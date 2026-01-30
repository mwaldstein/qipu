use crate::commands::context::types::SelectedNote;
use qipu_core::error::{QipuError, Result};
use qipu_core::note::{LinkType, Note};
use std::collections::{HashMap, HashSet};

/// Mutable state maintained during note selection
pub struct SelectionState<'a> {
    pub selected_notes: Vec<SelectedNote<'a>>,
    pub seen_ids: HashSet<String>,
    pub via_map: HashMap<String, String>,
}

impl<'a> SelectionState<'a> {
    pub fn new() -> Self {
        Self {
            selected_notes: Vec::new(),
            seen_ids: HashSet::new(),
            via_map: HashMap::new(),
        }
    }

    pub fn add_note(
        &mut self,
        _id: &str,
        resolved_id: String,
        note_map: &'a HashMap<&'a str, &'a Note>,
        via: Option<String>,
        link_type: Option<LinkType>,
    ) -> Result<()> {
        let via_for_map = via.as_ref().and_then(|v| {
            if v.starts_with("backlink:") || (v.contains(':') && !v.starts_with("walk:")) {
                Some(v.clone())
            } else {
                None
            }
        });

        let is_new = self.seen_ids.insert(resolved_id.clone());
        if is_new {
            let note =
                note_map
                    .get(resolved_id.as_str())
                    .ok_or_else(|| QipuError::NoteNotFound {
                        id: resolved_id.clone(),
                    })?;
            self.selected_notes.push(SelectedNote {
                note,
                via,
                link_type,
            });
        }

        if let Some(v) = via_for_map {
            self.via_map.entry(resolved_id).or_insert(v);
        }

        Ok(())
    }
}

/// Apply via_map to selected notes that have entries
pub fn apply_via_map(state: &mut SelectionState<'_>) {
    for selected in &mut state.selected_notes {
        if let Some(via) = state.via_map.get(selected.note.id()) {
            selected.via = Some(via.clone());
        }
    }
}
