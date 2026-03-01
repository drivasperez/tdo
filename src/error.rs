use std::fmt;

// r[error.db-not-found]
// r[error.db-locked]
// r[error.auth-missing]
// r[test.error-cases]

#[derive(Debug)]
pub enum Error {
    DbNotFound,
    DbLocked,
    AuthMissing,
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DbNotFound => write!(
                f,
                "Things 3 database not found. Is Things installed?\n\
                 Hint: use --db-path to specify the database location manually."
            ),
            Error::DbLocked => write!(
                f,
                "Things 3 database is locked (sync in progress?). \
                 Retried 3 times but it remains locked."
            ),
            Error::AuthMissing => write!(
                f,
                "Auth token required for write operations.\n\
                 Set via --auth-token flag or TDO_AUTH_TOKEN environment variable.\n\
                 Find your token in Things > Settings > General > Enable Things URLs > Authentication Token."
            ),
            Error::Sqlite(e) => write!(f, "Database error: {e}"),
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::NotFound(id) => write!(
                f,
                "Item not found: {id}\n\
                 Hint: use `tdo search <query>` to find items and their UUIDs."
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sqlite(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}
