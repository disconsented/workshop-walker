> **⚠️ AI-Generated Code**
>
> This crate was built with [Claude](https://anthropic.com) (Anthropic AI assistance).
> It has not been independently audited. Review the logic carefully before deploying
> to a production environment or trusting it with critical data.

# migrations-tool

A type-driven, async, **up-only** migration runner for [SurrealDB](https://surrealdb.com).
Works with any connection type (`Mem`, `RocksDb`, `Ws`, `Http`, …) via the generic
`Surreal<C: Connection>` API.

## Quickstart

```rust
use futures::StreamExt as _;
use migrations_tool::{Migrator, Outcome};

// Connect to SurrealDB however you like, then:
let plan = Migrator::from_files("./migrations")?
    .with_table("_migrations")       // default — can be omitted
    .ignore_checksum_changes(false)  // default — can be omitted
    .validate()?
    .plan(&db)
    .await?;

println!("will apply {} migrations", plan.pending().len());

let mut stream = std::pin::pin!(plan.execute(&db));
while let Some(outcome) = stream.next().await {
    match outcome? {
        Outcome::Applied { id, duration } => println!("applied {id} in {duration:?}"),
        Outcome::Skipped { id, .. }       => println!("skipped {id}"),
    }
}
```

## File layout

Place `.surql` files in a single directory. The runner loads every `*.surql` file
**non-recursively** and sorts them **lexicographically by filename**. No timestamp
parsing is performed — sort order is entirely determined by the filename string.

A timestamp prefix (e.g. `1779584187_baseline.surql`) works well because it is
fixed-width for any foreseeable epoch value.

- `Migration::id` = full filename (`1779584187_baseline.surql`)
- `Migration::name` = same as `id`

## Typestate flow

```
Migrator<Unvalidated>          ← from_files() / from_strings()
    .with_table("…")           ┐
    .ignore_checksum_changes(…)┘  builder methods — only here
    .validate()?               →  Migrator<Validated>
    .plan(&db).await?          →  Plan
    .execute(&db)              →  impl Stream<Item = Result<Outcome, Error>>
```

- **`validate()`** — connection-free; checks for duplicate IDs and empty content.
  Collects all duplicate IDs in a single error rather than stopping at the first.
- **`plan()`** — queries the state table; classifies each migration as pending,
  skipped, or errors with `ChecksumMismatch`.
- **`execute()`** — drives the stream; applies one migration per `next()` call.
  Halts on the first error.

## Checksums

Every migration's `content` is hashed with **BLAKE2b-256** (32-byte output,
lowercase hex) when it is first applied. On subsequent runs the stored hash is
compared against the hash of the file as it exists on disk.

A mismatch means the migration was edited after it was applied and `plan()` returns
`Error::ChecksumMismatch`.

To downgrade the mismatch from an error to a skip with a warning:

```rust
Migrator::from_files("./migrations")?
    .ignore_checksum_changes(true)
    .validate()?
    .plan(&db)
    .await?;
```

## State table schema

The runner maintains a table (default `_migrations`) with one record per applied
migration:

| field         | type     | description                          |
|---------------|----------|--------------------------------------|
| `id`          | record   | `_migrations:⟨migration-id⟩`         |
| `name`        | string   | human label (= `id` for file source) |
| `checksum`    | string   | BLAKE2b-256 hex of `content`         |
| `applied_at`  | datetime | server-side `time::now()`            |
| `duration_ms` | int      | wall-clock ms from tx start to commit|

## Error handling

All errors are variants of the single `Error` enum (powered by
[snafu](https://docs.rs/snafu)):

| variant             | when                                                     |
|---------------------|----------------------------------------------------------|
| `Io`                | reading a migration file fails                           |
| `InvalidUtf8`       | a migration file is not valid UTF-8                      |
| `DuplicateIds`      | two migrations share the same ID                         |
| `EmptyContent`      | a migration has blank content                            |
| `ChecksumMismatch`  | content changed after it was applied; use `ignore_…(true)` to bypass |
| `StateQuery`        | querying the state table fails                           |
| `MigrationFailed`   | executing a migration's SurrealQL fails                  |
| `Surreal`           | other SurrealDB client errors                            |

## Testing

```sh
# in-memory engine
cargo test --test mem

# RocksDB engine
cargo test --test rocksdb
```
