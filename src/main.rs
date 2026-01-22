use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp, Zoned};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, instrument, warn};
use ulid::Ulid;

#[derive(rust_embed::RustEmbed)]
#[folder = "ui/dist"]
struct UiAssets;

static HTTP: once_cell::sync::Lazy<reqwest::Client> =
    once_cell::sync::Lazy::new(reqwest::Client::new);

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the reverse proxy.
    Serve {
        /// Path to the config file.
        config: String,

        /// The address to bind to.
        #[arg(short, long, default_value = "127.0.0.1:3030")]
        address: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    pub address: SocketAddr,
    pub health_check: String,
    pub command: AppCommand,

    #[serde(default = "default_wait_period")]
    pub wait_period: SignedDuration,
    #[serde(default = "default_start_timeout")]
    pub start_timeout: SignedDuration,
    #[serde(default = "default_stop_timeout")]
    pub stop_timeout: SignedDuration,

    #[serde(skip)]
    pub kill_task: Option<tokio::task::JoinHandle<()>>,
}

fn default_wait_period() -> SignedDuration {
    SignedDuration::from_mins(10)
}

fn default_start_timeout() -> SignedDuration {
    SignedDuration::from_secs(30)
}

fn default_stop_timeout() -> SignedDuration {
    SignedDuration::from_secs(30)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AppCommand {
    Start(Box<CommandSpec>),
    StartEnd {
        start: Box<CommandSpec>,
        end: Box<CommandSpec>,
    },
}

#[derive(Debug)]
pub struct CommandSpec {
    program: String,
    args: Vec<String>,

    collect_stdout: Option<tokio::task::JoinHandle<()>>,
    collect_stderr: Option<tokio::task::JoinHandle<()>>,

    child: Option<tokio::process::Child>,
}

impl Serialize for CommandSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let program = &self.program;
        let args = shell_words::join(self.args.as_slice());
        let command = format!("{program} {args}");

        serializer.serialize_str(&command)
    }
}

impl FromStr for CommandSpec {
    type Err = shell_words::ParseError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let mut words = shell_words::split(command)?;

        words.reverse();

        let program = words.pop().unwrap_or_else(|| command.to_owned());

        words.reverse();

        Ok(Self {
            program,
            args: words,
            collect_stdout: None,
            collect_stderr: None,
            child: None,
        })
    }
}

impl<'de> Deserialize<'de> for CommandSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let command = String::deserialize(deserializer)?;
        CommandSpec::from_str(&command).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
struct RunOptions<C: Collector> {
    run_id: RunId,
    collector: C,
}

impl<C: Collector> RunOptions<C> {
    async fn append_stdout(&self, line: String) {
        self.collector.append_stdout(&self.run_id, line).await;
    }

    async fn append_stderr(&self, line: String) {
        self.collector.append_stderr(&self.run_id, line).await;
    }
}

impl CommandSpec {
    fn is_child_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            },
            None => false,
        }
    }

    #[instrument(skip(self), fields(program = %self.program))]
    fn run<C: Collector>(&mut self, opts: Option<RunOptions<C>>) {
        let should_spawn = match self.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(exit)) => {
                    warn!("child process exited with code {exit}, need to spawn new one");
                    true
                }
                Ok(None) => {
                    debug!("child process already exists, skipping spawn");
                    false
                }
                Err(err) => {
                    error!("failed to wait for child process: {err}");
                    true
                }
            },
            None => true,
        };

        if !should_spawn {
            return;
        };

        info!(args = ?self.args, "spawning command");
        match tokio::process::Command::new(&self.program)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(opts) = opts {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout).lines();

                        let opts = opts.clone();
                        self.collect_stdout = Some(tokio::task::spawn(async move {
                            while let Ok(Some(line)) = reader.next_line().await {
                                opts.append_stdout(line).await;
                            }
                        }));
                    }

                    if let Some(stderr) = child.stderr.take() {
                        let mut reader = BufReader::new(stderr).lines();

                        let opts = opts.clone();
                        self.collect_stderr = Some(tokio::task::spawn(async move {
                            while let Ok(Some(line)) = reader.next_line().await {
                                opts.append_stderr(line).await;
                            }
                        }));
                    }
                }

                self.child = Some(child);
                debug!("command spawned successfully");
            }
            Err(err) => {
                error!("failed to spawn command: {err}");
            }
        };
    }

    #[instrument(skip(self), fields(program = %self.program))]
    async fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            info!("killing process");

            match child.kill().await {
                Ok(()) => {
                    debug!("process killed successfully");
                }
                Err(err) => {
                    error!("failed to kill process: {err}");
                }
            };
        } else {
            debug!("no child process to kill");
        }

        if let Some(stdout) = self.collect_stdout.take() {
            stdout.abort();
        }

        if let Some(stderr) = self.collect_stderr.take() {
            stderr.abort();
        }
    }
}

