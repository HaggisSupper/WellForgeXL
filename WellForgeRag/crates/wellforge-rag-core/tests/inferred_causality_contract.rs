use chrono::{TimeZone, Utc};
use serde_json::json;
use tempfile::tempdir;
use wellforge_rag_core::{
    InferredCausalityActor, InferredCausalityClaimLevel, InferredCausalityInput,
    InferredCausalityOutcome, SqliteStore,
};

#[test]
fn inferred_causality_is_persisted_as_noncanonical_episode_evidence() {
    let root = tempdir().unwrap();
    let store = SqliteStore::open(root.path().join("rag.sqlite3")).unwrap();
    let input = InferredCausalityInput {
        episode_key: "well-a/2026-09-05T12:00:00Z/torque-response".to_owned(),
        start_at: Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap(),
        end_at: Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 30).unwrap(),
        actor: InferredCausalityActor::ManualInferred,
        claim_level: InferredCausalityClaimLevel::LikelyReaction,
        situation: json!({"torque": "rising", "rop": "falling"}),
        intervention: json!({"wob": "reduced", "rpm": "increased"}),
        observed_response: json!({"torque": "fell", "rop": "recovered"}),
        outcome: InferredCausalityOutcome::Effective,
        confidence: 0.91,
        inference_method: "rule-window-v1".to_owned(),
        canonical_state_ref: Some("rig-state:rotary-drilling:segment-42".to_owned()),
        source_artifact_id: None,
    };

    let record = store.upsert_inferred_causality(input).unwrap();
    let reread = store.get_inferred_causality(record.id).unwrap().unwrap();

    assert_eq!(reread.episode_key, record.episode_key);
    assert_eq!(reread.actor, InferredCausalityActor::ManualInferred);
    assert_eq!(
        reread.claim_level,
        InferredCausalityClaimLevel::LikelyReaction
    );
    assert_eq!(reread.outcome, InferredCausalityOutcome::Effective);
    assert_eq!(reread.confidence, 0.91);
    assert_eq!(reread.authority, "Inferred_Causality");
    assert!(!reread.is_canonical);
}

#[test]
fn inferred_causality_rejects_invalid_confidence() {
    let root = tempdir().unwrap();
    let store = SqliteStore::open(root.path().join("rag.sqlite3")).unwrap();
    let input = InferredCausalityInput {
        episode_key: "bad-confidence".to_owned(),
        start_at: Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 0).unwrap(),
        end_at: Utc.with_ymd_and_hms(2026, 9, 5, 12, 0, 1).unwrap(),
        actor: InferredCausalityActor::Unknown,
        claim_level: InferredCausalityClaimLevel::ObservedSequence,
        situation: json!({}),
        intervention: json!({}),
        observed_response: json!({}),
        outcome: InferredCausalityOutcome::Inconclusive,
        confidence: 1.25,
        inference_method: "test".to_owned(),
        canonical_state_ref: None,
        source_artifact_id: None,
    };

    let error = store
        .upsert_inferred_causality(input)
        .unwrap_err()
        .to_string();
    assert!(error.contains("confidence"));
}
