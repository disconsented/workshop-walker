#![allow(dead_code)]

use futures::StreamExt as _;
use migrations_tool::{Error, Migration, Migrator, Outcome, SkipReason};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

fn m(id: &str, content: &str) -> Migration {
    Migration {
        id: id.to_string(),
        name: id.to_string(),
        content: content.to_string(),
    }
}

/// Scenario 1 — empty migration set is a no-op.
pub async fn empty_set(db: &Surreal<Any>) {
    let plan = Migrator::from_strings(vec![])
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    assert_eq!(plan.pending().len(), 0);
    assert_eq!(plan.skipped().len(), 0);

    let results: Vec<_> = plan.execute(db).collect().await;
    assert!(results.is_empty());
}

/// Scenario 2 — apply 3 migrations and verify state table contents.
pub async fn apply_three(db: &Surreal<Any>) {
    let migrations = vec![
        m("001_a.surql", "DEFINE TABLE a SCHEMALESS;"),
        m("002_b.surql", "DEFINE TABLE b SCHEMALESS;"),
        m("003_c.surql", "DEFINE TABLE c SCHEMALESS;"),
    ];

    let plan = Migrator::from_strings(migrations)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    assert_eq!(plan.pending().len(), 3);

    let outcomes: Vec<_> = plan.execute(db).collect().await;
    assert_eq!(outcomes.len(), 3);
    for o in &outcomes {
        assert!(o.is_ok(), "unexpected error: {o:?}");
    }

    // State table must record all three.
    let mut resp = db
        .query("SELECT meta::id(id) AS id FROM _migrations ORDER BY id ASC")
        .await
        .unwrap();
    let ids: Vec<serde_json::Value> = resp.take(0).unwrap();
    assert_eq!(ids.len(), 3);
}

/// Scenario 3 — re-running the same set skips everything.
pub async fn rerun_all_skipped(db: &Surreal<Any>) {
    let migrations = vec![
        m("001_a.surql", "DEFINE TABLE a SCHEMALESS;"),
        m("002_b.surql", "DEFINE TABLE b SCHEMALESS;"),
    ];

    // First run.
    let plan = Migrator::from_strings(migrations.clone())
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();
    let _: Vec<_> = plan.execute(db).collect().await;

    // Second run.
    let plan2 = Migrator::from_strings(migrations)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    assert_eq!(plan2.pending().len(), 0);
    assert_eq!(plan2.skipped().len(), 2);
    for (_, reason) in plan2.skipped() {
        assert_eq!(*reason, SkipReason::AlreadyApplied);
    }

    let results: Vec<_> = plan2.execute(db).collect().await;
    assert!(results.is_empty());
}

/// Scenario 4 — modified content with default config → ChecksumMismatch from plan().
pub async fn checksum_mismatch_error(db: &Surreal<Any>) {
    // Apply original.
    let original = vec![m("001_a.surql", "DEFINE TABLE a SCHEMALESS;")];
    let plan = Migrator::from_strings(original)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();
    let _: Vec<_> = plan.execute(db).collect().await;

    // Try to plan with modified content.
    let modified = vec![m("001_a.surql", "DEFINE TABLE a_modified SCHEMALESS;")];
    let result = Migrator::from_strings(modified).validate().unwrap().plan(db).await;

    assert!(
        matches!(result, Err(Error::ChecksumMismatch { .. })),
        "expected ChecksumMismatch, got: {result:?}"
    );
}

/// Scenario 5 — modified content with ignore → skip with ChecksumChangedButIgnored.
pub async fn checksum_mismatch_ignored(db: &Surreal<Any>) {
    // Apply original.
    let original = vec![m("001_a.surql", "DEFINE TABLE a SCHEMALESS;")];
    let plan = Migrator::from_strings(original)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();
    let _: Vec<_> = plan.execute(db).collect().await;

    // Re-plan with modified content and ignore flag set.
    let modified = vec![m("001_a.surql", "DEFINE TABLE a_modified SCHEMALESS;")];
    let plan2 = Migrator::from_strings(modified)
        .ignore_checksum_changes(true)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    assert_eq!(plan2.pending().len(), 0);
    assert_eq!(plan2.skipped().len(), 1);
    assert_eq!(plan2.skipped()[0].1, SkipReason::ChecksumChangedButIgnored);
}

