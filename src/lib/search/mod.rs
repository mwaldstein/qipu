//! High-level search functionality for post-processing search results

use crate::lib::compaction::CompactionContext;
use crate::lib::index::SearchResult;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Process search results with compaction resolution, filtering, and sorting
#[tracing::instrument(skip(results, store, compaction_ctx, _compaction_note_map, cli), fields(results_count = results.len(), exclude_mocs, sort = ?sort))]
pub fn process_search_results(
    results: Vec<SearchResult>,
    cli: &crate::cli::Cli,
    store: &Store,
    compaction_ctx: &Option<CompactionContext>,
    _compaction_note_map: &Option<HashMap<&str, &crate::lib::note::Note>>,
    exclude_mocs: bool,
    sort: Option<&str>,
) -> (
    Vec<SearchResult>,
    HashMap<String, crate::lib::note::Note>,
    usize,
) {
    let mut results = results;

    let mut compacts_count = 0;

    if !cli.no_resolve_compaction {
        if let Some(ref ctx) = compaction_ctx {
            let mut canonical_results: HashMap<String, SearchResult> = HashMap::new();

            for mut result in results {
                let canonical_id = ctx.canon(&result.id);

                if let Ok(canonical_id) = canonical_id {
                    if canonical_id != result.id {
                        if let Ok(Some(digest_meta)) = store.db().get_note_metadata(&canonical_id) {
                            result.via = Some(result.id.clone());
                            result.id = canonical_id.clone();
                            result.title = digest_meta.title.clone();
                            result.note_type = digest_meta.note_type;
                            result.tags = digest_meta.tags.clone();
                            result.path = digest_meta.path.clone();
                        }
                    }

                    canonical_results
                        .entry(result.id.clone())
                        .and_modify(|existing| {
                            if result.relevance > existing.relevance {
                                *existing = result.clone();
                            } else if result.via.is_some() && existing.via.is_none() {
                                existing.via = result.via.clone();
                            }
                        })
                        .or_insert(result);
                }
            }

            let mut sorted_entries: Vec<_> = canonical_results.into_iter().collect();
            sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));
            results = sorted_entries
                .into_iter()
                .map(|(_, result)| result)
                .collect();

            results.sort_by(|a, b| {
                b.relevance
                    .partial_cmp(&a.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.id.cmp(&b.id))
            });
        }
    }

    if let Some(sort_field) = sort {
        if sort_field == "value" {
            results.sort_by(|a, b| {
                let value_a = a.value.unwrap_or(50);
                let value_b = b.value.unwrap_or(50);
                value_b.cmp(&value_a).then_with(|| a.id.cmp(&b.id))
            });
        }
    }

    if exclude_mocs {
        results.retain(|r| r.note_type != NoteType::Moc);
    }

    let mut notes_cache: HashMap<String, crate::lib::note::Note> = HashMap::new();
    if let Some(ref ctx) = compaction_ctx {
        for result in &results {
            let count = ctx.get_compacts_count(&result.id);
            compacts_count += count;
            if count > 0 && !notes_cache.contains_key(&result.id) {
                if let Ok(note) = store.get_note(&result.id) {
                    notes_cache.insert(result.id.clone(), note);
                }
            }
        }
    }

    (results, notes_cache, compacts_count)
}
