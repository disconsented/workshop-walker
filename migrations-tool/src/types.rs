use std::time::Duration;

/// A single migration to be applied to the database.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Unique identifier used for sort order and record lookup.
    ///
    /// When loaded from files this equals the full filename
    /// (e.g. `1779584187_baseline.surql`).
    pub id: String,

    /// Human-readable label stored in the state table.
    pub name: String,

    /// Raw SurrealQL to execute.
    pub content: String,
}

/// The result of processing a single migration.
#[derive(Debug)]
pub enum Outcome {
    /// The migration was executed and committed.
    Applied {
        /// The migration id.
        id: String,
        /// Wall-clock time from transaction start to commit.
        duration: Duration,
    },
    /// The migration was skipped.
    Skipped {
        /// The migration id.
        id: String,
        /// Why it was skipped.
        reason: SkipReason,
    },
}

/// Why a migration was not executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The migration is already recorded in the state table with a matching checksum.
    AlreadyApplied,
    /// The migration is recorded in the state table but its content has changed.
    ///
    /// This only occurs when `ignore_checksum_changes(true)` is set.
    ChecksumChangedButIgnored,
}