impl AppCommand {
    fn is_child_running(&mut self) -> bool {
        match self {
            AppCommand::Start(start) => start.is_child_running(),
            AppCommand::StartEnd { start, .. } => start.is_child_running(),
        }
    }

    #[instrument(skip(self))]
    fn start<C: Collector>(&mut self, opts: Option<RunOptions<C>>) {
        debug!("starting app command");
        let start = match self {
            AppCommand::Start(start) => start.as_mut(),
            AppCommand::StartEnd { start, end: _ } => start.as_mut(),
        };

        start.run(opts);
    }

    #[instrument(skip(self))]
    async fn stop(&mut self) {
        debug!("stopping app command");
        match self {
            AppCommand::Start(start) => start.kill().await,
            AppCommand::StartEnd { start, end } => {
                start.kill().await;
                end.run::<SqliteDatabase>(None)
            }
        };
    }
}

impl App {
    #[instrument(skip(self), fields(address = %self.address, health_check = %self.health_check))]
    async fn is_running(&self) -> bool {
        let address = self.address;
        let health_check_path = self.health_check.as_str();

        let health_check_url = format!("http://{address}{health_check_path}");

        debug!(url = %health_check_url, "performing health check");

        let resp = HTTP
            .get(&health_check_url)
            .send()
            .await
            .ok()
            .map(|r| r.status())
            .unwrap_or_else(|| http::StatusCode::SERVICE_UNAVAILABLE);

        let is_ok = resp == http::StatusCode::OK;
        debug!(status = %resp, is_running = is_ok, "health check result");

        is_ok
    }

    #[instrument(skip(self), fields(timeout = ?self.start_timeout))]
    async fn wait_for_running(&self) -> Result<(), pingora::time::Elapsed> {
        let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(10)
            .max_delay(Duration::from_secs(2))
            .map(tokio_retry::strategy::jitter);

        debug!("waiting for app to become ready");
        let wait_for_running = tokio_retry::Retry::spawn(strategy, async || -> Result<(), ()> {
            if self.is_running().await {
                Ok(())
            } else {
                Err(())
            }
        });

        let result = pingora::time::timeout(self.start_timeout.unsigned_abs(), wait_for_running)
            .await
            .map(|_| ());
        if result.is_ok() {
            info!("app is now running");
        } else {
            warn!("timed out waiting for app to start");
        }

        result
    }

    #[instrument(skip(self), fields(timeout = ?self.start_timeout))]
    async fn wait_for_stopped(&self) -> Result<(), pingora::time::Elapsed> {
        let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(10)
            .max_delay(Duration::from_secs(2))
            .map(tokio_retry::strategy::jitter);

        debug!("waiting for app to stop");
        let wait_for_stopping = tokio_retry::Retry::spawn(strategy, async || -> Result<(), ()> {
            if self.is_running().await {
                Err(())
            } else {
                Ok(())
            }
        });

        let result = pingora::time::timeout(self.stop_timeout.unsigned_abs(), wait_for_stopping)
            .await
            .map(|_| ());
        if result.is_ok() {
            info!("app is now stopped");
        } else {
            warn!("timed out waiting for app to stop");
        }

        result
    }

