use std::fmt::Debug;

use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use tracing::error;

use crate::db::SqliteDatabase;
use crate::types::{Host, RunId};

#[async_trait::async_trait]
pub trait Collector: Sync + Send + Clone + Debug + 'static {
    async fn app_started(&self, host: &Host) -> RunId;
    async fn app_stopped(&self, host: &Host);

    async fn app_start_failed(&self, host: &Host);
    async fn app_stop_failed(&self, host: &Host);

    async fn append_stdout(&self, run_id: &RunId, line: String);
    async fn append_stderr(&self, run_id: &RunId, line: String);
}

#[async_trait::async_trait]
impl Collector for SqliteDatabase {
    async fn app_started(&self, host: &Host) -> RunId {
        let run_id = RunId::new();
        let started_at = Zoned::new(Timestamp::now(), TimeZone::UTC)
            .timestamp()
            .as_millisecond();

        if let Err(e) = sqlx::query("INSERT INTO runs (run_id, host, started_at) VALUES (?, ?, ?)")
            .bind(&run_id.0)
            .bind(&host.0)
            .bind(started_at)
            .execute(&self.pool)
            .await
        {
            error!("failed to insert run record: {e}");
        }

        run_id
    }

    async fn app_stopped(&self, host: &Host) {
        let stopped_at = Zoned::new(Timestamp::now(), TimeZone::UTC)
            .timestamp()
            .as_millisecond();

        if let Err(e) = sqlx::query(
            "UPDATE runs SET stopped_at = ? WHERE run_id = (SELECT run_id FROM runs WHERE host = ? AND stopped_at IS NULL ORDER BY started_at DESC LIMIT 1)",
        )
        .bind(stopped_at)
        .bind(&host.0)
        .execute(&self.pool)
        .await
        {
            error!("failed to update run record: {e}");
        }
    }

    async fn app_start_failed(&self, host: &Host) {
        if let Err(e) = sqlx::query(
            "UPDATE runs SET start_failed = 1 WHERE run_id = (SELECT run_id FROM runs WHERE host = ? AND stopped_at IS NULL ORDER BY started_at DESC LIMIT 1)",
        )
        .bind(&host.0)
        .execute(&self.pool)
        .await
        {
            error!("failed to update run record: {e}");
        }
    }

    async fn app_stop_failed(&self, host: &Host) {
        if let Err(e) = sqlx::query(
            "UPDATE runs SET stop_failed = 1 WHERE run_id = (SELECT run_id FROM runs WHERE host = ? AND stopped_at IS NULL ORDER BY started_at DESC LIMIT 1)",
        )
        .bind(&host.0)
        .execute(&self.pool)
        .await
        {
            error!("failed to update run record: {e}");
        }
    }

    async fn append_stdout(&self, run_id: &RunId, line: String) {
        let timestamp = Zoned::new(Timestamp::now(), TimeZone::UTC)
            .timestamp()
            .as_millisecond();

        if let Err(e) = sqlx::query("INSERT INTO stdout (run_id, line, timestamp) VALUES (?, ?, ?)")
            .bind(&run_id.0)
            .bind(&line)
            .bind(timestamp)
            .execute(&self.pool)
            .await
        {
            error!("failed to insert stdout line: {e}");
        }
    }

    async fn append_stderr(&self, run_id: &RunId, line: String) {
        let timestamp = Zoned::new(Timestamp::now(), TimeZone::UTC)
            .timestamp()
            .as_millisecond();

        if let Err(e) = sqlx::query("INSERT INTO stderr (run_id, line, timestamp) VALUES (?, ?, ?)")
            .bind(&run_id.0)
            .bind(&line)
            .bind(timestamp)
            .execute(&self.pool)
            .await
        {
            error!("failed to insert stderr line: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::Reporter;

    async fn create_test_db() -> SqliteDatabase {
        SqliteDatabase::new("sqlite::memory:")
            .await
            .expect("failed to create in-memory database")
    }

    #[tokio::test]
    async fn app_started_creates_run_record() {
        let db = create_test_db().await;
        let host = Host("test-app.local".to_string());

        let run_id = db.app_started(&host).await;

        // Verify via reporter that the run exists
        let response = db
            .app_runs(&host, None, crate::reporter::PaginationParams::default())
            .await;
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].run_id, run_id.0);
    }

    #[tokio::test]
    async fn app_stopped_updates_run_record() {
        let db = create_test_db().await;
        let host = Host("test-app.local".to_string());

        db.app_started(&host).await;
        db.app_stopped(&host).await;

        // Verify via reporter - a stopped run should have awake time > 0
        let overview = db.app_overview(&host, None).await.unwrap();
        assert_eq!(overview.total_runs, 1);
    }

    #[tokio::test]
    async fn app_start_failed_sets_flag() {
        let db = create_test_db().await;
        let host = Host("test-app.local".to_string());

        db.app_started(&host).await;
        db.app_start_failed(&host).await;

        let overview = db.app_overview(&host, None).await.unwrap();
        assert_eq!(overview.total_start_failures, 1);
    }

    #[tokio::test]
    async fn append_stdout_captured_in_logs() {
        let db = create_test_db().await;
        let host = Host("test-app.local".to_string());

        let run_id = db.app_started(&host).await;
        db.append_stdout(&run_id, "Hello from stdout".to_string())
            .await;
        db.append_stdout(&run_id, "Another line".to_string()).await;

        let logs = db.run_logs(&run_id).await.unwrap();
        assert_eq!(logs.stdout.len(), 2);
        assert_eq!(logs.stdout[0].line, "Hello from stdout");
        assert_eq!(logs.stdout[1].line, "Another line");
    }

    #[tokio::test]
    async fn append_stderr_captured_in_logs() {
        let db = create_test_db().await;
        let host = Host("test-app.local".to_string());

        let run_id = db.app_started(&host).await;
        db.append_stderr(&run_id, "Error occurred".to_string())
            .await;
        db.append_stderr(&run_id, "Stack trace here".to_string())
            .await;

        let logs = db.run_logs(&run_id).await.unwrap();
        assert_eq!(logs.stderr.len(), 2);
        assert_eq!(logs.stderr[0].line, "Error occurred");
        assert_eq!(logs.stderr[1].line, "Stack trace here");
    }

    #[tokio::test]
    async fn multiple_hosts_tracked_separately() {
        let db = create_test_db().await;
        let host1 = Host("app1.local".to_string());
        let host2 = Host("app2.local".to_string());

        db.app_started(&host1).await;
        db.app_started(&host2).await;
        db.app_stopped(&host1).await;

        let apps = db.apps_overview(None).await;
        assert_eq!(apps.len(), 2);

        let app1 = apps.iter().find(|a| a.host == "app1.local").unwrap();
        let app2 = apps.iter().find(|a| a.host == "app2.local").unwrap();

        assert_eq!(app1.total_runs, 1);
        assert_eq!(app2.total_runs, 1);
    }
}
