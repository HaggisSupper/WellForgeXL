use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use wellforge_3d::SceneDocumentV1;
use wellforge_ac::{ClosestApproach, SpatialStation, closest_approach_scan};
use wellforge_core::{
    ApiError, CalculationBackend, CalculationContext, CalculationReceipt, InputRevision, Metres,
    PlotPreferences, PlotSpec, Radians, UnitPreferences,
};
use wellforge_formats::{
    Diagnostic, DiagnosticSeverity, ParseError, ParseOptions, parse_bha, parse_project,
};
use wellforge_storage::{StoredCalculationReceiptAudit, StoredProjectRevisionAudit};
use wellforge_survey::build_survey_scene as build_survey_scene_document;
use wellforge_survey::{
    Displacement, SurveyPosition, SurveyStation, build_plan_section_plot,
    calculate_displacement_minimum_curvature,
};

use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
pub struct PingResponse {
    pub message: &'static str,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimumCurvatureRequest {
    pub start: SurveyStation,
    pub end: SurveyStation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyPlotRequest {
    pub stations: Vec<SurveyPosition>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveySceneRequest {
    pub stations: Vec<SurveyPosition>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcScanRequest {
    pub reference: Vec<SpatialStation>,
    pub offsets: Vec<SpatialStation>,
}

const INSPECTION_MAX_BYTES: usize = 16 * 1024 * 1024;
const MINIMUM_CURVATURE_ALGORITHM: &str = "minimum-curvature";
const MINIMUM_CURVATURE_ALGORITHM_VERSION: &str = "2026.1";
const CANONICAL_SI_UNIT_SYSTEM: &str = "si";
const CANONICAL_SURVEY_CRS: &str = "EPSG:4979";
const LOCAL_WORKSTATION_ACTOR_ID: &str = "local-workstation";

/// Stable projection of a parser diagnostic for the desktop boundary.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
}

/// Read-only typed facts about a supported portable document.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInspection {
    pub document_type: String,
    pub root_name: String,
    pub caption: Option<String>,
    pub survey_count: usize,
    pub component_count: usize,
    pub diagnostics: Vec<InspectionDiagnostic>,
}

/// Safe display identity for the selected project. The local path remains backend-owned.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedProject {
    pub name: String,
}

/// The inspection belongs to the same native selection as the display identity.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSelectionResult {
    pub project: SelectedProject,
    pub inspection: DocumentInspection,
}

/// A deterministic survey result with Rust-owned provenance for the active project artifact.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimumCurvatureCalculation {
    pub result: MinimumCurvatureResult,
    pub receipt: CalculationReceipt,
}

/// Camel-case desktop projection of the raw survey displacement API.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimumCurvatureResult {
    pub north_m: Metres,
    pub east_m: Metres,
    pub tvd_m: Metres,
    pub dogleg_rad: Radians,
    pub dogleg_severity_rad_per_m: f64,
}

/// Read-only, metadata-only project history for the desktop audit workspace.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAudit {
    pub revisions: Vec<ProjectRevisionAudit>,
    pub calculation_receipts: Vec<CalculationReceiptAudit>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRevisionAudit {
    pub id: String,
    pub parent_revision_id: Option<String>,
    pub content_sha256: String,
    pub created_at: String,
    pub actor_id: String,
}

