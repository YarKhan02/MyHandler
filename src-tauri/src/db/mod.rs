use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;
use uuid::Uuid;
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
    sql: &str,
) -> rusqlite::Result<Vec<crate::structs::task_struct::Task>> {
    use crate::structs::task_struct::Task;
    
    let mut stmt = conn.prepare(sql)?;
    let task_iter = stmt.query_map([&start, &end], |row| Task::from_row(row))?;
    
    task_iter.collect()
}

// Delete Task by ID
pub fn delete_task_by_id(
    conn: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<usize> {
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let sql = include_str!("../db/sql/delete_task_by_id.sql");
    
    let rows_affected = conn.execute(sql, [&uuid]).map_err(|e| {
        eprintln!("Failed to delete task with ID {}: {}", task_id, e);
        eprintln!("SQL: {}", sql);
        e
    })?;
    
    if rows_affected == 0 {
        eprintln!("Warning: No task found with ID {}", task_id);
    }
    
    Ok(rows_affected)
}

// Handles: start, pause, resume, complete transitions
pub fn update_task_status(
    conn: &rusqlite::Connection,
    task_id: &str,
    new_status: crate::structs::task_struct::Status,
) -> rusqlite::Result<crate::structs::task_struct::Task> {
    use crate::structs::task_struct::{Task, Status};
    
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let now = chrono::Utc::now();
    
    // Load SQL based on the status transition using include_str! macro
    let sql = match new_status {
        Status::Ongoing => include_str!("../db/sql/update_status_ongoing.sql"),
        Status::Paused => include_str!("../db/sql/update_status_paused.sql"),
        Status::Completed => include_str!("../db/sql/update_status_completed.sql"),
        Status::NotStarted => include_str!("../db/sql/update_status_not_started.sql"),
    };
    
    let rows_affected = if new_status == Status::NotStarted {
        conn.execute(sql, rusqlite::params![&new_status, &now, &uuid])
    } else {
        conn.execute(sql, rusqlite::params![&new_status, &now, &now, &uuid])
    }.map_err(|e| {
        eprintln!("Failed to update task status to {:?} for ID {}: {}", new_status, task_id, e);
        e
    })?;
    
    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    
    // Fetch and return the updated task
    let fetch_sql = include_str!("../db/sql/get_task_by_id.sql");
    
    conn.query_row(fetch_sql, [&uuid], |row| Task::from_row(row))
}