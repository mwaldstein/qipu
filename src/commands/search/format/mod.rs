//! Search command output formatting modules

pub mod human;
pub mod json;
pub mod records;

pub use human::output_human;
pub use json::output_json;
pub use records::output_records;