impl From<StoredProjectRevisionAudit> for ProjectRevisionAudit {
    fn from(value: StoredProjectRevisionAudit) -> Self {
        Self {
            id: value.id,
            parent_revision_id: value.parent_revision_id,
            content_sha256: value.content_sha256,
            created_at: value.created_at,
            actor_id: value.actor_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationReceiptAudit {
    pub id: String,
    pub project_revision_id: String,
    pub project_revision_content_sha256: String,
    pub content_sha256: String,
    pub recorded_at: String,
    pub algorithm: String,
    pub algorithm_version: String,
    pub actor_id: String,
    pub output_sha256: String,
    pub warning_count: usize,
}

impl From<StoredCalculationReceiptAudit> for CalculationReceiptAudit {
    fn from(value: StoredCalculationReceiptAudit) -> Self {
        Self {
            id: value.id,
            project_revision_id: value.project_revision_id,
            project_revision_content_sha256: value.project_revision_content_sha256,
            content_sha256: value.content_sha256,
            recorded_at: value.recorded_at,
            algorithm: value.algorithm,
            algorithm_version: value.algorithm_version,
            actor_id: value.actor_id,
            output_sha256: value.output_sha256,
            warning_count: value.warnings.len(),
        }
    }
}

impl From<Displacement> for MinimumCurvatureResult {
    fn from(displacement: Displacement) -> Self {
        Self {
            north_m: displacement.north_m,
            east_m: displacement.east_m,
            tvd_m: displacement.tvd_m,
            dogleg_rad: displacement.dogleg_rad,
            dogleg_severity_rad_per_m: displacement.dogleg_severity_rad_per_m,
        }
    }
}

#[cfg(test)]
mod selection_contract_tests {
    use super::{
        DocumentInspection, InspectionDiagnostic, ProjectSelectionResult, SelectedProject,
    };

    #[test]
    fn selection_result_exposes_only_a_display_name_and_its_bound_inspection() {
        let result = ProjectSelectionResult {
            project: SelectedProject {
                name: "fixture.drillproj".to_owned(),
            },
            inspection: DocumentInspection {
                document_type: "project".to_owned(),
                root_name: "DrillProject".to_owned(),
                caption: None,
                survey_count: 1,
                component_count: 0,
                diagnostics: vec![InspectionDiagnostic {
                    severity: "warning".to_owned(),
                    code: "MISSING_UWI".to_owned(),
                    message: "No UWI was provided.".to_owned(),
                }],
            },
        };

        let json = serde_json::to_value(result).expect("selection result serializes");

        assert_eq!(json["project"]["name"], "fixture.drillproj");
        assert!(json["project"].get("path").is_none());
        assert_eq!(json["inspection"]["surveyCount"], 1);
    }
}

#[cfg(test)]
mod minimum_curvature_receipt_tests {
    use super::{
        MinimumCurvatureRequest, calculate_minimum_curvature_for_active_project,
        parse_minimum_curvature_request,
    };
    use crate::state::AppState;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    use wellforge_core::{CalculationReceipt, Metres, Radians};
    use wellforge_storage::LocalStore;
    use wellforge_survey::SurveyStation;

    #[test]
    fn minimum_curvature_returns_a_receipt_with_no_local_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wellforge-receipt-{nonce}.drillproj"));
        fs::write(&path, b"abc").expect("portable project fixture writes");
        let state = AppState::new_for_test();
        state
            .set_project_from_native_selection(path.clone())
            .expect("native selection opens fixture");
        let response = calculate_minimum_curvature_for_active_project(
            &state,
            MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).expect("finite measured depth"),
                    Radians::try_new(0.0).expect("finite inclination"),
                    Radians::try_new(0.0).expect("finite azimuth"),
                ),
                end: SurveyStation::new(
                    Metres::try_new(30.0).expect("finite measured depth"),
                    Radians::try_new(0.1).expect("finite inclination"),
                    Radians::try_new(0.2).expect("finite azimuth"),
                ),
            },
        )
        .expect("valid survey stations calculate");
        let changed_request_response = calculate_minimum_curvature_for_active_project(
            &state,
            MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).expect("finite measured depth"),
                    Radians::try_new(0.0).expect("finite inclination"),
                    Radians::try_new(0.0).expect("finite azimuth"),
                ),
                end: SurveyStation::new(
                    Metres::try_new(31.0).expect("finite measured depth"),
                    Radians::try_new(0.1).expect("finite inclination"),
                    Radians::try_new(0.2).expect("finite azimuth"),
                ),
            },
        )
        .expect("a distinct valid request calculates");
        fs::remove_file(&path).expect("portable project fixture removes");

        let json = serde_json::to_value(response).expect("response serializes");
        let changed_request_json =
            serde_json::to_value(changed_request_response).expect("response serializes");

        assert!(json.get("result").is_some());
        assert_eq!(json["receipt"]["algorithm"], "minimum-curvature");
        assert_eq!(json["receipt"]["algorithmVersion"], "2026.1");
        assert_eq!(json["receipt"]["context"]["unitSystem"], "si");
        assert_eq!(json["receipt"]["context"]["backend"], "cpu");
        assert_eq!(json["receipt"]["context"]["actorId"], "local-workstation");
        assert_eq!(
            json["receipt"]["inputRevisions"][0]["kind"],
            "project_revision"
        );
        assert!(
            json["receipt"]["inputRevisions"][0]["id"]
                .as_str()
                .is_some_and(|id| id.contains(":revision-event:"))
        );
        assert_eq!(
            json["receipt"]["inputRevisions"][0]["contentSha256"],
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            json["receipt"]["inputRevisions"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(
            json["receipt"]["inputRevisions"][1]["kind"],
            "minimum_curvature_request"
        );
        assert_eq!(
            json["receipt"]["inputRevisions"][1]["id"],
            json["receipt"]["inputRevisions"][1]["contentSha256"]
        );
        assert_ne!(
            json["receipt"]["inputRevisions"][1]["contentSha256"],
            changed_request_json["receipt"]["inputRevisions"][1]["contentSha256"]
        );
        assert!(
            !json
                .to_string()
                .contains(&path.to_string_lossy().replace('\\', "\\\\"))
        );
    }

    #[test]
    fn minimum_curvature_returns_a_structured_error_when_selected_content_cannot_be_read() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("wellforge-receipt-unreadable-{nonce}.drillproj"));
        fs::write(&path, b"abc").expect("portable project fixture writes");
        let state = AppState::new_for_test();
        state
            .set_project_from_native_selection(path.clone())
            .expect("native selection opens fixture");
        fs::remove_file(&path).expect("fixture removal simulates unavailable content");

        let error = calculate_minimum_curvature_for_active_project(
            &state,
            MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).expect("finite measured depth"),
                    Radians::try_new(0.0).expect("finite inclination"),
                    Radians::try_new(0.0).expect("finite azimuth"),
                ),
                end: SurveyStation::new(
                    Metres::try_new(30.0).expect("finite measured depth"),
                    Radians::try_new(0.1).expect("finite inclination"),
                    Radians::try_new(0.2).expect("finite azimuth"),
                ),
            },
        )
        .expect_err("missing selected artifact fails safely");

        assert_eq!(error.code, "PROJECT_CONTENT_READ_FAILED");
        assert!(error.details.is_none());
    }

    #[test]
    fn minimum_curvature_requires_an_active_native_selection() {
        let error = calculate_minimum_curvature_for_active_project(
            &AppState::new_for_test(),
            MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).expect("finite measured depth"),
                    Radians::try_new(0.0).expect("finite inclination"),
                    Radians::try_new(0.0).expect("finite azimuth"),
                ),
                end: SurveyStation::new(
                    Metres::try_new(30.0).expect("finite measured depth"),
                    Radians::try_new(0.1).expect("finite inclination"),
                    Radians::try_new(0.2).expect("finite azimuth"),
                ),
            },
        )
        .expect_err("calculation requires a native project selection");

        assert_eq!(error.code, "NO_OPEN_PROJECT");
    }

    #[test]
    fn minimum_curvature_maps_a_malformed_request_to_a_structured_api_error() {
        let error = parse_minimum_curvature_request(serde_json::json!({
            "start": { "md_m": 0.0, "inclination_rad": 0.0, "azimuth_true_rad": 0.0 }
        }))
        .expect_err("an incomplete request is not a calculation request");

        assert_eq!(error.code, "MALFORMED_CALCULATION_REQUEST");
        assert!(error.details.is_none());
    }

    #[test]
    fn minimum_curvature_persists_its_typed_receipt_across_restart() {
        let root = tempfile::tempdir().expect("temporary directory creates");
        let data_dir = root.path().join("app-data");
        let project_path = root.path().join("receipt-restart.drillproj");
        fs::write(&project_path, b"<DrillProject/>").expect("project fixture writes");
        let state = AppState::open(&data_dir).expect("file-backed state opens");
        state
            .set_project_from_native_selection(project_path)
            .expect("native selection opens fixture");

        let response = calculate_minimum_curvature_for_active_project(
            &state,
            MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).unwrap(),
                    Radians::try_new(0.0).unwrap(),
                    Radians::try_new(0.0).unwrap(),
                ),
                end: SurveyStation::new(
                    Metres::try_new(30.0).unwrap(),
                    Radians::try_new(0.1).unwrap(),
                    Radians::try_new(0.2).unwrap(),
                ),
            },
        )
        .expect("calculation succeeds");
        let receipt = response.receipt.clone();
        let revision_id = receipt.input_revisions()[0].id.clone();
        drop(state);

        let store = LocalStore::open(data_dir.join("local-authority.sqlite3"))
            .expect("authority store reopens");
        let persisted = store
            .latest_core_calculation_receipt_for_revision(&revision_id)
            .expect("receipt query succeeds")
            .expect("receipt persists");
        assert_eq!(persisted.receipt, receipt);
        assert_eq!(persisted.project_revision_id, revision_id);
        assert_eq!(store.calculation_receipt_count().unwrap(), 1);
        assert_eq!(
            serde_json::from_slice::<CalculationReceipt>(
                &serde_json::to_vec(&persisted.receipt).unwrap()
            )
            .unwrap(),
            receipt
        );
    }
}

