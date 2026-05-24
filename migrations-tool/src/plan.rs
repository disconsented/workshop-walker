use std::time::Instant;

use async_stream::try_stream;
use futures::Stream;
use snafu::ResultExt as _;
use surrealdb::{Connection, Surreal};
use tracing::{info, info_span, trace, Instrument as _};

use crate::checksum;
use crate::error::{MigrationFailedSnafu, SurrealSnafu};
use crate::types::{Migration, Outcome, SkipReason};
use crate::Error;

/// A classified set of migrations ready to execute.
///
/// Obtain via [`crate::Migrator::plan`]. Inspect [`Plan::pending`] and
/// [`Plan::skipped`] before calling [`Plan::execute`].
#[derive(Debug)]
pub struct Plan {
    pub(crate) table: String,
    pub(crate) pending: Vec<Migration>,
    pub(crate) skipped: Vec<(Migration, SkipReason)>,
}

impl Plan {
    /// Migrations that will be applied when [`Plan::execute`] is called.
    pub fn pending(&self) -> &[Migration] {
        &self.pending
    }

    /// Migrations that will be skipped and why.
    pub fn skipped(&self) -> &[(Migration, SkipReason)] {
        &self.skipped
    }

    /// Execute pending migrations in order, yielding one [`Outcome`] per migration.
    ///
    /// Each migration runs inside a `BEGIN / COMMIT TRANSACTION` block that also
    /// inserts the state row, so a failed migration leaves no state behind.
    ///
    /// The stream halts on the first error — subsequent migrations are not attempted.
    pub fn execute<'db, C>(self, db: &'db Surreal<C>) -> impl Stream<Item = Result<Outcome, Error>> + 'db
    where
        C: Connection,
    {
        let table = self.table;
        let pending = self.pending;

        try_stream! {
            for migration in pending {
                let duration = run_migration(db, &table, &migration).await?;
                yield Outcome::Applied { id: migration.id, duration };
            }
        }
    }
}

async fn run_migration<C: Connection>(
    db: &Surreal<C>,
    table: &str,
    migration: &Migration,
) -> Result<std::time::Duration, Error> {
    let span = info_span!("migration", id = %migration.id);

    async {
        trace!(content = %migration.content, "executing migration");

        let checksum = checksum::compute(&migration.content);
        let t0 = Instant::now();

        let tx = db
            .clone()
            .begin()
            .await
            .context(SurrealSnafu)?;

        let migration_result = tx
            .query(&migration.content)
            .await
            .and_then(|r| r.check())
            .context(MigrationFailedSnafu { id: migration.id.clone() });

        let duration = t0.elapsed();

        if let Err(e) = migration_result {
            let _ = tx.cancel().await;
            return Err(e);
        }

        let insert_result = tx
            .query(
                "CREATE type::record($t, $id) SET \
                 name = $name, \
                 checksum = $checksum, \
                 applied_at = time::now(), \
                 duration_ms = $duration_ms",
            )
            .bind(("t", table.to_string()))
            .bind(("id", migration.id.clone()))
            .bind(("name", migration.name.clone()))
            .bind(("checksum", checksum))
            .bind(("duration_ms", duration.as_millis() as i64))
            .await
            .and_then(|r| r.check())
            .context(MigrationFailedSnafu { id: migration.id.clone() });

        if let Err(e) = insert_result {
            let _ = tx.cancel().await;
            return Err(e);
        }

        tx.commit()
            .await
            .context(MigrationFailedSnafu { id: migration.id.clone() })?;

        info!(
            id = %migration.id,
            duration_ms = duration.as_millis(),
            "applied migration"
        );

        Ok(duration)
    }
    .instrument(span)
    .await
}
