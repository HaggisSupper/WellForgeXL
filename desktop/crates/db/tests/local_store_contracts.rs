use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    time::{SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use wellforge_core::{CalculationBackend, CalculationContext, CalculationReceipt, InputRevision};
use wellforge_storage::{
    CoreCalculationReceiptInput, LocalStore, NewProject, ProjectRevisionInput, ReplayEventInput,
    StorageError, SyncEnvelope, SyncOperation,
};

const PROJECT_CONTENT: &[u8] = br#"<project id="alpha"/>"#;
const PROJECT_SHA256: &str =
    "sha256:80908047ab4e9c1b9a92d5953d32dc1e5bb574537e5d421cfe60b0b35d1ff775";

struct TestStore {
    store: LocalStore,
    _directory: TempDir,
}

impl std::ops::Deref for TestStore {
    type Target = LocalStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl std::ops::DerefMut for TestStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}

fn test_store() -> TestStore {
    let directory = tempfile::tempdir().expect("temporary store directory creates");
    let store = LocalStore::open(directory.path().join("authority.sqlite3"))
        .expect("temporary file store opens");
    TestStore {
        store,
        _directory: directory,
    }
}

fn project() -> NewProject {
    NewProject {
        id: "project-alpha".to_owned(),
        name: "Alpha Field Plan".to_owned(),
        created_at: "2026-08-25T14:30:00Z".to_owned(),
    }
}

fn revision() -> ProjectRevisionInput {
    ProjectRevisionInput {
        id: "revision-001".to_owned(),
        project_id: "project-alpha".to_owned(),
        parent_revision_id: None,
        content: PROJECT_CONTENT.to_vec(),
        content_sha256: PROJECT_SHA256.to_owned(),
        created_at: "2026-08-25T14:31:00Z".to_owned(),
        actor_id: "engineer-42".to_owned(),
    }
}

fn temporary_database_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("wellforge-contract-{label}-{nonce}.sqlite3"))
}

fn remove_database_files(path: &Path) {
    for candidate in [
        path.to_path_buf(),
        PathBuf::from(format!("{}-wal", path.display())),
        PathBuf::from(format!("{}-shm", path.display())),
    ] {
        if candidate.exists() {
            fs::remove_file(candidate).expect("temporary database file removes");
        }
    }
}

fn revision_with(
    id: &str,
    parent_revision_id: Option<&str>,
    content: &[u8],
) -> ProjectRevisionInput {
    ProjectRevisionInput {
        id: id.to_owned(),
        project_id: "project-alpha".to_owned(),
        parent_revision_id: parent_revision_id.map(str::to_owned),
        content: content.to_vec(),
        content_sha256: format!("sha256:{:x}", Sha256::digest(content)),
        created_at: "2026-08-25T14:31:00Z".to_owned(),
        actor_id: "engineer-42".to_owned(),
    }
}

#[test]
fn local_store_persists_a_project_revision_and_authority_records() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision())
        .expect("revision is appended");

    let (receipt, output) = core_receipt("revision-001", &PROJECT_SHA256[7..]);
    store
        .record_core_calculation_receipt(CoreCalculationReceiptInput {
            id: "receipt-001".to_owned(),
            project_revision_id: "revision-001".to_owned(),
            receipt,
            output,
            recorded_at: "2026-08-25T14:32:00Z".to_owned(),
        })
        .expect("typed receipt is appended");
    store
        .append_replay_event(ReplayEventInput {
            id: "event-001".to_owned(),
            project_id: "project-alpha".to_owned(),
            project_revision_id: Some("revision-001".to_owned()),
            sequence: 1,
            event_type: "revision.created".to_owned(),
            payload: json!({"revisionId": "revision-001"}),
            occurred_at: "2026-08-25T14:31:00Z".to_owned(),
            actor_id: "engineer-42".to_owned(),
        })
        .expect("event is appended");
    store
        .enqueue_sync_envelope(SyncEnvelope {
            artifact_hash: PROJECT_SHA256.to_owned(),
            parent_revision_id: Some("revision-001".to_owned()),
            idempotency_key: "sync-001".to_owned(),
            actor_id: "engineer-42".to_owned(),
            occurred_at: "2026-08-25T14:33:00Z".to_owned(),
            operation: SyncOperation::Upsert,
        })
        .expect("envelope is appended");

    assert_eq!(store.project_count().unwrap(), 1);
    assert_eq!(store.project_revision_count().unwrap(), 1);
    assert_eq!(store.calculation_receipt_count().unwrap(), 1);
    assert_eq!(store.replay_event_count().unwrap(), 1);
    assert_eq!(store.sync_envelope_count().unwrap(), 1);
}

#[test]
fn local_store_rejects_content_hashes_that_do_not_match_content() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    let mut invalid = revision();
    invalid.content_sha256 =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned();

    let error = store
        .append_project_revision(invalid)
        .expect_err("wrong content hash must not be persisted");

    assert_eq!(error, StorageError::ContentHashMismatch);
    assert_eq!(store.project_revision_count().unwrap(), 0);
}

