//! Database migrations for session storage

use sqlx::{Any, Pool, Postgres, Sqlite};

/// SQL for creating the sessions table
const CREATE_SESSIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    app_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    id TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT '{}',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (app_name, user_id, id)
);
"#;

/// SQL for creating the events table
const CREATE_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    invocation_id TEXT NOT NULL,
    author TEXT NOT NULL,
    actions TEXT NOT NULL DEFAULT '{}',
    long_running_tool_ids TEXT,
    branch TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content TEXT,
    grounding_metadata TEXT,
    custom_metadata TEXT,
    usage_metadata TEXT,
    citation_metadata TEXT,
    partial INTEGER,
    turn_complete INTEGER,
    error_code TEXT,
    error_message TEXT,
    interrupted INTEGER,
    PRIMARY KEY (id, app_name, user_id, session_id),
    FOREIGN KEY (app_name, user_id, session_id) REFERENCES sessions(app_name, user_id, id) ON DELETE CASCADE
);
"#;

/// SQL for creating the app_states table
const CREATE_APP_STATES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS app_states (
    app_name TEXT NOT NULL PRIMARY KEY,
    state TEXT NOT NULL DEFAULT '{}',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

/// SQL for creating the user_states table
const CREATE_USER_STATES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS user_states (
    app_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT '{}',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (app_name, user_id)
);
"#;

/// Run migrations for PostgreSQL
pub async fn run_postgres_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(CREATE_SESSIONS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_EVENTS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_APP_STATES_TABLE).execute(pool).await?;
    sqlx::query(CREATE_USER_STATES_TABLE).execute(pool).await?;
    Ok(())
}

/// Run migrations for SQLite
pub async fn run_sqlite_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(CREATE_SESSIONS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_EVENTS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_APP_STATES_TABLE).execute(pool).await?;
    sqlx::query(CREATE_USER_STATES_TABLE).execute(pool).await?;
    Ok(())
}

/// Run migrations for the database (auto-detects database type)
pub async fn run_migrations(pool: &Pool<Any>) -> Result<(), sqlx::Error> {
    sqlx::query(CREATE_SESSIONS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_EVENTS_TABLE).execute(pool).await?;
    sqlx::query(CREATE_APP_STATES_TABLE).execute(pool).await?;
    sqlx::query(CREATE_USER_STATES_TABLE).execute(pool).await?;
    Ok(())
}

/// Create an index on events for faster queries
pub async fn create_events_index_postgres(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_session ON events(app_name, user_id, session_id, timestamp)")
        .execute(pool)
        .await?;
    Ok(())
}

/// Create an index on events for faster queries (SQLite)
pub async fn create_events_index_sqlite(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_session ON events(app_name, user_id, session_id, timestamp)")
        .execute(pool)
        .await?;
    Ok(())
}
