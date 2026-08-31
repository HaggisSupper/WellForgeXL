//! Errors exposed by the BHA interchange boundary.

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum InterchangeError {
    #[error("invalid BHA XML: {0}")]
    InvalidXml(String),
    #[error("DTD and external entities are not permitted")]
    ProhibitedXmlConstruct,
    #[error("unsupported BHA XML root: {0}")]
    UnsupportedRoot(String),
    #[error("required field is missing: {0}")]
    MissingField(&'static str),
    #[error("invalid field {field}: {value}")]
    InvalidField { field: &'static str, value: String },
    #[error("invalid BHA geometry: {0}")]
    InvalidGeometry(String),
}