    #[instrument(skip(app))]
    async fn start_app(
        host: &Host,
        app: &Arc<RwLock<App>>,
        collector: impl Collector,
    ) -> pingora::Result<()> {
        // Fast path: if child process is already running, skip health check
        if app.write().await.command.is_child_running() {
            debug!("child process already running, skipping health check");
            return Ok(());
        }

        // Slow path: no running child, do health check to confirm app state
        if !app.read().await.is_running().await {
            let address = app.read().await.address;
            let run_id = collector.app_started(host).await;

            info!(%address, "app not running, starting it");
            app.write().await.command.start(Some(RunOptions {
                run_id,
                collector: collector.clone(),
            }));

            if app.read().await.wait_for_running().await.is_err() {
                error!("failed to start app within timeout");
                collector.app_start_failed(host).await;
                return Err(pingora::Error::explain(
                    pingora::ErrorType::ConnectError,
                    "failed to start app",
                ));
            }
        } else {
            let address = app.read().await.address;
            debug!(%address, "app already running");
        }

        Ok(())
    }

    #[instrument(skip(app))]
    async fn schedule_kill(host: &Host, app: &Arc<RwLock<App>>, collector: impl Collector) {
        let mut app_guard = app.write().await;

        if let Some(task) = app_guard.kill_task.take() {
            debug!("aborting previous kill task");
            task.abort();
        }

        let wait_period = app_guard.wait_period;
        info!(wait_period = ?wait_period, "scheduling app shutdown");

        let handle = {
            let app = app.clone();
            let host = host.clone();
            tokio::spawn(async move {
                let wait_period = app.read().await.wait_period.unsigned_abs();
                pingora::time::sleep(wait_period).await;
                info!("wait period elapsed, stopping app");

                app.write().await.command.stop().await;
                collector.app_stopped(&host).await;

                if app.read().await.wait_for_stopped().await.is_err() {
                    error!("failed to stop app within timeout");
                    collector.app_stop_failed(&host).await;
                }
            })
        };

        app_guard.kill_task = Some(handle);
    }
}

fn deserialize_apps<'de, D>(deserializer: D) -> Result<HashMap<String, Arc<RwLock<App>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = HashMap::<String, App>::deserialize(deserializer)?;

    Ok(raw
        .into_iter()
        .map(|(k, v)| (k, Arc::new(RwLock::new(v))))
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_address: Option<SocketAddr>,

    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(flatten, deserialize_with = "deserialize_apps")]
    pub apps: HashMap<String, Arc<RwLock<App>>>,
}

fn default_database_url() -> String {
    "sqlite://penny.db".to_owned()
}

impl Config {
    async fn get_proxy_context(&self, host: &str) -> Option<ProxyContext> {
        let app = self.apps.get(host)?.clone();
        let proxy_context = ProxyContext::new(host, app).await;
        Some(proxy_context)
    }
}

struct YarpProxy<C> {
    config: Config,
    collector: C,
}

impl<C> YarpProxy<C>
where
    C: Collector,
{
    fn new(config: Config, collector: C) -> Self {
        Self { config, collector }
    }
}

fn get_host(session: &pingora::prelude::Session) -> Option<&str> {
    session
        .get_header(http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|host| host.split(':').next())
        .or(session.req_header().uri.host())
}

#[derive(Debug, Clone)]
pub struct Host(pub String);

#[derive(Debug, Clone)]
pub struct RunId(String);

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl RunId {
    fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone)]
struct SqliteDatabase {
    pool: sqlx::SqlitePool,
}