#[test]
fn local_store_rejects_duplicate_idempotency_keys_and_unknown_parents() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    let mut child = revision();
    child.parent_revision_id = Some("missing-revision".to_owned());

    assert!(matches!(
        store.append_project_revision(child),
        Err(StorageError::InvalidRelationship)
    ));

    let envelope = SyncEnvelope {
        artifact_hash: PROJECT_SHA256.to_owned(),
        parent_revision_id: None,
        idempotency_key: "sync-001".to_owned(),
        actor_id: "engineer-42".to_owned(),
        occurred_at: "2026-08-25T14:33:00Z".to_owned(),
        operation: SyncOperation::Create,
    };
    store
        .enqueue_sync_envelope(envelope.clone())
        .expect("first envelope is appended");

    assert!(matches!(
        store.enqueue_sync_envelope(envelope),
        Err(StorageError::Duplicate)
    ));
}

#[test]
fn local_store_returns_the_latest_immutable_revision_for_parent_chaining() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision())
        .expect("first revision is appended");
    let second_content = br#"<project id="alpha" revision="2"/>"#.to_vec();
    let second_hash = format!("sha256:{:x}", Sha256::digest(&second_content));
    store
        .append_project_revision(ProjectRevisionInput {
            id: "revision-002".to_owned(),
            project_id: "project-alpha".to_owned(),
            parent_revision_id: Some("revision-001".to_owned()),
            content: second_content,
            content_sha256: second_hash.clone(),
            created_at: "2026-08-25T14:32:00Z".to_owned(),
            actor_id: "engineer-42".to_owned(),
        })
        .expect("second revision is appended");

    let latest = store
        .latest_project_revision("project-alpha")
        .expect("latest revision query succeeds")
        .expect("latest revision exists");

    assert_eq!(latest.id, "revision-002");
    assert_eq!(latest.parent_revision_id.as_deref(), Some("revision-001"));
    assert_eq!(latest.content_sha256, second_hash);
}

#[test]
fn revision_events_allow_a_revert_to_prior_content_as_a_new_head() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision_with("event-a-1", None, b"A"))
        .expect("first content event appends");
    store
        .append_project_revision(revision_with("event-b", Some("event-a-1"), b"B"))
        .expect("second content event appends");
    store
        .append_project_revision(revision_with("event-a-2", Some("event-b"), b"A"))
        .expect("reverted content appends as a new event");

    let latest = store
        .latest_project_revision("project-alpha")
        .expect("head query succeeds")
        .expect("head exists");
    assert_eq!(store.project_revision_count().unwrap(), 3);
    assert_eq!(latest.id, "event-a-2");
    assert_eq!(latest.parent_revision_id.as_deref(), Some("event-b"));
    assert_eq!(
        latest.content_sha256,
        revision_with("ignored", None, b"A").content_sha256
    );
}

#[test]
fn revision_append_rejects_a_second_root_and_a_stale_parent() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision_with("root", None, b"root"))
        .expect("root appends");

    assert_eq!(
        store
            .append_project_revision(revision_with("second-root", None, b"other-root"))
            .expect_err("a second root is rejected"),
        StorageError::InvalidRelationship
    );
    store
        .append_project_revision(revision_with("child", Some("root"), b"child"))
        .expect("current-head child appends");
    assert_eq!(
        store
            .append_project_revision(revision_with("stale-child", Some("root"), b"stale"))
            .expect_err("a stale parent is rejected"),
        StorageError::InvalidRelationship
    );
    assert_eq!(store.project_revision_count().unwrap(), 2);
}

#[test]
fn concurrent_sibling_appends_allow_exactly_one_head_advance() {
    let path = temporary_database_path("concurrent-head");
    let mut setup = LocalStore::open(&path).expect("database opens");
    setup.create_project(project()).expect("project is created");
    setup
        .append_project_revision(revision_with("root", None, b"root"))
        .expect("root appends");
    drop(setup);

    let barrier = Arc::new(Barrier::new(2));
    let handles = [
        ("sibling-a", b"A".as_slice()),
        ("sibling-b", b"B".as_slice()),
    ]
    .into_iter()
    .map(|(id, content)| {
        let path = path.clone();
        let barrier = Arc::clone(&barrier);
        let revision = revision_with(id, Some("root"), content);
        std::thread::spawn(move || {
            let mut store = LocalStore::open(path).expect("concurrent connection opens");
            barrier.wait();
            store.append_project_revision(revision)
        })
    })
    .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().expect("append thread completes"))
        .collect::<Vec<_>>();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(StorageError::InvalidRelationship)))
            .count(),
        1
    );
    let store = LocalStore::open(&path).expect("database reopens");
    assert_eq!(store.project_revision_count().unwrap(), 2);
    drop(store);
    remove_database_files(&path);
}

