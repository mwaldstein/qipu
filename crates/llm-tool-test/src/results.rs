pub mod cache;
pub mod db;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod test_helpers;

pub use cache::Cache;
pub use db::ResultsDB;
pub use types::*;
pub use utils::{generate_run_id, get_qipu_version};

#[cfg(test)]
mod tests {
    // Tests have been moved to their respective submodule test modules:
    // - utils::tests for estimate_cost_from_tokens
    // - types::tests for CacheKey and ResultRecord
    // - db::tests for ResultsDB
}
