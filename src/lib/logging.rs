use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize structured logging based on CLI arguments
pub fn init_tracing(
    verbose: bool,
    log_level: Option<&str>,
    log_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine log level from CLI arguments
    let level = match (verbose, log_level) {
        (true, None) => "qipu=debug",
        (false, None) => "qipu=warn",
        (_, Some(level)) => return init_with_level(level, log_json),
    };

    init_with_level(level, log_json)
}

fn init_with_level(level: &str, log_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Support QIPU_LOG environment variable override
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_from_env("QIPU_LOG"))
        .unwrap_or_else(|_| {
            EnvFilter::new(if level.contains('=') {
                level.to_string()
            } else {
                format!("qipu={}", level)
            })
        });

    let registry = tracing_subscriber::registry().with(filter);

    if log_json {
        registry
            .with(fmt::layer().json().with_ansi(false))
            .try_init()?;
    } else {
        registry
            .with(fmt::layer().compact().with_target(false).with_ansi(false))
            .try_init()?;
    }

    Ok(())
}