impl SqliteDatabase {
    async fn new(database_url: &str) -> color_eyre::Result<Self> {
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

#[async_trait::async_trait]
trait Collector: Sync + Send + Clone + Debug + 'static {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl TimeRange {
    pub fn new(start: Option<i64>, end: Option<i64>) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TotalOverview {
    pub total_runs: i64,
    pub total_awake_time_ms: i64,
    pub total_sleep_time_ms: i64,
    pub total_start_failures: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AppOverview {
    pub host: String,
    pub total_runs: i64,
    pub total_awake_time_ms: i64,
    pub total_sleep_time_ms: i64,
    pub total_start_failures: i64,
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

    async fn app_runs(&self, host: &Host, time_range: Option<TimeRange>) -> Vec<AppRun>;

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
                COALESCE(SUM(start_failed), 0) as total_start_failures
            FROM ordered_runs
        "#;

        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(query)
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok((total_runs, total_awake_time_ms, total_sleep_time_ms, total_start_failures)) => {
                TotalOverview {
                    total_runs,
                    total_awake_time_ms,
                    total_sleep_time_ms,
                    total_start_failures,
                }
            }
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
                COALESCE(SUM(o.start_failed), 0) as total_start_failures
            FROM ordered_runs o
            GROUP BY o.host
            ORDER BY o.host
        "#;

        let rows = sqlx::query_as::<_, (String, i64, i64, i64, i64)>(query)
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
                    )| AppOverview {
                        host,
                        total_runs,
                        total_awake_time_ms,
                        total_sleep_time_ms,
                        total_start_failures,
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
                COALESCE(SUM(start_failed), 0) as total_start_failures
            FROM ordered_runs
        "#;

        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(query)
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
                })
            }
            Ok(None) => None,
            Err(e) => {
                error!("failed to query app overview: {e}");
                None
            }
        }
    }

    async fn app_runs(&self, host: &Host, time_range: Option<TimeRange>) -> Vec<AppRun> {
        let time_range = time_range.unwrap_or_default();

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
            ORDER BY started_at DESC
        "#;

        let rows = sqlx::query_as::<_, (String, i64, i64, i64)>(query)
            .bind(&host.0)
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_all(&self.pool)
            .await;

        match rows {
            Ok(rows) => rows
                .into_iter()
                .map(
                    |(run_id, start_time_ms, end_time_ms, total_awake_time_ms)| AppRun {
                        run_id,
                        start_time_ms,
                        end_time_ms,
                        total_awake_time_ms,
                    },
                )
                .collect(),
            Err(e) => {
                error!("failed to query app runs: {e}");
                Vec::new()
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

#[derive(Debug, Clone, Serialize)]
struct VersionResponse {
    version: &'static str,
}

async fn version_handler() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn static_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file first
    if let Some(content) = UiAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
            content.data.into_owned(),
        )
            .into_response();
    }

    // SPA fallback: serve index.html for all other routes
    match UiAssets::get("index.html") {
        Some(content) => (
            [(axum::http::header::CONTENT_TYPE, "text/html")],
            content.data.into_owned(),
        )
            .into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn total_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    Query(time_range): Query<TimeRange>,
) -> Json<TotalOverview> {
    let time_range = if time_range.start.is_some() || time_range.end.is_some() {
        Some(time_range)
    } else {
        None
    };
    Json(reporter.total_overview(time_range).await)
}

async fn apps_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    Query(time_range): Query<TimeRange>,
) -> Json<Vec<AppOverview>> {
    let time_range = if time_range.start.is_some() || time_range.end.is_some() {
        Some(time_range)
    } else {
        None
    };
    Json(reporter.apps_overview(time_range).await)
}

async fn app_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(host): axum::extract::Path<String>,
    Query(time_range): Query<TimeRange>,
) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    let time_range = if time_range.start.is_some() || time_range.end.is_some() {
        Some(time_range)
    } else {
        None
    };

    match reporter.app_overview(&Host(host), time_range).await {
        Some(overview) => Json(overview).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn app_runs_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(host): axum::extract::Path<String>,
    Query(time_range): Query<TimeRange>,
) -> Json<Vec<AppRun>> {
    let time_range = if time_range.start.is_some() || time_range.end.is_some() {
        Some(time_range)
    } else {
        None
    };

    Json(reporter.app_runs(&Host(host), time_range).await)
}

async fn run_logs_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    match reporter.run_logs(&RunId::from_string(run_id)).await {
        Some(logs) => Json(logs).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

fn create_api_router<R: Reporter>(reporter: R) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/version", get(version_handler))
        .route("/api/total-overview", get(total_overview_handler::<R>))
        .route("/api/apps-overview", get(apps_overview_handler::<R>))
        .route("/api/app-overview/{host}", get(app_overview_handler::<R>))
        .route("/api/app-runs/{host}", get(app_runs_handler::<R>))
        .route("/api/run-logs/{run_id}", get(run_logs_handler::<R>))
        .fallback(static_handler)
        .layer(cors)
        .with_state(reporter)
}

