mod common;

use tempfile::TempDir;

async fn db() -> (surrealdb::Surreal<surrealdb::engine::any::Any>, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir for rocksdb");
    let path = format!("rocksdb://{}", dir.path().display());
    let db = surrealdb::engine::any::connect(path)
        .await
        .expect("connect rocksdb");
    db.use_ns("test").use_db("test").await.expect("use_ns/use_db");
    (db, dir)
}

#[tokio::test]
async fn empty_set() {
    let (db, _dir) = db().await;
    common::empty_set(&db).await;
}

#[tokio::test]
async fn apply_three() {
    let (db, _dir) = db().await;
    common::apply_three(&db).await;
}

#[tokio::test]
async fn rerun_all_skipped() {
    let (db, _dir) = db().await;
    common::rerun_all_skipped(&db).await;
}

#[tokio::test]
async fn checksum_mismatch_error() {
    let (db, _dir) = db().await;
    common::checksum_mismatch_error(&db).await;
}

#[tokio::test]
async fn checksum_mismatch_ignored() {
    let (db, _dir) = db().await;
    common::checksum_mismatch_ignored(&db).await;
}

#[tokio::test]
async fn bad_sql_no_state_row() {
    let (db, _dir) = db().await;
    common::bad_sql_no_state_row(&db).await;
}

#[tokio::test]
async fn duplicate_ids() {
    let (db, _dir) = db().await;
    common::duplicate_ids(&db).await;
}

#[tokio::test]
async fn lexical_sort() {
    let (db, _db_dir) = db().await;
    let mig_dir = tempfile::tempdir().expect("tempdir for migrations");
    common::lexical_sort(&db, mig_dir.path()).await;
}

#[tokio::test]
async fn stream_halts_on_error() {
    let (db, _dir) = db().await;
    common::stream_halts_on_error(&db).await;
}
