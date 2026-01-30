use crate::cli::Cli;
use crate::commands::context::types::{ContextOptions, SelectedNote};
use qipu_core::note::LinkType;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

type CustomFilter = Arc<dyn Fn(&HashMap<String, serde_yaml::Value>) -> bool>;

/// Filter and sort selected notes based on min-value, custom filters, and sorting criteria
pub fn filter_and_sort_selected_notes(
    cli: &Cli,
    selected_notes: &mut Vec<SelectedNote<'_>>,
    options: &ContextOptions<'_>,
) {
    if let Some(min_value) = options.min_value {
        let before_count = selected_notes.len();
        selected_notes.retain(|selected| {
            let note_value = selected.note.frontmatter.value.unwrap_or(50);
            note_value >= min_value
        });
        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            debug!(
                min_value,
                before_count,
                after_count,
                filtered = before_count - after_count,
                "min_value_filter"
            );
        }
    }

    if !options.custom_filter.is_empty() {
        let before_count = selected_notes.len();

        let filters: Vec<CustomFilter> = options
            .custom_filter
            .iter()
            .map(|filter_expr| {
                crate::commands::context::filter::parse_custom_filter_expression(filter_expr)
            })
            .collect::<Result<_, _>>()
            .unwrap();

        selected_notes.retain(|selected| {
            filters
                .iter()
                .all(|filter| filter(&selected.note.frontmatter.custom))
        });

        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            debug!(
                filter_count = options.custom_filter.len(),
                before_count,
                after_count,
                filtered = before_count - after_count,
                "custom_filters"
            );
        }
    }

    selected_notes.sort_by(|a, b| {
        let a_verified = a.note.frontmatter.verified.unwrap_or(false);
        let b_verified = b.note.frontmatter.verified.unwrap_or(false);

        let link_priority = |link_type: &Option<LinkType>| -> u8 {
            match link_type {
                Some(lt) if lt.as_str() == "part-of" || lt.as_str() == "supports" => 0,
                Some(lt) if lt.as_str() != "related" => 1,
                Some(_) => 2,
                None => 1,
            }
        };

        let a_priority = link_priority(&a.link_type);
        let b_priority = link_priority(&b.link_type);

        b_verified
            .cmp(&a_verified)
            .then_with(|| a_priority.cmp(&b_priority))
            .then_with(
                || match (&a.note.frontmatter.created, &b.note.frontmatter.created) {
                    (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
            )
            .then_with(|| a.note.id().cmp(b.note.id()))
    });
}
