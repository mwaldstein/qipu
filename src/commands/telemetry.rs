//! Telemetry command handlers

use qipu_core::config::GlobalConfig;
use qipu_core::error::Result;

#[derive(Debug, Clone)]
pub enum TelemetrySource {
    Environment,
    Config,
    Default,
}

pub fn handle_enable() -> Result<()> {
    let mut config = GlobalConfig::load()?;
    config.set_telemetry_enabled(true);
    config.save()?;
    println!("Telemetry enabled");
    Ok(())
}

pub fn handle_disable() -> Result<()> {
    let mut config = GlobalConfig::load()?;
    config.set_telemetry_enabled(false);
    config.save()?;
    println!("Telemetry disabled");
    Ok(())
}

pub fn handle_status() -> Result<()> {
    let source;
    let enabled;

    if std::env::var("QIPU_NO_TELEMETRY").is_ok() {
        source = TelemetrySource::Environment;
        enabled = false;
    } else {
        match GlobalConfig::load() {
            Ok(config) => {
                source = TelemetrySource::Config;
                enabled = config.get_telemetry_enabled().unwrap_or(false);
            }
            Err(_) => {
                source = TelemetrySource::Default;
                enabled = false;
            }
        }
    }

    println!(
        "Telemetry: {}",
        if enabled { "enabled" } else { "disabled" }
    );

    let source_str = match source {
        TelemetrySource::Environment => "QIPU_NO_TELEMETRY environment variable".to_string(),
        TelemetrySource::Config => GlobalConfig::source_display(),
        TelemetrySource::Default => "default (disabled)".to_string(),
    };
    println!("Source: {}", source_str);

    Ok(())
}
