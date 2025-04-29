use std::{
    env,
    fmt::Write,
    ops::Add,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bbscope::{BBCode, BBCodeTagConfig};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use salvo::__private::tracing::{debug, error, info};
use snafu::{Whatever, prelude::*};
use surrealdb::{
    RecordId, Surreal,
    engine::local::{Db, RocksDb},
    opt::auth::Root,
    sql::{Data, Value, statements::InsertStatement, to_value},
};
use tokio::{
    io::AsyncWriteExt,
    spawn,
    sync::mpsc::UnboundedSender,
    time::{Instant, sleep, sleep_until},
};

use crate::{
    language::detect,
    model::{Dependencies, WorkshopItem},
    steam::{EPublishedFileQueryType, GetPage, SteamRoot, Struct},
};

mod app_config;
mod language;
mod model;
mod steam;
mod web;

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = Whatever;
#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_default())
        .try_init();
    let settings: app_config::Config = config::Config::builder()
        .add_source(config::File::with_name("config/config.toml"))
        .build()
        .whatever_context("finding config")?
        .try_deserialize()
        .whatever_context("deserializing config")?;

    let db_exists = tokio::fs::metadata("./workshopdb").await.is_ok()
        && std::fs::read_dir("./workshopdb").iter().count() > 1;

    let db = Surreal::new::<RocksDb>("./workshopdb")
        .await
        .whatever_context("connecting to db")?;

    // Select a specific namespace / database
    db.use_ns("workshop")
        .use_db("workshop")
        .await
        .whatever_context("using ns/db")?;

    if !db_exists {
        info!("creating db");
        db.query(format!(
            "DEFINE USER {} ON ROOT PASSWORD '{}' ROLES OWNER DURATION FOR TOKEN 1h, FOR SESSION \
             NONE;",
            settings.database.user, settings.database.password
        ))
        .await
        .whatever_context("creating root user")?;
        db.query(include_str!("../migrations/001-create-database.surql"))
            .await
            .whatever_context("running creation")?;
    }
    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: &settings.database.user,
        password: &settings.database.password,
    })
    .await
    .whatever_context("signing in to db")?;

    {
        let db = db.clone();
        let token = settings.steam.api_token.clone();
        spawn(async move {
            if settings.download_workshop {
                download_to_disk(&token, settings.steam.appid, GetPage::default())
                    .await
                    .unwrap();
            }

            if settings.read_from_cache {
                info!("reading data from cache");
                let mut readdir = tokio::fs::read_dir("data_cache").await.unwrap();
                let mut i = 0;

                let bbcode = BBCode::from_config(BBCodeTagConfig::extended(), None).unwrap();
                // SurrealDB can't handle these queries in parallel, so serial for now
                while let Ok(Some(entry)) = readdir.next_entry().await {
                    if let Err(e) = read_and_insert_data(&db, entry.path(), &bbcode).await {
                        println!("read failed: {e:#?}");
                    }
                    i += 1;
                    print!("Progress: {i}\r")
                }
                println!("\rfinished");
            }
        });
    }
    if settings.updater {
        let db = db.clone();
        let token = settings.steam.api_token.clone();
        spawn(async move {
            let token = token;
            let timestamp: Option<u64> = db
                .query("SELECT last_updated FROM workshop_items ORDER BY last_updated DESC LIMIT 1")
                .await
                .unwrap()
                .take((0, "last_updated"))
                .unwrap();
            let timestamp = timestamp.unwrap_or(0);
            let time_since = SystemTime::now()
                .duration_since(UNIX_EPOCH.add(Duration::from_secs(timestamp)))
                .unwrap();
            let h12 = Duration::from_secs(60 * 60 * 12);
            if time_since < h12 {
                let awake_in = h12 - time_since;
                info!(
                    "last_update from under 12 hours ago, sleeping for: {}",
                    humantime::Duration::from(awake_in)
                );
                sleep(awake_in).await;
            } else {
                info!("last_updated: {}", humantime::Duration::from(time_since))
            }
            loop {
                info!("Starting update");
                let next_run = Instant::now() + Duration::from_secs(60 * 60 * 12); // 12 hours later
                let mut first_page = GetPage::default();
                first_page.query_type = EPublishedFileQueryType::RankedByLastUpdatedDate;
                let bbcode = BBCode::from_config(BBCodeTagConfig::extended(), None).unwrap();
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SteamRoot>();
                let to_database = {
                    let db = db.clone();
                    tokio::spawn(async move {
                        let mut i = 0;
                        while let Some(steam_root) = rx.recv().await {
                            for item in steam_root.response.publishedfiledetails {
                                match serde_json::from_value(item) {
                                    Ok(item) => {
                                        if let Err(e) = insert_data(&db, &bbcode, item).await {
                                            error!("Inserting updated data with err: {e:?}")
                                        }
                                        i += 1;
                                    }
                                    Err(e) => error!("Parsing item: {e:?}"),
                                };
                            }
                            debug!("Insert progress: {i}");
                        }
                    })
                };
                if let Err(e) = download(&token, settings.steam.appid, first_page, tx).await {
                    error!("Periodic downloads failed with error: {e:?}")
                }
                let _ = to_database.await;
                info!("Finished updating");
                sleep_until(next_run).await;
            }
        });
    }
    web::start(db).await;
    Ok(())
}

