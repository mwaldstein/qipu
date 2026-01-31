//! Qipu Core Library
//!
//! Reusable library providing core domain logic for the Qipu knowledge management system.
//!
//! This library encapsulates data persistence, note management, graph operations,
//! indexing, and search functionality. It can be used independently or as part of
//! the qipu CLI application.

pub mod compaction;
pub mod config;
pub mod db;
pub mod error;
pub mod format;
pub mod git;
pub mod graph;
pub mod id;
pub mod index;
pub mod logging;
pub mod note;
pub mod ontology;
pub mod query;
pub mod records;
pub mod search;
pub mod similarity;
pub mod store;
pub mod telemetry;

mod text;
