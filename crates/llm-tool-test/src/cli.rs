use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a test scenario
    Run {
        /// Path to scenario file or name
        #[arg(long, short)]
        scenario: Option<String>,

        /// Filter scenarios by tags
        #[arg(long)]
        tags: Vec<String>,

        /// Filter scenarios by tier (0=smoke, 1=quick, 2=standard, 3=comprehensive)
        #[arg(long, default_value = "0")]
        tier: usize,

        /// Tool to test (e.g., amp, claude-code, opencode)
        #[arg(long, default_value = "opencode")]
        tool: String,

        /// Model to use with the tool (e.g., claude-sonnet-4-20250514, gpt-4o)
        #[arg(long)]
        model: Option<String>,

        /// Multiple tools for matrix run (comma-separated, e.g., opencode,amp)
        #[arg(long)]
        tools: Option<String>,

        /// Multiple models for matrix run (comma-separated, e.g., claude-sonnet-4-20250514,gpt-4o)
        #[arg(long)]
        models: Option<String>,

        /// Maximum cost in USD
        #[arg(long)]
        max_usd: Option<f64>,

        /// Dry run (don't execute LLM calls)
        #[arg(long)]
        dry_run: bool,

        /// Disable caching
        #[arg(long)]
        no_cache: bool,

        /// Judge model for LLM-as-judge evaluation
        #[arg(long)]
        judge_model: Option<String>,

        /// Maximum execution time in seconds per command
        #[arg(long, default_value = "300")]
        timeout_secs: u64,
    },
    /// List available scenarios
    List {
        /// Filter by tags
        #[arg(long)]
        tags: Vec<String>,

        /// Filter scenarios by tier (0=smoke, 1=quick, 2=standard, 3=comprehensive)
        #[arg(long, default_value = "0")]
        tier: usize,

        /// Show runs pending human review
        #[arg(long)]
        pending_review: bool,
    },
    /// Show details of a scenario
    Show {
        /// Name of the scenario
        #[arg(required = true)]
        name: String,
    },
    /// Compare results of runs
    Compare {
        /// Run IDs to compare
        #[arg(required = true)]
        run_ids: Vec<String>,
    },
    /// Clean up artifacts
    Clean,
    /// Add human review to a run
    Review {
        /// Run ID to review
        #[arg(required = true)]
        run_id: String,

        /// Dimension scores (e.g., accuracy=0.9, clarity=0.8)
        #[arg(long, value_parser = parse_key_value)]
        dimension: Vec<(String, f64)>,

        /// Review notes
        #[arg(long)]
        notes: Option<String>,
    },
}

fn parse_key_value(s: &str) -> Result<(String, f64), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value pair: {}", s));
    }
    let key = parts[0].to_string();
    let value = parts[1]
        .parse::<f64>()
        .map_err(|e| format!("Invalid float value for key {}: {}", key, e))?;
    if !(0.0..=1.0).contains(&value) {
        return Err(format!("Score for key {} must be between 0.0 and 1.0", key));
    }
    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_value_valid() {
        let result = parse_key_value("accuracy=0.9").unwrap();
        assert_eq!(result.0, "accuracy");
        assert_eq!(result.1, 0.9);
    }

    #[test]
    fn test_parse_key_value_multiple_keys() {
        let result = parse_key_value("clarity=0.75").unwrap();
        assert_eq!(result.0, "clarity");
        assert_eq!(result.1, 0.75);
    }

    #[test]
    fn test_parse_key_value_no_equals() {
        let result = parse_key_value("accuracy");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid key=value pair"));
    }

    #[test]
    fn test_parse_key_value_multiple_equals() {
        let result = parse_key_value("accuracy=0.9=extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_key_value_invalid_float() {
        let result = parse_key_value("accuracy=abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid float value"));
    }

    #[test]
    fn test_parse_key_value_score_too_high() {
        let result = parse_key_value("accuracy=1.5");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_parse_key_value_score_negative() {
        let result = parse_key_value("accuracy=-0.1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_parse_key_value_score_zero() {
        let result = parse_key_value("accuracy=0.0").unwrap();
        assert_eq!(result.1, 0.0);
    }

    #[test]
    fn test_parse_key_value_score_one() {
        let result = parse_key_value("accuracy=1.0").unwrap();
        assert_eq!(result.1, 1.0);
    }

    #[test]
    fn test_parse_key_value_key_with_underscore() {
        let result = parse_key_value("first_try=0.8").unwrap();
        assert_eq!(result.0, "first_try");
        assert_eq!(result.1, 0.8);
    }
}
