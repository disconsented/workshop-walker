use std::{fmt::Write, sync::Arc};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;
use snafu::{Whatever, prelude::*};
use surrealdb::{
    RecordId, Surreal,
    engine::local::{Db, RocksDb},
    opt::auth::Root,
};
use tokio::{fs::DirEntry, io::AsyncWriteExt, spawn, sync::Barrier};

use crate::{
    model::{Dependencies, WorkshopItem},
    steam::{GetPage, SteamRoot, Struct},
};
use crate::language::detect;

mod app_config;
mod model;
mod steam;
mod web;
mod language;

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
                let mut entries = tokio::fs::read_dir("data_cache").await.unwrap();
                let mut i = 0;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Err(e) = insert_data(&db, &mut i, entry).await {
                        println!("Insert Data: {e:#?}")
                    }
                }
            }
        });
    }
    web::start(db).await;
    Ok(())
}

async fn insert_data(db: &Surreal<Db>, i: &mut i32, entry: DirEntry) -> Result<(), Whatever> {
    let raw_json = tokio::fs::read_to_string(entry.path())
        .await
        .whatever_context("reading file")?;
    let data: Struct = serde_json::from_str(&raw_json).whatever_context("parsing from disk")?;

    let id = RecordId::from_table_key("workshop_items", data.publishedfileid);
    let description = data.file_description.unwrap_or_default();
    let item: WorkshopItem<RecordId> = WorkshopItem {
        appid: data.creator_appid.whatever_context("missing app id")?,
        author: data.creator.unwrap(),
        language: detect(&description),
        description,
        id: id.clone(),
        title: data.title.whatever_context("Missing title")?,
        preview_url: data.preview_url,
        last_updated: data.time_updated.unwrap_or_default() as _,
    };
    let _: Option<WorkshopItem<RecordId>> = db
        .upsert(item.id.clone())
        .content(item)
        .await
        .whatever_context("upserting item")?;

    let _: Vec<Dependencies> = db
        .insert("item_dependencies")
        .relation(
            data.children
                .into_iter()
                .map(|child| Dependencies {
                    this: id.clone(),
                    dependency: RecordId::from_table_key("workshop_items", child.publishedfileid),
                })
                .collect::<Vec<_>>(),
        )
        .await
        .whatever_context("inserting dependencies")?;

    *i += 1;
    print!("Progress: {i}\r");
    Ok(())
}

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

    let barrier = Arc::new(Barrier::new(2));
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
        match serde_json::from_str(&txt) {
            Ok(json) => {
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
    println!("finished downloading {downloaded}/total");
    barrier.wait().await;
    Ok(())
}
