//! Output format handling for qipu
//!
//! Supports three output formats per spec (specs/cli-tool.md):
//! - human: Readable, concise output for terminal use
//! - json: Stable, machine-readable JSON
//! - records: Line-oriented format optimized for LLM context injection

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::lib::error::QipuError;

/// Output format for qipu commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable output (default)
    #[default]
    Human,
    /// JSON output for machine consumption
    Json,
    /// Records output for LLM context injection
    Records,
}

impl FromStr for OutputFormat {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "records" => Ok(OutputFormat::Records),
            other => Err(QipuError::UnknownFormat(other.to_string())),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Human => write!(f, "human"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Records => write!(f, "records"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            "human".parse::<OutputFormat>().unwrap(),
            OutputFormat::Human
        );
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "records".parse::<OutputFormat>().unwrap(),
            OutputFormat::Records
        );
        assert_eq!(
            "HUMAN".parse::<OutputFormat>().unwrap(),
            OutputFormat::Human
        );
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    }

    #[test]
    fn test_unknown_format() {
        let err = "invalid".parse::<OutputFormat>().unwrap_err();
        assert!(matches!(err, QipuError::UnknownFormat(_)));
    }

    #[test]
    fn test_format_display() {
        assert_eq!(OutputFormat::Human.to_string(), "human");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Records.to_string(), "records");
    }
}