#[cfg(test)]
mod project_audit_tests {
    use super::project_audit_for_active_project;
    use crate::state::AppState;
    use std::fs;
    use wellforge_core::{Metres, Radians};
    use wellforge_survey::SurveyStation;

    #[test]
    fn project_audit_returns_only_metadata_for_the_active_project() {
        let root = tempfile::tempdir().expect("temporary directory creates");
        let project_path = root.path().join("audit.xml");
        fs::write(&project_path, b"<DrillProject/>").expect("project fixture writes");
        let state = AppState::new_for_test();
        state
            .set_project_from_native_selection(project_path)
            .expect("project selection succeeds");
        super::calculate_minimum_curvature_for_active_project(
            &state,
            super::MinimumCurvatureRequest {
                start: SurveyStation::new(
                    Metres::try_new(0.0).unwrap(),
                    Radians::try_new(0.0).unwrap(),
                    Radians::try_new(0.0).unwrap(),
                ),
                end: SurveyStation::new(
                    Metres::try_new(30.0).unwrap(),
                    Radians::try_new(0.1).unwrap(),
                    Radians::try_new(0.2).unwrap(),
                ),
            },
        )
        .expect("calculation persists a receipt");

        let audit = project_audit_for_active_project(&state).expect("audit is available");
        let json = serde_json::to_value(audit).expect("audit serializes");

        assert_eq!(json["revisions"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            json["calculationReceipts"].as_array().map(Vec::len),
            Some(1)
        );
        assert!(json["calculationReceipts"][0]["algorithm"].is_string());
        assert_eq!(json["calculationReceipts"][0]["warningCount"], 1);
        assert!(json["calculationReceipts"][0].get("warnings").is_none());
        assert!(json["calculationReceipts"][0].get("receipt").is_none());
        assert!(json["calculationReceipts"][0].get("output").is_none());
        assert!(!json.to_string().contains("local-authority.sqlite3"));
        assert!(!json.to_string().contains("audit.xml"));
    }
}

#[tauri::command]
pub fn ping() -> PingResponse {
    PingResponse {
        message: "wellforge-ready",
    }
}

#[tauri::command]
pub fn inspect_document(state: State<'_, AppState>) -> Result<DocumentInspection, ApiError> {
    inspect_active_document(&state)
}

#[tauri::command]
pub async fn select_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ProjectSelectionResult>, ApiError> {
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("WellForge project", &["drillproj", "bha", "xml"])
            .blocking_pick_file()
    })
    .await
    .map_err(|_| {
        api_error(
            "PROJECT_SELECTION_FAILED",
            "The native picker did not complete",
        )
    })?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|_| {
        api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document does not have a usable local path",
        )
    })?;
    let selected_project = state.set_project_from_native_selection(path)?;
    let inspection = inspect_document_file(&selected_project.canonical_path)?;
    Ok(Some(ProjectSelectionResult {
        project: SelectedProject {
            name: selected_project.summary.name,
        },
        inspection,
    }))
}

