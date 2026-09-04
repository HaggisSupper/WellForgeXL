//! Concrete cross-boundary contracts shared by WellForge capability crates.

use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors returned when a value cannot be represented by a finite SI type.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SiValueError {
    #[error("metres must be finite")]
    NonFiniteMetres,
    #[error("radians must be finite")]
    NonFiniteRadians,
}

/// A finite length in metres. Its JSON representation is a bare number.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Metres(f64);

impl Metres {
    pub fn try_new(value: f64) -> Result<Self, SiValueError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(SiValueError::NonFiniteMetres)
        }
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Metres {
    type Error = SiValueError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<Metres> for f64 {
    fn from(value: Metres) -> Self {
        value.get()
    }
}

impl<'de> Deserialize<'de> for Metres {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// A finite angle in radians. Its JSON representation is a bare number.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Radians(f64);

impl Radians {
    pub fn try_new(value: f64) -> Result<Self, SiValueError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(SiValueError::NonFiniteRadians)
        }
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Radians {
    type Error = SiValueError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<Radians> for f64 {
    fn from(value: Radians) -> Self {
        value.get()
    }
}

impl<'de> Deserialize<'de> for Radians {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// UI-selectable unit systems. Engineering values will remain SI internally.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    Oilfield,
    #[default]
    Si,
    Custom,
}

/// Persistable unit settings owned by the active project.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitPreferences {
    pub system: UnitSystem,
}

/// Persistable plot settings owned by the active project.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotPreferences {
    pub palette: String,
    pub show_risk_bands: bool,
}

/// A renderer-neutral plotting contract. Rust calculation crates return this
/// structure; the UI may render or export it but must not recreate its data.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotSpec {
    pub title: String,
    pub traces: Vec<PlotTrace>,
    pub bands: Vec<PlotBand>,
    pub annotations: Vec<PlotAnnotation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotTrace {
    pub id: String,
    pub name: String,
    pub points: Vec<PlotPoint>,
    pub layer: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotBand {
    pub id: String,
    pub lower: f64,
    pub upper: f64,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotAnnotation {
    pub text: String,
    pub point: PlotPoint,
}

impl Default for PlotPreferences {
    fn default() -> Self {
        Self {
            palette: "wellforge-dark".to_owned(),
            show_risk_bands: true,
        }
    }
}

/// A local WellForge project selected by the desktop shell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub path: String,
    pub name: String,
}

impl ProjectSummary {
    pub fn from_path(path: String) -> Self {
        let name = Path::new(&path)
            .file_name()
            .and_then(|component| component.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(&path)
            .to_owned();

        Self { path, name }
    }
}

/// A structured JSON error safe to return across the Tauri command boundary.
#[derive(Clone, Debug, Deserialize, Error, Eq, PartialEq, Serialize)]
#[error("{code}: {message}")]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Immutable provenance for one deterministic engineering calculation.
///
/// The receipt is created at the Rust calculation boundary. Presentation
/// clients may render it but must not generate or alter its output hash.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "CalculationReceiptWire")]
pub struct CalculationReceipt {
    algorithm: String,
    algorithm_version: String,
    input_revisions: Vec<InputRevision>,
    context: CalculationContext,
    output_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalculationReceiptWire {
    algorithm: String,
    algorithm_version: String,
    input_revisions: Vec<InputRevision>,
    context: CalculationContext,
    output_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputRevision {
    pub kind: String,
    pub id: String,
    pub content_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationContext {
    pub unit_system: String,
    pub crs: String,
    pub backend: CalculationBackend,
    pub actor_id: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalculationBackend {
    Cpu,
    Cuda,
    Vulkan,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ReceiptError {
    #[error("calculation algorithm must not be blank")]
    InvalidAlgorithm,
    #[error("calculation algorithm version must not be blank")]
    InvalidAlgorithmVersion,
    #[error("at least one input revision is required")]
    MissingInputRevision,
    #[error("input revision fields must be nonblank and contain a SHA-256 digest")]
    InvalidInputRevision,
    #[error("calculation context fields must be nonblank")]
    InvalidContext,
    #[error("calculation warning text must not be blank")]
    InvalidWarning,
    #[error("calculation output cannot be serialized")]
    OutputSerialization,
    #[error("calculation output hash must be a SHA-256 digest")]
    InvalidOutputHash,
}

impl CalculationReceipt {
    pub fn create(
        algorithm: impl Into<String>,
        algorithm_version: impl Into<String>,
        input_revisions: Vec<InputRevision>,
        context: CalculationContext,
        output: &Value,
    ) -> Result<Self, ReceiptError> {
        let algorithm = algorithm.into();
        let algorithm_version = algorithm_version.into();
        validate_receipt_fields(&algorithm, &algorithm_version, &input_revisions, &context)?;
        Self::from_parts(
            algorithm,
            algorithm_version,
            input_revisions,
            context,
            Self::canonical_json_sha256(output)?,
        )
    }

    pub fn canonical_json_sha256(value: &Value) -> Result<String, ReceiptError> {
        let canonical_output = canonicalize_json(value);
        let output_bytes =
            serde_json::to_vec(&canonical_output).map_err(|_| ReceiptError::OutputSerialization)?;
        Ok(hex_digest(&output_bytes))
    }

    pub fn verifies_output(&self, output: &Value) -> Result<bool, ReceiptError> {
        Ok(self.output_sha256 == Self::canonical_json_sha256(output)?)
    }

    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
    pub fn algorithm_version(&self) -> &str {
        &self.algorithm_version
    }
    pub fn input_revisions(&self) -> &[InputRevision] {
        &self.input_revisions
    }
    pub fn context(&self) -> &CalculationContext {
        &self.context
    }
    pub fn output_sha256(&self) -> &str {
        &self.output_sha256
    }

    fn from_parts(
        algorithm: String,
        algorithm_version: String,
        input_revisions: Vec<InputRevision>,
        context: CalculationContext,
        output_sha256: String,
    ) -> Result<Self, ReceiptError> {
        validate_receipt_fields(&algorithm, &algorithm_version, &input_revisions, &context)?;
        if !is_sha256(&output_sha256) {
            return Err(ReceiptError::InvalidOutputHash);
        }
        Ok(Self {
            algorithm,
            algorithm_version,
            input_revisions,
            context,
            output_sha256,
        })
    }
}

impl TryFrom<CalculationReceiptWire> for CalculationReceipt {
    type Error = ReceiptError;

    fn try_from(value: CalculationReceiptWire) -> Result<Self, Self::Error> {
        Self::from_parts(
            value.algorithm,
            value.algorithm_version,
            value.input_revisions,
            value.context,
            value.output_sha256,
        )
    }
}

fn validate_receipt_fields(
    algorithm: &str,
    algorithm_version: &str,
    input_revisions: &[InputRevision],
    context: &CalculationContext,
) -> Result<(), ReceiptError> {
    if algorithm.trim().is_empty() {
        return Err(ReceiptError::InvalidAlgorithm);
    }
    if algorithm_version.trim().is_empty() {
        return Err(ReceiptError::InvalidAlgorithmVersion);
    }
    if input_revisions.is_empty() {
        return Err(ReceiptError::MissingInputRevision);
    }
    if input_revisions.iter().any(|revision| {
        revision.kind.trim().is_empty()
            || revision.id.trim().is_empty()
            || !is_sha256(&revision.content_sha256)
    }) {
        return Err(ReceiptError::InvalidInputRevision);
    }
    if context.unit_system.trim().is_empty()
        || context.crs.trim().is_empty()
        || context.actor_id.trim().is_empty()
    {
        return Err(ReceiptError::InvalidContext);
    }
    if context
        .warnings
        .iter()
        .any(|warning| warning.trim().is_empty())
    {
        return Err(ReceiptError::InvalidWarning);
    }
    Ok(())
}

fn canonicalize_json(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_json).collect()),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), canonicalize_json(value)))
                .collect(),
        ),
        primitive => primitive.clone(),
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl ApiError {
    pub fn no_open_project() -> Self {
        Self {
            code: "NO_OPEN_PROJECT".to_owned(),
            message: "No project is currently open".to_owned(),
            details: None,
        }
    }

    pub fn invalid_project_path() -> Self {
        Self {
            code: "INVALID_PROJECT_PATH".to_owned(),
            message: "Project path must not be empty".to_owned(),
            details: None,
        }
    }
}
