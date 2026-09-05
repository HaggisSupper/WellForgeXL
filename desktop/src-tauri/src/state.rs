use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use wellforge_core::{
    ApiError, CalculationReceipt, PlotPreferences, ProjectSummary, UnitPreferences,
};
use wellforge_storage::{
    CoreCalculationReceiptInput, LocalStore, NewProject, ProjectRevisionInput, StorageError,
    StoredCalculationReceiptAudit, StoredProjectRevisionAudit,
};

static REVISION_EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);
const PROJECT_ARTIFACT_MAX_BYTES: usize = 4 * 1024 * 1024;

/// Application-owned mutable state. Persistent storage is introduced by the db crate.
pub struct AppState {
    project: RwLock<Option<ActiveProject>>,
    units: RwLock<UnitPreferences>,
    plots: RwLock<PlotPreferences>,
    storage: Mutex<LocalStore>,
    #[cfg(test)]
    _test_data_dir: Option<tempfile::TempDir>,
}

#[derive(Clone, Debug)]
struct ActiveProject {
    canonical_path: PathBuf,
    summary: ProjectSummary,
}

#[derive(Clone, Debug)]
pub(crate) struct SelectedProject {
    pub canonical_path: PathBuf,
    pub summary: ProjectSummary,
}

#[derive(Clone, Debug)]
pub(crate) struct SavedProjectRevision {
    pub summary: ProjectSummary,
    pub revision_id: String,
    pub content_sha256: String,
}

/// Metadata-only history for the active project. Storage keeps document and receipt payloads
/// private; this snapshot is safe for the desktop audit view.
pub(crate) struct ActiveProjectAudit {
    pub revisions: Vec<StoredProjectRevisionAudit>,
    pub calculation_receipts: Vec<StoredCalculationReceiptAudit>,
}

impl AppState {
    pub fn open(app_data_dir: impl AsRef<Path>) -> Result<Self, ApiError> {
        let app_data_dir = app_data_dir.as_ref();
        fs::create_dir_all(app_data_dir).map_err(|_| {
            api_error(
                "LOCAL_STORAGE_INIT_FAILED",
                "The application data directory could not be created",
            )
        })?;
        let storage =
            LocalStore::open(app_data_dir.join("local-authority.sqlite3")).map_err(|_| {
                api_error(
                    "LOCAL_STORAGE_INIT_FAILED",
                    "The local authority store could not be opened",
                )
            })?;
        Ok(Self::from_store(storage))
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        let test_data_dir = tempfile::tempdir().expect("test application-data directory creates");
        let storage = LocalStore::open(test_data_dir.path().join("local-authority.sqlite3"))
            .expect("test file-backed local storage opens");
        let mut state = Self::from_store(storage);
        state._test_data_dir = Some(test_data_dir);
        state
    }

    fn from_store(storage: LocalStore) -> Self {
        Self {
            project: RwLock::new(None),
            units: RwLock::new(UnitPreferences::default()),
            plots: RwLock::new(PlotPreferences::default()),
            storage: Mutex::new(storage),
            #[cfg(test)]
            _test_data_dir: None,
        }
    }

    /// Accepts only a path returned by the native desktop selection flow.
    /// This is intentionally not exposed as a Tauri command.
    pub(crate) fn set_project_from_native_selection(
        &self,
        selected_path: PathBuf,
    ) -> Result<SelectedProject, ApiError> {
        let canonical_path = validate_project_file(&selected_path)?;
        let summary = ProjectSummary::from_path(canonical_path.to_string_lossy().into_owned());
        let selected_project = SelectedProject {
            canonical_path: canonical_path.clone(),
            summary: summary.clone(),
        };
        let project = ActiveProject {
            canonical_path,
            summary: summary.clone(),
        };
        let mut active_project = self.project.write().expect("project lock poisoned");
        *active_project = Some(project.clone());
        Ok(selected_project)
    }

    pub fn save_project(&self) -> Result<ProjectSummary, ApiError> {
        self.save_active_project_revision()
            .map(|saved| saved.summary)
    }

