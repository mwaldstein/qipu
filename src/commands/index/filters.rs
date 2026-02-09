//! Filters for selective indexing

use qipu_core::error::{QipuError, Result};
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::time::SystemTime;

pub fn filter_quick_index(_store: &Store, notes: &[Note]) -> Vec<Note> {
    let mut mocs = Vec::new();
    let mut others: Vec<(SystemTime, Note)> = Vec::new();

    for note in notes {
        if note.note_type().is_moc() {
            mocs.push(note.clone());
        } else if let Some(path) = &note.path {
            if let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) {
                others.push((mtime, note.clone()));
            }
        }
    }

    others.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = mocs;
    for (_, note) in others.into_iter().take(100) {
        result.push(note);
    }

    result
}

pub fn filter_by_moc(store: &Store, notes: &[Note], moc_id: &str) -> Vec<Note> {
    let mut result = Vec::new();

    let moc = notes.iter().find(|n| n.id() == moc_id);
    if let Some(m) = moc {
        result.push(m.clone());

        let outbound_edges = store.db().get_outbound_edges(moc_id).unwrap_or_default();
        for edge in outbound_edges {
            if let Some(note) = notes.iter().find(|n| n.id() == edge.to) {
                result.push(note.clone());
            }
        }
    }

    result
}

pub fn filter_by_recent(notes: &[Note], n: usize) -> Vec<Note> {
    let mut notes_with_mtime: Vec<(SystemTime, Note)> = Vec::new();

    for note in notes {
        if let Some(path) = &note.path {
            if let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) {
                notes_with_mtime.push((mtime, note.clone()));
            }
        }
    }

    notes_with_mtime.sort_by(|a, b| b.0.cmp(&a.0));
    notes_with_mtime
        .into_iter()
        .take(n)
        .map(|(_, note)| note)
        .collect()
}

/// Parse a time string like "24 hours ago", "2 days ago", "1 week ago", or ISO 8601 timestamp
pub fn parse_modified_since(s: &str) -> Result<SystemTime> {
    use std::time::Duration;

    let now = SystemTime::now();
    let s_lower = s.to_lowercase();

    // Try to parse relative time expressions
    if s_lower.ends_with(" ago") {
        let parts: Vec<&str> = s_lower
            .trim_end_matches(" ago")
            .split_whitespace()
            .collect();
        if parts.len() == 2 {
            if let Ok(amount) = parts[0].parse::<u64>() {
                let duration = match parts[1] {
                    "second" | "seconds" => Duration::from_secs(amount),
                    "minute" | "minutes" => Duration::from_secs(amount * 60),
                    "hour" | "hours" => Duration::from_secs(amount * 60 * 60),
                    "day" | "days" => Duration::from_secs(amount * 24 * 60 * 60),
                    "week" | "weeks" => Duration::from_secs(amount * 7 * 24 * 60 * 60),
                    _ => {
                        return Err(QipuError::Other(format!(
                            "Unknown time unit: {}. Use seconds, minutes, hours, days, or weeks",
                            parts[1]
                        )));
                    }
                };
                return now.checked_sub(duration).ok_or_else(|| {
                    QipuError::Other("Invalid time duration: too far in the past".to_string())
                });
            }
        }
    }

    // Try ISO 8601 format
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(datetime.into());
    }

    // Try simpler ISO format (2024-01-15T10:30:00)
    if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(
            SystemTime::UNIX_EPOCH + Duration::from_secs(datetime.and_utc().timestamp() as u64)
        );
    }

    // Try date-only format (2024-01-15)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(
            SystemTime::UNIX_EPOCH + Duration::from_secs(datetime.and_utc().timestamp() as u64)
        );
    }

    Err(QipuError::Other(format!(
        "Cannot parse time: '{}'. Use formats like '24 hours ago', '2 days ago', '2024-01-15', or ISO 8601",
        s
    )))
}
