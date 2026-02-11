use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use jiff::SignedDuration;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::collector::Collector;
use crate::db::SqliteDatabase;
use crate::proxy::ProxyContext;
use crate::types::{Host, RunId};

const SHORT_WINDOW_MINUTES: u64 = 5;
const LONG_WINDOW_MINUTES: u64 = 30;

#[derive(Debug, Default)]
pub struct RequestTracker {
    /// Request counts bucketed by minute (minute_epoch, count)
    buckets: VecDeque<(u64, u64)>,
}

impl RequestTracker {
    fn current_minute() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 60
    }

    pub fn record_request(&mut self) {
        let now = Self::current_minute();

        if let Some(last) = self.buckets.back_mut()
            && last.0 == now
        {
            last.1 += 1;
            return;
        }

        self.buckets.push_back((now, 1));

        // Prune buckets older than the long window
        let cutoff = now.saturating_sub(LONG_WINDOW_MINUTES);
        while let Some(front) = self.buckets.front() {
            if front.0 < cutoff {
                self.buckets.pop_front();
            } else {
                break;
            }
        }
    }

    /// Returns (short_rate, long_rate) in requests per minute.
    pub fn request_rates(&self) -> (f64, f64) {
        let now = Self::current_minute();
        let short_cutoff = now.saturating_sub(SHORT_WINDOW_MINUTES);
        let long_cutoff = now.saturating_sub(LONG_WINDOW_MINUTES);

        let mut short_total: u64 = 0;
        let mut long_total: u64 = 0;

        for &(minute, count) in &self.buckets {
            if minute >= long_cutoff {
                long_total += count;
                if minute >= short_cutoff {
                    short_total += count;
                }
            }
        }

        let short_rate = short_total as f64 / SHORT_WINDOW_MINUTES as f64;
        let long_rate = long_total as f64 / LONG_WINDOW_MINUTES as f64;

        (short_rate, long_rate)
    }

    /// Total requests within the long window, for logging.
    pub fn total_recent_requests(&self) -> u64 {
        let now = Self::current_minute();
        let cutoff = now.saturating_sub(LONG_WINDOW_MINUTES);

        self.buckets
            .iter()
            .filter(|(minute, _)| *minute >= cutoff)
            .map(|(_, count)| count)
            .sum()
    }
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

    #[serde(default = "default_health_check_initial_backoff_ms")]
    pub health_check_initial_backoff_ms: u64,
    #[serde(default = "default_health_check_max_backoff_secs")]
    pub health_check_max_backoff_secs: u64,

    #[serde(default)]
    pub cold_start_page: bool,

    #[serde(default)]
    pub cold_start_page_path: Option<PathBuf>,

    #[serde(skip)]
    pub cold_start_page_html: Option<String>,

    #[serde(default)]
    pub adaptive_wait: bool,

    #[serde(default)]
    pub min_wait_period: Option<SignedDuration>,

    #[serde(default)]
    pub max_wait_period: Option<SignedDuration>,

    #[serde(default)]
    pub low_req_per_hour: Option<f64>,

    #[serde(default)]
    pub high_req_per_hour: Option<f64>,

    #[serde(default)]
    pub also_warm: Vec<String>,

    #[serde(skip)]
    pub request_tracker: RequestTracker,

    #[serde(skip)]
    pub confirmed_healthy: bool,

    #[serde(skip)]
    pub kill_task: Option<KillTask>,
}

/// Handle for a scheduled kill task. Dropping the `cancel` sender
/// cancels only the sleep phase; the stop/cleanup phase runs to completion.
pub struct KillTask {
    // Dropped to signal cancellation — never read directly.
    #[allow(dead_code)]
    cancel: tokio::sync::oneshot::Sender<()>,
    _handle: tokio::task::JoinHandle<()>,
}

impl std::fmt::Debug for KillTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KillTask").finish()
    }
}

pub fn default_wait_period() -> SignedDuration {
    SignedDuration::from_mins(10)
}

pub fn default_start_timeout() -> SignedDuration {
    SignedDuration::from_secs(30)
}

pub fn default_stop_timeout() -> SignedDuration {
    SignedDuration::from_secs(30)
}

fn default_min_wait_period() -> SignedDuration {
    SignedDuration::from_mins(5)
}

fn default_max_wait_period() -> SignedDuration {
    SignedDuration::from_mins(30)
}

fn default_health_check_initial_backoff_ms() -> u64 {
    10
}

