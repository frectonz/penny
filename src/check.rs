use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, info};

use crate::collector::Collector;
use crate::config::{App, Config};
use crate::types::{Host, RunId};

/// A collector that does nothing (no database needed for check).
#[derive(Debug, Clone)]
pub struct NoOpCollector;

#[async_trait::async_trait]
impl Collector for NoOpCollector {
    async fn app_started(&self, _host: &Host) -> RunId {
        RunId::default()
    }

    async fn app_stopped(&self, _host: &Host) {}

    async fn app_start_failed(&self, _host: &Host) {}

    async fn app_stop_failed(&self, _host: &Host) {}

    async fn append_stdout(&self, _run_id: &RunId, _line: String) {}

    async fn append_stderr(&self, _run_id: &RunId, _line: String) {}
}

/// Tracks check results for a single app.
#[derive(Debug)]
pub struct AppCheckResult {
    pub hostname: String,
    pub start_success: bool,
    pub health_check_success: bool,
    pub stop_success: bool,
    pub start_error: Option<String>,
    pub health_check_error: Option<String>,
    pub stop_error: Option<String>,
}

impl AppCheckResult {
    pub fn new(hostname: String) -> Self {
        Self {
            hostname,
            start_success: false,
            health_check_success: false,
            stop_success: false,
            start_error: None,
            health_check_error: None,
            stop_error: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.start_success && self.health_check_success && self.stop_success
    }
}

/// Runs the check for a single app.
async fn check_app(hostname: &str, app: &Arc<RwLock<App>>) -> AppCheckResult {
    let mut result = AppCheckResult::new(hostname.to_string());

    // Start the app
    info!(hostname = %hostname, "starting app");
    app.write().await.command.start::<NoOpCollector>(None);
    result.start_success = true;

    // Wait for healthy
    info!(hostname = %hostname, "waiting for health check");
    match app.read().await.wait_for_running().await {
        Ok(()) => {
            result.health_check_success = true;
        }
        Err(_) => {
            result.health_check_error = Some("Health check timed out".to_string());
            error!(hostname = %hostname, "health check failed");
        }
    }

    // Stop the app
    info!(hostname = %hostname, "stopping app");
    app.write().await.command.stop().await;

    // Wait for stopped
    info!(hostname = %hostname, "waiting for app to stop");
    match app.read().await.wait_for_stopped().await {
        Ok(()) => {
            result.stop_success = true;
        }
        Err(_) => {
            result.stop_error = Some("Stop timed out".to_string());
            error!(hostname = %hostname, "stop timed out");
        }
    }

    result
}

/// Prints the result for a single app.
fn print_app_result(result: &AppCheckResult) {
    println!("========================================");
    println!("Checking: {}", result.hostname);
    println!("========================================");

    if result.start_success {
        println!("  \u{2713} Start command executed");
    } else {
        let error = result.start_error.as_deref().unwrap_or("Unknown error");
        println!("  \u{2717} Start command failed: {}", error);
    }

    if result.health_check_success {
        println!("  \u{2713} Health check passed");
    } else {
        let error = result
            .health_check_error
            .as_deref()
            .unwrap_or("Unknown error");
        println!("  \u{2717} Health check failed: {}", error);
    }

    if result.stop_success {
        println!("  \u{2713} Stop completed");
    } else {
        let error = result.stop_error.as_deref().unwrap_or("Unknown error");
        println!("  \u{2717} Stop failed: {}", error);
    }

    println!();
}

/// Prints the summary of all check results.
fn print_summary(results: &[AppCheckResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.is_success()).count();
    let failed = total - passed;

    println!("========================================");
    println!("Summary");
    println!("========================================");
    println!("Total: {} | Passed: {} | Failed: {}", total, passed, failed);
}

/// Main entry point for the check command.
pub async fn run_check(
    config_path: &str,
    apps_filter: Option<Vec<String>>,
) -> color_eyre::Result<()> {
    let config_content = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    info!(apps_count = config.apps.len(), "loaded configuration");

    // Filter apps if specified
    let apps_to_check: Vec<_> = if let Some(filter) = apps_filter {
        config
            .apps
            .iter()
            .filter(|(hostname, _)| filter.contains(hostname))
            .collect()
    } else {
        config.apps.iter().collect()
    };

    if apps_to_check.is_empty() {
        println!("No apps to check.");
        return Ok(());
    }

    let mut results = Vec::new();

    for (hostname, app) in apps_to_check {
        let result = check_app(hostname, app).await;
        print_app_result(&result);
        results.push(result);
    }

    print_summary(&results);

    // Return error if any checks failed
    let failed_count = results.iter().filter(|r| !r.is_success()).count();
    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
