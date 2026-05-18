use anyhow::Result;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

use crate::cli::Cli;
use crate::config::AppConfig;

pub fn init(cli: &Cli, config: &AppConfig) -> Result<()> {
    let level = match cli.verbose {
        0 => config.logging.level.clone(),
        1 => "debug".to_string(),
        _ => "trace".to_string(),
    };
    let filter = EnvFilter::try_new(level).or_else(|_| EnvFilter::try_new("info"))?;
    let builder = fmt().with_env_filter(filter);

    if cli.log_json || config.logging.json {
        builder.json().init();
    } else {
        builder.compact().init();
    }

    Ok(())
}
