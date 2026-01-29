//! Field weighting constants for text indexing and search
//!
//! Per spec (specs/similarity-ranking.md):
//! - `title`: Weight 2.0
//! - `tags`: Weight 1.5
//! - `body`: Weight 1.0 (baseline)

/// Weight multiplier for title fields
pub const TITLE_WEIGHT: f64 = 2.0;

/// Weight multiplier for tag fields
pub const TAGS_WEIGHT: f64 = 1.5;

/// Weight multiplier for body fields (baseline)
pub const BODY_WEIGHT: f64 = 1.0;
