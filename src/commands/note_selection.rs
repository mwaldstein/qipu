//! Shared note selection for commands that operate on note sets.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use qipu_core::error::{QipuError, Result};
use qipu_core::graph::{Direction, HopCost, TreeOptions};
use qipu_core::index::{Index, LinkSource};
use qipu_core::note::Note;
use qipu_core::store::Store;
use qipu_core::text::markdown::{
    extract_qipu_id_from_target, is_external_or_anchor_target, markdown_links,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptySelection {
    Error,
    FullStore,
}

pub struct NoteSelection<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub query_limit: usize,
    pub empty_selection: EmptySelection,
    pub traversal: Option<TraversalSelection<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TraversalSelection<'a> {
    pub direction: Direction,
    pub max_hops: u32,
    pub type_include: &'a [String],
    pub typed_only: bool,
    pub inline_only: bool,
}

impl NoteSelection<'_> {
    fn has_selector(&self) -> bool {
        !self.note_ids.is_empty()
            || self.tag.is_some()
            || self.moc_id.is_some()
            || self.query.is_some()
    }
}

pub fn collect_notes(
    store: &Store,
    index: &Index,
    all_notes: &[Note],
    selection: &NoteSelection,
) -> Result<Vec<Note>> {
    let mut selected_notes = Vec::new();
    let mut seen_ids = HashSet::new();

    for id in selection.note_ids {
        add_note(store, &mut selected_notes, &mut seen_ids, id)?;
    }

    if let Some(tag_name) = selection.tag {
        for note in all_notes {
            if note.frontmatter.tags.iter().any(|tag| tag == tag_name) {
                add_note(store, &mut selected_notes, &mut seen_ids, note.id())?;
            }
        }
    }

    if let Some(moc_id) = selection.moc_id {
        for note in get_moc_linked_notes(store, index, moc_id)? {
            if seen_ids.insert(note.id().to_string()) {
                selected_notes.push(note);
            }
        }
    }

    if let Some(query) = selection.query {
        let results = store.db().search(
            query,
            None,
            None,
            None,
            None,
            selection.query_limit,
            &store.config().search,
        )?;
        for result in results {
            add_note(store, &mut selected_notes, &mut seen_ids, &result.id)?;
        }
    }

    if !selection.has_selector() {
        match selection.empty_selection {
            EmptySelection::Error => {
                return Err(QipuError::Other(
                    "no selection criteria provided. Use --note, --tag, --moc, or --query"
                        .to_string(),
                ))
            }
            EmptySelection::FullStore => {
                for note in all_notes {
                    add_note(store, &mut selected_notes, &mut seen_ids, note.id())?;
                }
            }
        }
    }

    if let Some(traversal) = selection.traversal {
        if traversal.max_hops > 0 && !selected_notes.is_empty() {
            expand_by_traversal(store, index, traversal, &mut selected_notes, &mut seen_ids)?;
        }
    }

    Ok(selected_notes)
}

pub fn sort_notes_by_created_id(notes: &mut [Note]) {
    notes.sort_by(|a, b| {
        match (&a.frontmatter.created, &b.frontmatter.created) {
            (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.id().cmp(b.id()))
    });
}

fn add_note(
    store: &Store,
    selected_notes: &mut Vec<Note>,
    seen_ids: &mut HashSet<String>,
    id: &str,
) -> Result<()> {
    if seen_ids.insert(id.to_string()) {
        match store.get_note(id) {
            Ok(note) => selected_notes.push(note),
            Err(_) => return Err(QipuError::NoteNotFound { id: id.to_string() }),
        }
    }
    Ok(())
}

fn get_moc_linked_notes(store: &Store, index: &Index, root_id: &str) -> Result<Vec<Note>> {
    let root = store.get_note(root_id)?;
    // `--moc` is a legacy flag name. The selector treats the supplied note as a
    // linked collection root and returns the root plus its outbound children.
    MocLinkResolver::new(store, index).linked_notes(&root)
}

struct MocLinkResolver<'a> {
    store: &'a Store,
    index: &'a Index,
    path_to_id: HashMap<PathBuf, String>,
}

impl<'a> MocLinkResolver<'a> {
    fn new(store: &'a Store, index: &'a Index) -> Self {
        let path_to_id = index
            .metadata
            .values()
            .map(|meta| {
                (
                    normalize_path(store.root().join(&meta.path)),
                    meta.id.clone(),
                )
            })
            .collect();

        Self {
            store,
            index,
            path_to_id,
        }
    }

    fn linked_notes(&self, moc: &Note) -> Result<Vec<Note>> {
        let mut linked_notes = Vec::new();
        let mut seen_ids = HashSet::new();

        self.add_linked_note(moc.id(), &mut seen_ids, &mut linked_notes);
        self.extract_typed_links(moc, &mut seen_ids, &mut linked_notes);
        self.extract_wiki_links(moc, &mut seen_ids, &mut linked_notes)?;
        self.extract_markdown_links(moc, &mut seen_ids, &mut linked_notes);

        Ok(linked_notes)
    }

