//! Registry primitives for canonical request/result schema identities.

use std::collections::BTreeMap;

use semver::Version;
use serde_json::Value;
use thiserror::Error;

use crate::{
    CompatibilityPolicy, ContractId, GovernanceError, SchemaFingerprint, fingerprint_schema,
};

/// Canonical identity and schema fingerprints for one contract version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDescriptor {
    /// Stable dotted contract identifier.
    pub id: ContractId,
    /// Canonical semantic version represented by these schemas.
    pub version: Version,
    /// Compatibility rule advertised by the contract family.
    pub compatibility: CompatibilityPolicy,
    /// Deterministic fingerprint of the request schema.
    pub request_schema_fingerprint: SchemaFingerprint,
    /// Deterministic fingerprint of the result schema.
    pub result_schema_fingerprint: SchemaFingerprint,
}

impl ContractDescriptor {
    /// Constructs one canonical descriptor from request and result JSON schemas.
    ///
    /// # Errors
    /// Returns a governance error when the contract ID or semantic version is malformed.
    pub fn new(
        id: &str,
        version: &str,
        compatibility: CompatibilityPolicy,
        request_schema: &Value,
        result_schema: &Value,
    ) -> Result<Self, GovernanceError> {
        Ok(Self {
            id: id.parse()?,
            version: Version::parse(version)
                .map_err(|_| GovernanceError::InvalidSemanticVersion)?,
            compatibility,
            request_schema_fingerprint: fingerprint_schema(request_schema),
            result_schema_fingerprint: fingerprint_schema(result_schema),
        })
    }
}

/// Registry insertion failure.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RegistryError {
    /// The same contract ID/version was registered with different canonical metadata.
    #[error("conflicting canonical contract registration")]
    Conflict,
}

/// Deterministic registry of canonical contract versions.
#[derive(Clone, Debug, Default)]
pub struct ContractRegistry {
    descriptors: BTreeMap<(ContractId, Version), ContractDescriptor>,
}

impl ContractRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            descriptors: BTreeMap::new(),
        }
    }

    /// Registers one descriptor. Re-registering the identical descriptor is idempotent.
    ///
    /// # Errors
    /// Returns [`RegistryError::Conflict`] when the ID/version already exists with different
    /// fingerprints or compatibility metadata.
    pub fn register(&mut self, descriptor: ContractDescriptor) -> Result<(), RegistryError> {
        let key = (descriptor.id.clone(), descriptor.version.clone());
        if let Some(existing) = self.descriptors.get(&key) {
            if existing == &descriptor {
                return Ok(());
            }
            return Err(RegistryError::Conflict);
        }
        self.descriptors.insert(key, descriptor);
        Ok(())
    }

    /// Returns the registered descriptor for an exact ID/version pair.
    #[must_use]
    pub fn get(&self, id: &ContractId, version: &Version) -> Option<&ContractDescriptor> {
        self.descriptors.get(&(id.clone(), version.clone()))
    }

    /// Number of registered canonical versions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Returns true when no contracts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    /// Iterates over registered descriptors in deterministic ID/version order.
    pub fn iter(&self) -> impl Iterator<Item = &ContractDescriptor> {
        self.descriptors.values()
    }
}