    pub(crate) fn save_active_project_revision(&self) -> Result<SavedProjectRevision, ApiError> {
        let project = self
            .project
            .read()
            .expect("project lock poisoned")
            .as_ref()
            .map(|project| project.summary.clone())
            .ok_or_else(ApiError::no_open_project)?;
        let path = self.active_project_path()?;
        let content = read_project_content(&path)?;
        let digest = format!("sha256:{:x}", Sha256::digest(&content));
        let project_id = Self::project_id_for_path(&path);
        let timestamp = OffsetDateTime::now_utc().format(&Rfc3339).map_err(|_| {
            api_error(
                "PROJECT_SAVE_FAILED",
                "The save timestamp could not be created",
            )
        })?;
        let mut storage = self.storage.lock().expect("storage lock poisoned");
        match storage.create_project(NewProject {
            id: project_id.clone(),
            name: project.name.clone(),
            created_at: timestamp.clone(),
        }) {
            Ok(()) | Err(StorageError::Duplicate) => {}
            Err(_) => {
                return Err(api_error(
                    "PROJECT_SAVE_FAILED",
                    "The local authority store could not persist project metadata",
                ));
            }
        };
        let latest = storage.latest_project_revision(&project_id).map_err(|_| {
            api_error(
                "PROJECT_SAVE_FAILED",
                "The local authority store could not inspect project history",
            )
        })?;
        if latest
            .as_ref()
            .is_some_and(|revision| revision.content_sha256 == digest)
        {
            let revision_id = latest.expect("matching latest revision exists").id;
            return Ok(SavedProjectRevision {
                summary: project,
                revision_id,
                content_sha256: digest,
            });
        }
        let parent_revision_id = latest.map(|revision| revision.id);
        let revision_id = Self::revision_event_id(&project_id);
        storage
            .append_project_revision(ProjectRevisionInput {
                id: revision_id.clone(),
                project_id,
                parent_revision_id,
                content,
                content_sha256: digest.clone(),
                created_at: timestamp,
                actor_id: "local-workstation".to_owned(),
            })
            .map_err(|_| {
                api_error(
                    "PROJECT_SAVE_FAILED",
                    "The local authority store could not persist the project revision",
                )
            })?;
        Ok(SavedProjectRevision {
            summary: project,
            revision_id,
            content_sha256: digest,
        })
    }

    fn project_id_for_path(path: &Path) -> String {
        format!(
            "project-{:x}",
            Sha256::digest(path.to_string_lossy().as_bytes())
        )
    }

    fn revision_event_id(project_id: &str) -> String {
        let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos();
        let sequence = REVISION_EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!(
            "{project_id}:revision-event:{timestamp:x}-{:x}-{sequence:x}",
            std::process::id()
        )
    }

    pub(crate) fn persist_calculation_receipt(
        &self,
        project_revision_id: String,
        receipt: CalculationReceipt,
        output: Value,
    ) -> Result<(), ApiError> {
        let receipt_bytes = serde_json::to_vec(&receipt).map_err(|_| {
            api_error(
                "RECEIPT_PERSISTENCE_FAILED",
                "The calculation receipt could not be serialized for persistence",
            )
        })?;
        let receipt_sha256 = format!("{:x}", Sha256::digest(&receipt_bytes));
        let id = format!("receipt:{project_revision_id}:{receipt_sha256}");
        let recorded_at = OffsetDateTime::now_utc().format(&Rfc3339).map_err(|_| {
            api_error(
                "RECEIPT_PERSISTENCE_FAILED",
                "The receipt timestamp could not be created",
            )
        })?;
        let mut storage = self.storage.lock().expect("storage lock poisoned");
        match storage.record_core_calculation_receipt(CoreCalculationReceiptInput {
            id,
            project_revision_id,
            receipt,
            output,
            recorded_at,
        }) {
            Ok(()) | Err(StorageError::Duplicate) => Ok(()),
            Err(_) => Err(api_error(
                "RECEIPT_PERSISTENCE_FAILED",
                "The calculation receipt could not be persisted",
            )),
        }
    }