fn inspect_active_document(state: &AppState) -> Result<DocumentInspection, ApiError> {
    let path = state.active_project_path()?;
    inspect_document_file(Path::new(&path))
}

fn inspect_document_file(path: &Path) -> Result<DocumentInspection, ApiError> {
    reject_reparse_point(path)?;
    let canonical_path = path.canonicalize().map_err(|_| {
        api_error(
            "FORMAT_READ_FAILED",
            "The active document could not be resolved for inspection",
        )
    })?;
    let extension = canonical_path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| {
            api_error(
                "UNSUPPORTED_FILE_TYPE",
                "Only .drillproj, .bha, and .xml files can be inspected",
            )
        })?;
    if !matches!(extension.as_str(), "drillproj" | "bha" | "xml") {
        return Err(api_error(
            "UNSUPPORTED_FILE_TYPE",
            "Only .drillproj, .bha, and .xml files can be inspected",
        ));
    }

    let bytes = read_inspection_bytes(&canonical_path)?;
    match extension.as_str() {
        "drillproj" => inspect_project(&bytes),
        "bha" => inspect_bha(&bytes),
        "xml" => match inspect_project(&bytes) {
            Ok(document) => Ok(document),
            Err(error) if error.code == "FORMAT_UNSUPPORTED_ROOT" => inspect_bha(&bytes),
            Err(error) => Err(error),
        },
        _ => unreachable!("extension was validated"),
    }
}

