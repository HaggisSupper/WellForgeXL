use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, bail};
use arrow_ipc::reader::{FileReader as ArrowFileReader, StreamReader as ArrowStreamReader};
use parquet::file::reader::{FileReader as ParquetFileReader, SerializedFileReader};
use quick_xml::{Reader as XmlReader, events::Event};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MAX_TEXT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_XML_ELEMENTS: usize = 50_000;
const MAX_CSV_ROWS: u64 = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactFamily {
    Document,
    StructuredData,
    DrillingData,
    AnalyticalData,
    Database,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtractionStatus {
    Extracted,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextSection {
    pub locator: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnProfile {
    pub name: String,
    pub data_type: Option<String>,
    pub nullable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataProfile {
    pub name: String,
    pub row_count: Option<u64>,
    pub columns: Vec<ColumnProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionEnvelope {
    pub family: ArtifactFamily,
    pub status: ExtractionStatus,
    pub backend: String,
    pub text_sections: Vec<TextSection>,
    pub profiles: Vec<DataProfile>,
    pub metadata: Value,
    pub warnings: Vec<String>,
}

impl ExtractionEnvelope {
    fn extracted(family: ArtifactFamily, backend: &str) -> Self {
        Self {
            family,
            status: ExtractionStatus::Extracted,
            backend: backend.to_owned(),
            text_sections: Vec::new(),
            profiles: Vec::new(),
            metadata: json!({}),
            warnings: Vec::new(),
        }
    }

    fn unsupported(family: ArtifactFamily, reason: impl Into<String>) -> Self {
        Self {
            family,
            status: ExtractionStatus::Unsupported,
            backend: "native-router".to_owned(),
            text_sections: Vec::new(),
            profiles: Vec::new(),
            metadata: json!({}),
            warnings: vec![reason.into()],
        }
    }
}

pub fn extract_path(path: impl AsRef<Path>) -> Result<ExtractionEnvelope> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "txt" | "md" | "markdown" | "html" | "htm" => extract_text(path),
        "json" => extract_json(path),
        "jsonl" | "ndjson" => extract_json_lines(path),
        "yaml" | "yml" => extract_yaml(path),
        "toml" => extract_toml(path),
        "csv" => extract_delimited(path, b','),
        "tsv" => extract_delimited(path, b'\t'),
        "xml" | "witsml" => extract_xml(path, extension == "witsml"),
        "las" => extract_las(path),
        "parquet" => extract_parquet(path),
        "arrow" | "ipc" | "feather" => extract_arrow(path),
        "sqlite" | "sqlite3" | "db" => extract_sqlite(path),
        "dlis" => Ok(ExtractionEnvelope::unsupported(
            ArtifactFamily::DrillingData,
            "DLIS is recognized but requires a verified binary parser before content extraction",
        )),
        "duckdb" => Ok(ExtractionEnvelope::unsupported(
            ArtifactFamily::Database,
            "DuckDB is recognized but requires a verified DuckDB adapter before content extraction",
        )),
        "pdf" | "docx" | "pptx" | "xlsx" | "xlsm" | "xlsb" | "png" | "jpg" | "jpeg" | "tif"
        | "tiff" => Ok(ExtractionEnvelope::unsupported(
            ArtifactFamily::Document,
            "document/image format is routed to the configured non-executing extraction sidecar",
        )),
        _ => Ok(ExtractionEnvelope::unsupported(
            ArtifactFamily::Unknown,
            format!("no verified extractor is registered for .{extension}"),
        )),
    }
}

fn bounded_text(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("cannot stat text artifact {}", path.display()))?;
    if metadata.len() > MAX_TEXT_BYTES {
        bail!(
            "text artifact {} exceeds {} byte native extraction limit",
            path.display(),
            MAX_TEXT_BYTES
        );
    }
    fs::read_to_string(path).with_context(|| format!("cannot read UTF-8 text {}", path.display()))
}

fn extract_text(path: &Path) -> Result<ExtractionEnvelope> {
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::Document, "rust-text");
    result.text_sections.push(TextSection {
        locator: "document".to_owned(),
        text: bounded_text(path)?,
    });
    Ok(result)
}

fn extract_json(path: &Path) -> Result<ExtractionEnvelope> {
    let value: Value = serde_json::from_str(&bounded_text(path)?)
        .with_context(|| format!("invalid JSON {}", path.display()))?;
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::StructuredData, "rust-json");
    result.text_sections.push(TextSection {
        locator: "$".to_owned(),
        text: serde_json::to_string_pretty(&value)?,
    });
    result.metadata = json!({ "json_type": json_kind(&value) });
    Ok(result)
}

fn extract_json_lines(path: &Path) -> Result<ExtractionEnvelope> {
    let text = bounded_text(path)?;
    let mut count = 0_u64;
    let mut first = Vec::new();
    for (index, line) in text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
    {
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("invalid JSONL row {} in {}", index + 1, path.display()))?;
        count += 1;
        if first.len() < 10 {
            first.push(value);
        }
    }
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::StructuredData, "rust-jsonl");
    result.metadata = json!({ "row_count": count });
    result.text_sections.push(TextSection {
        locator: "sample:first-10".to_owned(),
        text: serde_json::to_string_pretty(&first)?,
    });
    Ok(result)
}

