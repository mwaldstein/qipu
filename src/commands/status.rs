//! `qipu status` readiness probe.

use std::path::{Path, PathBuf};

use crate::cli::{Cli, OutputFormat};
use crate::commands::context::path_relative_to_cwd;
use qipu_core::config::StoreConfig;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::paths::{
    discover_store, ATTACHMENTS_DIR, CONFIG_FILE, MOCS_DIR, NOTES_DIR, TEMPLATES_DIR,
};
use rusqlite::{Connection, OpenFlags};

struct StatusReport {
    ready: bool,
    store_found: bool,
    store: Option<String>,
    schema_version: Option<i64>,
    notes: Option<i64>,
    basic_indexed: Option<i64>,
    full_indexed: Option<i64>,
    warnings: Vec<String>,
}

pub fn execute(cli: &Cli, root: &Path) -> Result<()> {
    match collect_status(cli, root) {
        Ok(report) => {
            print_status(cli.format, &report)?;
            Ok(())
        }
        Err((report, error)) => {
            print_status(cli.format, &report)?;
            Err(error)
        }
    }
}

fn collect_status(
    cli: &Cli,
    root: &Path,
) -> std::result::Result<StatusReport, (StatusReport, QipuError)> {
    let store_path = match resolve_store_path(cli, root) {
        Ok(path) => path,
        Err(error) => {
            return Err((
                StatusReport {
                    ready: false,
                    store_found: false,
                    store: None,
                    schema_version: None,
                    notes: None,
                    basic_indexed: None,
                    full_indexed: None,
                    warnings: vec!["run `qipu init` to create a store".to_string()],
                },
                error,
            ));
        }
    };

    let store_display = path_relative_to_cwd(&store_path);
    let warnings = Vec::new();

    if !store_path.exists() {
        return Err((
            StatusReport {
                ready: false,
                store_found: false,
                store: None,
                schema_version: None,
                notes: None,
                basic_indexed: None,
                full_indexed: None,
                warnings: vec!["run `qipu init` to create a store".to_string()],
            },
            QipuError::StoreNotFound {
                search_root: root.to_path_buf(),
            },
        ));
    }

    if let Err(reason) = validate_layout(&store_path) {
        return Err((
            StatusReport {
                ready: false,
                store_found: true,
                store: Some(store_display),
                schema_version: None,
                notes: None,
                basic_indexed: None,
                full_indexed: None,
                warnings,
            },
            QipuError::InvalidStore { reason },
        ));
    }

    if let Err(error) = read_config(&store_path) {
        return Err((
            StatusReport {
                ready: false,
                store_found: true,
                store: Some(store_display),
                schema_version: None,
                notes: None,
                basic_indexed: None,
                full_indexed: None,
                warnings,
            },
            QipuError::InvalidStore {
                reason: format!("config is unreadable: {}", error),
            },
        ));
    }

    let database = match inspect_database(&store_path) {
        Ok(database) => database,
        Err(error) => {
            return Err((
                StatusReport {
                    ready: false,
                    store_found: true,
                    store: Some(store_display),
                    schema_version: None,
                    notes: None,
                    basic_indexed: None,
                    full_indexed: None,
                    warnings,
                },
                QipuError::InvalidStore {
                    reason: error.to_string(),
                },
            ));
        }
    };

    Ok(StatusReport {
        ready: true,
        store_found: true,
        store: Some(store_display),
        schema_version: Some(database.schema_version),
        notes: Some(database.notes),
        basic_indexed: Some(database.basic_indexed),
        full_indexed: Some(database.full_indexed),
        warnings,
    })
}

fn resolve_store_path(cli: &Cli, root: &Path) -> Result<PathBuf> {
    let base_store = if let Some(path) = &cli.store {
        if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        }
    } else {
        discover_store(root)?
    };

    if let Some(workspace_name) = &cli.workspace {
        Ok(base_store.join("workspaces").join(workspace_name))
    } else {
        Ok(base_store)
    }
}

