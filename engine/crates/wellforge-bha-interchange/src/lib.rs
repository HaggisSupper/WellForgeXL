//! Clean-room BHA XML-to-JSON interchange boundary.

mod error;
mod model;
mod project;
mod sanitize;
pub mod structural_json;
mod validate;
mod xml_tree;

pub use error::InterchangeError;
pub use model::{
    BhaAssembly, BhaComponentRecord, ComponentDetail, ComponentKind, MotorDetail,
    RotarySteerableDetail, StabilizerDetail, TubularSection,
};
pub use project::project_bha;
pub use sanitize::{SanitizationPolicy, SanitizationReport, sanitize_tree};
pub use xml_tree::{StructuralNode, parse_xml};

/// The sanitized structural document, canonical projection, and audit report
/// produced by [`convert_xml`].
#[derive(Debug, Clone, PartialEq)]
pub struct InterchangeOutput {
    /// Order-preserving sanitized XML tree.
    pub structural: StructuralNode,
    /// Validated neutral BHA assembly projection.
    pub canonical: BhaAssembly,
    /// Counts of metadata removed during sanitization.
    pub report: SanitizationReport,
}

/// Parse, sanitize, and project a neutral BHA XML document.
///
/// The caller supplies token fingerprints through `policy`; the converter
/// never needs plaintext restricted values. The canonical projection is built
/// from the sanitized tree so removed metadata cannot enter downstream data.
///
/// # Errors
///
/// Returns [`InterchangeError`] when parsing, sanitization, or canonical
/// projection validation fails.
pub fn convert_xml(
    input: &str,
    policy: &SanitizationPolicy,
) -> Result<InterchangeOutput, InterchangeError> {
    let tree = parse_xml(input)?;
    let (structural, report) = sanitize_tree(tree, policy)?;
    let canonical = project_bha(&structural)?;
    Ok(InterchangeOutput {
        structural,
        canonical,
        report,
    })
}
