//! Re-exports for backward compatibility
//!
//! Individual check functions are now organized by category:
//! - `structure::` - store directory structure checks
//! - `database::` - database consistency checks
//! - `content::` - note content validation checks

pub use crate::commands::doctor::content::*;
pub use crate::commands::doctor::database::*;
pub use crate::commands::doctor::structure::*;