fn default_health_check_max_backoff_secs() -> u64 {
    2
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
        let mut words = shell_words::split(command)?.into_iter();
        let program = words.next().unwrap_or_else(|| command.to_owned());

        Ok(Self {
            program,
            args: words.collect(),
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
pub struct RunOptions<C: Collector> {
    pub run_id: RunId,
    pub collector: C,
}

impl<C: Collector> RunOptions<C> {
    pub async fn append_stdout(&self, line: String) {
        if let Err(e) = self.collector.append_stdout(&self.run_id, line).await {
            error!("failed to append stdout: {e}");
        }
    }

    pub async fn append_stderr(&self, line: String) {
        if let Err(e) = self.collector.append_stderr(&self.run_id, line).await {
            error!("failed to append stderr: {e}");
        }
    }
}

impl CommandSpec {
    pub fn is_child_running(&mut self) -> bool {
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
    pub fn run<C: Collector>(&mut self, opts: Option<RunOptions<C>>) {
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
    pub async fn kill(&mut self) {
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
    pub fn is_child_running(&mut self) -> bool {
        match self {
            AppCommand::Start(start) => start.is_child_running(),
            AppCommand::StartEnd { start, .. } => start.is_child_running(),
        }
    }

    #[instrument(skip(self))]
    pub fn start<C: Collector>(&mut self, opts: Option<RunOptions<C>>) {
        debug!("starting app command");
        let start = match self {
            AppCommand::Start(start) => start.as_mut(),
            AppCommand::StartEnd { start, .. } => start.as_mut(),
        };

        start.run(opts);
    }

    #[instrument(skip(self))]
    pub async fn stop(&mut self) {
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

static HTTP: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(reqwest::Client::new);

impl App {
    pub fn effective_wait_period(&self) -> Duration {
        if !self.adaptive_wait {
            return self.wait_period.unsigned_abs();
        }

        // When adaptive, wait_period is ignored; use dedicated min/max bounds
        let min_wait = self
            .min_wait_period
            .unwrap_or(default_min_wait_period())
            .unsigned_abs();
        let max_wait = self
            .max_wait_period
            .unwrap_or(default_max_wait_period())
            .unsigned_abs();

        let (short_rate, long_rate) = self.request_tracker.request_rates();
        let effective_rate = short_rate.max(long_rate);

        // Convert user-facing req/hr thresholds to req/min for comparison with rates
        let low = self.low_req_per_hour.unwrap_or(12.0) / 60.0;
        let high = self.high_req_per_hour.unwrap_or(300.0) / 60.0;

        // Smoothstep (Hermite): S-curve, gentle at extremes, steeper in middle
        let t = ((effective_rate - low) / (high - low)).clamp(0.0, 1.0);
        let factor = t * t * (3.0 - 2.0 * t);

        let min_secs = min_wait.as_secs_f64();
        let max_secs = max_wait.as_secs_f64();
        Duration::from_secs_f64(min_secs + (max_secs - min_secs) * factor)
    }

    #[instrument(skip(self), fields(address = %self.address, health_check = %self.health_check))]
    pub async fn is_running(&self) -> bool {
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
    pub async fn wait_for_running(&self) -> Result<(), pingora::time::Elapsed> {
        let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(
            self.health_check_initial_backoff_ms,
        )
        .max_delay(Duration::from_secs(self.health_check_max_backoff_secs))
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
    pub async fn wait_for_stopped(&self) -> Result<(), pingora::time::Elapsed> {
        let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(
            self.health_check_initial_backoff_ms,
        )
        .max_delay(Duration::from_secs(self.health_check_max_backoff_secs))
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

    async fn wait_for_healthy(app: &Arc<RwLock<App>>) -> pingora::Result<()> {
        if app.read().await.wait_for_running().await.is_err() {
            error!("failed to start app within timeout");
            return Err(pingora::Error::explain(
                pingora::ErrorType::ConnectError,
                "failed to start app",
            ));
        }
        app.write().await.confirmed_healthy = true;
        Ok(())
    }

    #[instrument(skip(app))]
    pub async fn start_app(
        host: &Host,
        app: &Arc<RwLock<App>>,
        collector: impl Collector,
    ) -> pingora::Result<()> {
        let mut guard = app.write().await;

        // Fast path: if child process is already running, skip health check
        if guard.command.is_child_running() {
            if !guard.cold_start_page || guard.confirmed_healthy {
                debug!("child process already running, skipping health check");
                return Ok(());
            }
            // cold_start_page app started by loading page flow, not yet confirmed healthy
            drop(guard);
            return Self::wait_for_healthy(app).await;
        }

        // Slow path: no running child, do health check to confirm app state
        let needs_start = !guard.is_running().await;

        if needs_start {
            let address = guard.address;
            let run_id = collector.app_started(host).await.map_err(|e| {
                pingora::Error::explain(
                    pingora::ErrorType::ConnectError,
                    format!("failed to record app start: {e}"),
                )
            })?;

            info!(%address, "app not running, starting it");
            guard.command.start(Some(RunOptions {
                run_id,
                collector: collector.clone(),
            }));

            drop(guard);
            if let Err(e) = Self::wait_for_healthy(app).await {
                if let Err(e) = collector.app_start_failed(host).await {
                    error!("failed to record app start failure: {e}");
                }
                return Err(e);
            }
        } else {
            let address = guard.address;
            debug!(%address, "app already running");
        }

        Ok(())
    }

    #[instrument(skip(app))]
    pub async fn begin_start_app(
        host: &Host,
        app: &Arc<RwLock<App>>,
        collector: impl Collector,
    ) -> pingora::Result<bool> {
        let mut guard = app.write().await;

        // Fast path: child running and confirmed healthy
        if guard.command.is_child_running() && guard.confirmed_healthy {
            debug!("child process running and confirmed healthy");
            return Ok(true);
        }

        // Child running but not yet confirmed healthy
        if guard.command.is_child_running() {
            debug!("child process running but not yet confirmed healthy");
            return Ok(false);
        }

        // No child running, check if externally managed process is healthy
        if guard.is_running().await {
            debug!("externally managed process is healthy");
            guard.confirmed_healthy = true;
            return Ok(true);
        }

        // Need to start the app
        let run_id = collector.app_started(host).await.map_err(|e| {
            pingora::Error::explain(
                pingora::ErrorType::ConnectError,
                format!("failed to record app start: {e}"),
            )
        })?;
        let address = guard.address;

        info!(%address, "app not running, starting it (non-blocking)");
        guard.command.start(Some(RunOptions {
            run_id,
            collector: collector.clone(),
        }));

        drop(guard);

        // Spawn background task to wait for health and set confirmed_healthy
        let app = app.clone();
        let host = host.clone();
        tokio::spawn(async move {
            if app.read().await.wait_for_running().await.is_ok() {
                app.write().await.confirmed_healthy = true;
                info!(host = %host, "app confirmed healthy in background");
            } else {
                error!(host = %host, "app failed to start in background");
                if let Err(e) = collector.app_start_failed(&host).await {
                    error!(host = %host, "failed to record app start failure: {e}");
                }
            }
        });

        Ok(false)
    }

    #[instrument(skip(app))]
    pub async fn schedule_kill(host: &Host, app: &Arc<RwLock<App>>, collector: impl Collector) {
        let mut app_guard = app.write().await;

        if let Some(prev) = app_guard.kill_task.take() {
            debug!("cancelling previous kill task");
            drop(prev);
        }

        app_guard.request_tracker.record_request();
        let wait_period = app_guard.effective_wait_period();
        let (short_rate, long_rate) = app_guard.request_tracker.request_rates();
        let total_reqs = app_guard.request_tracker.total_recent_requests();
        info!(
            ?wait_period,
            short_rate = format!("{short_rate:.2}"),
            long_rate = format!("{long_rate:.2}"),
            total_reqs,
            adaptive = app_guard.adaptive_wait,
            "scheduling app shutdown"
        );

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let handle = {
            let app = app.clone();
            let host = host.clone();
            tokio::spawn(async move {
                let wait_period = app.read().await.effective_wait_period();

                // CANCELLABLE: sleep races against cancellation
                tokio::select! {
                    _ = pingora::time::sleep(wait_period) => {}
                    _ = cancel_rx => {
                        debug!("kill task cancelled during sleep");
                        return;
                    }
                }

                // CRITICAL SECTION: runs to completion, never aborted
                info!("wait period elapsed, stopping app");

                let mut guard = app.write().await;
                guard.command.stop().await;
                guard.confirmed_healthy = false;
                drop(guard);
                if let Err(e) = collector.app_stopped(&host).await {
                    error!("failed to record app stop: {e}");
                }

                if app.read().await.wait_for_stopped().await.is_err() {
                    error!("failed to stop app within timeout");
                    if let Err(e) = collector.app_stop_failed(&host).await {
                        error!("failed to record app stop failure: {e}");
                    }
                }
            })
        };

        app_guard.kill_task = Some(KillTask {
            cancel: cancel_tx,
            _handle: handle,
        });
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

/// TLS configuration for automatic certificate provisioning.
#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    /// Enable automatic TLS certificate provisioning.
    #[serde(default)]
    pub enabled: bool,

    /// Contact email for ACME account registration.
    pub acme_email: String,

    /// Use Let's Encrypt staging environment (for testing).
    #[serde(default)]
    pub staging: bool,

    /// Directory to store certificates.
    #[serde(default = "default_certs_dir")]
    pub certs_dir: PathBuf,

    /// Days before expiry to renew certificates.
    #[serde(default = "default_renewal_days")]
    pub renewal_days: u32,

    /// Hours between certificate renewal checks.
    #[serde(default = "default_renewal_check_interval_hours")]
    pub renewal_check_interval_hours: u64,

    /// Seconds between order status poll attempts.
    #[serde(default = "default_order_poll_interval_secs")]
    pub order_poll_interval_secs: u64,

    /// Maximum number of order status poll retries.
    #[serde(default = "default_order_poll_max_retries")]
    pub order_poll_max_retries: u32,

    /// Seconds between certificate readiness poll attempts.
    #[serde(default = "default_cert_poll_interval_secs")]
    pub cert_poll_interval_secs: u64,

    /// Maximum number of certificate readiness poll retries.
    #[serde(default = "default_cert_poll_max_retries")]
    pub cert_poll_max_retries: u32,
}

fn default_certs_dir() -> PathBuf {
    PathBuf::from("./certs")
}

fn default_renewal_days() -> u32 {
    30
}

fn default_renewal_check_interval_hours() -> u64 {
    12
}

fn default_order_poll_interval_secs() -> u64 {
    2
}

fn default_order_poll_max_retries() -> u32 {
    20
}

fn default_cert_poll_interval_secs() -> u64 {
    1
}

fn default_cert_poll_max_retries() -> u32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_address: Option<SocketAddr>,

    #[serde(default)]
    pub api_domain: Option<String>,

    #[serde(default = "default_database_url")]
    pub database_url: String,

    /// TLS configuration for automatic certificate provisioning.
    #[serde(default)]
    pub tls: Option<TlsConfig>,

    /// Default page size for paginated API responses.
    #[serde(default = "default_page_limit")]
    pub default_page_limit: u32,

    /// Maximum allowed page size for paginated API responses.
    #[serde(default = "default_max_page_limit")]
    pub max_page_limit: u32,

    #[serde(flatten, deserialize_with = "deserialize_apps")]
    pub apps: HashMap<String, Arc<RwLock<App>>>,
}

pub fn default_database_url() -> String {
    "sqlite://penny.db".to_owned()
}

fn default_page_limit() -> u32 {
    20
}

fn default_max_page_limit() -> u32 {
    100
}

impl Config {
    pub fn tls_domains(&self) -> Vec<String> {
        let mut domains: Vec<String> = self.apps.keys().cloned().collect();
        if let Some(api_domain) = &self.api_domain
            && self.api_address.is_some()
        {
            domains.push(api_domain.clone());
        }
        domains
    }

    pub fn load_cold_start_pages(&mut self) -> color_eyre::Result<()> {
        for (host, app) in &self.apps {
            let mut guard = app.blocking_write();
            if let Some(path) = &guard.cold_start_page_path {
                let html = std::fs::read_to_string(path).map_err(|e| {
                    color_eyre::eyre::eyre!(
                        "failed to read cold start page for {host} at {}: {e}",
                        path.display()
                    )
                })?;

                if !html.contains("<meta http-equiv=\"refresh\"") {
                    warn!(
                        host = %host,
                        path = %path.display(),
                        "custom cold start page is missing <meta http-equiv=\"refresh\" ...> tag; page won't auto-refresh"
                    );
                }

                guard.cold_start_page = true;
                guard.cold_start_page_html = Some(html);
            }
        }
        Ok(())
    }

    pub async fn get_proxy_context(&self, host: &str) -> Option<ProxyContext> {
        if let Some(app) = self.apps.get(host) {
            return Some(ProxyContext::new(host, app.clone()).await);
        }

        if let Some(api_domain) = &self.api_domain
            && host == api_domain
            && let Some(api_address) = self.api_address
        {
            return Some(ProxyContext::new_api(host, api_address));
        }

        None
    }
}
