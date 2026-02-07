use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SERVICE_NAME: &str = "penny.service";

/// Options for generating the systemd unit file, mirroring `serve` flags.
pub struct InstallOpts {
    pub config: String,
    pub address: String,
    pub https_address: String,
    pub no_tls: bool,
    pub password: Option<String>,
}

fn user_service_dir() -> color_eyre::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| color_eyre::eyre::eyre!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(".config/systemd/user"))
}

fn service_file_path() -> color_eyre::Result<PathBuf> {
    Ok(user_service_dir()?.join(SERVICE_NAME))
}

fn penny_binary_path() -> color_eyre::Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| color_eyre::eyre::eyre!("failed to resolve penny binary path: {e}"))
}

fn login_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_owned())
}

fn run_cmd(program: &str, args: &[&str]) -> color_eyre::Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run `{program}`: {e}"))?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!(
            "`{program} {}` exited with {}",
            args.join(" "),
            status
        ));
    }

    Ok(())
}

fn generate_unit_file(opts: &InstallOpts) -> color_eyre::Result<String> {
    let config_path = fs::canonicalize(&opts.config).map_err(|e| {
        color_eyre::eyre::eyre!(
            "config file '{}' not found or inaccessible: {e}",
            opts.config
        )
    })?;

    // Validate the config file parses correctly.
    let config_content = fs::read_to_string(&config_path)?;
    let _config: crate::config::Config = toml::from_str(&config_content)
        .map_err(|e| color_eyre::eyre::eyre!("invalid config file: {e}"))?;

    let penny_bin = penny_binary_path()?;
    let shell = login_shell();
    let working_dir = config_path
        .parent()
        .unwrap_or(Path::new("/"))
        .to_string_lossy();

    let mut serve_args = format!(
        "serve {} --address {} --https-address {}",
        config_path.display(),
        opts.address,
        opts.https_address,
    );
    if opts.no_tls {
        serve_args.push_str(" --no-tls");
    }

    let exec_start = format!("{shell} -lc 'exec {} {serve_args}'", penny_bin.display(),);

    let mut environment_lines = String::new();
    if let Some(ref password) = opts.password {
        environment_lines.push_str(&format!("Environment=PENNY_PASSWORD={password}\n"));
    }
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        environment_lines.push_str(&format!("Environment=RUST_LOG={rust_log}\n"));
    }

    Ok(format!(
        "\
[Unit]
Description=Penny reverse proxy
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={exec_start}
Restart=on-failure
RestartSec=5
WorkingDirectory={working_dir}
{environment_lines}
[Install]
WantedBy=default.target
"
    ))
}

pub fn install(opts: InstallOpts) -> color_eyre::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(color_eyre::eyre::eyre!(
            "the `systemd` command is only available on Linux"
        ));
    }

    let service_path = service_file_path()?;
    if service_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "service already installed at {}, run `penny systemd uninstall` first",
            service_path.display()
        ));
    }

    let unit_content = generate_unit_file(&opts)?;

    // Ensure the directory exists.
    let service_dir = user_service_dir()?;
    fs::create_dir_all(&service_dir)?;

    fs::write(&service_path, &unit_content)?;
    println!("wrote unit file to {}", service_path.display());

    run_cmd("systemctl", &["--user", "daemon-reload"])?;
    println!("reloaded systemd daemon");

    run_cmd("systemctl", &["--user", "enable", SERVICE_NAME])?;
    println!("enabled {SERVICE_NAME}");

    run_cmd("systemctl", &["--user", "start", SERVICE_NAME])?;
    println!("started {SERVICE_NAME}");

    // enable-linger is non-fatal — service still works when logged in.
    if let Ok(user) = std::env::var("USER") {
        if let Err(e) = run_cmd("loginctl", &["enable-linger", &user]) {
            eprintln!(
                "warning: failed to enable linger (service won't start at boot without a login session): {e}"
            );
        } else {
            println!("enabled linger for user {user}");
        }
    }

    println!("\npenny service installed and running.");
    println!("use `penny systemd status` to check status");
    println!("use `penny systemd logs --follow` to watch logs");

    Ok(())
}

pub fn uninstall() -> color_eyre::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(color_eyre::eyre::eyre!(
            "the `systemd` command is only available on Linux"
        ));
    }

    let service_path = service_file_path()?;
    if !service_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "service not installed (no unit file at {})",
            service_path.display()
        ));
    }

    // Stop and disable (ignore errors — service might already be stopped).
    let _ = run_cmd("systemctl", &["--user", "stop", SERVICE_NAME]);
    println!("stopped {SERVICE_NAME}");

    let _ = run_cmd("systemctl", &["--user", "disable", SERVICE_NAME]);
    println!("disabled {SERVICE_NAME}");

    fs::remove_file(&service_path)?;
    println!("removed {}", service_path.display());

    run_cmd("systemctl", &["--user", "daemon-reload"])?;
    println!("reloaded systemd daemon");

    println!("\npenny service uninstalled.");

    Ok(())
}

pub fn status() -> color_eyre::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(color_eyre::eyre::eyre!(
            "the `systemd` command is only available on Linux"
        ));
    }

    let service_path = service_file_path()?;
    if !service_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "service not installed (no unit file at {})",
            service_path.display()
        ));
    }

    // Pass through directly — let systemctl print its output.
    let status = Command::new("systemctl")
        .args(["--user", "status", SERVICE_NAME])
        .status()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run systemctl: {e}"))?;

    // systemctl status exits with 3 when service is inactive, which is not an error.
    if !status.success() && status.code() != Some(3) {
        return Err(color_eyre::eyre::eyre!(
            "systemctl status exited with {}",
            status
        ));
    }

    Ok(())
}

pub fn logs(follow: bool) -> color_eyre::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(color_eyre::eyre::eyre!(
            "the `systemd` command is only available on Linux"
        ));
    }

    let mut args = vec!["--user-unit", SERVICE_NAME];
    if follow {
        args.push("--follow");
    }

    let status = Command::new("journalctl")
        .args(&args)
        .status()
        .map_err(|e| color_eyre::eyre::eyre!("failed to run journalctl: {e}"))?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!("journalctl exited with {}", status));
    }

    Ok(())
}
