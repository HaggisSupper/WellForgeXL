//! Errors exposed by the BHA interchange boundary.

#[derive(Debug, thiserror::Error, PartialEq)]
/// Errors returned while converting BHA interchange documents.
pub enum InterchangeError {
    /// The XML document could not be parsed.
    #[error("invalid BHA XML: {0}")]
    InvalidXml(String),
    /// The document contains a DTD or external entity declaration.
    #[error("DTD and external entities are not permitted")]
    ProhibitedXmlConstruct,
    /// The XML root element is not supported by the interchange contract.
    #[error("unsupported BHA XML root: {0}")]
    UnsupportedRoot(String),
    /// A required interchange field was absent.
    #[error("required field is missing: {0}")]
    MissingField(&'static str),
    /// A field value failed validation.
    #[error("invalid field {field}: {value}")]
    InvalidField {
        /// The name of the field that failed validation.
        field: &'static str,
        /// The rejected field value.
        value: String,
    },
    /// The BHA geometry failed validation.
    #[error("invalid BHA geometry: {0}")]
    InvalidGeometry(String),
}