#[test]
fn local_store_rejects_cross_project_parent_revisions_and_self_parents() {
    let mut store = test_store();
    store
        .create_project(project())
        .expect("first project is created");
    store
        .create_project(NewProject {
            id: "project-beta".to_owned(),
            name: "Beta Field Plan".to_owned(),
            created_at: "2026-08-25T14:30:00Z".to_owned(),
        })
        .expect("second project is created");
    store
        .append_project_revision(revision())
        .expect("first revision is appended");

    let mut cross_project = revision();
    cross_project.id = "revision-beta".to_owned();
    cross_project.project_id = "project-beta".to_owned();
    cross_project.parent_revision_id = Some("revision-001".to_owned());
    let cross_project_error = store
        .append_project_revision(cross_project)
        .expect_err("a parent from another project is rejected");

    let mut self_parent = revision();
    self_parent.id = "revision-self".to_owned();
    self_parent.parent_revision_id = Some("revision-self".to_owned());
    let self_parent_error = store
        .append_project_revision(self_parent)
        .expect_err("a revision cannot parent itself");

    assert_eq!(cross_project_error, StorageError::InvalidRelationship);
    assert_eq!(self_parent_error, StorageError::InvalidRelationship);
    assert_eq!(store.project_revision_count().unwrap(), 1);
}

#[test]
fn local_store_rejects_replay_events_bound_to_another_project_revision() {
    let mut store = test_store();
    store
        .create_project(project())
        .expect("first project is created");
    store
        .create_project(NewProject {
            id: "project-beta".to_owned(),
            name: "Beta Field Plan".to_owned(),
            created_at: "2026-08-25T14:30:00Z".to_owned(),
        })
        .expect("second project is created");
    store
        .append_project_revision(revision())
        .expect("first revision is appended");

    let error = store
        .append_replay_event(ReplayEventInput {
            id: "event-beta".to_owned(),
            project_id: "project-beta".to_owned(),
            project_revision_id: Some("revision-001".to_owned()),
            sequence: 1,
            event_type: "revision.created".to_owned(),
            payload: json!({"revisionId": "revision-001"}),
            occurred_at: "2026-08-25T14:31:00Z".to_owned(),
            actor_id: "engineer-42".to_owned(),
        })
        .expect_err("a replay event cannot reference another project's revision");

    assert_eq!(error, StorageError::InvalidRelationship);
    assert_eq!(store.replay_event_count().unwrap(), 0);
}

fn core_receipt(
    revision_id: &str,
    revision_content_sha256: &str,
) -> (CalculationReceipt, serde_json::Value) {
    let output = json!({"result": 42});
    let receipt = CalculationReceipt::create(
        "minimum-curvature",
        "2026.1",
        vec![InputRevision {
            kind: "project_revision".to_owned(),
            id: revision_id.to_owned(),
            content_sha256: revision_content_sha256.to_owned(),
        }],
        CalculationContext {
            unit_system: "si".to_owned(),
            crs: "EPSG:4979".to_owned(),
            backend: CalculationBackend::Cpu,
            actor_id: "engineer-42".to_owned(),
            warnings: Vec::new(),
        },
        &output,
    )
    .expect("receipt is valid");
    (receipt, output)
}

#[test]
fn local_store_persists_and_loads_a_typed_core_receipt_bound_to_its_revision() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision())
        .expect("revision is appended");
    let revision_digest = PROJECT_SHA256
        .strip_prefix("sha256:")
        .expect("fixture hash is prefixed");
    let (receipt, output) = core_receipt("revision-001", revision_digest);

    store
        .record_core_calculation_receipt(CoreCalculationReceiptInput {
            id: "core-receipt-001".to_owned(),
            project_revision_id: "revision-001".to_owned(),
            receipt: receipt.clone(),
            output,
            recorded_at: "2026-08-25T14:32:00Z".to_owned(),
        })
        .expect("typed receipt is persisted");

    let stored = store
        .load_core_calculation_receipt("core-receipt-001")
        .expect("typed receipt loads")
        .expect("typed receipt exists");
    assert_eq!(stored.project_revision_id, "revision-001");
    assert_eq!(stored.receipt, receipt);
    assert_eq!(store.calculation_receipt_count().unwrap(), 1);
}

#[test]
fn local_store_rejects_a_typed_receipt_with_a_mismatched_revision_binding() {
    let mut store = test_store();
    store.create_project(project()).expect("project is created");
    store
        .append_project_revision(revision())
        .expect("revision is appended");
    let revision_digest = PROJECT_SHA256
        .strip_prefix("sha256:")
        .expect("fixture hash is prefixed");
    let (receipt, output) = core_receipt("another-revision", revision_digest);

    let error = store
        .record_core_calculation_receipt(CoreCalculationReceiptInput {
            id: "core-receipt-mismatch".to_owned(),
            project_revision_id: "revision-001".to_owned(),
            receipt,
            output,
            recorded_at: "2026-08-25T14:32:00Z".to_owned(),
        })
        .expect_err("receipt input identity must bind to the stored revision");

    assert_eq!(error, StorageError::InvalidCalculationReceipt);
    assert_eq!(store.calculation_receipt_count().unwrap(), 0);
}

#[test]
fn typed_receipt_fixture_hash_matches_the_revision_bytes() {
    assert_eq!(
        format!("{:x}", Sha256::digest(PROJECT_CONTENT)),
        &PROJECT_SHA256[7..]
    );
}
