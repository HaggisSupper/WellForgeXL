//! Removal of caller-identified restricted metadata from a structural tree.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{InterchangeError, StructuralNode};

/// SHA-256 fingerprints identifying tokens to remove.
#[derive(Debug, Clone, Default)]
pub struct SanitizationPolicy {
    fingerprints: BTreeSet<[u8; 32]>,
}

impl SanitizationPolicy {
    /// Construct a policy from caller-provided SHA-256 token fingerprints.
    #[must_use]
    pub fn new(fingerprints: BTreeSet<[u8; 32]>) -> Self {
        Self { fingerprints }
    }

    fn matches(&self, value: &str) -> bool {
        tokenize(value).any(|token| {
            let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
            self.fingerprints.contains(&digest)
        })
    }
}

/// Auditable counts of removed metadata, without retaining removed content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SanitizationReport {
    /// Number of elements and descendants removed.
    pub removed_elements: usize,
    /// Number of attributes removed.
    pub removed_attributes: usize,
    /// Number of scalar text values removed.
    pub removed_values: usize,
}

/// Sanitize a tree recursively according to a fingerprint policy.
///
/// # Errors
///
/// This currently returns no errors, but retains a fallible boundary for future
/// policy and tree validation.
pub fn sanitize_tree(
    mut tree: StructuralNode,
    policy: &SanitizationPolicy,
) -> Result<(StructuralNode, SanitizationReport), InterchangeError> {
    if policy.matches(&tree.name) {
        return Err(InterchangeError::SanitizedRoot);
    }
    let mut report = SanitizationReport::default();
    sanitize_node(&mut tree, policy, &mut report);
    Ok((tree, report))
}

fn sanitize_node(
    node: &mut StructuralNode,
    policy: &SanitizationPolicy,
    report: &mut SanitizationReport,
) {
    node.attributes.retain(|(name, value)| {
        let remove = policy.matches(name) || policy.matches(value);
        if remove {
            report.removed_attributes += 1;
        }
        !remove
    });
    if node.text.as_ref().is_some_and(|text| policy.matches(text)) {
        node.text = None;
        report.removed_values += 1;
    }
    let mut kept = Vec::with_capacity(node.children.len());
    for mut child in node.children.drain(..) {
        if policy.matches(&child.name) {
            report.removed_elements += 1;
        } else {
            sanitize_node(&mut child, policy, report);
            kept.push(child);
        }
    }
    node.children = kept;
}

fn tokenize(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
}
