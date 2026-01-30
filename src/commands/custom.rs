//! Custom metadata commands

use crate::cli::OutputFormat;
use crate::commands::format::output_by_format_result;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

/// Parse a value string using YAML for automatic type detection
///
/// This allows natural CLI usage:
/// - `1` → int: 1
/// - `3.14` → float: 3.14
/// - `true` → bool: true
/// - `hello` → string: "hello"
/// - `'[1, 2, 3]'` → array: [1, 2, 3]
/// - `'{"k": "v"}'` → object: {"k": "v"}
fn parse_custom_value(value: &str) -> serde_yaml::Value {
    // Try to parse as YAML value
    match serde_yaml::from_str(value) {
        Ok(v) => v,
        // If parsing fails, treat as a string
        Err(_) => serde_yaml::Value::String(value.to_string()),
    }
}

/// Set a custom metadata field on a note
pub fn set_custom_field(
    store: &Store,
    id_or_path: &str,
    key: &str,
    value: &str,
    format: OutputFormat,
    quiet: bool,
) -> Result<()> {
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

    let note_id = note.id().to_string();
    let parsed_value = parse_custom_value(value);

    // Insert the custom field
    note.frontmatter
        .custom
        .insert(key.to_string(), parsed_value.clone());

    // Save the note
    store.save_note(&mut note)?;

    if !quiet {
        // Display disclaimer on first use (only for human format)
        if matches!(format, OutputFormat::Human) {
            eprintln!();
            eprintln!("Note: Custom metadata is for applications building on qipu.");
            eprintln!(
                "For standard note management, use 'qipu value', 'qipu tag', or 'qipu link'."
            );
            eprintln!();
        }

        // Format the value for display
        let display_value = match &parsed_value {
            serde_yaml::Value::String(s) => s.clone(),
            _ => serde_yaml::to_string(&parsed_value)
                .unwrap_or_else(|_| value.to_string())
                .trim()
                .to_string(),
        };

        output_by_format_result!(format,
            json => {
                let output = serde_json::json!({
                    "id": note_id,
                    "key": key,
                    "value": parsed_value
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok::<(), QipuError>(())
            },
            human => {
                println!("Set {} custom.{} = {}", note_id, key, display_value);
            },
            records => {
                println!(
                    "T id=\"{}\" key=\"{}\" value={:?}",
                    note_id, key, display_value
                );
            }
        )?;
    }

    Ok(())
}

/// Get a custom metadata field from a note
pub fn get_custom_field(
    store: &Store,
    id_or_path: &str,
    key: &str,
    format: OutputFormat,
) -> Result<()> {
    let note = store.load_note_by_id_or_path(id_or_path)?;

    let note_id = note.id().to_string();

    match note.frontmatter.custom.get(key) {
        Some(value) => {
            output_by_format_result!(format,
                json => {
                    let output = serde_json::json!({
                        "id": note_id,
                        "key": key,
                        "value": value
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    Ok(())
                },
                human => {
                    // Format output based on value type
                    let output = match value {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(value)
                            .unwrap_or_else(|_| format!("{:?}", value))
                            .trim()
                            .to_string(),
                    };
                    println!("{}", output);
                },
                records => {
                    let output = match value {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(value)
                            .unwrap_or_else(|_| format!("{:?}", value))
                            .trim()
                            .to_string(),
                    };
                    println!("T id=\"{}\" key=\"{}\" value={:?}", note_id, key, output);
                }
            )
        }
        None => Err(QipuError::UsageError(format!(
            "Custom field '{}' not found on note {}",
            key, note_id
        ))),
    }
}

/// Show all custom metadata for a note
pub fn show_custom_fields(store: &Store, id_or_path: &str, format: OutputFormat) -> Result<()> {
    let note = store.load_note_by_id_or_path(id_or_path)?;

    let note_id = note.id().to_string();

    if note.frontmatter.custom.is_empty() {
        output_by_format_result!(format,
            json => {
                let output = serde_json::json!({
                    "id": note_id,
                    "custom": {}
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok(())
            },
            human => {
                println!("{}:", note_id);
                println!("  (no custom fields)");
            },
            records => {
                println!("T id=\"{}\"", note_id);
            }
        )
    } else {
        output_by_format_result!(format,
            json => {
                let output = serde_json::json!({
                    "id": note_id,
                    "custom": note.frontmatter.custom
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok(())
            },
            human => {
                println!("{}:", note_id);
                // Sort keys for deterministic output
                let mut keys: Vec<_> = note.frontmatter.custom.keys().collect();
                keys.sort();

                for key in keys {
                    let value = &note.frontmatter.custom[key];
                    let display_value = match value {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(value)
                            .unwrap_or_else(|_| format!("{:?}", value))
                            .trim()
                            .to_string(),
                    };
                    println!("  {}: {}", key, display_value);
                }
            },
            records => {
                // Sort keys for deterministic output
                let mut keys: Vec<_> = note.frontmatter.custom.keys().collect();
                keys.sort();

                println!("T id=\"{}\"", note_id);
                for key in keys {
                    let value = &note.frontmatter.custom[key];
                    let display_value = match value {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(value)
                            .unwrap_or_else(|_| format!("{:?}", value))
                            .trim()
                            .to_string(),
                    };
                    println!("F custom.{}={:?}", key, display_value);
                }
            }
        )
    }
}

/// Remove a custom metadata field from a note
pub fn unset_custom_field(
    store: &Store,
    id_or_path: &str,
    key: &str,
    format: OutputFormat,
    quiet: bool,
) -> Result<()> {
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

    let note_id = note.id().to_string();

    if note.frontmatter.custom.remove(key).is_some() {
        store.save_note(&mut note)?;
        if !quiet {
            output_by_format_result!(format,
                json => {
                    let output = serde_json::json!({
                        "id": note_id,
                        "key": key,
                        "removed": true
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    Ok::<(), QipuError>(())
                },
                human => {
                    println!("Removed {} custom.{}", note_id, key);
                },
                records => {
                    println!("T id=\"{}\" key=\"{}\" removed=true", note_id, key);
                }
            )?;
        }
        Ok(())
    } else {
        Err(QipuError::UsageError(format!(
            "Custom field '{}' not found on note {}",
            key, note_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_custom_value_numbers() {
        // Integers
        let val = parse_custom_value("1");
        assert_eq!(val, serde_yaml::Value::Number(1.into()));

        let val = parse_custom_value("-5");
        assert_eq!(val, serde_yaml::Value::Number((-5).into()));

        // Floats
        let val = parse_custom_value("3.14");
        assert!(matches!(val, serde_yaml::Value::Number(_)));
    }

    #[test]
    fn test_parse_custom_value_booleans() {
        let val = parse_custom_value("true");
        assert_eq!(val, serde_yaml::Value::Bool(true));

        let val = parse_custom_value("false");
        assert_eq!(val, serde_yaml::Value::Bool(false));
    }

    #[test]
    fn test_parse_custom_value_strings() {
        let val = parse_custom_value("hello");
        assert_eq!(val, serde_yaml::Value::String("hello".to_string()));

        let val = parse_custom_value("in-progress");
        assert_eq!(val, serde_yaml::Value::String("in-progress".to_string()));
    }

    #[test]
    fn test_parse_custom_value_null() {
        let val = parse_custom_value("null");
        assert_eq!(val, serde_yaml::Value::Null);
    }

    #[test]
    fn test_parse_custom_value_arrays() {
        let val = parse_custom_value("[1, 2, 3]");
        if let serde_yaml::Value::Sequence(seq) = val {
            assert_eq!(seq.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_custom_value_objects() {
        let val = parse_custom_value(r#"{"key": "value"}"#);
        if let serde_yaml::Value::Mapping(_) = val {
            // Success
        } else {
            panic!("Expected object");
        }
    }
}