fn read_inspection_bytes(path: &Path) -> Result<Vec<u8>, ApiError> {
    let mut file = File::open(path).map_err(|_| {
        api_error(
            "FORMAT_READ_FAILED",
            "The active document could not be read",
        )
    })?;
    let metadata = file.metadata().map_err(|_| {
        api_error(
            "FORMAT_READ_FAILED",
            "The active document could not be read",
        )
    })?;
    if !metadata.is_file() {
        return Err(api_error(
            "FORMAT_READ_FAILED",
            "The active document is not a regular file",
        ));
    }
    if metadata.len() > INSPECTION_MAX_BYTES as u64 {
        return Err(api_error(
            "FORMAT_FILE_TOO_LARGE",
            "The selected document exceeds the inspection size limit",
        ));
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take((INSPECTION_MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            api_error(
                "FORMAT_READ_FAILED",
                "The active document could not be read",
            )
        })?;
    if bytes.len() > INSPECTION_MAX_BYTES {
        return Err(api_error(
            "FORMAT_FILE_TOO_LARGE",
            "The selected document exceeds the inspection size limit",
        ));
    }
    Ok(bytes)
}

#[cfg(windows)]
fn reject_reparse_point(path: &Path) -> Result<(), ApiError> {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        api_error(
            "FORMAT_READ_FAILED",
            "The active document could not be resolved for inspection",
        )
    })?;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(api_error(
            "FORMAT_READ_FAILED",
            "The active document cannot be a reparse point",
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn reject_reparse_point(_path: &Path) -> Result<(), ApiError> {
    Ok(())
}

fn inspect_project(bytes: &[u8]) -> Result<DocumentInspection, ApiError> {
    let document = parse_project(
        bytes,
        ParseOptions {
            max_bytes: INSPECTION_MAX_BYTES,
            ..ParseOptions::default()
        },
    )
    .map_err(map_format_error)?;
    let diagnostics = document.validate().iter().map(map_diagnostic).collect();
    Ok(DocumentInspection {
        document_type: "project".to_owned(),
        root_name: document.root.name,
        caption: document.caption,
        survey_count: document.surveys.len(),
        component_count: 0,
        diagnostics,
    })
}

fn inspect_bha(bytes: &[u8]) -> Result<DocumentInspection, ApiError> {
    let document = parse_bha(
        bytes,
        ParseOptions {
            max_bytes: INSPECTION_MAX_BYTES,
            ..ParseOptions::default()
        },
    )
    .map_err(map_format_error)?;
    let diagnostics = document.validate().iter().map(map_diagnostic).collect();
    Ok(DocumentInspection {
        document_type: "bha".to_owned(),
        root_name: document.root.name,
        caption: document.caption,
        survey_count: 0,
        component_count: document.components.len(),
        diagnostics,
    })
}

fn map_diagnostic(diagnostic: &Diagnostic) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        }
        .to_owned(),
        code: diagnostic.code.to_owned(),
        message: diagnostic.message.clone(),
    }
}

fn map_format_error(error: ParseError) -> ApiError {
    let (code, message) = match error {
        ParseError::ByteLimitExceeded { .. } => (
            "FORMAT_FILE_TOO_LARGE",
            "The selected document exceeds the inspection size limit".to_owned(),
        ),
        ParseError::ForbiddenMarkup => (
            "FORMAT_FORBIDDEN_MARKUP",
            "The selected document contains unsupported XML markup".to_owned(),
        ),
        ParseError::DepthLimitExceeded { .. } | ParseError::NodeLimitExceeded { .. } => (
            "FORMAT_STRUCTURE_LIMIT",
            "The selected document exceeds the inspection structure limits".to_owned(),
        ),
        ParseError::Encoding | ParseError::UnsupportedEncoding { .. } => (
            "FORMAT_UNSUPPORTED_ENCODING",
            "The selected document uses an unsupported XML encoding".to_owned(),
        ),
        ParseError::Malformed(_) | ParseError::Xml(_) => (
            "FORMAT_MALFORMED_XML",
            "The selected document is not well-formed XML".to_owned(),
        ),
        ParseError::UnexpectedRoot { .. } => (
            "FORMAT_UNSUPPORTED_ROOT",
            "The selected document does not have a supported root element".to_owned(),
        ),
    };
    api_error(code, &message)
}

