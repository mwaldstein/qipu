//! Custom filter expression parsing for context command
//!
//! Supports:
//! - Equality: `key=value`
//! - Existence: `key` (present), `!key` (absent)
//! - Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`

use crate::lib::error::{QipuError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Comparison operators for custom filter expressions
#[derive(Debug, Clone)]
pub enum ComparisonOp {
    GreaterEqual,
    Greater,
    LessEqual,
    Less,
}

/// Parse a custom filter expression and return a predicate function
///
/// Supported formats:
/// - Equality: `key=value`
/// - Existence: `key` (present), `!key` (absent)
/// - Numeric comparisons: `key>n`, `key>=n`, `key<n`, `key<=n`
#[allow(clippy::type_complexity)]
pub fn parse_custom_filter_expression(
    expr: &str,
) -> Result<Arc<dyn Fn(&HashMap<String, serde_yaml::Value>) -> bool + 'static>> {
    let expr = expr.trim();

    // Check for absence (!key)
    if let Some(key) = expr.strip_prefix('!') {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(QipuError::Other(
                "custom filter expression '!key' is missing key".to_string(),
            ));
        }
        return Ok(Arc::new(
            move |custom: &HashMap<String, serde_yaml::Value>| !custom.contains_key(&key),
        ));
    }

    // Check for numeric comparisons (key>n, key>=n, key<n, key<=n) - must be checked before equality!
    let (_op_str, op, key, value) = if let Some((k, v)) = expr.split_once(">=") {
        (">=", ComparisonOp::GreaterEqual, k.trim(), v.trim())
    } else if let Some((k, v)) = expr.split_once(">") {
        (">", ComparisonOp::Greater, k.trim(), v.trim())
    } else if let Some((k, v)) = expr.split_once("<=") {
        ("<=", ComparisonOp::LessEqual, k.trim(), v.trim())
    } else if let Some((k, v)) = expr.split_once("<") {
        ("<", ComparisonOp::Less, k.trim(), v.trim())
    } else if let Some((k, v)) = expr.split_once('=') {
        // Equality check (key=value)
        let key = k.trim().to_string();
        let value = v.trim().to_string();
        if key.is_empty() {
            return Err(QipuError::Other(
                "custom filter expression 'key=value' is missing key".to_string(),
            ));
        }
        return Ok(Arc::new(
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
        ));
    } else {
        // No comparison operator found, check for existence
        let key = expr.trim().to_string();
        if key.is_empty() {
            return Err(QipuError::Other(
                "custom filter expression is empty".to_string(),
            ));
        }
        return Ok(Arc::new(
            move |custom: &HashMap<String, serde_yaml::Value>| custom.contains_key(&key),
        ));
    };

    let key = key.to_string();
    let value = value.to_string();

    if key.is_empty() {
        return Err(QipuError::Other(format!(
            "custom filter expression '{}' is missing key",
            expr
        )));
    }
    if value.is_empty() {
        return Err(QipuError::Other(format!(
            "custom filter expression '{}' is missing value",
            expr
        )));
    }

    let target_value: f64 = value.parse().map_err(|_| {
        QipuError::Other(format!(
            "custom filter expression '{}' has invalid numeric value '{}'",
            expr, value
        ))
    })?;

    let compare_fn = match op {
        ComparisonOp::GreaterEqual => |a: f64, b: f64| a >= b,
        ComparisonOp::Greater => |a: f64, b: f64| a > b,
        ComparisonOp::LessEqual => |a: f64, b: f64| a <= b,
        ComparisonOp::Less => |a: f64, b: f64| a < b,
    };

    Ok(Arc::new(
        move |custom: &HashMap<String, serde_yaml::Value>| {
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
        },
    ))
}
