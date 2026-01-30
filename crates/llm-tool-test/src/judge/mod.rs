//! Judge module for LLM-as-judge evaluation.
//!
//! This module provides rubric-based evaluation functionality for assessing
//! LLM tool performance. Currently supports loading rubrics from YAML files
//! with future support for direct LLM API-based evaluation.
//!
//! # Example
//!
//! ```rust,no_run
//! use llm_tool_test::judge;
//! use std::path::Path;
//!
//! let rubric = judge::load_rubric(Path::new("rubrics/quality.yaml")).unwrap();
//! println!("Loaded rubric with {} criteria", rubric.criteria.len());
//! ```

pub mod eval;
pub mod rubric;
pub mod types;

pub use rubric::*;
pub use types::*;

#[cfg(test)]
mod tests;