    fn add_linked_note(
        &self,
        id: &str,
        seen_ids: &mut HashSet<String>,
        linked_notes: &mut Vec<Note>,
    ) {
        if !id.is_empty() && seen_ids.insert(id.to_string()) && self.index.contains(id) {
            if let Ok(note) = self.store.get_note(id) {
                linked_notes.push(note);
            }
        }
    }

    fn extract_typed_links(
        &self,
        moc: &Note,
        seen_ids: &mut HashSet<String>,
        linked_notes: &mut Vec<Note>,
    ) {
        for typed_link in &moc.frontmatter.links {
            self.add_linked_note(&typed_link.id, seen_ids, linked_notes);
        }
    }

    fn extract_wiki_links(
        &self,
        moc: &Note,
        seen_ids: &mut HashSet<String>,
        linked_notes: &mut Vec<Note>,
    ) -> Result<()> {
        use regex::Regex;

        let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").map_err(|e| {
            QipuError::FailedOperation {
                operation: "compile wiki link regex".to_string(),
                reason: e.to_string(),
            }
        })?;

        for cap in wiki_link_re.captures_iter(&moc.body) {
            self.add_linked_note(cap[1].trim(), seen_ids, linked_notes);
        }

        Ok(())
    }

    fn extract_markdown_links(
        &self,
        moc: &Note,
        seen_ids: &mut HashSet<String>,
        linked_notes: &mut Vec<Note>,
    ) {
        for link in markdown_links(&moc.body) {
            let target = link.target.as_str();
            if is_external_or_anchor_target(target) {
                continue;
            }

            let Some(id) = extract_qipu_id_from_target(target)
                .or_else(|| self.resolve_relative_target(target, moc))
            else {
                continue;
            };

            self.add_linked_note(&id, seen_ids, linked_notes);
        }
    }

    fn resolve_relative_target(&self, target: &str, source_note: &Note) -> Option<String> {
        if !target.ends_with(".md") {
            return None;
        }

        let source_path = source_note.path.as_ref()?;
        let source_dir = source_path.parent()?;
        let target_path = normalize_path(source_dir.join(target));

        self.path_to_id.get(&target_path).cloned()
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn expand_by_traversal(
    store: &Store,
    index: &Index,
    traversal: TraversalSelection,
    selected_notes: &mut Vec<Note>,
    seen_ids: &mut HashSet<String>,
) -> Result<()> {
    let initial_ids: Vec<String> = selected_notes.iter().map(|n| n.id().to_string()).collect();
    let opts = TreeOptions {
        direction: traversal.direction,
        max_hops: HopCost::from(traversal.max_hops),
        type_include: traversal.type_include,
        type_exclude: Vec::new(),
        typed_only: traversal.typed_only,
        inline_only: traversal.inline_only,
        max_nodes: None,
        max_edges: None,
        max_fanout: None,
        max_chars: None,
        semantic_inversion: true,
        min_value: None,
        ignore_value: false,
    };

    for initial_id in &initial_ids {
        perform_simple_traversal(index, initial_id, &opts, store, selected_notes, seen_ids)?;
    }

    Ok(())
}

fn perform_simple_traversal(
    index: &Index,
    root: &str,
    opts: &TreeOptions,
    store: &Store,
    selected_notes: &mut Vec<Note>,
    seen_ids: &mut HashSet<String>,
) -> Result<()> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back((root.to_string(), HopCost::from(0)));
    visited.insert(root.to_string());

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        for edge in &index.edges {
            if !edge_matches_traversal(edge, &current_id, opts) {
                continue;
            }

            let neighbor_id = if edge.from == current_id {
                &edge.to
            } else {
                &edge.from
            };

            if visited.insert(neighbor_id.clone()) {
                let edge_cost =
                    qipu_core::graph::get_link_type_cost(edge.link_type.as_str(), store.config());
                queue.push_back((neighbor_id.clone(), accumulated_cost + edge_cost));
                add_note(store, selected_notes, seen_ids, neighbor_id)?;
            }
        }
    }

    Ok(())
}

fn edge_matches_traversal(
    edge: &qipu_core::index::Edge,
    current_id: &str,
    opts: &TreeOptions,
) -> bool {
    let direction_matches = match opts.direction {
        Direction::Out => edge.from == current_id,
        Direction::In => edge.to == current_id,
        Direction::Both => edge.from == current_id || edge.to == current_id,
    };
    if !direction_matches {
        return false;
    }

    if !opts.type_include.is_empty()
        && !opts
            .type_include
            .iter()
            .any(|t| t == edge.link_type.as_str())
    {
        return false;
    }

    let is_inline = matches!(edge.source, LinkSource::Inline);
    if opts.inline_only && !is_inline {
        return false;
    }
    if opts.typed_only && is_inline {
        return false;
    }

    true
}
