use tempfile::tempdir;
use uuid::Uuid;
use wellforge_rag_core::{LanceVectorIndex, VectorRecord};

#[tokio::test]
async fn lance_index_rebuilds_and_returns_nearest_chunk() {
    let root = tempdir().unwrap();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let index = LanceVectorIndex::new(root.path().join("lancedb"), 4).unwrap();

    index
        .rebuild(&[
            VectorRecord {
                chunk_id: first,
                vector: vec![1.0, 0.0, 0.0, 0.0],
            },
            VectorRecord {
                chunk_id: second,
                vector: vec![0.0, 1.0, 0.0, 0.0],
            },
        ])
        .await
        .unwrap();

    let hits = index.search(&[0.9, 0.1, 0.0, 0.0], 2).await.unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].chunk_id, first);
    assert!(hits[0].distance <= hits[1].distance);
}

#[tokio::test]
async fn lance_index_rejects_wrong_vector_dimension() {
    let root = tempdir().unwrap();
    let index = LanceVectorIndex::new(root.path().join("lancedb"), 4).unwrap();
    let error = index.search(&[1.0, 0.0], 1).await.unwrap_err().to_string();
    assert!(error.contains("dimension"));
}
