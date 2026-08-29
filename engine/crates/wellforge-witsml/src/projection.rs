//! Offline projection of supported WITSML 2.x XML object identity.

use quick_xml::{Reader, XmlVersion, events::Event};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{SourceObjectRef, WitsmlObjectType};

/// WITSML XML projection failure.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// XML is not well-formed.
    #[error("invalid XML: {0}")]
    InvalidXml(String),
    /// Root object type is outside the Release 1 subset.
    #[error("unsupported WITSML object root: {0}")]
    UnsupportedRoot(String),
    /// Required UUID is absent.
    #[error("WITSML object UUID is missing")]
    MissingUuid,
    /// UUID is invalid.
    #[error(transparent)]
    Identity(#[from] crate::IdentityError),
}

fn object_type(local_name: &[u8]) -> Option<WitsmlObjectType> {
    match local_name {
        b"Well" => Some(WitsmlObjectType::Well),
        b"Wellbore" => Some(WitsmlObjectType::Wellbore),
        b"Trajectory" => Some(WitsmlObjectType::Trajectory),
        b"WellboreGeometry" => Some(WitsmlObjectType::WellboreGeometry),
        b"Tubular" => Some(WitsmlObjectType::Tubular),
        b"BhaRun" => Some(WitsmlObjectType::BhaRun),
        b"Log" => Some(WitsmlObjectType::Log),
        b"ChannelSet" => Some(WitsmlObjectType::ChannelSet),
        b"Channel" => Some(WitsmlObjectType::Channel),
        _ => None,
    }
}

/// Projects the authoritative identity of one supported WITSML XML object.
///
/// This deliberately does not claim full WITSML schema validation or ETP transport.
///
/// # Errors
///
/// Returns [`ProjectionError`] for malformed XML, unsupported roots, missing UUIDs or invalid identity fields.
pub fn project_source_identity(
    xml: &str,
    source_system: &str,
) -> Result<SourceObjectRef, ProjectionError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut citation = String::new();
    let mut root_type = None;
    let mut uuid = None;
    let mut in_name = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) if root_type.is_none() => {
                let local = element.local_name();
                root_type = object_type(local.as_ref());
                if root_type.is_none() {
                    return Err(ProjectionError::UnsupportedRoot(
                        String::from_utf8_lossy(local.as_ref()).into_owned(),
                    ));
                }
                for attribute in element.attributes().with_checks(true) {
                    let attribute = attribute
                        .map_err(|error| ProjectionError::InvalidXml(error.to_string()))?;
                    if attribute.key.local_name().as_ref() == b"uuid" {
                        uuid = Some(
                            attribute
                                .normalized_value(XmlVersion::Implicit1_0)
                                .map_err(|error| ProjectionError::InvalidXml(error.to_string()))?
                                .into_owned(),
                        );
                    }
                }
            }
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"name" => in_name = true,
            Ok(Event::Text(text)) if in_name => {
                citation = text
                    .decode()
                    .map_err(|error| ProjectionError::InvalidXml(error.to_string()))?
                    .into_owned();
            }
            Ok(Event::End(element)) if element.local_name().as_ref() == b"name" => in_name = false,
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(ProjectionError::InvalidXml(error.to_string())),
        }
    }
    let object_type =
        root_type.ok_or_else(|| ProjectionError::UnsupportedRoot("missing".to_owned()))?;
    let uuid = uuid.ok_or(ProjectionError::MissingUuid)?;
    let mut reference =
        SourceObjectRef::new(&uuid, object_type, Some(&format!("eml:///witsml/{uuid}")))?;
    reference.content_hash = format!("sha256:{:x}", Sha256::digest(xml.as_bytes()));
    reference.citation_name = citation;
    source_system.clone_into(&mut reference.source_system);
    reference.validate()?;
    Ok(reference)
}
