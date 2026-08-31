//! Safe, order-preserving projection of neutral XML into a structural tree.

use quick_xml::{Reader, events::Event};
use serde::Serialize;

use crate::InterchangeError;

/// An XML element retaining names, attributes, text, and child order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuralNode {
    /// Element name as written in the source document.
    pub name: String,
    /// Attributes in source order.
    pub attributes: Vec<(String, String)>,
    /// Text content encountered directly under this element.
    pub text: Option<String>,
    /// Child elements in source order.
    pub children: Vec<StructuralNode>,
}

/// Parse one supported neutral BHA XML document without resolving external data.
///
/// # Errors
///
/// Returns an error for malformed XML, unsupported roots, prohibited constructs,
/// or text outside the document root.
#[allow(clippy::too_many_lines)]
pub fn parse_xml(input: &str) -> Result<StructuralNode, InterchangeError> {
    preflight(input)?;
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<StructuralNode> = Vec::new();
    let mut root: Option<StructuralNode> = None;

    loop {
        let event = reader
            .read_event()
            .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?;
        match event {
            Event::Start(element) => {
                let name = decode_name(reader.decoder(), element.name().as_ref())?;
                let attributes = element
                    .attributes()
                    .map(|attribute| {
                        let attribute = attribute
                            .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?;
                        let key = decode_name(reader.decoder(), attribute.key.as_ref())?;
                        let value = attribute
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Implicit1_0,
                                reader.decoder(),
                            )
                            .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?
                            .into_owned();
                        Ok((key, value))
                    })
                    .collect::<Result<Vec<_>, InterchangeError>>()?;
                stack.push(StructuralNode {
                    name,
                    attributes,
                    text: None,
                    children: Vec::new(),
                });
            }
            Event::Empty(element) => {
                let name = decode_name(reader.decoder(), element.name().as_ref())?;
                let attributes = element
                    .attributes()
                    .map(|attribute| {
                        let attribute = attribute
                            .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?;
                        let key = decode_name(reader.decoder(), attribute.key.as_ref())?;
                        let value = attribute
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Implicit1_0,
                                reader.decoder(),
                            )
                            .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?
                            .into_owned();
                        Ok((key, value))
                    })
                    .collect::<Result<Vec<_>, InterchangeError>>()?;
                attach(
                    StructuralNode {
                        name,
                        attributes,
                        text: None,
                        children: Vec::new(),
                    },
                    &mut stack,
                    &mut root,
                )?;
            }
            Event::End(end) => {
                let node = stack
                    .pop()
                    .ok_or_else(|| invalid("unexpected closing element"))?;
                let closing = decode_name(reader.decoder(), end.name().as_ref())?;
                if node.name != closing {
                    return Err(invalid("mismatched closing element"));
                }
                attach(node, &mut stack, &mut root)?;
            }
            Event::Text(text) => {
                let decoded = text
                    .decode()
                    .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?;
                let value = quick_xml::escape::unescape(&decoded)
                    .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?
                    .into_owned();
                if let Some(node) = stack.last_mut() {
                    node.text = Some(node.text.take().unwrap_or_default() + &value);
                } else if !value.trim().is_empty() {
                    return Err(invalid("text outside document root"));
                }
            }
            Event::DocType(_) | Event::GeneralRef(_) | Event::PI(_) => {
                return Err(InterchangeError::ProhibitedXmlConstruct);
            }
            Event::CData(text) => {
                let value = String::from_utf8(text.into_inner().into_owned())
                    .map_err(|error| InterchangeError::InvalidXml(error.to_string()))?;
                if let Some(node) = stack.last_mut() {
                    node.text = Some(node.text.take().unwrap_or_default() + &value);
                } else if !value.trim().is_empty() {
                    return Err(invalid("text outside document root"));
                }
            }
            Event::Decl(_) => {
                if root.is_some() {
                    return Err(InterchangeError::ProhibitedXmlConstruct);
                }
            }
            Event::Comment(_) => {}
            Event::Eof => break,
        }
    }
    if !stack.is_empty() {
        return Err(invalid("unclosed element"));
    }
    root.ok_or_else(|| invalid("document has no root element"))
}

fn preflight(input: &str) -> Result<(), InterchangeError> {
    let bytes = input.as_bytes();
    let mut index = 0;
    let mut markup_seen = false;
    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"<!--") {
            markup_seen = true;
            index = input[index + 4..]
                .find("-->")
                .map_or(bytes.len(), |end| index + 4 + end + 3);
            continue;
        }
        if bytes[index..].starts_with(b"<![CDATA[") {
            markup_seen = true;
            index = input[index + 9..]
                .find("]]>")
                .map_or(bytes.len(), |end| index + 9 + end + 3);
            continue;
        }
        let tail = &input[index + 1..];
        let trimmed = tail.trim_start_matches(char::is_whitespace);
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("!doctype") || lower.starts_with("!entity") {
            return Err(InterchangeError::ProhibitedXmlConstruct);
        }
        if let Some(stripped) = trimmed.strip_prefix('?') {
            let declaration = stripped.trim_start();
            let is_xml_declaration = declaration.to_ascii_lowercase().starts_with("xml")
                && declaration
                    .as_bytes()
                    .get(3)
                    .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'?');
            if !is_xml_declaration || markup_seen {
                return Err(InterchangeError::ProhibitedXmlConstruct);
            }
        }
        markup_seen = true;
        index += 1;
    }
    Ok(())
}

#[allow(clippy::ptr_arg)]
fn attach(
    node: StructuralNode,
    stack: &mut Vec<StructuralNode>,
    root: &mut Option<StructuralNode>,
) -> Result<(), InterchangeError> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else if root.is_some() {
        return Err(invalid("multiple root elements"));
    } else if node.name != "BHA" && node.name != "Component" {
        return Err(InterchangeError::UnsupportedRoot(node.name));
    } else {
        *root = Some(node);
    }
    Ok(())
}

fn decode_name(
    decoder: quick_xml::encoding::Decoder,
    bytes: &[u8],
) -> Result<String, InterchangeError> {
    decoder
        .decode(bytes)
        .map(std::borrow::Cow::into_owned)
        .map_err(|error| InterchangeError::InvalidXml(error.to_string()))
}

fn invalid(message: &str) -> InterchangeError {
    InterchangeError::InvalidXml(message.to_owned())
}