fn extract_yaml(path: &Path) -> Result<ExtractionEnvelope> {
    let text = bounded_text(path)?;
    let value: yaml_serde::Value =
        yaml_serde::from_str(&text).with_context(|| format!("invalid YAML {}", path.display()))?;
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::StructuredData, "rust-yaml");
    result.text_sections.push(TextSection {
        locator: "$".to_owned(),
        text: serde_json::to_string_pretty(&value)?,
    });
    Ok(result)
}

fn extract_toml(path: &Path) -> Result<ExtractionEnvelope> {
    let value: toml::Value = toml::from_str(&bounded_text(path)?)
        .with_context(|| format!("invalid TOML {}", path.display()))?;
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::StructuredData, "rust-toml");
    result.text_sections.push(TextSection {
        locator: "$".to_owned(),
        text: serde_json::to_string_pretty(&value)?,
    });
    Ok(result)
}

fn extract_delimited(path: &Path, delimiter: u8) -> Result<ExtractionEnvelope> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("cannot stat delimited file {}", path.display()))?;
    if metadata.len() > 512 * 1024 * 1024 {
        bail!(
            "delimited artifact {} exceeds 512 MiB limit",
            path.display()
        );
    }
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_path(path)
        .with_context(|| format!("cannot open delimited file {}", path.display()))?;
    let headers = reader.headers()?.clone();
    let columns = headers
        .iter()
        .map(|name| ColumnProfile {
            name: name.to_owned(),
            data_type: None,
            nullable: None,
        })
        .collect::<Vec<_>>();
    let mut rows = 0_u64;
    let mut truncated = false;
    for record in reader.records() {
        record?;
        rows += 1;
        if rows >= MAX_CSV_ROWS {
            truncated = true;
            break;
        }
    }
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::StructuredData, "rust-csv");
    result.profiles.push(DataProfile {
        name: "table".to_owned(),
        row_count: Some(rows),
        columns,
    });
    if truncated {
        result.warnings.push(format!(
            "row count stopped at configured profiling ceiling {MAX_CSV_ROWS}"
        ));
    }
    Ok(result)
}

fn extract_xml(path: &Path, drilling: bool) -> Result<ExtractionEnvelope> {
    let file = File::open(path).with_context(|| format!("cannot open XML {}", path.display()))?;
    let mut reader = XmlReader::from_reader(BufReader::new(file));
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut seen = 0_usize;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(event) => {
                let name = String::from_utf8_lossy(event.name().as_ref()).to_string();
                *counts.entry(name.clone()).or_default() += 1;
                stack.push(name);
                seen += 1;
            }
            Event::Empty(event) => {
                let name = String::from_utf8_lossy(event.name().as_ref()).to_string();
                *counts.entry(name).or_default() += 1;
                seen += 1;
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        if seen >= MAX_XML_ELEMENTS {
            break;
        }
        buffer.clear();
    }
    let family = if drilling {
        ArtifactFamily::DrillingData
    } else {
        ArtifactFamily::StructuredData
    };
    let mut result = ExtractionEnvelope::extracted(family, "rust-xml");
    result.metadata = json!({ "element_count_profiled": seen, "element_counts": counts });
    result.text_sections.push(TextSection {
        locator: "xml:element-profile".to_owned(),
        text: serde_json::to_string_pretty(&result.metadata)?,
    });
    if seen >= MAX_XML_ELEMENTS {
        result.warnings.push(format!(
            "XML element profiling stopped at {MAX_XML_ELEMENTS}"
        ));
    }
    Ok(result)
}

fn extract_las(path: &Path) -> Result<ExtractionEnvelope> {
    let file = File::open(path).with_context(|| format!("cannot open LAS {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut section = String::new();
    let mut curves = Vec::new();
    let mut well_fields = Vec::new();
    let mut data_rows = 0_u64;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('~') {
            section = rest
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            continue;
        }
        if section.starts_with('c') {
            if let Some(curve) = parse_las_mnemonic(trimmed) {
                curves.push(curve);
            }
        } else if section.starts_with('w') {
            if let Some(field) = parse_las_mnemonic(trimmed) {
                well_fields.push(field);
            }
        } else if section.starts_with('a') {
            data_rows += 1;
        }
    }

    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::DrillingData, "rust-las");
    result.metadata = json!({
        "curve_count": curves.len(),
        "data_rows": data_rows,
        "well_field_count": well_fields.len()
    });
    result.text_sections.push(TextSection {
        locator: "las:well".to_owned(),
        text: well_fields.join("\n"),
    });
    result.text_sections.push(TextSection {
        locator: "las:curves".to_owned(),
        text: curves.join("\n"),
    });
    Ok(result)
}

