use crate::index::types::NoteMetadata;
use crate::index::{Edge, Index};

/// Trait for providing graph adjacency and metadata
pub trait GraphProvider {
    fn get_outbound_edges(&self, id: &str) -> Vec<Edge>;
    fn get_inbound_edges(&self, id: &str) -> Vec<Edge>;
    fn get_metadata(&self, id: &str) -> Option<NoteMetadata>;
}

impl GraphProvider for Index {
    fn get_outbound_edges(&self, id: &str) -> Vec<Edge> {
        self.get_outbound_edges(id).into_iter().cloned().collect()
    }

    fn get_inbound_edges(&self, id: &str) -> Vec<Edge> {
        self.get_inbound_edges(id).into_iter().cloned().collect()
    }

    fn get_metadata(&self, id: &str) -> Option<NoteMetadata> {
        self.get_metadata(id).cloned()
    }
}