fn validate_layout(store_path: &Path) -> std::result::Result<(), String> {
    if !store_path.is_dir() {
        return Err(format!(
            "store path is not a directory: {}",
            store_path.display()
        ));
    }

    let mut missing = Vec::new();
    for dir_name in [NOTES_DIR, MOCS_DIR, ATTACHMENTS_DIR, TEMPLATES_DIR] {
        if !store_path.join(dir_name).is_dir() {
            missing.push(dir_name);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing required store dirs: {} (store_root={})",
            missing.join(", "),
            store_path.display()
        ))
    }
}

fn read_config(store_path: &Path) -> Result<()> {
    let config_path = store_path.join(CONFIG_FILE);
    if config_path.exists() {
        StoreConfig::load(&config_path)?;
    }
    Ok(())
}

struct DatabaseStatus {
    schema_version: i64,
    notes: i64,
    basic_indexed: i64,
    full_indexed: i64,
}

fn inspect_database(store_path: &Path) -> Result<DatabaseStatus> {
    let db_path = store_path.join("qipu.db");
    if !db_path.is_file() {
        return Err(QipuError::InvalidStore {
            reason: format!("database is missing: {}", db_path.display()),
        });
    }

    let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| QipuError::Other(format!("failed to open database read-only: {}", e)))?;

    let schema_version = conn
        .query_row(
            "SELECT value FROM index_meta WHERE key = 'schema_version'",
            [],
            |row| {
                let value: String = row.get(0)?;
                Ok(value.parse().unwrap_or(6))
            },
        )
        .map_err(|e| QipuError::Other(format!("failed to get schema version: {}", e)))?;
    let notes = query_count(&conn, "SELECT COUNT(*) FROM notes", "get note count")?;
    let basic_indexed = query_count(
        &conn,
        "SELECT COUNT(*) FROM notes WHERE index_level = 1",
        "count basic indexed notes",
    )?;
    let full_indexed = query_count(
        &conn,
        "SELECT COUNT(*) FROM notes WHERE index_level = 2",
        "count full indexed notes",
    )?;

    Ok(DatabaseStatus {
        schema_version,
        notes,
        basic_indexed,
        full_indexed,
    })
}

fn query_count(conn: &Connection, sql: &str, action: &str) -> Result<i64> {
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|e| QipuError::Other(format!("failed to {}: {}", action, e)))
}

fn print_status(format: OutputFormat, report: &StatusReport) -> Result<()> {
    match format {
        OutputFormat::Json => print_json(report),
        OutputFormat::Human => {
            print_human(report);
            Ok(())
        }
        OutputFormat::Records => {
            print_records(report);
            Ok(())
        }
    }
}

fn print_json(report: &StatusReport) -> Result<()> {
    let output = serde_json::json!({
        "ready": report.ready,
        "store_found": report.store_found,
        "store": report.store,
        "database": {
            "schema_version": report.schema_version,
        },
        "notes": report.notes,
        "index": {
            "total_notes": report.notes,
            "basic_indexed": report.basic_indexed,
            "full_indexed": report.full_indexed,
        },
        "warnings": report.warnings,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_human(report: &StatusReport) {
    println!("Ready: {}", if report.ready { "yes" } else { "no" });

    match &report.store {
        Some(store) => println!("Store: {}", store),
        None => println!("Store: not found"),
    }

    if let Some(notes) = report.notes {
        println!("Notes: {}", notes);
    }

    if report.basic_indexed.is_some() || report.full_indexed.is_some() {
        println!(
            "Index: total={} basic={} full={}",
            report.notes.unwrap_or(0),
            report.basic_indexed.unwrap_or(0),
            report.full_indexed.unwrap_or(0)
        );
    }

    if let Some(schema_version) = report.schema_version {
        println!("Database: schema_version={}", schema_version);
    }

    if !report.ready && !report.store_found {
        println!("Next: qipu init");
    }

    for warning in &report.warnings {
        println!("Warning: {}", warning);
    }
}

fn print_records(report: &StatusReport) {
    let store = report.store.as_deref().unwrap_or("-");
    println!(
        "H qipu=1 records=1 mode=status ready={} store_found={} store={}",
        report.ready, report.store_found, store
    );
    println!(
        "D database.schema_version={} notes={} index.total={} index.basic={} index.full={}",
        report
            .schema_version
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        report
            .notes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        report
            .notes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        report
            .basic_indexed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        report
            .full_indexed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    for warning in &report.warnings {
        println!("W {}", warning);
    }
}
