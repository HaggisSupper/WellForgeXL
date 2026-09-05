use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use uuid::Uuid;
use wait_timeout::ChildExt;

use crate::{
    ArtifactFamily, ArtifactInput, ChunkInput, CitationInput, ConceptInput, CorpusStats,
    ExtractionEnvelope, ExtractionStatus, IngestReport, OkfExportReport, RagConfig, SearchHit,
    SqliteStore, extract_path,
};

const CHUNK_CHAR_LIMIT: usize = 1_600;
const CHUNK_OVERLAP_CHARS: usize = 200;

#[derive(Clone)]
pub struct RagService {
    config: RagConfig,
    store: SqliteStore,
}

impl RagService {
    pub fn open(config: RagConfig) -> Result<Self> {
        fs::create_dir_all(&config.storage.okf).with_context(|| {
            format!("cannot create OKF directory {}", config.storage.okf.display())
        })?;
        fs::create_dir_all(&config.storage.lancedb).with_context(|| {
            format!(
                "cannot create LanceDB directory {}",
                config.storage.lancedb.display()
            )
        })?;
        let store = SqliteStore::open(&config.storage.sqlite)?;
        Ok(Self { config, store })
    }

    pub fn config(&self) -> &RagConfig {
        &self.config
    }

    pub fn store(&self) -> &SqliteStore {
        &self.store
    }

    pub fn stats(&self) -> Result<CorpusStats> {
        self.store.stats()
    }

    pub fn ingest_path(&self, path: impl AsRef<Path>) -> Result<IngestReport> {
        let canonical = fs::canonicalize(path.as_ref())
            .with_context(|| format!("cannot resolve ingestion path {}", path.as_ref().display()))?;
        self.enforce_ingestion_roots(&canonical)?;
        let metadata = fs::metadata(&canonical)?;
        if !metadata.is_file() {
            bail!("ingestion path is not a file: {}", canonical.display());
        }
        if metadata.len() > self.config.ingest.max_file_bytes {
            bail!(
                "artifact {} exceeds configured ingestion size limit",
                canonical.display()
            );
        }

        let sha256 = sha256_file(&canonical)?;
        let mut extraction = extract_path(&canonical)?;
        if extraction.status == ExtractionStatus::Unsupported
            && matches!(extraction.family, ArtifactFamily::Document)
            && is_sidecar_extension(&canonical)
        {
            extraction = self.extract_with_sidecar(&canonical)?;
        }

        let artifact = self.store.upsert_artifact(ArtifactInput {
            sha256,
            source_uri: canonical.to_string_lossy().into_owned(),
            display_name: canonical
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| "artifact".to_owned()),
            mime_type: mime_type_for(&canonical).to_owned(),
            family: family_name(&extraction.family).to_owned(),
            size_bytes: metadata.len(),
            modified_at: None,
            extraction_backend: extraction.backend.clone(),
        })?;

        let title = extraction_title(&canonical, &extraction);
        let domain = infer_domain(&extraction);
        let source_body = source_concept_body(&extraction);
        let source_concept = self.store.upsert_concept(ConceptInput {
            concept_path: format!("artifacts/{}", artifact.id),
            concept_type: "SourceArtifact".to_owned(),
            title,
            domain: domain.clone(),
            body: source_body,
            frontmatter: json!({
                "artifact_id": artifact.id,
                "artifact_sha256": artifact.sha256,
                "source_uri": canonical.to_string_lossy(),
                "extraction_backend": extraction.backend,
                "extraction_status": extraction.status,
            }),
            provenance_state: "source-derived".to_owned(),
            trust_state: "unverified".to_owned(),
            lifecycle_state: "active".to_owned(),
            source_confidence: None,
        })?;

        let mut concepts_written = 1_u64;
        let mut chunks_written = 0_u64;
        let mut ordinal = 0_u64;

