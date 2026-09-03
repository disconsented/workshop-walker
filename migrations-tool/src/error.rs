use snafu::Snafu;

/// All errors produced by the migration runner.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// An I/O error reading a migration file.
    #[snafu(display("I/O error reading '{}': {source}", path.display()))]
    Io {
        source: std::io::Error,
        path: std::path::PathBuf,
    },

    /// A migration file contained invalid UTF-8.
    #[snafu(display("Invalid UTF-8 in '{}': {source}", path.display()))]
    InvalidUtf8 {
        source: std::string::FromUtf8Error,
        path: std::path::PathBuf,
    },

    /// One or more migration IDs appeared more than once.
    #[snafu(display("Duplicate migration IDs: {}", ids.join(", ")))]
    DuplicateIds { ids: Vec<String> },

    /// A migration has no SurrealQL content.
    #[snafu(display("Migration '{}' has empty content", id))]
    EmptyContent { id: String },

    /// The on-disk checksum differs from the stored checksum.
    ///
    /// Call `.ignore_checksum_changes(true)` on the migrator to treat this as
    /// a skip instead of an error.
    #[snafu(display(
        "Checksum mismatch for migration '{}'. Set ignore_checksum_changes(true) to bypass.",
        id
    ))]
    ChecksumMismatch { id: String },

    /// A general SurrealDB client error.
    #[snafu(display("SurrealDB error: {source}"))]
    Surreal {
        #[snafu(source(from(surrealdb::Error, Box::new)))]
        source: Box<surrealdb::Error>,
    },

    /// An error querying the migration state table.
    #[snafu(display("Error querying migration state: {source}"))]
    StateQuery {
        #[snafu(source(from(surrealdb::Error, Box::new)))]
        source: Box<surrealdb::Error>,
    },

    /// A migration failed during execution.
    #[snafu(display("Migration '{}' failed: {source}", id))]
    MigrationFailed {
        id: String,
        #[snafu(source(from(surrealdb::Error, Box::new)))]
        source: Box<surrealdb::Error>,
    },
}