    pub(crate) fn active_project_audit(&self) -> Result<ActiveProjectAudit, ApiError> {
        let project_path = self.active_project_path()?;
        let project_id = Self::project_id_for_path(&project_path);
        let storage = self.storage.lock().expect("storage lock poisoned");
        let revisions = storage
            .list_project_revision_audits(&project_id)
            .map_err(|_| {
                api_error(
                    "AUDIT_READ_FAILED",
                    "The project audit history could not be read",
                )
            })?;
        let calculation_receipts = storage
            .list_calculation_receipt_audits(&project_id)
            .map_err(|_| {
                api_error(
                    "AUDIT_READ_FAILED",
                    "The project audit history could not be read",
                )
            })?;
        Ok(ActiveProjectAudit {
            revisions,
            calculation_receipts,
        })
    }

    /// Returns the only document path that read-only format commands may inspect.
    pub fn active_project_path(&self) -> Result<PathBuf, ApiError> {
        self.project
            .read()
            .expect("project lock poisoned")
            .as_ref()
            .map(|project| project.canonical_path.clone())
            .ok_or_else(ApiError::no_open_project)
    }

    pub fn units(&self) -> UnitPreferences {
        self.units.read().expect("unit lock poisoned").clone()
    }

    pub fn plot_preferences(&self) -> PlotPreferences {
        self.plots
            .read()
            .expect("plot preference lock poisoned")
            .clone()
    }
}

fn read_project_content(path: &Path) -> Result<Vec<u8>, ApiError> {
    let source_metadata = fs::symlink_metadata(path).map_err(|_| {
        api_error(
            "PROJECT_CONTENT_READ_FAILED",
            "The selected project artifact could not be read",
        )
    })?;
    if source_is_reparse_point(&source_metadata) {
        return Err(api_error(
            "PROJECT_CONTENT_READ_FAILED",
            "The selected project artifact cannot be a reparse point",
        ));
    }
    let mut file = File::open(path).map_err(|_| {
        api_error(
            "PROJECT_CONTENT_READ_FAILED",
            "The selected project artifact could not be read",
        )
    })?;
    let metadata = file.metadata().map_err(|_| {
        api_error(
            "PROJECT_CONTENT_READ_FAILED",
            "The selected project artifact could not be inspected",
        )
    })?;
    if !metadata.is_file() {
        return Err(api_error(
            "PROJECT_CONTENT_READ_FAILED",
            "The selected project artifact is not a regular file",
        ));
    }
    if metadata.len() > PROJECT_ARTIFACT_MAX_BYTES as u64 {
        return Err(api_error(
            "PROJECT_CONTENT_TOO_LARGE",
            "The selected project artifact exceeds the project size limit",
        ));
    }
    let mut content = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take((PROJECT_ARTIFACT_MAX_BYTES + 1) as u64)
        .read_to_end(&mut content)
        .map_err(|_| {
            api_error(
                "PROJECT_CONTENT_READ_FAILED",
                "The selected project artifact could not be read",
            )
        })?;
    if content.len() > PROJECT_ARTIFACT_MAX_BYTES {
        return Err(api_error(
            "PROJECT_CONTENT_TOO_LARGE",
            "The selected project artifact exceeds the project size limit",
        ));
    }
    Ok(content)
}

fn validate_project_file(path: &Path) -> Result<PathBuf, ApiError> {
    if path.as_os_str().is_empty() {
        return Err(api_error(
            "INVALID_PROJECT_PATH",
            "Project path must not be empty",
        ));
    }
    let source_metadata = fs::symlink_metadata(path).map_err(|_| {
        api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document could not be resolved",
        )
    })?;
    if source_is_reparse_point(&source_metadata) {
        return Err(api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document cannot be a reparse point",
        ));
    }
    let canonical_path = path.canonicalize().map_err(|_| {
        api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document could not be resolved",
        )
    })?;
    let extension = canonical_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(supported_type_error)?;
    if !matches!(extension.as_str(), "drillproj" | "bha" | "xml") {
        return Err(supported_type_error());
    }
    let file = File::open(&canonical_path).map_err(|_| {
        api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document could not be opened",
        )
    })?;
    if !file
        .metadata()
        .map_err(|_| {
            api_error(
                "PROJECT_SELECTION_FAILED",
                "The selected document could not be inspected",
            )
        })?
        .is_file()
    {
        return Err(api_error(
            "PROJECT_SELECTION_FAILED",
            "The selected document is not a regular file",
        ));
    }
    Ok(canonical_path)
}

