use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tracing::error;

use crate::db::SqliteDatabase;
use crate::types::{Host, RunId};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationParams {
    pub cursor: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<i64>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TotalOverview {
    pub total_runs: i64,
    pub total_awake_time_ms: i64,
    pub total_sleep_time_ms: i64,
    pub total_start_failures: i64,
    pub total_stop_failures: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AppOverview {
    pub host: String,
    pub total_runs: i64,
    pub total_awake_time_ms: i64,
    pub total_sleep_time_ms: i64,
    pub total_start_failures: i64,
    pub total_stop_failures: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AppRun {
    pub run_id: String,
    pub start_time_ms: i64,
    pub end_time_ms: i64,
    pub total_awake_time_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub line: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RunLogs {
    pub stdout: Vec<LogEntry>,
    pub stderr: Vec<LogEntry>,
}

#[async_trait::async_trait]
pub trait Reporter: Sync + Send + Clone + Debug + 'static {
    async fn total_overview(&self, time_range: Option<TimeRange>) -> TotalOverview;

    async fn apps_overview(&self, time_range: Option<TimeRange>) -> Vec<AppOverview>;

    async fn app_overview(&self, host: &Host, time_range: Option<TimeRange>)
    -> Option<AppOverview>;

    async fn app_runs(
        &self,
        host: &Host,
        time_range: Option<TimeRange>,
        pagination: PaginationParams,
    ) -> PaginatedResponse<AppRun>;

    async fn run_logs(&self, run_id: &RunId) -> Option<RunLogs>;
}

#[async_trait::async_trait]
impl Reporter for SqliteDatabase {
    async fn total_overview(&self, time_range: Option<TimeRange>) -> TotalOverview {
        let time_range = time_range.unwrap_or_default();

        let query = r#"
            WITH ordered_runs AS (
                SELECT
                    started_at,
                    stopped_at,
                    start_failed,
                    stop_failed,
                    LAG(stopped_at) OVER (ORDER BY started_at) as prev_stopped_at
                FROM runs
                WHERE ($1 IS NULL OR started_at >= $1)
                  AND ($2 IS NULL OR started_at <= $2)
            ),
            current_sleep AS (
                SELECT
                    CASE
                        WHEN NOT EXISTS (SELECT 1 FROM runs WHERE stopped_at IS NULL)
                        THEN CAST(strftime('%s', 'now') * 1000 AS INTEGER) -
                             (SELECT MAX(stopped_at) FROM runs)
                        ELSE 0
                    END as ongoing_sleep_ms
            )
            SELECT
                COUNT(*) as total_runs,
                COALESCE(SUM(CASE WHEN stopped_at IS NOT NULL THEN stopped_at - started_at ELSE 0 END), 0) as total_awake_time_ms,
                COALESCE(SUM(CASE WHEN prev_stopped_at IS NOT NULL AND started_at > prev_stopped_at THEN started_at - prev_stopped_at ELSE 0 END), 0)
                    + (SELECT ongoing_sleep_ms FROM current_sleep) as total_sleep_time_ms,
                COALESCE(SUM(start_failed), 0) as total_start_failures,
                COALESCE(SUM(stop_failed), 0) as total_stop_failures
            FROM ordered_runs
        "#;

        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(query)
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok((
                total_runs,
                total_awake_time_ms,
                total_sleep_time_ms,
                total_start_failures,
                total_stop_failures,
            )) => TotalOverview {
                total_runs,
                total_awake_time_ms,
                total_sleep_time_ms,
                total_start_failures,
                total_stop_failures,
            },
            Err(e) => {
                error!("failed to query total overview: {e}");
                TotalOverview::default()
            }
        }
    }

    async fn apps_overview(&self, time_range: Option<TimeRange>) -> Vec<AppOverview> {
        let time_range = time_range.unwrap_or_default();

        let query = r#"
            WITH ordered_runs AS (
                SELECT
                    host,
                    started_at,
                    stopped_at,
                    start_failed,
                    stop_failed,
                    LAG(stopped_at) OVER (PARTITION BY host ORDER BY started_at) as prev_stopped_at
                FROM runs
                WHERE ($1 IS NULL OR started_at >= $1)
                  AND ($2 IS NULL OR started_at <= $2)
            ),
            latest_per_host AS (
                SELECT
                    host,
                    MAX(stopped_at) as last_stopped_at,
                    MAX(CASE WHEN stopped_at IS NULL THEN 1 ELSE 0 END) as has_running
                FROM runs
                GROUP BY host
            ),
            current_sleep_per_host AS (
                SELECT
                    host,
                    CASE
                        WHEN has_running = 0 AND last_stopped_at IS NOT NULL
                        THEN CAST(strftime('%s', 'now') * 1000 AS INTEGER) - last_stopped_at
                        ELSE 0
                    END as ongoing_sleep_ms
                FROM latest_per_host
            )
            SELECT
                o.host,
                COUNT(*) as total_runs,
                COALESCE(SUM(CASE WHEN o.stopped_at IS NOT NULL THEN o.stopped_at - o.started_at ELSE 0 END), 0) as total_awake_time_ms,
                COALESCE(SUM(CASE WHEN o.prev_stopped_at IS NOT NULL AND o.started_at > o.prev_stopped_at THEN o.started_at - o.prev_stopped_at ELSE 0 END), 0)
                    + COALESCE((SELECT ongoing_sleep_ms FROM current_sleep_per_host WHERE host = o.host), 0) as total_sleep_time_ms,
                COALESCE(SUM(o.start_failed), 0) as total_start_failures,
                COALESCE(SUM(o.stop_failed), 0) as total_stop_failures
            FROM ordered_runs o
            GROUP BY o.host
            ORDER BY o.host
        "#;

        let rows = sqlx::query_as::<_, (String, i64, i64, i64, i64, i64)>(query)
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_all(&self.pool)
            .await;

        match rows {
            Ok(rows) => rows
                .into_iter()
                .map(
                    |(
                        host,
                        total_runs,
                        total_awake_time_ms,
                        total_sleep_time_ms,
                        total_start_failures,
                        total_stop_failures,
                    )| AppOverview {
                        host,
                        total_runs,
                        total_awake_time_ms,
                        total_sleep_time_ms,
                        total_start_failures,
                        total_stop_failures,
                    },
                )
                .collect(),
            Err(e) => {
                error!("failed to query apps overview: {e}");
                Vec::new()
            }
        }
    }

    async fn app_overview(
        &self,
        host: &Host,
        time_range: Option<TimeRange>,
    ) -> Option<AppOverview> {
        let time_range = time_range.unwrap_or_default();

        let query = r#"
            WITH ordered_runs AS (
                SELECT
                    host,
                    started_at,
                    stopped_at,
                    start_failed,
                    stop_failed,
                    LAG(stopped_at) OVER (ORDER BY started_at) as prev_stopped_at
                FROM runs
                WHERE host = $1
                  AND ($2 IS NULL OR started_at >= $2)
                  AND ($3 IS NULL OR started_at <= $3)
            ),
            latest_info AS (
                SELECT
                    MAX(stopped_at) as last_stopped_at,
                    MAX(CASE WHEN stopped_at IS NULL THEN 1 ELSE 0 END) as has_running
                FROM runs
                WHERE host = $1
            ),
            current_sleep AS (
                SELECT
                    CASE
                        WHEN has_running = 0 AND last_stopped_at IS NOT NULL
                        THEN CAST(strftime('%s', 'now') * 1000 AS INTEGER) - last_stopped_at
                        ELSE 0
                    END as ongoing_sleep_ms
                FROM latest_info
            )
            SELECT
                COUNT(*) as total_runs,
                COALESCE(SUM(CASE WHEN stopped_at IS NOT NULL THEN stopped_at - started_at ELSE 0 END), 0) as total_awake_time_ms,
                COALESCE(SUM(CASE WHEN prev_stopped_at IS NOT NULL AND started_at > prev_stopped_at THEN started_at - prev_stopped_at ELSE 0 END), 0)
                    + COALESCE((SELECT ongoing_sleep_ms FROM current_sleep), 0) as total_sleep_time_ms,
                COALESCE(SUM(start_failed), 0) as total_start_failures,
                COALESCE(SUM(stop_failed), 0) as total_stop_failures
            FROM ordered_runs
        "#;

        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(query)
            .bind(&host.0)
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_optional(&self.pool)
            .await;

        match row {
            Ok(Some((
                total_runs,
                total_awake_time_ms,
                total_sleep_time_ms,
                total_start_failures,
                total_stop_failures,
            ))) => {
                if total_runs == 0 {
                    return None;
                }
                Some(AppOverview {
                    host: host.0.clone(),
                    total_runs,
                    total_awake_time_ms,
                    total_sleep_time_ms,
                    total_start_failures,
                    total_stop_failures,
                })
            }
            Ok(None) => None,
            Err(e) => {
                error!("failed to query app overview: {e}");
                None
            }
        }
    }

    async fn app_runs(
        &self,
        host: &Host,
        time_range: Option<TimeRange>,
        pagination: PaginationParams,
    ) -> PaginatedResponse<AppRun> {
        let time_range = time_range.unwrap_or_default();
        let limit = pagination.limit.unwrap_or(20).min(100) as i64;
        let fetch_limit = limit + 1; // Fetch one extra to detect if more pages exist

        let query = r#"
            SELECT
                run_id,
                started_at,
                COALESCE(stopped_at, CAST(strftime('%s', 'now') * 1000 AS INTEGER)) as end_time,
                CASE
                    WHEN stopped_at IS NOT NULL THEN stopped_at - started_at
                    ELSE CAST(strftime('%s', 'now') * 1000 AS INTEGER) - started_at
                END as awake_time
            FROM runs
            WHERE host = $1
              AND ($2 IS NULL OR started_at >= $2)
              AND ($3 IS NULL OR started_at <= $3)
              AND ($4 IS NULL OR started_at < $4)
            ORDER BY started_at DESC
            LIMIT $5
        "#;

        let rows = sqlx::query_as::<_, (String, i64, i64, i64)>(query)
            .bind(&host.0)
            .bind(time_range.start)
            .bind(time_range.end)
            .bind(pagination.cursor)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await;

        match rows {
            Ok(mut rows) => {
                let has_more = rows.len() as i64 > limit;
                if has_more {
                    rows.pop(); // Remove the extra item used for detection
                }

                let next_cursor = if has_more {
                    rows.last().map(|(_, start_time_ms, _, _)| *start_time_ms)
                } else {
                    None
                };

                let items = rows
                    .into_iter()
                    .map(
                        |(run_id, start_time_ms, end_time_ms, total_awake_time_ms)| AppRun {
                            run_id,
                            start_time_ms,
                            end_time_ms,
                            total_awake_time_ms,
                        },
                    )
                    .collect();

                PaginatedResponse {
                    items,
                    next_cursor,
                    has_more,
                }
            }
            Err(e) => {
                error!("failed to query paginated app runs: {e}");
                PaginatedResponse {
                    items: Vec::new(),
                    next_cursor: None,
                    has_more: false,
                }
            }
        }
    }

    async fn run_logs(&self, run_id: &RunId) -> Option<RunLogs> {
        let exists_query = "SELECT 1 FROM runs WHERE run_id = $1";
        let exists = sqlx::query_scalar::<_, i32>(exists_query)
            .bind(&run_id.0)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
            .is_some();

        if !exists {
            return None;
        }

        let stdout_query = r#"
            SELECT line, timestamp
            FROM stdout
            WHERE run_id = $1
            ORDER BY timestamp ASC
        "#;

        let stderr_query = r#"
            SELECT line, timestamp
            FROM stderr
            WHERE run_id = $1
            ORDER BY timestamp ASC
        "#;

        let stdout = sqlx::query_as::<_, (String, i64)>(stdout_query)
            .bind(&run_id.0)
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(line, timestamp)| LogEntry { line, timestamp })
                    .collect()
            })
            .unwrap_or_else(|e| {
                error!("failed to query stdout logs: {e}");
                Vec::new()
            });

        let stderr = sqlx::query_as::<_, (String, i64)>(stderr_query)
            .bind(&run_id.0)
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(line, timestamp)| LogEntry { line, timestamp })
                    .collect()
            })
            .unwrap_or_else(|e| {
                error!("failed to query stderr logs: {e}");
                Vec::new()
            });

        Some(RunLogs { stdout, stderr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;

    async fn create_test_db() -> SqliteDatabase {
        SqliteDatabase::new("sqlite::memory:")
            .await
            .expect("failed to create in-memory database")
    }

    #[tokio::test]
    async fn total_overview_empty_database() {
        let db = create_test_db().await;

        let overview = db.total_overview(None).await;

        assert_eq!(overview.total_runs, 0);
        assert_eq!(overview.total_awake_time_ms, 0);
        assert_eq!(overview.total_start_failures, 0);
    }

    #[tokio::test]
    async fn total_overview_counts_runs_and_failures() {
        let db = create_test_db().await;
        let host1 = Host("app1.local".to_string());
        let host2 = Host("app2.local".to_string());

        db.app_started(&host1).await;
        db.app_stopped(&host1).await;

        db.app_started(&host2).await;
        db.app_stopped(&host2).await;

        db.app_started(&host1).await;
        db.app_start_failed(&host1).await;

        let overview = db.total_overview(None).await;

        assert_eq!(overview.total_runs, 3);
        assert_eq!(overview.total_start_failures, 1);
    }

    #[tokio::test]
    async fn apps_overview_groups_by_host() {
        let db = create_test_db().await;
        let host1 = Host("app1.local".to_string());
        let host2 = Host("app2.local".to_string());

        db.app_started(&host1).await;
        db.app_stopped(&host1).await;
        db.app_started(&host1).await;
        db.app_stopped(&host1).await;

        db.app_started(&host2).await;
        db.app_stopped(&host2).await;

        let overview = db.apps_overview(None).await;

        assert_eq!(overview.len(), 2);

        let app1 = overview.iter().find(|a| a.host == "app1.local").unwrap();
        assert_eq!(app1.total_runs, 2);

        let app2 = overview.iter().find(|a| a.host == "app2.local").unwrap();
        assert_eq!(app2.total_runs, 1);
    }

    #[tokio::test]
    async fn app_overview_returns_none_for_unknown_host() {
        let db = create_test_db().await;

        let overview = db
            .app_overview(&Host("unknown.local".to_string()), None)
            .await;

        assert!(overview.is_none());
    }

    #[tokio::test]
    async fn app_overview_returns_stats_for_host() {
        let db = create_test_db().await;
        let host = Host("myapp.local".to_string());
        let other = Host("other.local".to_string());

        db.app_started(&host).await;
        db.app_stopped(&host).await;

        db.app_started(&host).await;
        db.app_start_failed(&host).await;

        db.app_started(&other).await;
        db.app_stopped(&other).await;

        let overview = db.app_overview(&host, None).await;

        assert!(overview.is_some());
        let overview = overview.unwrap();
        assert_eq!(overview.host, "myapp.local");
        assert_eq!(overview.total_runs, 2);
        assert_eq!(overview.total_start_failures, 1);
    }

    #[tokio::test]
    async fn app_runs_returns_runs_for_host() {
        let db = create_test_db().await;
        let host = Host("myapp.local".to_string());

        let run_id1 = db.app_started(&host).await;
        db.app_stopped(&host).await;

        let run_id2 = db.app_started(&host).await;
        db.app_stopped(&host).await;

        let run_id3 = db.app_started(&host).await;
        db.app_stopped(&host).await;

        let response = db.app_runs(&host, None, PaginationParams::default()).await;

        assert_eq!(response.items.len(), 3);

        // Verify all run IDs are present
        let run_ids: Vec<&str> = response.items.iter().map(|r| r.run_id.as_str()).collect();
        assert!(run_ids.contains(&run_id1.0.as_str()));
        assert!(run_ids.contains(&run_id2.0.as_str()));
        assert!(run_ids.contains(&run_id3.0.as_str()));
    }

    #[tokio::test]
    async fn app_runs_filters_by_host() {
        let db = create_test_db().await;
        let host1 = Host("app1.local".to_string());
        let host2 = Host("app2.local".to_string());

        db.app_started(&host1).await;
        db.app_stopped(&host1).await;

        db.app_started(&host2).await;
        db.app_stopped(&host2).await;

        let response = db.app_runs(&host1, None, PaginationParams::default()).await;

        assert_eq!(response.items.len(), 1);
    }

    #[tokio::test]
    async fn run_logs_returns_none_for_unknown_run() {
        let db = create_test_db().await;

        let logs = db
            .run_logs(&RunId::from_string("nonexistent".to_string()))
            .await;

        assert!(logs.is_none());
    }

    #[tokio::test]
    async fn run_logs_returns_stdout_and_stderr() {
        let db = create_test_db().await;
        let host = Host("test.local".to_string());

        let run_id = db.app_started(&host).await;
        db.append_stdout(&run_id, "stdout line 1".to_string()).await;
        db.append_stdout(&run_id, "stdout line 2".to_string()).await;
        db.append_stderr(&run_id, "stderr line 1".to_string()).await;

        let logs = db.run_logs(&run_id).await;

        assert!(logs.is_some());
        let logs = logs.unwrap();
        assert_eq!(logs.stdout.len(), 2);
        assert_eq!(logs.stderr.len(), 1);
        assert_eq!(logs.stdout[0].line, "stdout line 1");
        assert_eq!(logs.stdout[1].line, "stdout line 2");
        assert_eq!(logs.stderr[0].line, "stderr line 1");
    }

    #[tokio::test]
    async fn run_logs_returns_empty_logs_for_run_without_output() {
        let db = create_test_db().await;
        let host = Host("test.local".to_string());

        let run_id = db.app_started(&host).await;

        let logs = db.run_logs(&run_id).await;

        assert!(logs.is_some());
        let logs = logs.unwrap();
        assert!(logs.stdout.is_empty());
        assert!(logs.stderr.is_empty());
    }

    #[tokio::test]
    async fn app_runs_returns_limited_results() {
        let db = create_test_db().await;
        let host = Host("myapp.local".to_string());

        // Create 5 runs
        for _ in 0..5 {
            db.app_started(&host).await;
            db.app_stopped(&host).await;
        }

        let pagination = PaginationParams {
            cursor: None,
            limit: Some(3),
        };
        let response = db.app_runs(&host, None, pagination).await;

        assert_eq!(response.items.len(), 3);
        assert!(response.has_more);
        assert!(response.next_cursor.is_some());
    }

    #[tokio::test]
    async fn app_runs_cursor_returns_next_page() {
        let db = create_test_db().await;
        let host = Host("myapp.local".to_string());

        // Create 5 runs
        for _ in 0..5 {
            db.app_started(&host).await;
            db.app_stopped(&host).await;
        }

        // Get first page
        let pagination = PaginationParams {
            cursor: None,
            limit: Some(3),
        };
        let first_page = db.app_runs(&host, None, pagination).await;
        assert_eq!(first_page.items.len(), 3);
        assert!(first_page.has_more);

        // Get second page using cursor
        let pagination = PaginationParams {
            cursor: first_page.next_cursor,
            limit: Some(3),
        };
        let second_page = db.app_runs(&host, None, pagination).await;
        assert_eq!(second_page.items.len(), 2);
        assert!(!second_page.has_more);
        assert!(second_page.next_cursor.is_none());

        // Ensure no overlap between pages
        let first_ids: Vec<&str> = first_page.items.iter().map(|r| r.run_id.as_str()).collect();
        for run in &second_page.items {
            assert!(!first_ids.contains(&run.run_id.as_str()));
        }
    }

    #[tokio::test]
    async fn app_runs_empty_result() {
        let db = create_test_db().await;
        let host = Host("unknown.local".to_string());

        let pagination = PaginationParams::default();
        let response = db.app_runs(&host, None, pagination).await;

        assert!(response.items.is_empty());
        assert!(!response.has_more);
        assert!(response.next_cursor.is_none());
    }
}
