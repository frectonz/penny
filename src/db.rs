use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    pub(crate) pool: sqlx::SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> color_eyre::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = sqlx::SqlitePool::connect_with(options).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS runs (
                run_id TEXT PRIMARY KEY,
                host TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                stopped_at INTEGER,
                start_failed INTEGER NOT NULL DEFAULT 0,
                stop_failed INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stdout (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT NOT NULL,
                line TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stderr (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT NOT NULL,
                line TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}
