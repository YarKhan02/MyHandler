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

// Trait for types that can update database records
pub trait Updatable {
    fn table_name() -> &'static str;
    fn update_columns_values(&self) -> Vec<(&'static str, &dyn rusqlite::ToSql)>;
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
        println!("Attempting to acquire database lock...");
        match self.conn.lock() {
            Ok(guard) => {
                println!("Database lock acquired successfully");
                guard
            }
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
        ("settings", include_str!("../db/tables/settings.sql")),
        ("calendar_credentials", include_str!("../db/tables/calendar_credentials.sql")),
        ("calendar_events", include_str!("../db/tables/calendar_events.sql")),
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

// Get a single task by ID
pub fn get_task_by_id(
    conn: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<crate::structs::task_struct::Task> {
    use crate::structs::task_struct::Task;
    
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let sql = include_str!("../db/sql/get_task_by_id.sql");
    
    conn.query_row(sql, [&uuid], |row| Task::from_row(row))
}

// Update task fields
pub fn update_task<T: Updatable>(
    conn: &rusqlite::Connection,
    task_id: &str,
    update_data: &T,
) -> rusqlite::Result<crate::structs::task_struct::Task> {
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let cols_vals = update_data.update_columns_values();
    
    if cols_vals.is_empty() {
        // No fields to update, just return the current task
        return get_task_by_id(conn, task_id);
    }
    
    let set_clauses: Vec<String> = cols_vals.iter()
        .map(|(col, _)| format!("{} = ?", col))
        .collect();
    let values: Vec<&dyn rusqlite::ToSql> = cols_vals.iter()
        .map(|(_, v)| *v)
        .collect();
    
    let sql = format!(
        "UPDATE {} SET {} WHERE id = ?",
        T::table_name(),
        set_clauses.join(", ")
    );
    
    let mut params = values;
    params.push(&uuid);
    
    let rows_affected = conn.execute(&sql, &params[..]).map_err(|e| {
        eprintln!("Failed to update task with ID {}: {}", task_id, e);
        eprintln!("SQL: {}", sql);
        e
    })?;
    
    if rows_affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        get_task_by_id(conn, task_id)
    }
}

// Handles: start, pause, resume, complete transitions
pub fn update_task_status(
    conn: &rusqlite::Connection,
    task_id: &str,
    new_status: crate::structs::task_struct::Status,
) -> rusqlite::Result<crate::structs::task_struct::Task> {
    use crate::structs::task_struct::Status;
    
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
    get_task_by_id(conn, task_id)
}

// Get settings from database
pub fn get_settings(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<crate::structs::settings::Settings> {
    use crate::structs::settings::Settings;
    
    let sql = include_str!("../db/sql/get_settings.sql");
    
    conn.query_row(sql, [], |row| Settings::from_row(row))
}

// Update settings in database
pub fn update_settings<T: Updatable>(
    conn: &rusqlite::Connection,
    update_data: &T,
) -> rusqlite::Result<crate::structs::settings::Settings> {
    let cols_vals = update_data.update_columns_values();
    
    if cols_vals.is_empty() {
        // No fields to update, just return current settings
        return get_settings(conn);
    }
    
    let set_clauses: Vec<String> = cols_vals.iter()
        .map(|(col, _)| format!("{} = ?", col))
        .collect();
    let values: Vec<&dyn rusqlite::ToSql> = cols_vals.iter()
        .map(|(_, v)| *v)
        .collect();
    
    let now = chrono::Utc::now();
    let sql = format!(
        "UPDATE {} SET {}, updated_at = ? WHERE id = 1",
        T::table_name(),
        set_clauses.join(", ")
    );
    
    let mut params = values;
    params.push(&now);
    
    let rows_affected = conn.execute(&sql, &params[..]).map_err(|e| {
        eprintln!("Failed to update settings: {}", e);
        eprintln!("SQL: {}", sql);
        e
    })?;
    
    if rows_affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        get_settings(conn)
    }
}

// Save calendar credentials to database
pub fn save_calendar_credentials(
    conn: &rusqlite::Connection,
    creds: &crate::structs::calendar::CalendarCredentials,
) -> rusqlite::Result<()> {
    let sql = include_str!("../db/sql/save_calendar_credentials.sql");
    
    conn.execute(
        sql,
        rusqlite::params![
            &creds.email,
            &creds.access_token,
            &creds.refresh_token,
            &creds.token_expiry,
        ],
    )?;
    
    // Enable calendar integration in settings
    let enable_sql = include_str!("../db/sql/enable_calendar_integration.sql");
    conn.execute(enable_sql, rusqlite::params![&creds.email])?;
    
    Ok(())
}

// Get calendar credentials from database
pub fn get_calendar_credentials(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<Option<crate::structs::calendar::CalendarCredentials>> {
    use crate::structs::calendar::CalendarCredentials;
    
    println!("get_calendar_credentials: Loading SQL...");
    let sql = include_str!("../db/sql/get_calendar_credentials.sql");
    println!("get_calendar_credentials: SQL loaded, executing query...");
    
    let result = conn.query_row(sql, [], |row| {
        println!("get_calendar_credentials: Processing row...");
        let email: String = row.get(0)?;
        let access_token: String = row.get(1)?;
        let refresh_token: String = row.get(2)?;
        let token_expiry: chrono::DateTime<chrono::Utc> = row.get(3)?;
        
        println!("get_calendar_credentials: Row data retrieved");
        
        // Check if credentials are actually set (not empty placeholder)
        if email.is_empty() || access_token.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        Ok(CalendarCredentials {
            email,
            access_token,
            refresh_token,
            token_expiry,
        })
    });
    
    println!("get_calendar_credentials: Query executed, processing result...");
    
    match result {
        Ok(creds) => {
            println!("get_calendar_credentials: Credentials found");
            Ok(Some(creds))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            println!("get_calendar_credentials: No credentials found (empty table)");
            Ok(None)
        }
        Err(e) => {
            eprintln!("get_calendar_credentials: Error occurred: {}", e);
            Err(e)
        }
    }
}

// Clear calendar credentials from database
pub fn clear_calendar_credentials(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let sql = include_str!("../db/sql/clear_calendar_credentials.sql");
    conn.execute(sql, [])?;
    
    // Disable calendar integration in settings
    let disable_sql = include_str!("../db/sql/disable_calendar_integration.sql");
    conn.execute(disable_sql, [])?;
    
    // Clear all calendar events
    clear_all_calendar_events(conn)?;
    
    Ok(())
}
// Update google_event_id for a task
pub fn update_task_google_event_id(
    conn: &rusqlite::Connection,
    task_id: &str,
    event_id: &str,
) -> rusqlite::Result<()> {
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let sql = include_str!("../db/sql/upsert_calendar_event.sql");
    conn.execute(sql, rusqlite::params![&uuid, event_id])?;
    
    Ok(())
}

// Clear google_event_id for a task
pub fn clear_task_google_event_id(
    conn: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<()> {
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let sql = include_str!("../db/sql/delete_calendar_event_by_task.sql");
    conn.execute(sql, rusqlite::params![&uuid])?;
    
    Ok(())
}

// Get google_event_id for a task
pub fn get_task_google_event_id(
    conn: &rusqlite::Connection,
    task_id: &str,
) -> rusqlite::Result<Option<String>> {
    let uuid = Uuid::parse_str(task_id)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid UUID: {}", e)))?;
    
    let sql = include_str!("../db/sql/get_calendar_event_by_task.sql");
    let result = conn.query_row(sql, rusqlite::params![&uuid], |row| row.get(0));
    
    match result {
        Ok(event_id) => Ok(Some(event_id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// Clear all calendar events (when disconnecting calendar)
pub fn clear_all_calendar_events(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<()> {
    let sql = include_str!("../db/sql/clear_all_calendar_events.sql");
    conn.execute(sql, [])?;
    Ok(())
}
