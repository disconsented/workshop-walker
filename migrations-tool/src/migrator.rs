use std::{collections::HashSet, marker::PhantomData, path::Path};

use snafu::ResultExt as _;
use surrealdb::{Connection, Surreal};
use tracing::{Instrument as _, debug, error, info, info_span, warn};

use crate::{
    Error, checksum,
    error::{InvalidUtf8Snafu, IoSnafu, StateQuerySnafu},
    plan::Plan,
    types::{Migration, SkipReason},
};

/// Typestate marker — builder methods are available, `plan()` is not.
#[derive(Debug)]
pub struct Unvalidated;

/// Typestate marker — validation passed, `plan()` is available.
#[derive(Debug)]
pub struct Validated;

/// Async migration runner, parameterised by a typestate `S`.
///
/// Start with [`Migrator::from_files`] or [`Migrator::from_strings`], configure
/// via builder methods, advance to [`Validated`] via [`Migrator::validate`],
/// then call [`Migrator::plan`].
#[derive(Debug)]
pub struct Migrator<S = Unvalidated> {
    pub(crate) migrations: Vec<Migration>,
    pub(crate) table: String,
    pub(crate) ignore_checksum_changes: bool,
    pub(crate) _state: PhantomData<S>,
}

impl Migrator<Unvalidated> {
    /// Load every `*.surql` file from `dir`, sorted lexicographically by
    /// filename.
    ///
    /// `Migration::id` and `Migration::name` are both set to the full filename.
    pub fn from_files(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();
        let mut paths = Vec::new();

        let entries = std::fs::read_dir(dir).context(IoSnafu {
            path: dir.to_path_buf(),
        })?;

        for entry in entries {
            let entry = entry.context(IoSnafu {
                path: dir.to_path_buf(),
            })?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("surql") {
                paths.push(path);
            }
        }

        // Lexicographic sort by filename only (not full path).
        paths.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });

        let mut migrations = Vec::with_capacity(paths.len());
        for path in paths {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let bytes = std::fs::read(&path).context(IoSnafu { path: path.clone() })?;
            let content =
                String::from_utf8(bytes).context(InvalidUtf8Snafu { path: path.clone() })?;

            migrations.push(Migration {
                id: filename.clone(),
                name: filename,
                content,
            });
        }

        Ok(Self::new(migrations))
    }

    /// Use a caller-supplied list of migrations. Order is preserved as given.
    pub fn from_strings(migrations: Vec<Migration>) -> Self {
        Self::new(migrations)
    }

    /// Override the state-tracking table name (default: `_migrations`).
    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = table.into();
        self
    }

    /// When `true`, a checksum mismatch for an already-applied migration is
    /// treated as a skip rather than an error (a warning is still emitted).
    pub fn ignore_checksum_changes(mut self, ignore: bool) -> Self {
        self.ignore_checksum_changes = ignore;
        self
    }

    /// Validate the migration set without touching the database.
    ///
    /// Collects **all** problems before returning — duplicate IDs are reported
    /// together rather than one at a time.
    pub fn validate(self) -> Result<Migrator<Validated>, Error> {
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut duplicate_ids: Vec<String> = Vec::new();

        for m in &self.migrations {
            if !seen_ids.insert(m.id.clone()) {
                duplicate_ids.push(m.id.clone());
            }
        }

        if !duplicate_ids.is_empty() {
            return Err(Error::DuplicateIds { ids: duplicate_ids });
        }

        for m in &self.migrations {
            if m.content.trim().is_empty() {
                return Err(Error::EmptyContent { id: m.id.clone() });
            }
        }

        debug!(
            count = self.migrations.len(),
            table = %self.table,
            "migrations validated"
        );

        Ok(Migrator {
            migrations: self.migrations,
            table: self.table,
            ignore_checksum_changes: self.ignore_checksum_changes,
            _state: PhantomData,
        })
    }

    fn new(migrations: Vec<Migration>) -> Self {
        Self {
            migrations,
            table: "_migrations".to_string(),
            ignore_checksum_changes: false,
            _state: PhantomData,
        }
    }
}

impl Migrator<Validated> {
    /// Query the database to determine which migrations have already been
    /// applied, then classify the remaining ones as pending or skipped.
    ///
    /// Returns [`Error::ChecksumMismatch`] if a migration's content has changed
    /// and `ignore_checksum_changes` is `false`.
    pub async fn plan<C: Connection>(&self, db: &Surreal<C>) -> Result<Plan, Error> {
        let span = info_span!("plan", table = %self.table);

        async {
            let applied = self.fetch_applied(db).await?;
            self.classify(applied)
        }
        .instrument(span)
        .await
    }

    async fn fetch_applied<C: Connection>(
        &self,
        db: &Surreal<C>,
    ) -> Result<std::collections::HashMap<String, String>, Error> {
        // Ensure the state table exists (no-op if already present).
        let define_sql = format!(
            "DEFINE TABLE IF NOT EXISTS `{}` SCHEMALESS",
            self.table.replace('`', "")
        );
        db.query(define_sql)
            .await
            .context(StateQuerySnafu)?
            .check()
            .context(StateQuerySnafu)?;

        let mut response = db
            .query("SELECT meta::id(id) AS id, checksum FROM type::table($table) ORDER BY id ASC")
            .bind(("table", self.table.clone()))
            .await
            .context(StateQuerySnafu)?;

        let records: Vec<surrealdb::types::Object> = response.take(0).context(StateQuerySnafu)?;

        debug!(count = records.len(), "fetched applied migrations");

        let applied = records
            .into_iter()
            .filter_map(|obj| {
                let id = obj.get("id")?.as_string()?.clone();
                let checksum = obj.get("checksum")?.as_string()?.clone();
                Some((id, checksum))
            })
            .collect();

        Ok(applied)
    }

    fn classify(&self, applied: std::collections::HashMap<String, String>) -> Result<Plan, Error> {
        let mut pending = Vec::new();
        let mut skipped = Vec::new();

        for migration in &self.migrations {
            match applied.get(&migration.id) {
                None => {
                    debug!(id = %migration.id, "pending");
                    pending.push(migration.clone());
                }
                Some(stored_checksum) => {
                    let current = checksum::compute(&migration.content);
                    if current == *stored_checksum {
                        debug!(id = %migration.id, "already applied");
                        skipped.push((migration.clone(), SkipReason::AlreadyApplied));
                    } else if self.ignore_checksum_changes {
                        warn!(
                            id = %migration.id,
                            stored = %stored_checksum,
                            current = %current,
                            "checksum changed but ignore_checksum_changes is set — skipping"
                        );
                        skipped.push((migration.clone(), SkipReason::ChecksumChangedButIgnored));
                    } else {
                        error!(
                            id = %migration.id,
                            "checksum mismatch; set ignore_checksum_changes(true) to bypass"
                        );
                        return Err(Error::ChecksumMismatch {
                            id: migration.id.clone(),
                        });
                    }
                }
            }
        }

        info!(
            pending = pending.len(),
            skipped = skipped.len(),
            "plan ready"
        );

        Ok(Plan {
            table: self.table.clone(),
            pending,
            skipped,
        })
    }
}
