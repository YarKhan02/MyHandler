use rusqlite::Error as SqliteError;
use std::fmt;

#[derive(Debug)]
pub enum DbError {
    Sqlite(SqliteError),
    Io(std::io::Error),
    PathError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbError::Sqlite(e) => write!(f, "Database error: {}", e),
            DbError::Io(e) => write!(f, "IO error: {}", e),
            DbError::PathError(e) => write!(f, "Path error: {}", e),
        }
    }
}

impl std::error::Error for DbError {}

impl From<SqliteError> for DbError {
    fn from(err: SqliteError) -> Self {
        DbError::Sqlite(err)
    }
}

impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::Io(err)
    }
}

pub type DbResult<T> = std::result::Result<T, DbError>;
