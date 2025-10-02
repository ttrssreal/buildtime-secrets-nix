#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use buildtime_secrets_nix::Provisioner;
use std::io::Write;
use std::sync::Mutex;
use tracing::{Subscriber, debug, warn};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt,
};

pub const DEFAULT_LOG_FILE: &str = "/var/log/buildtime-secrets-nix/log";

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("cannot read CONFIG_FILE environment variable: {0}")]
    ReadConfigVar(#[from] std::env::VarError),

    #[error("cannot open config file: {0}")]
    OpenConfigFile(#[from] std::io::Error),

    #[error("cannot parse config file: {0}")]
    ParseConfigFile(#[from] serde_json::Error),

    #[error("cannot find derivation path in program args")]
    GetDerivationPath,

    #[error("error during provisioning: {0}")]
    ProvisionSecrets(#[from] buildtime_secrets_nix::Error),
}

// Hijack the error reporting system!!
fn report_error<W, S>(mut w: W, msg: S) -> std::io::Result<()>
where
    W: Write,
    S: AsRef<str>,
{
    const PREFIX: &str = "unknown pre-build hook command '";
    const PREFIX_LEN: usize = PREFIX.len();

    // First remove the error message leaving only "error: "
    w.write_all(&b"\x08".repeat(PREFIX_LEN))?;

    // End ansi color mode
    w.write_all(b"\x1b[0m")?;

    // Replace the error with our error
    w.write_all(msg.as_ref().as_bytes())?;

    w.write_all(b"\n")?;

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            let msg = format!("buildtime-secrets-nix: failed to provision secrets: {err}");

            // Stderr will send logs to the nix-daemon's journalctl
            eprintln!("{msg}");

            // Report the error directly to the terminal
            report_error(std::io::stdout().lock(), msg)
                .expect("failed to report error: couldn't write to stdout");

            // Finish "successfully" so nix will then report the faulty pre-build
            // hook command
            std::process::exit(0);
        }
    }
}

fn export_mount_paths(provisioner: &Provisioner<'_>) -> Result<(), Error> {
    let path = provisioner.derivation_secret_directory()?;
    let derivation_secret_directory = path.to_string_lossy();

    println!("extra-sandbox-paths");
    println!("/secrets={derivation_secret_directory}");

    Ok(())
}

fn build_log_file_layer<S>() -> Option<Box<dyn Layer<S> + Send + Sync + 'static>>
where
    S: Subscriber + for<'span> LookupSpan<'span> + 'static,
{
    let log_file_path: std::path::PathBuf = std::env::var("LOG_FILE")
        .unwrap_or(DEFAULT_LOG_FILE.to_string())
        .into();

    let _ = log_file_path.parent().map(std::fs::create_dir_all);

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .map(|log_file| {
            tracing_subscriber::fmt::layer()
                .with_writer(Mutex::new(log_file))
                .with_ansi(false)
                .with_filter(EnvFilter::from_default_env())
                .boxed()
        })
        .ok()
}

fn run() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(build_log_file_layer())
        .init();

    let config_path = std::env::var("CONFIG_FILE")?;
    debug!("reading config file at {config_path:?}");

    let config_string = std::fs::read_to_string(&config_path)?;
    let mut config: buildtime_secrets_nix::Config = serde_json::from_str(&config_string)?;

    let mut args = std::env::args();
    let args_len = args.len();

    if args_len != 2 || args_len != 3 {
        warn!("expected to recive 2 or 3 program arguments but got {args_len:?}");
    }

    config.derivation = args.nth(1).ok_or(Error::GetDerivationPath)?;
    debug!("finished config: {config:?}");

    let provisioner = buildtime_secrets_nix::Provisioner::new(&config)?;

    provisioner.provision_all()?;

    // All secrets were successful
    if provisioner.required_secrets()?.is_some() {
        export_mount_paths(&provisioner)?;
    }

    Ok(())
}
