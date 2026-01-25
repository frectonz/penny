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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS acme_account (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                private_key_pem TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Gets the stored ACME account private key PEM if it exists.
    pub async fn get_acme_account(&self) -> color_eyre::Result<Option<String>> {
        let result: Option<(String,)> =
            sqlx::query_as(r#"SELECT private_key_pem FROM acme_account WHERE id = 1"#)
                .fetch_optional(&self.pool)
                .await?;

        Ok(result.map(|(pem,)| pem))
    }

    /// Saves the ACME account private key PEM.
    pub async fn save_acme_account(&self, private_key_pem: &str) -> color_eyre::Result<()> {
        let now = jiff::Timestamp::now().as_millisecond();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO acme_account (id, private_key_pem, created_at)
            VALUES (1, ?, ?)
            "#,
        )
        .bind(private_key_pem)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
