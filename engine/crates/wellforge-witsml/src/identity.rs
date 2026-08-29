//! Source object identity and supported WITSML object types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

/// WITSML object types accepted by BHA Release 1.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WitsmlObjectType {
    /// Well.
    Well,
    /// Wellbore.
    Wellbore,
    /// Directional trajectory.
    Trajectory,
    /// Hole geometry.
    WellboreGeometry,
    /// Tubular/BHA configuration.
    Tubular,
    /// BHA run context.
    BhaRun,
    /// Log container.
    Log,
    /// Channel set.
    ChannelSet,
    /// Channel.
    Channel,
}

/// Immutable reference to an authoritative source object.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObjectRef {
    /// WITSML object UUID.
    pub uuid: Uuid,
    /// Optional absolute Energistics URI.
    pub uri: Option<String>,
    /// Supported object type.
    pub object_type: WitsmlObjectType,
    /// SHA-256 of normalized source content.
    pub content_hash: String,
    /// Human-readable citation only; never used as a relationship key.
    pub citation_name: String,
    /// Source system name/version.
    pub source_system: String,
}

/// Source identity validation failure.
#[derive(Debug, Error, PartialEq)]
pub enum IdentityError {
    /// UUID text could not be parsed.
    #[error("invalid WITSML UUID: {0}")]
    InvalidUuid(String),
    /// UUID must not be the nil UUID.
    #[error("WITSML UUID must not be nil")]
    NilUuid,
    /// URI is not absolute.
    #[error("invalid absolute source URI: {0}")]
    InvalidUri(String),
    /// Content hash is not a SHA-256 digest in the canonical wire format.
    #[error("invalid SHA-256 source content hash: {0}")]
    InvalidContentHash(String),
    /// Citation name is blank.
    #[error("source citation name must not be blank")]
    BlankCitationName,
    /// Source system is blank.
    #[error("source system must not be blank")]
    BlankSourceSystem,
}

fn validate_uri(candidate: &str) -> Result<(), IdentityError> {
    let parsed =
        Url::parse(candidate).map_err(|_| IdentityError::InvalidUri(candidate.to_owned()))?;
    if matches!(parsed.scheme(), "eml" | "http" | "https") {
        Ok(())
    } else {
        Err(IdentityError::InvalidUri(candidate.to_owned()))
    }
}

fn is_sha256_content_hash(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64 && digest.as_bytes().iter().all(u8::is_ascii_hexdigit)
    })
}

impl SourceObjectRef {
    /// Creates a validated source reference with blank provenance fields for later projection.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError`] when the UUID is invalid or nil, or when the optional URI is not
    /// absolute and supported.
    pub fn new(
        uuid: &str,
        object_type: WitsmlObjectType,
        uri: Option<&str>,
    ) -> Result<Self, IdentityError> {
        let uuid =
            Uuid::parse_str(uuid).map_err(|_| IdentityError::InvalidUuid(uuid.to_owned()))?;
        if uuid.is_nil() {
            return Err(IdentityError::NilUuid);
        }
        let uri = uri.map(str::to_owned);
        if let Some(candidate) = &uri {
            validate_uri(candidate)?;
        }
        Ok(Self {
            uuid,
            uri,
            object_type,
            content_hash: String::new(),
            citation_name: String::new(),
            source_system: String::new(),
        })
    }

    /// Validates the complete source identity and provenance required at a contract boundary.
    ///
    /// `new` supports staged construction during WITSML projection, so callers accepting a
    /// deserialized source reference must invoke this method after all provenance is present.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError`] when the identity, URI, normalized content hash or provenance
    /// fields are invalid.
    pub fn validate(&self) -> Result<(), IdentityError> {
        if self.uuid.is_nil() {
            return Err(IdentityError::NilUuid);
        }
        if let Some(uri) = &self.uri {
            validate_uri(uri)?;
        }
        if !is_sha256_content_hash(&self.content_hash) {
            return Err(IdentityError::InvalidContentHash(self.content_hash.clone()));
        }
        if self.citation_name.trim().is_empty() {
            return Err(IdentityError::BlankCitationName);
        }
        if self.source_system.trim().is_empty() {
            return Err(IdentityError::BlankSourceSystem);
        }
        Ok(())
    }
}
