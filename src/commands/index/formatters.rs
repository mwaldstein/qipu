//! Formatters for index command output

use crate::commands::format::FormatDispatcher;
use qipu_core::error::Result;
use qipu_core::store::Store;

pub struct IndexFormatter<'a> {
    pub store: &'a Store,
    pub notes_count: usize,
}

impl<'a> FormatDispatcher for IndexFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "status": "ok",
            "notes_indexed": self.notes_count,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("Indexed {} notes", self.notes_count);
    }

    fn output_records(&self) {
        println!(
            "H qipu=1 records=1 store={} mode=index notes={}",
            self.store.root().display(),
            self.notes_count
        );
    }
}

pub struct IndexStatusFormatter<'a> {
    pub store: &'a Store,
    pub db_count: i64,
    pub basic_count: i64,
    pub full_count: i64,
}

impl<'a> FormatDispatcher for IndexStatusFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "total_notes": self.db_count,
            "basic_indexed": self.basic_count,
            "full_indexed": self.full_count,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("Index Status");
        println!("-------------");
        println!("Total notes: {}", self.db_count);
        println!(
            "Basic indexed: {} ({})",
            self.basic_count,
            if self.db_count > 0 {
                format!(
                    "{:.0}%",
                    (self.basic_count as f64) / (self.db_count as f64) * 100.0
                )
            } else {
                "N/A".to_string()
            }
        );
        println!(
            "Full-text indexed: {} ({})",
            self.full_count,
            if self.db_count > 0 {
                format!(
                    "{:.0}%",
                    (self.full_count as f64) / (self.db_count as f64) * 100.0
                )
            } else {
                "N/A".to_string()
            }
        );
    }

    fn output_records(&self) {
        println!(
            "H qipu=1 records=1 store={} mode=status total={} basic={} full={}",
            self.store.root().display(),
            self.db_count,
            self.basic_count,
            self.full_count
        );
    }
}
