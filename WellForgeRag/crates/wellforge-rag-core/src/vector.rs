use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result, bail};
use arrow_array::{
    FixedSizeListArray, Float32Array, RecordBatch, StringArray, types::Float32Type,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{
    database::CreateTableMode,
    query::{ExecutableQuery, QueryBase},
};
use uuid::Uuid;

const TABLE_NAME: &str = "chunks";

#[derive(Debug, Clone, PartialEq)]
pub struct VectorRecord {
    pub chunk_id: Uuid,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorHit {
    pub chunk_id: Uuid,
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct LanceVectorIndex {
    root: PathBuf,
    dimension: usize,
}

impl LanceVectorIndex {
    pub fn new(root: PathBuf, dimension: usize) -> Result<Self> {
        if dimension == 0 {
            bail!("vector dimension must be greater than zero");
        }
        std::fs::create_dir_all(&root)
            .with_context(|| format!("cannot create LanceDB directory {}", root.display()))?;
        Ok(Self { root, dimension })
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub async fn rebuild(&self, records: &[VectorRecord]) -> Result<()> {
        for record in records {
            self.validate_vector(&record.vector)?;
        }

        let dimension = i32::try_from(self.dimension).context("vector dimension exceeds i32")?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dimension,
                ),
                false,
            ),
        ]));

        let ids = StringArray::from_iter_values(records.iter().map(|record| record.chunk_id.to_string()));
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            records
                .iter()
                .map(|record| Some(record.vector.iter().copied().map(Some).collect::<Vec<_>>())),
            dimension,
        );
        let batch = RecordBatch::try_new(schema, vec![Arc::new(ids), Arc::new(vectors)])?;

        let db = lancedb::connect(self.root.to_string_lossy().as_ref())
            .execute()
            .await
            .context("cannot connect to local LanceDB")?;
        db.create_table(TABLE_NAME, batch)
            .mode(CreateTableMode::Overwrite)
            .execute()
            .await
            .context("cannot rebuild LanceDB chunk table")?;
        Ok(())
    }

    pub async fn search(&self, query: &[f32], limit: usize) -> Result<Vec<VectorHit>> {
        self.validate_vector(query)?;
        if limit == 0 {
            return Ok(Vec::new());
        }

        let db = lancedb::connect(self.root.to_string_lossy().as_ref())
            .execute()
            .await
            .context("cannot connect to local LanceDB")?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("LanceDB chunk table is not initialized; rebuild the vector index first")?;
        let batches = table
            .query()
            .nearest_to(query)?
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut hits = Vec::new();
        for batch in batches {
            let ids = batch
                .column_by_name("chunk_id")
                .and_then(|array| array.as_any().downcast_ref::<StringArray>())
                .context("LanceDB result missing UTF-8 chunk_id column")?;
            let distances = batch
                .column_by_name("_distance")
                .and_then(|array| array.as_any().downcast_ref::<Float32Array>())
                .context("LanceDB result missing Float32 _distance column")?;
            if ids.len() != distances.len() {
                bail!("LanceDB result column length mismatch");
            }
            for row in 0..ids.len() {
                let chunk_id = Uuid::parse_str(ids.value(row))
                    .with_context(|| format!("invalid chunk UUID returned by LanceDB: {}", ids.value(row)))?;
                hits.push(VectorHit {
                    chunk_id,
                    distance: distances.value(row),
                });
            }
        }
        Ok(hits)
    }

    fn validate_vector(&self, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimension {
            bail!(
                "vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            );
        }
        if vector.iter().any(|value| !value.is_finite()) {
            bail!("vector contains a non-finite value");
        }
        Ok(())
    }
}
