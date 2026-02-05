use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;
use crate::error::{DbError, DbResult};

// Trait for types that can be inserted into the database
pub trait Insertable {
    fn table_name() -> &'static str;
    fn columns_values(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)>;
}

// Global database connection wrapped in Mutex for thread safety
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app: &AppHandle) -> DbResult<Self> {
        let path = get_db_path(app)?;
        
        match Connection::open(&path) {
            Ok(conn) => {
                println!("Database connection opened");
                Ok(Database {
                    conn: Mutex::new(conn),
                })
            }
            Err(e) => {
                eprintln!("Failed to open database at {:?}: {}", path, e);
                Err(DbError::Sqlite(e))
            }
        }
    }

    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        match self.conn.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Database mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        }
    }
}

pub fn get_db_path(app: &AppHandle) -> DbResult<PathBuf> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| DbError::PathError(format!("Failed to get app data directory: {:?}", e)))?;
    
    if let Err(e) = fs::create_dir_all(&app_dir) {
        eprintln!("Failed to create app data directory: {}", e);
        return Err(DbError::Io(e));
    }
    
    let db_path = app_dir.join("myhandler.db");
    println!("Database path: {:?}", db_path);
    
    Ok(db_path)
}

pub fn init_db(app: &AppHandle) -> DbResult<()> {
    println!("Initializing database...");
    
    // Create global database connection
    let db = Database::new(app)?;
    
    // Initialize tables
    let conn = db.get_connection();
    
    let table_sql_files = [
        ("tasks", include_str!("../db/tables/tasks.sql")),
    ];

    for (table_name, sql) in table_sql_files {
        match conn.execute_batch(sql) {
            Ok(_) => println!("Table '{}' initialized", table_name),
            Err(e) => {
                eprintln!("Failed to initialize table '{}': {}", table_name, e);
                return Err(DbError::Sqlite(e));
            }
        }
    }
    
    // Drop the lock before storing in app state
    drop(conn);
    
    // Store database in app state for global access
    app.manage(db);
    
    Ok(())
}

// Global insert function for any Insertable struct
pub fn insert<T: Insertable>(conn: &rusqlite::Connection, item: &T) -> rusqlite::Result<()> {
    let cols_vals = item.columns_values();
    let columns: Vec<&str> = cols_vals.iter().map(|(c, _)| *c).collect();
    let values: Vec<&dyn rusqlite::ToSql> = cols_vals.iter().map(|(_, v)| *v).collect();

    let cols_str = columns.join(", ");
    let placeholders = vec!["?"; columns.len()].join(", ");
    let sql = format!("INSERT INTO {} ({}) VALUES ({})", T::table_name(), cols_str, placeholders);

    conn.execute(&sql, &values[..]).map_err(|e| {
        eprintln!("Failed to insert into {}: {}", T::table_name(), e);
        eprintln!("SQL: {}", sql);
        e
    })?;
    
    Ok(())
}

// Query tasks by date range
pub fn query_tasks_by_date_range(
    conn: &rusqlite::Connection,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> rusqlite::Result<Vec<crate::structs::task_struct::Task>> {
    use crate::structs::task_struct::Task;
    
    let sql = include_str!("../db/sql/get_tasks_by_date.sql");
    
    let mut stmt = conn.prepare(sql)?;
    let task_iter = stmt.query_map([&start, &end], |row| Task::from_row(row))?;
    
    task_iter.collect()
}
