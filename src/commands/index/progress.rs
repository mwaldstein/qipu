//! Progress tracking for indexing operations

use qipu_core::note::Note;
use std::time::Instant;

pub struct ProgressTracker {
    first_update_time: Option<Instant>,
    last_update_time: Option<Instant>,
    last_indexed: usize,
    notes_per_sec: f64,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            first_update_time: None,
            last_update_time: None,
            last_indexed: 0,
            notes_per_sec: 0.0,
        }
    }

    pub fn update(&mut self, indexed: usize, total: usize, note: &Note) {
        let now = Instant::now();

        if self.first_update_time.is_none() {
            self.first_update_time = Some(now);
        }

        if let Some(last_time) = self.last_update_time {
            let elapsed = now.duration_since(last_time).as_secs_f64();
            let indexed_delta = indexed - self.last_indexed;

            if elapsed > 0.0 && indexed_delta > 0 {
                self.notes_per_sec = indexed_delta as f64 / elapsed;
            }
        }

        self.last_update_time = Some(now);
        self.last_indexed = indexed;

        let percent = (indexed as f64 / total as f64) * 100.0;
        let remaining = total - indexed;

        let eta_str = if self.notes_per_sec > 0.0 {
            let eta_secs = remaining as f64 / self.notes_per_sec;
            if eta_secs < 1.0 {
                "1s".to_string()
            } else if eta_secs < 60.0 {
                format!("{:.0}s", eta_secs.ceil())
            } else {
                format!("{:.0}m {:.0}s", (eta_secs / 60.0).floor(), eta_secs % 60.0)
            }
        } else {
            "---".to_string()
        };

        let bar_width = 30;
        let filled = (bar_width as f64 * percent / 100.0) as usize;
        let filled = filled.min(bar_width);
        let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);

        eprintln!(
            "  [{}] {:.0}% ({} / {}) {:.0} notes/sec",
            bar, percent, indexed, total, self.notes_per_sec
        );
        eprintln!(
            "  ETA: {}  Last: {} \"{}\"",
            eta_str,
            note.id(),
            note.title()
        );
    }
}
