//! Custom metadata filter expression parsing.

use std::collections::HashMap;
use std::sync::Arc;

use crate::bail_invalid;
use crate::error::{QipuError, Result};

pub type CustomFilterPredicate = Arc<dyn Fn(&HashMap<String, serde_yaml::Value>) -> bool + 'static>;

#[derive(Debug, Clone)]
enum ComparisonOp {
    GreaterEqual,
    Greater,
    LessEqual,
    Less,
}

fn is_iso_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

/// Parse a custom filter expression and return a predicate function.
///
/// Supported formats:
/// - Equality: `key=value`
/// - Existence: `key` (present), `!key` (absent)
/// - Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
/// - Date comparisons: `key>YYYY-MM-DD`, `key>=YYYY-MM-DD`, `key<YYYY-MM-DD`, `key<=YYYY-MM-DD`
pub fn parse_custom_filter_expression(expr: &str) -> Result<CustomFilterPredicate> {
    let expr = expr.trim();

    if let Some(key) = expr.strip_prefix('!') {
        return parse_absence_filter(key);
    }

    if !expr.contains(">=") && !expr.contains("<=") && !expr.contains('>') && !expr.contains('<') {
        if let Some((key, value)) = expr.split_once('=') {
            return parse_equality_filter(key, value);
        }
    }

    let Some((op, key, value)) = parse_comparison_parts(expr)? else {
        return parse_existence_filter(expr);
    };

    if key.is_empty() {
        bail_invalid!(
            &format!("custom filter expression '{}'", expr),
            "missing key"
        );
    }
    if value.is_empty() {
        bail_invalid!(
            &format!("custom filter expression '{}'", expr),
            "missing value"
        );
    }

    if is_iso_date(&value) {
        return Ok(date_comparison_filter(key, value, op));
    }

    let target_value: f64 = value.parse().map_err(|_| {
        QipuError::invalid_value(
            &format!("custom filter expression '{}'", expr),
            format!(
                "invalid numeric or date value '{}' (dates must be YYYY-MM-DD)",
                value
            ),
        )
    })?;

    Ok(numeric_comparison_filter(key, target_value, op))
}

fn parse_absence_filter(key: &str) -> Result<CustomFilterPredicate> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(QipuError::UsageError(
            "custom filter expression '!key' is missing key".to_string(),
        ));
    }
    Ok(Arc::new(
        move |custom: &HashMap<String, serde_yaml::Value>| !custom.contains_key(&key),
    ))
}

fn parse_existence_filter(expr: &str) -> Result<CustomFilterPredicate> {
    let key = expr.trim().to_string();
    if key.is_empty() {
        return Err(QipuError::UsageError(
            "custom filter expression is empty".to_string(),
        ));
    }
    Ok(Arc::new(
        move |custom: &HashMap<String, serde_yaml::Value>| custom.contains_key(&key),
    ))
}

fn parse_equality_filter(key: &str, value: &str) -> Result<CustomFilterPredicate> {
    let key = key.trim().to_string();
    let value = value.trim().to_string();
    if key.is_empty() {
        return Err(QipuError::UsageError(
            "custom filter expression 'key=value' is missing key".to_string(),
        ));
    }
    if value.is_empty() {
        return Err(QipuError::UsageError(
            "custom filter expression 'key=value' is missing value".to_string(),
        ));
    }
    Ok(Arc::new(
        move |custom: &HashMap<String, serde_yaml::Value>| {
            custom
                .get(&key)
                .map(|v| match v {
                    serde_yaml::Value::String(s) => s == &value,
                    serde_yaml::Value::Number(num) => num.to_string() == value,
                    serde_yaml::Value::Bool(b) => b.to_string() == value,
                    _ => false,
                })
                .unwrap_or(false)
        },
    ))
}

fn parse_comparison_parts(expr: &str) -> Result<Option<(ComparisonOp, String, String)>> {
    let comparison = if let Some((k, v)) = expr.split_once(">=") {
        Some((ComparisonOp::GreaterEqual, k.trim(), v.trim()))
    } else if let Some((k, v)) = expr.split_once('>') {
        Some((ComparisonOp::Greater, k.trim(), v.trim()))
    } else if let Some((k, v)) = expr.split_once("<=") {
        Some((ComparisonOp::LessEqual, k.trim(), v.trim()))
    } else if let Some((k, v)) = expr.split_once('<') {
        Some((ComparisonOp::Less, k.trim(), v.trim()))
    } else {
        None
    };

    Ok(comparison.map(|(op, key, value)| (op, key.to_string(), value.to_string())))
}

fn date_comparison_filter(key: String, value: String, op: ComparisonOp) -> CustomFilterPredicate {
    let compare_fn: fn(&str, &str) -> bool = match op {
        ComparisonOp::GreaterEqual => |a, b| a >= b,
        ComparisonOp::Greater => |a, b| a > b,
        ComparisonOp::LessEqual => |a, b| a <= b,
        ComparisonOp::Less => |a, b| a < b,
    };

    Arc::new(move |custom: &HashMap<String, serde_yaml::Value>| {
        custom
            .get(&key)
            .and_then(|v| match v {
                serde_yaml::Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|actual_value| compare_fn(actual_value, &value))
            .unwrap_or(false)
    })
}

fn numeric_comparison_filter(
    key: String,
    target_value: f64,
    op: ComparisonOp,
) -> CustomFilterPredicate {
    let compare_fn = match op {
        ComparisonOp::GreaterEqual => |a: f64, b: f64| a >= b,
        ComparisonOp::Greater => |a: f64, b: f64| a > b,
        ComparisonOp::LessEqual => |a: f64, b: f64| a <= b,
        ComparisonOp::Less => |a: f64, b: f64| a < b,
    };

    Arc::new(move |custom: &HashMap<String, serde_yaml::Value>| {
        custom
            .get(&key)
            .and_then(|v| match v {
                serde_yaml::Value::Number(num) => num.as_f64(),
                serde_yaml::Value::String(s) => s.parse::<f64>().ok(),
                serde_yaml::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            })
            .map(|actual_value| compare_fn(actual_value, target_value))
            .unwrap_or(false)
    })
}

pub fn matches_custom_filter(custom: &HashMap<String, serde_yaml::Value>, expr: &str) -> bool {
    parse_custom_filter_expression(expr)
        .map(|filter| filter(custom))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom(fields: &[(&str, serde_yaml::Value)]) -> HashMap<String, serde_yaml::Value> {
        fields
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn matches_date_comparisons() {
        let custom = custom(&[(
            "publication_date",
            serde_yaml::Value::String("2024-06-20".to_string()),
        )]);

        assert!(matches_custom_filter(
            &custom,
            "publication_date>=2024-06-01"
        ));
        assert!(matches_custom_filter(
            &custom,
            "publication_date<2024-07-01"
        ));
        assert!(!matches_custom_filter(
            &custom,
            "publication_date<2024-06-01"
        ));
    }

    #[test]
    fn invalid_numeric_comparison_is_parse_error() {
        assert!(parse_custom_filter_expression("priority>high").is_err());
    }
}