        for text_section in &extraction.text_sections {
            for text in chunk_text(&text_section.text) {
                let content_hash = sha256_text(&text);
                let chunk = self.store.upsert_chunk(ChunkInput {
                    concept_id: Some(source_concept.id),
                    artifact_id: artifact.id,
                    ordinal,
                    section_locator: text_section.locator.clone(),
                    source_locator: text_section.locator.clone(),
                    token_estimate: estimate_tokens(&text),
                    text,
                    content_hash,
                    embedding_state: if self.config.embedding.enabled {
                        "pending".to_owned()
                    } else {
                        "disabled".to_owned()
                    },
                    embedding_model: self
                        .config
                        .embedding
                        .enabled
                        .then(|| self.config.embedding.model.clone()),
                })?;
                self.store.add_citation(CitationInput {
                    concept_id: Some(source_concept.id),
                    chunk_id: Some(chunk.id),
                    artifact_id: artifact.id,
                    locator_type: locator_type(&text_section.locator).to_owned(),
                    locator: text_section.locator.clone(),
                    label: Some(artifact.display_name.clone()),
                })?;
                ordinal += 1;
                chunks_written += 1;
            }
        }

        for (profile_index, profile) in extraction.profiles.iter().enumerate() {
            let profile_body = profile_body(profile);
            let profile_concept = self.store.upsert_concept(ConceptInput {
                concept_path: format!(
                    "artifacts/{}/profiles/{}-{}",
                    artifact.id,
                    profile_index + 1,
                    slug(&profile.name)
                ),
                concept_type: "DataProfile".to_owned(),
                title: profile.name.clone(),
                domain: domain.clone(),
                body: profile_body.clone(),
                frontmatter: json!({
                    "artifact_id": artifact.id,
                    "row_count": profile.row_count,
                    "columns": profile.columns,
                }),
                provenance_state: "source-derived".to_owned(),
                trust_state: "unverified".to_owned(),
                lifecycle_state: "active".to_owned(),
                source_confidence: None,
            })?;
            self.store.link_edge(
                profile_concept.id,
                source_concept.id,
                "derived_from",
                Some(artifact.id),
            )?;
            let content_hash = sha256_text(&profile_body);
            self.store.upsert_chunk(ChunkInput {
                concept_id: Some(profile_concept.id),
                artifact_id: artifact.id,
                ordinal,
                section_locator: format!("profile:{}", profile.name),
                source_locator: format!("profile:{}", profile.name),
                token_estimate: estimate_tokens(&profile_body),
                text: profile_body,
                content_hash,
                embedding_state: if self.config.embedding.enabled {
                    "pending".to_owned()
                } else {
                    "disabled".to_owned()
                },
                embedding_model: self
                    .config
                    .embedding
                    .enabled
                    .then(|| self.config.embedding.model.clone()),
            })?;
            ordinal += 1;
            concepts_written += 1;
            chunks_written += 1;
        }

