//! Safe intake of portable WellForge XML project and assembly files.
//!
//! The typed projection is deliberately partial: it exposes stable workflow
//! facts while retaining the original bytes on every successfully parsed
//! document. Only an unchanged document can be written with
//! [`WriteMode::PreserveUnchanged`], which returns those original bytes exactly.

use encoding_rs::Encoding;
use quick_xml::{Reader, events::Event};
use thiserror::Error;

const DEFAULT_MAX_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_DEPTH: usize = 128;
const DEFAULT_MAX_NODES: usize = 250_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseOptions {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_BYTES,
            max_depth: DEFAULT_MAX_DEPTH,
            max_nodes: DEFAULT_MAX_NODES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootInfo {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurveyPoint {
    pub measured_depth: Option<f64>,
    pub inclination: Option<f64>,
    pub azimuth: Option<f64>,
    pub invalid_numeric_fields: Vec<SurveyNumericField>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurveyNumericField {
    MeasuredDepth,
    Inclination,
    Azimuth,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub caption: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownNode {
    pub name: String,
    pub order: usize,
    pub depth: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDocument {
    /// Partial typed view; all unprojected XML remains in the preserved source.
    pub root: RootInfo,
    pub caption: Option<String>,
    pub surveys: Vec<SurveyPoint>,
    pub unknown_nodes: Vec<UnknownNode>,
    original: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BhaDocument {
    /// Partial typed view; all unprojected XML remains in the preserved source.
    pub root: RootInfo,
    pub caption: Option<String>,
    pub components: Vec<Component>,
    pub unknown_nodes: Vec<UnknownNode>,
    original: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteMode {
    PreserveUnchanged,
}

impl ProjectDocument {
    pub fn write(&self, mode: WriteMode) -> Result<Vec<u8>, WriteError> {
        write_original(&self.original, mode)
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        validate_project(self)
    }
}
impl BhaDocument {
    pub fn write(&self, mode: WriteMode) -> Result<Vec<u8>, WriteError> {
        write_original(&self.original, mode)
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        validate_bha(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML input is larger than the configured {limit}-byte limit (actual: {actual})")]
    ByteLimitExceeded { limit: usize, actual: usize },
    #[error("XML markup contains a DTD or entity declaration")]
    ForbiddenMarkup,
    #[error("XML nesting exceeds the configured {limit}-level limit")]
    DepthLimitExceeded { limit: usize },
    #[error("XML has more than the configured {limit} nodes")]
    NodeLimitExceeded { limit: usize },
    #[error("unsupported or invalid XML encoding")]
    Encoding,
    #[error("unsupported XML encoding: {label}")]
    UnsupportedEncoding { label: String },
    #[error("XML is malformed: {0}")]
    Malformed(&'static str),
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("expected {expected} document root, found {found}")]
    UnexpectedRoot {
        expected: &'static str,
        found: String,
    },
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("writing failed")]
    UnsupportedMode,
}

pub fn parse_project(input: &[u8], options: ParseOptions) -> Result<ProjectDocument, ParseError> {
    let parsed = parse(input, options, RootKind::Project)?;
    Ok(ProjectDocument {
        root: parsed.root,
        caption: parsed.caption,
        surveys: parsed.surveys,
        unknown_nodes: parsed.unknown_nodes,
        original: input.to_vec(),
    })
}

pub fn parse_bha(input: &[u8], options: ParseOptions) -> Result<BhaDocument, ParseError> {
    let parsed = parse(input, options, RootKind::Bha)?;
    Ok(BhaDocument {
        root: parsed.root,
        caption: parsed.caption,
        components: parsed.components,
        unknown_nodes: parsed.unknown_nodes,
        original: input.to_vec(),
    })
}

pub fn validate_project(document: &ProjectDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.caption.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "MISSING_CAPTION",
            message: "Project has no caption".to_owned(),
        });
    }
    let mut prior = None;
    for survey in &document.surveys {
        for field in &survey.invalid_numeric_fields {
            let (code, message) = match field {
                SurveyNumericField::MeasuredDepth => (
                    "INVALID_SURVEY_MEASURED_DEPTH",
                    "Survey measured depth must be a finite number",
                ),
                SurveyNumericField::Inclination => (
                    "INVALID_SURVEY_INCLINATION",
                    "Survey inclination must be a finite number",
                ),
                SurveyNumericField::Azimuth => (
                    "INVALID_SURVEY_AZIMUTH",
                    "Survey azimuth must be a finite number",
                ),
            };
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code,
                message: message.to_owned(),
            });
        }
        if let Some(depth) = survey.measured_depth {
            if prior.is_some_and(|previous| depth < previous) {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: "NON_MONOTONIC_DEPTH",
                    message: "Survey measured depth must not decrease".to_owned(),
                });
            }
            prior = Some(depth);
        }
    }
    diagnostics
}

pub fn validate_bha(document: &BhaDocument) -> Vec<Diagnostic> {
    if document.caption.as_deref().is_none_or(str::is_empty) {
        vec![Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "MISSING_CAPTION",
            message: "Assembly has no caption".to_owned(),
        }]
    } else {
        Vec::new()
    }
}