fn api_error(code: &str, message: &str) -> ApiError {
    ApiError {
        code: code.to_owned(),
        message: message.to_owned(),
        details: None,
    }
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>) -> Result<SelectedProject, ApiError> {
    state
        .save_project()
        .map(|project| SelectedProject { name: project.name })
}

/// Returns immutable project lineage and calculation-receipt metadata for the active project.
/// Document content, receipt payloads, calculation output, and local storage paths never cross
/// this desktop boundary.
#[tauri::command]
pub fn get_project_audit(state: State<'_, AppState>) -> Result<ProjectAudit, ApiError> {
    project_audit_for_active_project(&state)
}

fn project_audit_for_active_project(state: &AppState) -> Result<ProjectAudit, ApiError> {
    let audit = state.active_project_audit()?;
    Ok(ProjectAudit {
        revisions: audit.revisions.into_iter().map(Into::into).collect(),
        calculation_receipts: audit
            .calculation_receipts
            .into_iter()
            .map(Into::into)
            .collect(),
    })
}

#[tauri::command]
pub fn get_units(state: State<'_, AppState>) -> UnitPreferences {
    state.units()
}

#[tauri::command]
pub fn get_plot_preferences(state: State<'_, AppState>) -> PlotPreferences {
    state.plot_preferences()
}

#[tauri::command]
pub fn calculate_minimum_curvature(
    state: State<'_, AppState>,
    request: serde_json::Value,
) -> Result<MinimumCurvatureCalculation, ApiError> {
    calculate_minimum_curvature_for_active_project(
        &state,
        parse_minimum_curvature_request(request)?,
    )
}

fn parse_minimum_curvature_request(
    request: serde_json::Value,
) -> Result<MinimumCurvatureRequest, ApiError> {
    serde_json::from_value(request).map_err(|_| {
        api_error(
            "MALFORMED_CALCULATION_REQUEST",
            "The minimum-curvature request is malformed",
        )
    })
}

fn calculate_minimum_curvature_for_active_project(
    state: &AppState,
    request: MinimumCurvatureRequest,
) -> Result<MinimumCurvatureCalculation, ApiError> {
    let saved_revision = state.save_active_project_revision()?;
    let result = MinimumCurvatureResult::from(
        calculate_displacement_minimum_curvature(&request.start, &request.end).map_err(
            |error| ApiError {
                code: error.code().to_owned(),
                message: error.to_string(),
                details: None,
            },
        )?,
    );
    let project_sha256 = saved_revision
        .content_sha256
        .strip_prefix("sha256:")
        .expect("saved revision hashes are canonical")
        .to_owned();
    let request_value = serde_json::to_value(&request).map_err(|_| {
        api_error(
            "RECEIPT_CREATION_FAILED",
            "The calculation request could not be recorded",
        )
    })?;
    let request_sha256 =
        CalculationReceipt::canonical_json_sha256(&request_value).map_err(|_| {
            api_error(
                "RECEIPT_CREATION_FAILED",
                "The calculation request could not be recorded",
            )
        })?;
    let output = serde_json::to_value(result).map_err(|_| {
        api_error(
            "RECEIPT_CREATION_FAILED",
            "The calculation result could not be recorded",
        )
    })?;
    let receipt = CalculationReceipt::create(
        MINIMUM_CURVATURE_ALGORITHM,
        MINIMUM_CURVATURE_ALGORITHM_VERSION,
        vec![
            InputRevision {
                kind: "project_revision".to_owned(),
                id: saved_revision.revision_id.clone(),
                content_sha256: project_sha256,
            },
            InputRevision {
                kind: "minimum_curvature_request".to_owned(),
                id: request_sha256.clone(),
                content_sha256: request_sha256,
            },
        ],
        CalculationContext {
            unit_system: CANONICAL_SI_UNIT_SYSTEM.to_owned(),
            crs: CANONICAL_SURVEY_CRS.to_owned(),
            backend: CalculationBackend::Cpu,
            actor_id: LOCAL_WORKSTATION_ACTOR_ID.to_owned(),
            warnings: vec!["Actor identity is currently local to this workstation.".to_owned()],
        },
        &output,
    )
    .map_err(|_| {
        api_error(
            "RECEIPT_CREATION_FAILED",
            "The calculation receipt could not be created",
        )
    })?;

    state.persist_calculation_receipt(saved_revision.revision_id, receipt.clone(), output)?;

    Ok(MinimumCurvatureCalculation { result, receipt })
}

