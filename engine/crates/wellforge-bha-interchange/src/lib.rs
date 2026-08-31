//! Clean-room BHA XML-to-JSON interchange boundary.

mod error;
mod sanitize;
pub mod structural_json;
mod xml_tree;

pub use error::InterchangeError;
pub use sanitize::{SanitizationPolicy, SanitizationReport, sanitize_tree};
pub use xml_tree::{StructuralNode, parse_xml};