fn supported_type_error() -> ApiError {
    api_error(
        "UNSUPPORTED_FILE_TYPE",
        "Only .drillproj, .bha, and .xml files can be selected",
    )
}

fn api_error(code: &str, message: &str) -> ApiError {
    ApiError {
        code: code.to_owned(),
        message: message.to_owned(),
        details: None,
    }
}

#[cfg(windows)]
fn source_is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn source_is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wellforge-{label}-{nonce}"));
        fs::create_dir(&path).expect("temporary directory creates");
        path
    }

    #[test]
    fn save_project_fails_without_an_open_project() {
        let state = AppState::new_for_test();
        let error = state
            .save_project()
            .expect_err("save requires an active project");

        assert_eq!(error.code, "NO_OPEN_PROJECT");
    }

    #[test]
    fn native_project_selection_rejects_unsupported_files_and_leaves_no_active_project() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wellforge-untrusted-{nonce}.txt"));
        fs::write(&path, b"not a portable project").expect("synthetic file writes");
        let state = AppState::new_for_test();
        let error = state
            .set_project_from_native_selection(path.clone())
            .expect_err("unsupported native selection is rejected");
        fs::remove_file(path).expect("synthetic file removes");

        assert_eq!(error.code, "UNSUPPORTED_FILE_TYPE");
        assert_eq!(
            state
                .active_project_path()
                .expect_err("project remains unset")
                .code,
            "NO_OPEN_PROJECT"
        );
    }

    #[test]
    fn native_project_selection_rejects_a_directory_named_like_a_supported_document() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wellforge-untrusted-{nonce}.xml"));
        fs::create_dir(&path).expect("synthetic directory creates");
        let error = AppState::new_for_test()
            .set_project_from_native_selection(path.clone())
            .expect_err("directory selection is rejected");
        fs::remove_dir(path).expect("synthetic directory removes");

        assert_eq!(error.code, "PROJECT_SELECTION_FAILED");
    }

    #[test]
    fn plot_preferences_start_with_the_default_palette_and_risk_bands_visible() {
        let preferences = AppState::new_for_test().plot_preferences();

        assert_eq!(preferences.palette, "wellforge-dark");
        assert!(preferences.show_risk_bands);
    }

    #[test]
    fn file_backed_state_reopens_saved_project_authority_data() {
        let root = temporary_directory("durable-state");
        let data_dir = root.join("app-data");
        let project_path = root.join("durable.xml");
        fs::write(&project_path, b"<DrillProject/>").expect("project fixture writes");

        let state = AppState::open(&data_dir).expect("file-backed state opens");
        state
            .set_project_from_native_selection(project_path)
            .expect("project is selected");
        state.save_project().expect("project is saved");
        drop(state);

        let reopened = AppState::open(&data_dir).expect("file-backed state reopens");
        assert_eq!(
            reopened
                .storage
                .lock()
                .expect("storage lock is available")
                .project_revision_count()
                .expect("revision count is readable"),
            1
        );
        drop(reopened);
        fs::remove_dir_all(root).expect("temporary directory removes");
    }

    #[test]
    fn save_project_deduplicates_unchanged_content_and_chains_changed_content() {
        let root = temporary_directory("immutable-save");
        let data_dir = root.join("app-data");
        let project_path = root.join("immutable.xml");
        fs::write(&project_path, b"<DrillProject revision=\"1\"/>")
            .expect("project fixture writes");
        let state = AppState::open(&data_dir).expect("file-backed state opens");
        state
            .set_project_from_native_selection(project_path.clone())
            .expect("project is selected");

        state.save_project().expect("first save succeeds");
        state.save_project().expect("unchanged save is idempotent");
        fs::write(&project_path, b"<DrillProject revision=\"2\"/>")
            .expect("changed project fixture writes");
        state.save_project().expect("changed save succeeds");

        let project_id = AppState::project_id_for_path(
            &state
                .active_project_path()
                .expect("active project path is available"),
        );
        let storage = state.storage.lock().expect("storage lock is available");
        assert_eq!(storage.project_revision_count().unwrap(), 2);
        let latest = storage
            .latest_project_revision(&project_id)
            .expect("latest revision query succeeds")
            .expect("latest revision exists");
        assert!(latest.parent_revision_id.is_some());
        drop(storage);
        drop(state);
        fs::remove_dir_all(root).expect("temporary directory removes");
    }

    #[test]
    fn save_project_records_a_revert_as_a_new_event_after_restart() {
        let root = temporary_directory("restart-revert");
        let data_dir = root.join("app-data");
        let project_path = root.join("revert.xml");
        let content_a = b"<DrillProject revision=\"A\"/>";
        let content_b = b"<DrillProject revision=\"B\"/>";
        fs::write(&project_path, content_a).expect("initial project fixture writes");

        let first_run = AppState::open(&data_dir).expect("first state opens");
        first_run
            .set_project_from_native_selection(project_path.clone())
            .expect("project is selected");
        first_run.save_project().expect("A saves");
        let project_id = AppState::project_id_for_path(
            &first_run
                .active_project_path()
                .expect("active path is available"),
        );
        let first_a_id = first_run
            .storage
            .lock()
            .expect("storage lock is available")
            .latest_project_revision(&project_id)
            .expect("head query succeeds")
            .expect("A head exists")
            .id;
        fs::write(&project_path, content_b).expect("changed project fixture writes");
        first_run.save_project().expect("B saves");
        drop(first_run);

        let second_run = AppState::open(&data_dir).expect("second state opens");
        second_run
            .set_project_from_native_selection(project_path.clone())
            .expect("project is selected after restart");
        fs::write(&project_path, content_a).expect("reverted project fixture writes");
        second_run.save_project().expect("reverted A saves");

        let storage = second_run
            .storage
            .lock()
            .expect("storage lock is available");
        assert_eq!(storage.project_revision_count().unwrap(), 3);
        let reverted = storage
            .latest_project_revision(&project_id)
            .expect("head query succeeds")
            .expect("reverted head exists");
        assert_ne!(reverted.id, first_a_id);
        assert_eq!(
            reverted.content_sha256,
            format!("sha256:{:x}", Sha256::digest(content_a))
        );
        drop(storage);
        drop(second_run);
        fs::remove_dir_all(root).expect("temporary directory removes");
    }

    #[test]
    fn saving_rejects_content_that_grows_beyond_the_project_limit_after_selection() {
        let root = temporary_directory("oversized-save");
        let data_dir = root.join("app-data");
        let project_path = root.join("oversized.xml");
        fs::write(&project_path, b"<DrillProject/>").expect("project fixture writes");
        let state = AppState::open(&data_dir).expect("file-backed state opens");
        state
            .set_project_from_native_selection(project_path.clone())
            .expect("small project is selected");
        fs::write(
            &project_path,
            vec![b'x'; super::PROJECT_ARTIFACT_MAX_BYTES + 1],
        )
        .expect("oversized project fixture writes");

        let error = state
            .save_project()
            .expect_err("oversized project is rejected");

        assert_eq!(error.code, "PROJECT_CONTENT_TOO_LARGE");
        assert_eq!(
            state
                .storage
                .lock()
                .expect("storage lock is available")
                .project_revision_count()
                .unwrap(),
            0
        );
        drop(state);
        fs::remove_dir_all(root).expect("temporary directory removes");
    }
}
