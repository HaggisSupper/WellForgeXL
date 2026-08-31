//! Clean-room BHA XML-to-JSON interchange boundary.

mod error;
mod model;
mod project;
mod sanitize;
pub mod structural_json;
mod validate;
mod xml_tree;

pub use error::InterchangeError;
pub use model::{BhaAssembly, BhaComponentRecord, ComponentDetail, ComponentKind, TubularSection};
pub use project::project_bha;
pub use sanitize::{SanitizationPolicy, SanitizationReport, sanitize_tree};
pub use xml_tree::{StructuralNode, parse_xml};