#[tauri::command]
pub fn build_survey_plot(request: SurveyPlotRequest) -> Result<PlotSpec, ApiError> {
    build_plan_section_plot(&request.stations).map_err(|error| ApiError {
        code: error.code().to_owned(),
        message: error.to_string(),
        details: None,
    })
}

#[tauri::command]
pub fn build_survey_scene(request: SurveySceneRequest) -> Result<SceneDocumentV1, ApiError> {
    build_survey_scene_document(&request.stations).map_err(|error| ApiError {
        code: error.code().to_owned(),
        message: error.to_string(),
        details: None,
    })
}

#[tauri::command]
pub fn run_scan(request: AcScanRequest) -> Result<ClosestApproach, ApiError> {
    closest_approach_scan(&request.reference, &request.offsets).map_err(|error| ApiError {
        code: "AC_SCAN_FAILED".into(),
        message: error.to_string(),
        details: None,
    })
}

#[cfg(test)]
mod scene_tests {
    use super::SurveySceneRequest;
    use serde_json::json;

    #[test]
    fn scene_request_rejects_a_missing_coordinate() {
        let request = serde_json::from_value::<SurveySceneRequest>(json!({
            "stations": [{
                "mdM": 0.0,
                "northM": null,
                "eastM": 0.0,
                "tvdM": 0.0
            }]
        }));

        assert!(
            request.is_err(),
            "all scene coordinates must be finite metres"
        );
    }

    #[test]
    fn scene_request_rejects_an_out_of_range_coordinate_at_the_json_boundary() {
        let request = serde_json::from_str::<SurveySceneRequest>(
            r#"{"stations":[{"mdM":0.0,"northM":1e999,"eastM":0.0,"tvdM":0.0}]}"#,
        );

        assert!(
            request.is_err(),
            "out-of-range JSON values must be rejected"
        );
    }
}

#[cfg(test)]
mod format_inspection_tests {
    use super::inspect_active_document;
    use crate::state::AppState;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn synthetic_file(extension: &str, contents: &[u8]) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wellforge-inspect-{nonce}.{extension}"));
        fs::write(&path, contents).expect("synthetic fixture writes");
        path
    }

    #[test]
    fn inspection_requires_an_active_project() {
        let error =
            inspect_active_document(&AppState::new_for_test()).expect_err("no active project");

        assert_eq!(error.code, "NO_OPEN_PROJECT");
    }

    #[test]
    fn inspection_returns_a_typed_project_summary_for_an_uppercase_extension() {
        let path = synthetic_file(
            "DRILLPROJ",
            b"<DrillProject><Caption>Fixture well</Caption><Surveys><Survey MD=\"10\" Inc=\"0\" Azi=\"0\"/></Surveys></DrillProject>",
        );
        let state = AppState::new_for_test();
        state
            .set_project_from_native_selection(path.clone())
            .expect("active project opens");
        let inspection = inspect_active_document(&state).expect("synthetic project inspects");
        fs::remove_file(path).expect("synthetic fixture removes");

        assert_eq!(inspection.document_type, "project");
        assert_eq!(inspection.caption.as_deref(), Some("Fixture well"));
        assert_eq!(inspection.survey_count, 1);
        assert_eq!(inspection.component_count, 0);
    }

    #[test]
    fn inspection_maps_malformed_synthetic_xml_to_an_api_error() {
        let path = synthetic_file("bha", b"<BHA><Caption>Broken</BHA>");
        let state = AppState::new_for_test();
        state
            .set_project_from_native_selection(path.clone())
            .expect("active project opens");
        let error = inspect_active_document(&state).expect_err("malformed XML fails");
        fs::remove_file(path).expect("synthetic fixture removes");

        assert_eq!(error.code, "FORMAT_MALFORMED_XML");
    }
}
