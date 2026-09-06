//! Canonical contract-governance primitives for `WellForge` engine boundaries.
//!
//! This crate owns domain-independent contract identity, semantic-version compatibility,
//! deterministic JSON normalization, and schema fingerprints. Calculation-domain crates retain
//! ownership of their request/result schemas and physics.

mod registry;

pub use registry::{ContractDescriptor, ContractRegistry, RegistryError};

use semver::Version;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Governance validation failure.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum GovernanceError {
    /// A canonical contract identifier is malformed.
    #[error("invalid canonical contract identifier")]
    InvalidContractId,
    /// A contract-version string is not valid semantic versioning.
    #[error("invalid semantic version")]
    InvalidSemanticVersion,
    /// A semantic version is valid but outside the registered compatibility policy.
    #[error("unsupported contract version")]
    UnsupportedVersion,
}

/// Validated dotted canonical contract identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractId(String);

impl ContractId {
    /// Returns the canonical identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ContractId {
    type Err = GovernanceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let valid = !value.is_empty()
            && value.split('.').all(|segment| {
                let mut characters = segment.chars();
                characters
                    .next()
                    .is_some_and(|first| first.is_ascii_lowercase())
                    && characters.all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '_'
                    })
            });
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(GovernanceError::InvalidContractId)
        }
    }
}

/// Compatibility rule applied to incoming semantic versions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompatibilityPolicy {
    /// Only the registered canonical version is accepted.
    Exact,
    /// Any valid semantic version sharing the registered major version is accepted.
    SameMajor,
    /// Only versions explicitly listed here are accepted.
    ExplicitSet(Vec<Version>),
}

/// Registered canonical version and its compatibility rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVersionPolicy {
    canonical: Version,
    compatibility: CompatibilityPolicy,
}

impl ContractVersionPolicy {
    /// Creates a contract-version policy.
    #[must_use]
    pub const fn new(canonical: Version, compatibility: CompatibilityPolicy) -> Self {
        Self {
            canonical,
            compatibility,
        }
    }

    /// Returns the registered canonical version.
    #[must_use]
    pub const fn canonical(&self) -> &Version {
        &self.canonical
    }

    /// Parses and validates one incoming semantic-version string.
    ///
    /// # Errors
    /// Returns [`GovernanceError::InvalidSemanticVersion`] for malformed `SemVer` and
    /// [`GovernanceError::UnsupportedVersion`] when the parsed version does not satisfy policy.
    pub fn validate_str(&self, value: &str) -> Result<Version, GovernanceError> {
        let version = Version::parse(value).map_err(|_| GovernanceError::InvalidSemanticVersion)?;
        let accepted = match &self.compatibility {
            CompatibilityPolicy::Exact => version == self.canonical,
            CompatibilityPolicy::SameMajor => version.major == self.canonical.major,
            CompatibilityPolicy::ExplicitSet(versions) => versions.contains(&version),
        };
        if accepted {
            Ok(version)
        } else {
            Err(GovernanceError::UnsupportedVersion)
        }
    }
}

fn parse_registered(value: &str) -> Result<Version, GovernanceError> {
    Version::parse(value).map_err(|_| GovernanceError::InvalidSemanticVersion)
}

/// Validates that `value` exactly matches one registered semantic version.
///
/// # Errors
/// Returns a governance error for malformed or unsupported versions.
pub fn validate_exact(value: &str, canonical: &str) -> Result<Version, GovernanceError> {
    ContractVersionPolicy::new(parse_registered(canonical)?, CompatibilityPolicy::Exact)
        .validate_str(value)
}

/// Validates that `value` shares the registered major semantic version.
///
/// # Errors
/// Returns a governance error for malformed or unsupported versions.
pub fn validate_same_major(value: &str, canonical: &str) -> Result<Version, GovernanceError> {
    ContractVersionPolicy::new(parse_registered(canonical)?, CompatibilityPolicy::SameMajor)
        .validate_str(value)
}

/// Validates that `value` belongs to an explicit registered semantic-version set.
///
/// # Errors
/// Returns a governance error for malformed or unsupported versions.
pub fn validate_explicit(value: &str, allowed: &[&str]) -> Result<Version, GovernanceError> {
    let versions = allowed
        .iter()
        .map(|registered| parse_registered(registered))
        .collect::<Result<Vec<_>, _>>()?;
    let canonical = versions
        .last()
        .cloned()
        .ok_or(GovernanceError::UnsupportedVersion)?;
    ContractVersionPolicy::new(canonical, CompatibilityPolicy::ExplicitSet(versions))
        .validate_str(value)
}

/// Stable SHA-256 fingerprint of a normalized canonical schema.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SchemaFingerprint([u8; 32]);

impl SchemaFingerprint {
    /// Returns the lower-case hexadecimal fingerprint.
    #[must_use]
    pub fn to_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(64);
        for byte in self.0 {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }
}

impl fmt::Display for SchemaFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

/// Recursively normalizes JSON object key ordering while preserving array order and scalar values.
#[must_use]
pub fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort_unstable();
            let normalized = keys
                .into_iter()
                .map(|key| (key.clone(), normalize_json(&object[key])))
                .collect::<Map<String, Value>>();
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.iter().map(normalize_json).collect()),
        _ => value.clone(),
    }
}

/// Computes a deterministic SHA-256 fingerprint over normalized compact JSON.
///
/// # Panics
/// Panics only if serializing an already-materialized [`serde_json::Value`] unexpectedly fails.
#[must_use]
pub fn fingerprint_schema(schema: &Value) -> SchemaFingerprint {
    let normalized = normalize_json(schema);
    let encoded =
        serde_json::to_vec(&normalized).expect("serializing serde_json::Value cannot fail");
    let digest = Sha256::digest(encoded);
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest);
    SchemaFingerprint(bytes)
}
