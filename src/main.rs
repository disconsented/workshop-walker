use std::{fmt::Write, path::PathBuf, sync::Arc};
use bbscope::{BBCode, BBCodeTagConfig};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use salvo::__private::tracing::{error, info};
use snafu::{Whatever, prelude::*};
use surrealdb::{
    RecordId, Surreal,
    engine::local::{Db, RocksDb},
    opt::{Table, auth::Root},
    rpc::Method::Insert,
    sql::{
        Array, Data, Idiom, Value,
        statements::{InsertStatement, UpdateStatement, UpsertStatement},
        to_value,
    },
    syn::token::Keyword::{From, M},
};
use tokio::{
    fs::DirEntry,
    io::AsyncWriteExt,
    spawn,
    sync::Barrier,
    task::{JoinSet, spawn_blocking},
};

use crate::{
    language::detect,
    model::{Dependencies, Tag, WorkshopItem},
    steam::{GetPage, SteamRoot, Struct},
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
    tracing_subscriber::fmt().init();
    let settings: app_config::Config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .whatever_context("finding config")?
        .try_deserialize()
        .whatever_context("deserializing config")?;

    let db_exists = tokio::fs::metadata("./workshopdb").await.is_ok();
    let db = Surreal::new::<RocksDb>("./workshopdb")
        .await
        .whatever_context("connecting to db")?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: &settings.database.user,
        password: &settings.database.password,
    })
    .await
    .whatever_context("signing in to db")?;

    if !db_exists {
        info!("creating db");
        db.query(include_str!("../migrations/001-create-database.surql"))
            .await
            .whatever_context("running creation")?;
    }

    // Select a specific namespace / database
    db.use_ns("workshop")
        .use_db("workshop")
        .await
        .whatever_context("using ns/db")?;

    {
        let db = db.clone();
        spawn(async move {
            if settings.download_workshop {
                download(&settings.steam.api_token.clone(), settings.steam.appid)
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
                    if let Err(e) = insert_data(&db, entry.path(), &bbcode).await {
                        println!("read failed: {e:#?}");
                    }
                    i += 1;
                    print!("Progress: {i}\r")
                }
                println!("\rfinished");
            }
        });
    }
    web::start(db).await;
    Ok(())
}

// Read all cached steam files and load them into the database
async fn insert_data(db: &Surreal<Db>, path: PathBuf, bb: &BBCode) -> Result<(), Whatever> {
    let raw_json = tokio::fs::read_to_string(path)
        .await
        .whatever_context("reading file")?;
    let data: Struct = serde_json::from_str(&raw_json).whatever_context("parsing from disk")?;

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
        if id.eq(&RecordId::from_table_key("workshop_items", "3121742525")) {
            dbg!(&data.children);
        }

        let data = data
            .children
            .into_iter()
            .map(|child| {
                let dep_id = RecordId::from_table_key("workshop_items", child.publishedfileid);
                to_value(Dependencies {
                    id: RecordId::from_table_key(
                        "item_dependencies",
                        vec![
                            id.clone().into(),
                            dep_id.clone().into()
                        ],
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

    let foo = db
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
    let sql = format!("{foo:#?}");
    let mut response = foo.await.whatever_context("big insert query")?;

    let errors = response.take_errors();
    if errors.len() > 0 {
        error!("{sql}");
        error!("{errors:#?}");
        panic!("Errors: ")
    }

    Ok(())
}

// Download _all_ items from the steam workshop (for a given app) and cache them on disk.
async fn download(token: &str, appid: u32) -> Result<(), Whatever> {
    let client = Client::new();
    let mut first_page = GetPage::default();
    first_page.appid = appid;
    let first = first_page.into_request(&client, token).unwrap();
    let first_resp = client.execute(first).await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SteamRoot>();
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

    let barrier = Arc::new(Barrier::new(1));
    {
        let barrier = barrier.clone();
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
            barrier.wait().await;
            println!("Writer finished! Wrote {total} files");
        });
    }
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
    barrier.wait().await;
    Ok(())
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    #[test]
    fn test_serialize_ordering() {
        #[derive(Serialize)]
        pub enum Ordering {
            Random,
            Order(Vec<bool>),
        }

        dbg!(serde_json::to_string(&Ordering::Order(vec![true])).unwrap());
    }
}
