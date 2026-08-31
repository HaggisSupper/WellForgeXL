//! Clean-room BHA XML-to-JSON interchange boundary.

mod error;
mod xml_tree;

pub use error::InterchangeError;
pub use xml_tree::{StructuralNode, parse_xml};
