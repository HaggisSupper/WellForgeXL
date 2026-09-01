//! JSON projection of the order-preserving structural tree.

use serde_json::Value;

use crate::StructuralNode;

/// Serialize a structural tree while retaining child order.
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn to_value(tree: &StructuralNode) -> Value {
    serde_json::to_value(tree).expect("StructuralNode serialization is infallible")
}
