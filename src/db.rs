// r[data.read]
// r[global.db-path]
// r[test.fixture-db]

use crate::error::Error;
use rusqlite::{Connection, OpenFlags};
use std::thread;
use std::time::Duration;

/// Find the Things 3 database path using glob pattern.
pub fn find_db_path() -> Result<String, Error> {
    let home = std::env::var("HOME").map_err(|_| Error::DbNotFound)?;
    let pattern = format!(
        "{home}/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac/\
         ThingsData-*/Things Database.thingsdatabase/main.sqlite"
    );
    let mut paths: Vec<_> = glob::glob(&pattern)
        .map_err(|_| Error::DbNotFound)?
        .filter_map(|r| r.ok())
        .collect();
    paths.sort();
    paths
        .into_iter()
        .next()
        .map(|p| p.to_string_lossy().into_owned())
        .ok_or(Error::DbNotFound)
}

/// Open the Things database in read-only mode with retry on lock.
pub fn open_db(path: &str) -> Result<Connection, Error> {
    // r[error.db-locked]
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    for attempt in 0..3 {
        match Connection::open_with_flags(path, flags) {
            Ok(conn) => {
                // Test that we can actually query
                match conn.execute_batch("SELECT 1") {
                    Ok(()) => return Ok(conn),
                    Err(e) if is_locked(&e) && attempt < 2 => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) if is_locked(&e) => return Err(Error::DbLocked),
                    Err(e) => return Err(Error::Sqlite(e)),
                }
            }
            Err(e) if is_locked(&e) && attempt < 2 => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) if is_locked(&e) => return Err(Error::DbLocked),
            Err(e) => return Err(Error::Sqlite(e)),
        }
    }
    Err(Error::DbLocked)
}

fn is_locked(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked,
                ..
            },
            _
        )
    )
}
