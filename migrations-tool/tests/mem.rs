mod common;

use surrealdb::engine::any::connect;

async fn db() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
    let db = connect("mem://").await.expect("connect mem://");
    db.use_ns("test").use_db("test").await.expect("use_ns/use_db");
    db
}

#[tokio::test]
async fn empty_set() {
    common::empty_set(&db().await).await;
}

#[tokio::test]
async fn apply_three() {
    common::apply_three(&db().await).await;
}

#[tokio::test]
async fn rerun_all_skipped() {
    common::rerun_all_skipped(&db().await).await;
}

#[tokio::test]
async fn checksum_mismatch_error() {
    common::checksum_mismatch_error(&db().await).await;
}

#[tokio::test]
async fn checksum_mismatch_ignored() {
    common::checksum_mismatch_ignored(&db().await).await;
}

#[tokio::test]
async fn bad_sql_no_state_row() {
    common::bad_sql_no_state_row(&db().await).await;
}

#[tokio::test]
async fn duplicate_ids() {
    common::duplicate_ids(&db().await).await;
}

#[tokio::test]
async fn lexical_sort() {
    let dir = tempfile::tempdir().expect("tempdir");
    common::lexical_sort(&db().await, dir.path()).await;
}

#[tokio::test]
async fn stream_halts_on_error() {
    common::stream_halts_on_error(&db().await).await;
}