/// Scenario 6 — bad SurrealQL → error, transaction rolled back, no state row written.
pub async fn bad_sql_no_state_row(db: &Surreal<Any>) {
    let migrations = vec![m("001_bad.surql", "THIS IS NOT VALID SURQL !!!!!")];

    let plan = Migrator::from_strings(migrations)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    let results: Vec<_> = plan.execute(db).collect().await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err(), "expected error, got: {:?}", results[0]);

    // State table must be empty — transaction rolled back.
    let mut resp = db.query("SELECT * FROM _migrations").await.unwrap();
    let rows: Vec<serde_json::Value> = resp.take(0).unwrap();
    assert!(rows.is_empty(), "state row was written despite transaction failure");
}

/// Scenario 7 — duplicate IDs → validation error before any DB call.
pub async fn duplicate_ids(db: &Surreal<Any>) {
    let migrations = vec![
        m("001.surql", "DEFINE TABLE a SCHEMALESS;"),
        m("001.surql", "DEFINE TABLE b SCHEMALESS;"),
    ];

    let result = Migrator::from_strings(migrations).validate();
    assert!(
        matches!(result, Err(Error::DuplicateIds { .. })),
        "expected DuplicateIds, got: {result:?}"
    );

    // Ensure db was never touched.
    let _ = db; // consumed to suppress lint
}

/// Scenario 8 — file source: lexical sort is correct across different magnitudes.
pub async fn lexical_sort(db: &Surreal<Any>, dir: &std::path::Path) {
    // Five filenames that span orders of magnitude but sort correctly lexically
    // because the prefix is fixed-width.
    let files = [
        ("1000000000_first.surql", "DEFINE TABLE first SCHEMALESS;"),
        ("1000000001_second.surql", "DEFINE TABLE second SCHEMALESS;"),
        ("1000000002_third.surql", "DEFINE TABLE third SCHEMALESS;"),
        ("9000000000_fourth.surql", "DEFINE TABLE fourth SCHEMALESS;"),
        ("9999999999_fifth.surql", "DEFINE TABLE fifth SCHEMALESS;"),
    ];

    for (name, content) in &files {
        std::fs::write(dir.join(name), content).unwrap();
    }

    let plan = Migrator::from_files(dir)
        .unwrap()
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    let pending = plan.pending();
    assert_eq!(pending.len(), 5);
    assert_eq!(pending[0].id, "1000000000_first.surql");
    assert_eq!(pending[1].id, "1000000001_second.surql");
    assert_eq!(pending[2].id, "1000000002_third.surql");
    assert_eq!(pending[3].id, "9000000000_fourth.surql");
    assert_eq!(pending[4].id, "9999999999_fifth.surql");
}

/// Scenario 9 — stream halts on first error; subsequent migrations not attempted.
pub async fn stream_halts_on_error(db: &Surreal<Any>) {
    let migrations = vec![
        m("001_ok.surql", "DEFINE TABLE ok1 SCHEMALESS;"),
        m("002_bad.surql", "THIS IS INVALID SURQL"),
        m("003_ok.surql", "DEFINE TABLE ok3 SCHEMALESS;"),
    ];

    let plan = Migrator::from_strings(migrations)
        .validate()
        .unwrap()
        .plan(db)
        .await
        .unwrap();

    let results: Vec<_> = plan.execute(db).collect().await;

    // Should be: Ok(001 applied), Err(002 failed). 003 must NOT appear.
    assert_eq!(results.len(), 2, "expected 2 results, got {}", results.len());
    assert!(results[0].is_ok(), "first migration should succeed");
    assert!(results[1].is_err(), "second migration should fail");

    // 003 was not applied.
    let mut resp = db
        .query("SELECT meta::id(id) AS id FROM _migrations WHERE meta::id(id) = '003_ok.surql'")
        .await
        .unwrap();
    let rows: Vec<serde_json::Value> = resp.take(0).unwrap();
    assert!(rows.is_empty(), "migration 003 should not have been applied");
}

/// Helper used in outcome assertions.
pub fn outcome_id(outcome: &Outcome) -> &str {
    match outcome {
        Outcome::Applied { id, .. } => id,
        Outcome::Skipped { id, .. } => id,
    }
}
