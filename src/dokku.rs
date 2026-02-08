use std::fs;
use std::path::PathBuf;
use std::process::Command;

const DATA_DIR: &str = "/var/lib/dokku/data/penny-vhosts";
const CONFIG_FILE: &str = "penny.toml";

fn data_dir() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

fn config_file_path() -> PathBuf {
    data_dir().join(CONFIG_FILE)
}

fn run_cmd_output(program: &str, args: &[&str]) -> color_eyre::Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run `{program}`: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(color_eyre::eyre::eyre!(
            "`{program} {}` failed: {stderr}",
            args.join(" ")
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn read_property(app: &str, property: &str) -> Option<String> {
    let path = data_dir().join(app).join(property);
    fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn read_property_or(app: &str, property: &str, default: &str) -> String {
    read_property(app, property).unwrap_or_else(|| default.to_owned())
}

struct AppInfo {
    name: String,
    domains: Vec<String>,
    host_port: String,
}

fn get_penny_apps() -> color_eyre::Result<Vec<AppInfo>> {
    let apps_output = run_cmd_output("dokku", &["apps:list"])?;
    let mut result = Vec::new();

    for app in apps_output.lines().skip(1) {
        let app = app.trim();
        if app.is_empty() {
            continue;
        }

        // Check proxy-type == "penny"
        let proxy_type = match run_cmd_output("dokku", &["proxy:report", app, "--proxy-type"]) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if proxy_type != "penny" {
            continue;
        }

        // Check proxy-enabled == "true"
        let proxy_enabled = match run_cmd_output("dokku", &["proxy:report", app, "--proxy-enabled"])
        {
            Ok(e) => e,
            Err(_) => continue,
        };
        if proxy_enabled != "true" {
            continue;
        }

        // Get domains
        let domains_str =
            match run_cmd_output("dokku", &["domains:report", app, "--domains-app-vhosts"]) {
                Ok(d) => d,
                Err(_) => continue,
            };
        if domains_str.is_empty() {
            continue;
        }
        let domains: Vec<String> = domains_str
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();

        // Get container host port
        let host_port = match get_host_port(app) {
            Some(p) => p,
            None => continue,
        };

        result.push(AppInfo {
            name: app.to_owned(),
            domains,
            host_port,
        });
    }

    Ok(result)
}

fn get_host_port(app: &str) -> Option<String> {
    let container_id = run_cmd_output(
        "docker",
        &[
            "ps",
            "-q",
            "--filter",
            &format!("label=com.dokku.app-name={app}"),
            "--filter",
            "label=com.dokku.container-type=web",
        ],
    )
    .ok()?;

    let container_id = container_id.lines().next()?.trim();
    if container_id.is_empty() {
        return None;
    }

    let host_port = run_cmd_output(
        "docker",
        &[
            "inspect",
            "--format",
            "{{range $p, $conf := .NetworkSettings.Ports}}{{range $conf}}{{.HostPort}}{{end}}{{end}}",
            container_id,
        ],
    )
    .ok()?;

    let host_port = host_port.lines().next()?.trim().to_owned();
    if host_port.is_empty() {
        return None;
    }

    Some(host_port)
}

fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn generate_config(apps: &[AppInfo]) -> String {
    let mut config = String::new();

    for app in apps {
        let wait_period = read_property_or(&app.name, "wait-period", "10m");
        let health_check = read_property_or(&app.name, "health-check", "/");
        let cold_start_page = read_property_or(&app.name, "cold-start-page", "true");
        let start_timeout = read_property_or(&app.name, "start-timeout", "30s");
        let stop_timeout = read_property_or(&app.name, "stop-timeout", "30s");
        let adaptive_wait = read_property_or(&app.name, "adaptive-wait", "false");

        for domain in &app.domains {
            let escaped_domain = escape_toml_string(domain);
            config.push_str(&format!("[\"{escaped_domain}\"]\n"));
            config.push_str(&format!("address = \"127.0.0.1:{}\"\n", app.host_port));
            config.push_str(&format!(
                "health_check = \"{}\"\n",
                escape_toml_string(&health_check)
            ));
            config.push_str(&format!("wait_period = \"{wait_period}\"\n"));
            config.push_str(&format!("cold_start_page = {cold_start_page}\n"));
            config.push_str(&format!("start_timeout = \"{start_timeout}\"\n"));
            config.push_str(&format!("stop_timeout = \"{stop_timeout}\"\n"));
            config.push_str(&format!("adaptive_wait = {adaptive_wait}\n"));

            // Optional properties — only emit if set
            if let Some(val) = read_property(&app.name, "min-wait-period") {
                config.push_str(&format!("min_wait_period = \"{val}\"\n"));
            }
            if let Some(val) = read_property(&app.name, "max-wait-period") {
                config.push_str(&format!("max_wait_period = \"{val}\"\n"));
            }
            if let Some(val) = read_property(&app.name, "low-req-per-hour") {
                config.push_str(&format!("low_req_per_hour = {val}\n"));
            }
            if let Some(val) = read_property(&app.name, "high-req-per-hour") {
                config.push_str(&format!("high_req_per_hour = {val}\n"));
            }
            if let Some(val) = read_property(&app.name, "cold-start-page-path") {
                config.push_str(&format!(
                    "cold_start_page_path = \"{}\"\n",
                    escape_toml_string(&val)
                ));
            }

            config.push_str(&format!("\n[\"{escaped_domain}\".command]\n"));
            config.push_str(&format!(
                "start = \"dokku ps:start {}\"\n",
                escape_toml_string(&app.name)
            ));
            config.push_str(&format!(
                "end = \"dokku ps:stop {}\"\n",
                escape_toml_string(&app.name)
            ));
            config.push('\n');
        }
    }

    config
}

pub fn build_config() -> color_eyre::Result<()> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;

    let apps = get_penny_apps()?;
    let config = generate_config(&apps);

    let config_path = config_file_path();
    fs::write(&config_path, &config)?;
    println!("penny config written to {}", config_path.display());

    Ok(())
}

pub fn clear_config(app: &str) -> color_eyre::Result<()> {
    // Clear config is the same as build config — the cleared app simply won't
    // appear because it's no longer penny-proxied or has been removed.
    println!("clearing penny config for {app}");
    build_config()
}