        Ok(IngestReport {
            artifact_id: artifact.id,
            concepts_written,
            chunks_written,
            warnings: extraction.warnings,
        })
    }

    pub fn search_lexical(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        self.store.lexical_search(query, limit)
    }

    pub fn export_okf(&self) -> Result<OkfExportReport> {
        let concepts = self.store.list_concepts()?;
        let mut files = Vec::with_capacity(concepts.len());
        for concept in concepts {
            let relative = concept_file_path(&concept.concept_path)?;
            let destination = self.config.storage.okf.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut text = String::new();
            text.push_str("---\n");
            push_yaml_string(&mut text, "id", &concept.id.to_string())?;
            push_yaml_string(&mut text, "type", &concept.concept_type)?;
            push_yaml_string(&mut text, "title", &concept.title)?;
            push_yaml_string(&mut text, "domain", &concept.domain)?;
            push_yaml_string(
                &mut text,
                "provenance_state",
                &concept.provenance_state,
            )?;
            push_yaml_string(&mut text, "trust_state", &concept.trust_state)?;
            push_yaml_string(&mut text, "lifecycle_state", &concept.lifecycle_state)?;
            if let Some(confidence) = concept.source_confidence {
                text.push_str(&format!("source_confidence: {confidence}\n"));
            }
            text.push_str("---\n\n");
            text.push_str(&concept.body);
            if !text.ends_with('\n') {
                text.push('\n');
            }
            write_atomic(&destination, text.as_bytes())?;
            files.push(destination);
        }
        Ok(OkfExportReport {
            files_written: u64::try_from(files.len()).context("OKF file count overflow")?,
            files,
        })
    }

    fn enforce_ingestion_roots(&self, path: &Path) -> Result<()> {
        if self.config.ingest.roots.is_empty() {
            return Ok(());
        }
        for root in &self.config.ingest.roots {
            if let Ok(root) = fs::canonicalize(root)
                && path.starts_with(&root)
            {
                return Ok(());
            }
        }
        bail!(
            "ingestion path {} is outside configured ingestion roots",
            path.display()
        )
    }

    fn extract_with_sidecar(&self, path: &Path) -> Result<ExtractionEnvelope> {
        let adapter = &self.config.ingest.python_adapter;
        if !adapter.is_file() {
            bail!(
                "configured document extraction sidecar does not exist: {}",
                adapter.display()
            );
        }
        let stdout = NamedTempFile::new()?;
        let stderr = NamedTempFile::new()?;
        let mut child = Command::new(&self.config.ingest.python)
            .arg(adapter)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.reopen()?))
            .stderr(Stdio::from(stderr.reopen()?))
            .spawn()
            .with_context(|| {
                format!(
                    "cannot start document extraction sidecar {}",
                    self.config.ingest.python
                )
            })?;
        let timeout = Duration::from_secs(120);
        let status = match child.wait_timeout(timeout)? {
            Some(status) => status,
            None => {
                child.kill()?;
                child.wait()?;
                bail!("document extraction sidecar exceeded {timeout:?}");
            }
        };
        let stdout_bytes = fs::read(stdout.path())?;
        let stderr_bytes = fs::read(stderr.path())?;
        if stdout_bytes.len() as u64 > self.config.ingest.max_extraction_bytes {
            bail!("document extraction output exceeds configured size limit");
        }
        if !status.success() {
            bail!(
                "document extraction sidecar failed: {}",
                String::from_utf8_lossy(&stderr_bytes).trim()
            );
        }
        serde_json::from_slice(&stdout_bytes).context("invalid extraction sidecar response")
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}