#[derive(Clone, Copy)]
enum RootKind {
    Project,
    Bha,
}
struct Parsed {
    root: RootInfo,
    caption: Option<String>,
    surveys: Vec<SurveyPoint>,
    components: Vec<Component>,
    unknown_nodes: Vec<UnknownNode>,
}

fn parse(input: &[u8], options: ParseOptions, kind: RootKind) -> Result<Parsed, ParseError> {
    if input.len() > options.max_bytes {
        return Err(ParseError::ByteLimitExceeded {
            limit: options.max_bytes,
            actual: input.len(),
        });
    }
    let decoded = decode_xml(input)?;
    if contains_forbidden_markup(&decoded) {
        return Err(ParseError::ForbiddenMarkup);
    }
    let mut reader = Reader::from_str(&decoded);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut nodes = 0usize;
    let mut root = None;
    let mut caption = None;
    let mut surveys = Vec::new();
    let mut components = Vec::new();
    let mut unknown_nodes = Vec::new();
    let mut caption_depth = None::<usize>;
    loop {
        match reader.read_event()? {
            Event::Start(event) => {
                nodes += 1;
                if nodes > options.max_nodes {
                    return Err(ParseError::NodeLimitExceeded {
                        limit: options.max_nodes,
                    });
                }
                depth += 1;
                if depth > options.max_depth {
                    return Err(ParseError::DepthLimitExceeded {
                        limit: options.max_depth,
                    });
                }
                let name = event.name().as_ref().to_vec();
                let name = String::from_utf8_lossy(&name).into_owned();
                if root.is_none() {
                    ensure_root(&name, kind)?;
                    root = Some(RootInfo { name });
                } else {
                    process_start(
                        &name,
                        &event,
                        depth,
                        &mut surveys,
                        &mut components,
                        &mut unknown_nodes,
                        &mut caption_depth,
                    )?;
                }
            }
            Event::Empty(event) => {
                nodes += 1;
                if nodes > options.max_nodes {
                    return Err(ParseError::NodeLimitExceeded {
                        limit: options.max_nodes,
                    });
                }
                if depth + 1 > options.max_depth {
                    return Err(ParseError::DepthLimitExceeded {
                        limit: options.max_depth,
                    });
                }
                let name = event.name().as_ref().to_vec();
                let name = String::from_utf8_lossy(&name).into_owned();
                if root.is_none() {
                    ensure_root(&name, kind)?;
                    root = Some(RootInfo { name });
                } else {
                    process_start(
                        &name,
                        &event,
                        depth + 1,
                        &mut surveys,
                        &mut components,
                        &mut unknown_nodes,
                        &mut caption_depth,
                    )?;
                    caption_depth = None;
                }
            }
            Event::Text(event) => {
                if caption_depth == Some(depth) {
                    caption = Some(
                        event
                            .decode()
                            .map_err(quick_xml::Error::from)?
                            .trim()
                            .to_owned(),
                    );
                }
            }
            Event::End(_) => {
                if caption_depth == Some(depth) {
                    caption_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::DocType(_) | Event::GeneralRef(_) => return Err(ParseError::ForbiddenMarkup),
            Event::Eof => break,
            _ => {}
        }
    }
    if depth != 0 {
        return Err(ParseError::Malformed("unclosed element"));
    }
    Ok(Parsed {
        root: root.ok_or(ParseError::Malformed("missing root"))?,
        caption,
        surveys,
        components,
        unknown_nodes,
    })
}

fn process_start(
    event_name: &str,
    event: &quick_xml::events::BytesStart<'_>,
    depth: usize,
    surveys: &mut Vec<SurveyPoint>,
    components: &mut Vec<Component>,
    unknown: &mut Vec<UnknownNode>,
    caption_depth: &mut Option<usize>,
) -> Result<(), ParseError> {
    let normalized = event_name.to_ascii_lowercase();
    match normalized.as_str() {
        "caption" if depth == 2 => *caption_depth = Some(depth),
        "survey" => {
            let (measured_depth, invalid_measured_depth) =
                parse_attribute(event, &["md", "measureddepth"]);
            let (inclination, invalid_inclination) =
                parse_attribute(event, &["inc", "inclination"]);
            let (azimuth, invalid_azimuth) = parse_attribute(event, &["azi", "azimuth"]);
            let mut invalid_numeric_fields = Vec::new();
            if invalid_measured_depth {
                invalid_numeric_fields.push(SurveyNumericField::MeasuredDepth);
            }
            if invalid_inclination {
                invalid_numeric_fields.push(SurveyNumericField::Inclination);
            }
            if invalid_azimuth {
                invalid_numeric_fields.push(SurveyNumericField::Azimuth);
            }
            surveys.push(SurveyPoint {
                measured_depth,
                inclination,
                azimuth,
                invalid_numeric_fields,
            });
        }
        "component" => components.push(Component {
            caption: string_attribute(event, &["caption", "name"]).unwrap_or_default(),
        }),
        "surveys" | "components" => {}
        _ => unknown.push(UnknownNode {
            name: event_name.to_owned(),
            order: unknown.len(),
            depth,
        }),
    }
    Ok(())
}

fn parse_attribute(
    event: &quick_xml::events::BytesStart<'_>,
    names: &[&str],
) -> (Option<f64>, bool) {
    let Some(value) = string_attribute(event, names) else {
        return (None, false);
    };
    match value.parse::<f64>() {
        Ok(value) if value.is_finite() => (Some(value), false),
        _ => (None, true),
    }
}
fn string_attribute(event: &quick_xml::events::BytesStart<'_>, names: &[&str]) -> Option<String> {
    event
        .attributes()
        .with_checks(false)
        .flatten()
        .find_map(|attribute| {
            let key = String::from_utf8_lossy(attribute.key.as_ref());
            names
                .iter()
                .any(|name| key.eq_ignore_ascii_case(name))
                .then(|| String::from_utf8_lossy(attribute.value.as_ref()).into_owned())
        })
}
fn ensure_root(found: &str, kind: RootKind) -> Result<(), ParseError> {
    let valid = match kind {
        RootKind::Project => {
            found.eq_ignore_ascii_case("DrillProject") || found.eq_ignore_ascii_case("Project")
        }
        RootKind::Bha => found.eq_ignore_ascii_case("BHA"),
    };
    if valid {
        Ok(())
    } else {
        Err(ParseError::UnexpectedRoot {
            expected: match kind {
                RootKind::Project => "DrillProject or Project",
                RootKind::Bha => "BHA",
            },
            found: found.to_owned(),
        })
    }
}
fn contains_forbidden_markup(input: &str) -> bool {
    let lowered = input.to_ascii_lowercase();
    lowered.contains("<!doctype") || lowered.contains("<!entity")
}
fn decode_xml(input: &[u8]) -> Result<String, ParseError> {
    if input.starts_with(&[0xfe, 0xff]) || input.starts_with(&[0xff, 0xfe]) {
        return Err(ParseError::UnsupportedEncoding {
            label: "UTF-16".to_owned(),
        });
    }
    let head = String::from_utf8_lossy(&input[..input.len().min(512)]);
    let label = xml_encoding_label(&head);
    match label {
        Some(label)
            if !label.eq_ignore_ascii_case("utf-8") && !label.eq_ignore_ascii_case("utf8") =>
        {
            if label.to_ascii_lowercase().starts_with("utf-16") {
                return Err(ParseError::UnsupportedEncoding {
                    label: label.to_owned(),
                });
            }
            let encoding = Encoding::for_label(label.as_bytes()).ok_or(ParseError::Encoding)?;
            let (decoded, _, errors) = encoding.decode(input);
            if errors {
                Err(ParseError::Encoding)
            } else {
                Ok(decoded.into_owned())
            }
        }
        _ => String::from_utf8(input.to_vec()).map_err(|_| ParseError::Encoding),
    }
}

fn xml_encoding_label(head: &str) -> Option<&str> {
    let declaration_start = head.find("<?xml")?;
    let declaration = &head[declaration_start + "<?xml".len()..];
    let declaration = &declaration[..declaration.find("?>")?];
    let bytes = declaration.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if declaration[index..]
            .to_ascii_lowercase()
            .starts_with("encoding")
        {
            let mut value_start = index + "encoding".len();
            while bytes.get(value_start).is_some_and(u8::is_ascii_whitespace) {
                value_start += 1;
            }
            if bytes.get(value_start) != Some(&b'=') {
                index += 1;
                continue;
            }
            value_start += 1;
            while bytes.get(value_start).is_some_and(u8::is_ascii_whitespace) {
                value_start += 1;
            }
            let quote = *bytes.get(value_start)?;
            if quote != b'\'' && quote != b'"' {
                return None;
            }
            let value_start = value_start + 1;
            let value_end = declaration[value_start..].find(quote as char)? + value_start;
            return Some(&declaration[value_start..value_end]);
        }
        index += 1;
    }
    None
}
fn write_original(original: &[u8], mode: WriteMode) -> Result<Vec<u8>, WriteError> {
    match mode {
        WriteMode::PreserveUnchanged => Ok(original.to_vec()),
    }
}
