//! Query and filtering utilities for notes

pub mod custom_filter;
pub mod filter;

pub use custom_filter::{
    matches_custom_filter, parse_custom_filter_expression, CustomFilterPredicate,
};
pub use filter::NoteFilter;

#[cfg(test)]
mod tests;