fn sha256_text(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn chunk_text(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.is_empty() {
        return Vec::new();
    }
    let chars = value.char_indices().collect::<Vec<_>>();
    if chars.len() <= CHUNK_CHAR_LIMIT {
        return vec![value.to_owned()];
    }
    let mut chunks = Vec::new();
    let mut start_char = 0_usize;
    while start_char < chars.len() {
        let end_char = (start_char + CHUNK_CHAR_LIMIT).min(chars.len());
        let start_byte = chars[start_char].0;
        let end_byte = if end_char == chars.len() {
            value.len()
        } else {
            chars[end_char].0
        };
        let chunk = value[start_byte..end_byte].trim();
        if !chunk.is_empty() {
            chunks.push(chunk.to_owned());
        }
        if end_char == chars.len() {
            break;
        }
        start_char = end_char.saturating_sub(CHUNK_OVERLAP_CHARS);
    }
    chunks
}

fn estimate_tokens(value: &str) -> u64 {
    u64::try_from(value.chars().count().div_ceil(4)).unwrap_or(u64::MAX)
}

fn extraction_title(path: &Path, extraction: &ExtractionEnvelope) -> String {
    for section in &extraction.text_sections {
        if let Some(title) = section
            .text
            .lines()
            .map(str::trim)
            .find_map(|line| line.strip_prefix("# ").filter(|title| !title.trim().is_empty()))
        {
            return title.trim().to_owned();
        }
    }
    path.file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Artifact".to_owned())
}

fn source_concept_body(extraction: &ExtractionEnvelope) -> String {
    let mut body = extraction
        .text_sections
        .iter()
        .map(|section| section.text.trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if body.is_empty() {
        body.push_str("Structured artifact. Use linked data-profile concepts for schema evidence.");
    }
    if !extraction.warnings.is_empty() {
        body.push_str("\n\n## Extraction warnings\n\n");
        for warning in &extraction.warnings {
            body.push_str("- ");
            body.push_str(warning);
            body.push('\n');
        }
    }
    body
}

fn profile_body(profile: &crate::DataProfile) -> String {
    let mut body = format!("# {}\n\n", profile.name);
    if let Some(rows) = profile.row_count {
        body.push_str(&format!("Rows profiled: {rows}\n\n"));
    }
    body.push_str("Columns:\n");
    for column in &profile.columns {
        body.push_str("- ");
        body.push_str(&column.name);
        if let Some(data_type) = &column.data_type {
            body.push_str(": ");
            body.push_str(data_type);
        }
        body.push('\n');
    }
    body
}

fn infer_domain(extraction: &ExtractionEnvelope) -> String {
    let sample = extraction
        .text_sections
        .iter()
        .take(8)
        .map(|section| section.text.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    for (domain, terms) in [
        ("well-control", &["kick", "maasp", "kill weight", "leak off", "fit/lot"][..]),
        ("hydraulics", &["ecd", "pressure loss", "rheology", "nozzle", "surge", "swab"]),
        ("directional", &["trajectory", "dogleg", "toolface", "inclination", "azimuth"]),
        ("torque-drag-bha", &["torque", "drag", "buckling", "bha", "campbell", "frf"]),
        ("drilling-performance", &["mse", "rop", "d-exponent", "rig state", "wob"]),
        ("production", &["production", "choke", "separator", "orifice"]),
    ] {
        if terms.iter().any(|term| sample.contains(term)) {
            return domain.to_owned();
        }
    }
    match extraction.family {
        ArtifactFamily::DrillingData => "drilling-data".to_owned(),
        ArtifactFamily::AnalyticalData => "analytical-data".to_owned(),
        ArtifactFamily::Database => "database".to_owned(),
        _ => "engineering-reference".to_owned(),
    }
}

fn family_name(family: &ArtifactFamily) -> &'static str {
    match family {
        ArtifactFamily::Document => "document",
        ArtifactFamily::StructuredData => "structured-data",
        ArtifactFamily::DrillingData => "drilling-data",
        ArtifactFamily::AnalyticalData => "analytical-data",
        ArtifactFamily::Database => "database",
        ArtifactFamily::Unknown => "unknown",
    }
}

fn mime_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "pdf" => "application/pdf",
        "json" | "jsonl" | "ndjson" => "application/json",
        "xml" | "witsml" => "application/xml",
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "md" | "markdown" => "text/markdown",
        "txt" | "las" => "text/plain",
        "parquet" => "application/vnd.apache.parquet",
        "arrow" | "ipc" | "feather" => "application/vnd.apache.arrow.file",
        "sqlite" | "sqlite3" | "db" => "application/vnd.sqlite3",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "xlsx" | "xlsm" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "xlsb" => "application/vnd.ms-excel.sheet.binary.macroEnabled.12",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

fn is_sidecar_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "pdf"
            | "docx"
            | "pptx"
            | "xlsx"
            | "xlsm"
            | "xls"
            | "xlsb"
            | "rtf"
            | "png"
            | "jpg"
            | "jpeg"
            | "tif"
            | "tiff"
    )
}

fn locator_type(locator: &str) -> &'static str {
    if locator.starts_with("page:") {
        "page"
    } else if locator.starts_with("slide:") {
        "slide"
    } else if locator.starts_with("sheet:") || locator.starts_with("profile:") {
        "table"
    } else if locator.starts_with("las:") {
        "las-section"
    } else {
        "section"
    }
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            output.push(character);
            separator = false;
        } else if !separator && !output.is_empty() {
            output.push('-');
            separator = true;
        }
    }
    output.trim_matches('-').to_owned()
}

fn concept_file_path(concept_path: &str) -> Result<PathBuf> {
    if concept_path.is_empty() || concept_path.starts_with('/') || concept_path.contains("..") {
        bail!("unsafe concept path {concept_path}");
    }
    let mut path = PathBuf::new();
    for segment in concept_path.split('/') {
        let segment = slug(segment);
        if segment.is_empty() {
            bail!("concept path contains an empty unsafe segment");
        }
        path.push(segment);
    }
    path.set_extension("md");
    Ok(path)
}

fn push_yaml_string(output: &mut String, key: &str, value: &str) -> Result<()> {
    output.push_str(key);
    output.push_str(": ");
    output.push_str(&serde_json::to_string(value)?);
    output.push('\n');
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file_mut().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| anyhow::anyhow!("cannot replace {}: {}", path.display(), error.error))?;
    Ok(())
}
