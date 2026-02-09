//! Telemetry command handlers

use qipu_core::config::GlobalConfig;
use qipu_core::error::Result;
use qipu_core::telemetry::{TelemetryCollector, TelemetryConfig, TelemetryEvent};
use std::fs;

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

pub fn handle_show() -> Result<()> {
    let config = TelemetryConfig::default();
    let collector = TelemetryCollector::new(config.clone());

    println!(
        "Telemetry status: {}\n",
        if config.enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    // Load events from disk
    let events_dir = &config.events_dir;
    let events_file = events_dir.join("events.jsonl");

    let mut all_events: Vec<TelemetryEvent> = Vec::new();

    if events_file.exists() {
        let content = fs::read_to_string(&events_file).map_err(qipu_core::error::QipuError::Io)?;

        for line in content.lines() {
            if let Ok(event) = serde_json::from_str::<TelemetryEvent>(line) {
                all_events.push(event);
            }
        }
    }

    // Also add in-memory events
    let pending = collector.get_pending_events();
    all_events.extend(pending);

    if all_events.is_empty() {
        println!("No pending telemetry events.");
        return Ok(());
    }

    println!("Pending telemetry events ({}):", all_events.len());
    println!("{}", "=".repeat(50));

    for (i, event) in all_events.iter().enumerate() {
        match event {
            TelemetryEvent::CommandExecuted {
                timestamp,
                command,
                success,
                duration,
                error,
            } => {
                let dt = chrono::DateTime::from_timestamp(*timestamp, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| timestamp.to_string());

                let status = if *success {
                    "✓ success"
                } else {
                    "✗ failed"
                };
                let err_str = error
                    .map(|e| format!(" [{}]", format!("{:?}", e).to_lowercase()))
                    .unwrap_or_default();

                println!(
                    "{}. [{}] Command: {} - {} (duration: {:?}){}",
                    i + 1,
                    dt,
                    command.as_str(),
                    status,
                    duration,
                    err_str
                );
            }
            TelemetryEvent::SessionStats {
                timestamp,
                os_platform,
                app_version,
                workspace_count,
                note_count,
            } => {
                let dt = chrono::DateTime::from_timestamp(*timestamp, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| timestamp.to_string());

                println!(
                    "{}. [{}] Session stats - OS: {}, Version: {}, Workspaces: {:?}, Notes: {:?}",
                    i + 1,
                    dt,
                    os_platform,
                    app_version,
                    workspace_count,
                    note_count
                );
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("Total events ready for upload: {}", all_events.len());

    Ok(())
}

pub fn handle_upload() -> Result<()> {
    use qipu_core::telemetry::{EndpointConfig, TelemetryUploader};
    use std::sync::Arc;

    let config = TelemetryConfig::default();
    let collector = TelemetryCollector::new(config.clone());

    if !collector.is_enabled() {
        println!("Telemetry is disabled. Enable with: qipu telemetry enable");
        return Ok(());
    }

    let endpoint_config = EndpointConfig::from_env();
    if !endpoint_config.is_configured() {
        println!("No telemetry endpoint configured.");
        println!("Set QIPU_TELEMETRY_ENDPOINT to enable remote upload.");
        println!("Example: export QIPU_TELEMETRY_ENDPOINT=https://telemetry.example.com/v1/batch");
        return Ok(());
    }

    // Load events from disk into collector
    collector.rotate_events().ok();

    let events = collector.get_pending_events();
    if events.is_empty() {
        println!("No telemetry events to upload.");
        return Ok(());
    }

    println!("Uploading {} telemetry events...", events.len());

    let uploader = TelemetryUploader::new(Arc::new(collector));

    match uploader.upload_immediate() {
        Ok(()) => {
            println!("✓ Upload successful");
            println!("  Local events cleared.");
        }
        Err(e) => {
            println!("✗ Upload failed: {}", e);
            println!("  Events retained for retry.");
        }
    }

    Ok(())
}