pub struct ProxyContext {
    host: Host,
    app: Arc<RwLock<App>>,
    peer: Box<pingora::prelude::HttpPeer>,
}

impl ProxyContext {
    async fn new(host: &str, app: Arc<RwLock<App>>) -> Self {
        let address = app.read().await.address;

        Self {
            app,
            host: Host(host.to_owned()),
            peer: Box::new(pingora::prelude::HttpPeer::new(
                address,
                false,
                host.to_owned(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl<C> pingora::prelude::ProxyHttp for YarpProxy<C>
where
    C: Collector,
{
    type CTX = Option<ProxyContext>;

    fn new_ctx(&self) -> Self::CTX {
        None
    }

    async fn request_filter(
        &self,
        session: &mut pingora::prelude::Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        let host = get_host(session).ok_or_else(|| {
            warn!("request missing host header");
            pingora::Error::explain(pingora::ErrorType::InvalidHTTPHeader, "failed to get host")
        })?;

        debug!(host = %host, "processing request");
        *ctx = self.config.get_proxy_context(host).await;

        if ctx.is_none() {
            warn!(host = %host, "no app configured for host");
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut pingora::proxy::Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<pingora::prelude::HttpPeer>> {
        let ctx = ctx.take().ok_or_else(|| {
            error!("no proxy context available");
            pingora::Error::explain(
                pingora::ErrorType::ConnectError,
                "failed to get proxy context",
            )
        })?;

        info!(host = %ctx.host, "proxying request");

        App::start_app(&ctx.host, &ctx.app, self.collector.clone()).await?;
        App::schedule_kill(&ctx.host, &ctx.app, self.collector.clone()).await;

        let address = ctx.app.read().await.address;
        debug!(host = %ctx.host, upstream = %address, "connecting to upstream");

        Ok(ctx.peer.clone())
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,penny=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let Args {
        command: Command::Serve { config, address },
    } = Args::parse();

    info!(config = %config, address = %address, "starting penny proxy");

    let mut server = pingora::server::Server::new(None).unwrap();
    server.bootstrap();

    let config_content = std::fs::read_to_string(&config)?;
    let config: Config = toml::from_str(&config_content)?;

    info!(apps_count = config.apps.len(), "loaded configuration");
    for (host, app) in &config.apps {
        let app = app.blocking_read();
        info!(
            host = %host,
            address = %app.address,
            health_check = %app.health_check,
            "registered app"
        );
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let collector = runtime.block_on(SqliteDatabase::new(&config.database_url))?;

    if let Some(api_address) = config.api_address {
        let api_collector = collector.clone();
        runtime.spawn(async move {
            let router = create_api_router(api_collector);
            let listener = tokio::net::TcpListener::bind(api_address).await.unwrap();
            info!(address = %api_address, "API server listening");
            axum::serve(listener, router).await.unwrap();
        });
    }

    let proxy = YarpProxy::new(config, collector);

    let mut proxy_service = pingora::prelude::http_proxy_service(&server.configuration, proxy);
    proxy_service.add_tcp(&address);

    server.add_service(proxy_service);

    info!(address = %address, "proxy server listening");
    server.run_forever()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> SqliteDatabase {
        SqliteDatabase::new("sqlite::memory:")
            .await
            .expect("failed to create in-memory database")
    }

    mod collector_tests {
        use super::*;

        #[tokio::test]
        async fn app_started_creates_run_record() {
            let db = create_test_db().await;
            let host = Host("test-app.local".to_string());

            let run_id = db.app_started(&host).await;

            // Verify via reporter that the run exists
            let runs = db.app_runs(&host, None).await;
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].run_id, run_id.0);
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

    mod reporter_tests {
        use super::*;

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

            let runs = db.app_runs(&host, None).await;

            assert_eq!(runs.len(), 3);

            // Verify all run IDs are present
            let run_ids: Vec<&str> = runs.iter().map(|r| r.run_id.as_str()).collect();
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

            let runs = db.app_runs(&host1, None).await;

            assert_eq!(runs.len(), 1);
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
    }
}