fn parse_las_mnemonic(line: &str) -> Option<String> {
    let (left, description) = line.split_once(':').unwrap_or((line, ""));
    let (mnemonic, rest) = left.split_once('.').unwrap_or((left, ""));
    let mnemonic = mnemonic.trim();
    if mnemonic.is_empty() {
        return None;
    }
    let unit = rest.split_whitespace().next().unwrap_or_default();
    Some(format!(
        "{}{}{}",
        mnemonic,
        if unit.is_empty() { "" } else { "." },
        if unit.is_empty() {
            description.trim().to_owned()
        } else if description.trim().is_empty() {
            unit.to_owned()
        } else {
            format!("{unit} : {}", description.trim())
        }
    ))
}

fn extract_parquet(path: &Path) -> Result<ExtractionEnvelope> {
    let file = File::open(path)
        .with_context(|| format!("cannot open Parquet artifact {}", path.display()))?;
    let reader = SerializedFileReader::new(file)
        .with_context(|| format!("invalid Parquet artifact {}", path.display()))?;
    let metadata = reader.metadata();
    let file_metadata = metadata.file_metadata();
    let columns = file_metadata
        .schema_descr()
        .columns()
        .iter()
        .map(|column| ColumnProfile {
            name: column.path().string(),
            data_type: Some(format!("{:?}", column.physical_type())),
            nullable: None,
        })
        .collect();
    let rows = u64::try_from(file_metadata.num_rows()).context("negative Parquet row count")?;
    let mut result = ExtractionEnvelope::extracted(ArtifactFamily::AnalyticalData, "rust-parquet");
    result.profiles.push(DataProfile {
        name: "parquet".to_owned(),
        row_count: Some(rows),
        columns,
    });
    result.metadata = json!({ "row_groups": metadata.num_row_groups() });
    Ok(result)
}

fn extract_arrow(path: &Path) -> Result<ExtractionEnvelope> {
    let file = File::open(path)
        .with_context(|| format!("cannot open Arrow artifact {}", path.display()))?;
    if let Ok(mut reader) = ArrowFileReader::try_new(file, None) {
        let schema = reader.schema();
        let columns = arrow_columns(&schema);
        let mut rows = 0_u64;
        for batch in &mut reader {
            rows = rows
                .checked_add(u64::try_from(batch?.num_rows())?)
                .context("Arrow row count overflow")?;
        }
        let mut result =
            ExtractionEnvelope::extracted(ArtifactFamily::AnalyticalData, "rust-arrow-ipc-file");
        result.profiles.push(DataProfile {
            name: "arrow".to_owned(),
            row_count: Some(rows),
            columns,
        });
        return Ok(result);
    }

    let file = File::open(path)?;
    let mut reader = ArrowStreamReader::try_new(file, None)
        .with_context(|| format!("invalid Arrow IPC artifact {}", path.display()))?;
    let schema = reader.schema();
    let columns = arrow_columns(&schema);
    let mut rows = 0_u64;
    for batch in &mut reader {
        rows = rows
            .checked_add(u64::try_from(batch?.num_rows())?)
            .context("Arrow row count overflow")?;
    }
    let mut result =
        ExtractionEnvelope::extracted(ArtifactFamily::AnalyticalData, "rust-arrow-ipc-stream");
    result.profiles.push(DataProfile {
        name: "arrow".to_owned(),
        row_count: Some(rows),
        columns,
    });
    Ok(result)
}

fn arrow_columns(schema: &arrow_schema::SchemaRef) -> Vec<ColumnProfile> {
    schema
        .fields()
        .iter()
        .map(|field| ColumnProfile {
            name: field.name().clone(),
            data_type: Some(format!("{:?}", field.data_type())),
            nullable: Some(field.is_nullable()),
        })
        .collect()
}

fn extract_sqlite(path: &Path) -> Result<ExtractionEnvelope> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("cannot open SQLite artifact {}", path.display()))?;
    let mut objects = connection.prepare(
        "SELECT name, type FROM sqlite_schema
         WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%'
         ORDER BY name",
    )?;
    let object_rows = objects
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(objects);

    let mut result =
        ExtractionEnvelope::extracted(ArtifactFamily::Database, "rust-sqlite-schema");
    for (name, kind) in object_rows {
        let quoted = name.replace('"', "\"\"");
        let pragma = format!("PRAGMA table_info(\"{quoted}\")");
        let mut statement = connection.prepare(&pragma)?;
        let columns = statement
            .query_map([], |row| {
                let not_null: i64 = row.get(3)?;
                Ok(ColumnProfile {
                    name: row.get(1)?,
                    data_type: Some(row.get::<_, String>(2)?),
                    nullable: Some(not_null == 0),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        result.profiles.push(DataProfile {
            name: name.clone(),
            row_count: None,
            columns,
        });
        if let Some(object) = result.metadata.as_object_mut() {
            object.insert(name, json!({ "type": kind }));
        }
    }
    Ok(result)
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