// Read all cached steam files and load them into the database
async fn read_and_insert_data(
    db: &Surreal<Db>,
    path: PathBuf,
    bb: &BBCode,
) -> Result<(), Whatever> {
    let raw_json = tokio::fs::read_to_string(path)
        .await
        .whatever_context("reading file")?;
    let data: Struct = serde_json::from_str(&raw_json).whatever_context("parsing from text")?;

    insert_data(db, bb, data).await
}

async fn insert_data(db: &Surreal<Db>, bb: &BBCode, data: Struct) -> Result<(), Whatever> {
    let id = RecordId::from_table_key("workshop_items", data.publishedfileid);
    let description = data.file_description.unwrap_or_default();
    let item: WorkshopItem<RecordId> = WorkshopItem {
        appid: data.creator_appid.whatever_context("missing app id")?,
        author: data.creator.unwrap(),
        language: detect(&description),
        description: bb.parse(&description),
        id: id.clone(),
        title: data.title.whatever_context("Missing title")?,
        preview_url: data.preview_url,
        last_updated: data.time_updated.unwrap_or_default() as _,
        tags: vec![],
        score: data.vote_data.map(|votes|votes.score).unwrap_or_default()
    };
    let tags = data
        .tags
        .iter()
        .cloned()
        .map(|tag| crate::model::Tag {
            app_id: item.appid.clone() as _,
            display_name: tag.display_name,
            tag: tag.tag,
        })
        .collect::<Vec<_>>();
    let insert_tags = {
        let mut stmt = InsertStatement::default();
        stmt.into = Some(Value::Table("tags".into()));
        stmt.data = Data::SingleExpression(to_value(tags.clone()).unwrap());
        stmt.ignore = true;
        stmt
    };

    let upsert_item = {
        let mut stmt = InsertStatement::default();
        stmt.data = Data::SingleExpression(to_value(item).unwrap());
        stmt.into = Some(Value::Table("workshop_items".into()));
        stmt.ignore = true;
        stmt
    };

    let insert_item_deps = {
        let mut stmt = InsertStatement::default();
        stmt.relation = true;

        stmt.into = Some(Value::Table("item_dependencies".into()));
        let data = data
            .children
            .into_iter()
            .map(|child| {
                let dep_id = RecordId::from_table_key("workshop_items", child.publishedfileid);
                to_value(Dependencies {
                    id: RecordId::from_table_key(
                        "item_dependencies",
                        vec![id.clone().into(), dep_id.clone().into()],
                    ),
                    this: id.clone(),
                    dependency: dep_id,
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        stmt.data = Data::SingleExpression(Value::Array(data.into()));
        stmt.ignore = true;
        stmt
    };

    let query = db
        .query("BEGIN TRANSACTION")
        .query(insert_tags)
        .query(upsert_item)
        .query(insert_item_deps)
        .query("UPDATE $id SET tags=$tags")
        .bind(("id", id.clone()))
        .bind((
            "tags",
            tags.iter()
                .map(|tag| RecordId::from_table_key("tags", tag.tag.clone()))
                .collect::<Vec<_>>(),
        ))
        .query("COMMIT");
    let sql = format!("{query:#?}");
    let mut response = query.await.whatever_context("big insert query")?;

    let errors = response.take_errors();
    if errors.len() > 0 {
        error!("{sql}");
        error!("{errors:#?}");
        panic!("Errors: ")
    }

    Ok(())
}

// Download _all_ items from the steam workshop (for a given app) and cache them
// on disk.
async fn download_to_disk(token: &str, appid: u32, first_page: GetPage) -> Result<(), Whatever> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SteamRoot>();
    {
        spawn(async move {
            println!("running writer");
            let mut total = 0;
            while let Some(thing) = rx.recv().await {
                for file in thing.response.publishedfiledetails {
                    let file: Struct = serde_json::from_value(file).unwrap();
                    let file_path = format!("./data_cache/{}.json", file.publishedfileid);
                    let _ = tokio::fs::remove_file(&file_path).await;
                    let mut new_file = tokio::fs::File::create_new(file_path).await.unwrap();
                    let json = serde_json::to_string(&file).unwrap();
                    new_file.write_all(json.as_bytes()).await.unwrap();
                    total += 1;
                }
            }
            println!("Writer finished! Wrote {total} files");
        });
    }

    download(token, appid, first_page, tx).await
}
// Downloads all items from the workshop, forwarding them to another worker.
async fn download(
    token: &str,
    appid: u32,
    mut first_page: GetPage,
    tx: UnboundedSender<SteamRoot>,
) -> Result<(), Whatever> {
    let client = Client::new();
    first_page.appid = appid;
    let first = first_page.into_request(&client, token).unwrap();
    let first_resp = client.execute(first).await.unwrap();
    if !first_resp.status().is_success() {
        whatever!("Got error on first response: {first_resp:?}")
    }
    let first_root: SteamRoot = serde_json::from_str(&first_resp.text().await.unwrap()).unwrap();
    let total = first_root.response.total;
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} \
             ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );
    let mut downloaded = 0;
    let mut next = GetPage::try_from(&first_root)?;
    tx.send(first_root).unwrap();
    while total >= downloaded {
        next.appid = appid;
        let response = client
            .execute(next.into_request(&client, token).unwrap())
            .await
            .unwrap();
        let txt = response.text().await.unwrap();
        match serde_json::from_str::<SteamRoot>(&txt) {
            Ok(json) => {
                if json.response.publishedfiledetails.is_empty() {
                    info!("Got fewer than expected items; exiting early");
                    break;
                }
                next = GetPage::try_from(&json)?;
                downloaded += json.response.publishedfiledetails.len() as i64;
                let _ = tx.send(json).unwrap();
                pb.set_position(downloaded as u64);
            }
            Err(e) => {
                use std::io::Write;
                let mut f = std::fs::File::create_new("dead.json").unwrap();
                f.write(txt.as_bytes()).unwrap();
                return Err(e).whatever_context("deser response from steam");
            }
        }
    }
    println!("finished downloading {downloaded}/{total}");
    Ok(())
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    #[test]
    fn test_serialize_ordering() {
        #[derive(Serialize)]
        pub enum Ordering {
            Order(Vec<bool>),
            Random,
        }

        dbg!(serde_json::to_string(&Ordering::Order(vec![true])).unwrap());
    }
}
