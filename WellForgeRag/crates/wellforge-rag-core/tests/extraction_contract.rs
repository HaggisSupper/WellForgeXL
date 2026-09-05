use std::{fs, fs::File, sync::Arc};

use arrow_array::{ArrayRef, Float64Array, RecordBatch};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use rusqlite::Connection;
use tempfile::tempdir;
use wellforge_rag_core::{ArtifactFamily, ExtractionStatus, extract_path};

#[test]
fn extracts_structured_text_and_drilling_formats_without_row_to_prose_expansion() {
    let root = tempdir().unwrap();

    let json_path = root.path().join("sample.json");
    fs::write(
        &json_path,
        r#"{"pressure_psi":1234.5,"state":"drilling"}"#,
    )
    .unwrap();
    let json = extract_path(&json_path).unwrap();
    assert_eq!(json.family, ArtifactFamily::StructuredData);
    assert_eq!(json.status, ExtractionStatus::Extracted);
    assert!(json.text_sections[0].text.contains("pressure_psi"));

    let csv_path = root.path().join("survey.csv");
    fs::write(&csv_path, "md_m,inc_deg\n0,0\n30,1.5\n60,3.0\n").unwrap();
    let csv = extract_path(&csv_path).unwrap();
    assert_eq!(csv.profiles[0].row_count, Some(3));
    assert_eq!(csv.profiles[0].columns[0].name, "md_m");

    let xml_path = root.path().join("trajectory.xml");
    fs::write(
        &xml_path,
        "<trajectory><station><md uom=\"m\">100</md></station></trajectory>",
    )
    .unwrap();
    let xml = extract_path(&xml_path).unwrap();
    assert_eq!(xml.family, ArtifactFamily::StructuredData);
    assert!(xml.text_sections.iter().any(|section| section.text.contains("trajectory")));

    let las_path = root.path().join("well.las");
    fs::write(
        &las_path,
        "~Version\nVERS. 2.0\n~Well\nWELL. TEST-1\n~Curve\nDEPT.M : Measured Depth\nGR.API : Gamma Ray\n~Ascii\n1000 55\n1001 57\n",
    )
    .unwrap();
    let las = extract_path(&las_path).unwrap();
    assert_eq!(las.family, ArtifactFamily::DrillingData);
    assert_eq!(las.metadata["curve_count"].as_u64(), Some(2));
    assert_eq!(las.metadata["data_rows"].as_u64(), Some(2));
    assert!(las.text_sections.iter().any(|section| section.text.contains("GR")));
}

#[test]
fn profiles_parquet_arrow_and_sqlite_without_materializing_every_row_as_text() {
    let root = tempdir().unwrap();
    let schema = Arc::new(Schema::new(vec![Field::new(
        "depth_m",
        DataType::Float64,
        false,
    )]));
    let values: ArrayRef = Arc::new(Float64Array::from(vec![1000.0, 1001.0]));
    let batch = RecordBatch::try_new(schema.clone(), vec![values]).unwrap();

    let parquet_path = root.path().join("samples.parquet");
    let mut parquet = ArrowWriter::try_new(File::create(&parquet_path).unwrap(), schema.clone(), None)
        .unwrap();
    parquet.write(&batch).unwrap();
    parquet.close().unwrap();
    let parquet = extract_path(&parquet_path).unwrap();
    assert_eq!(parquet.family, ArtifactFamily::AnalyticalData);
    assert_eq!(parquet.profiles[0].row_count, Some(2));
    assert_eq!(parquet.profiles[0].columns[0].name, "depth_m");
    assert!(parquet.text_sections.is_empty());

    let arrow_path = root.path().join("samples.arrow");
    let mut arrow = FileWriter::try_new(File::create(&arrow_path).unwrap(), &schema).unwrap();
    arrow.write(&batch).unwrap();
    arrow.finish().unwrap();
    let arrow = extract_path(&arrow_path).unwrap();
    assert_eq!(arrow.family, ArtifactFamily::AnalyticalData);
    assert_eq!(arrow.profiles[0].row_count, Some(2));
    assert_eq!(arrow.profiles[0].columns[0].name, "depth_m");

    let sqlite_path = root.path().join("field.db");
    let database = Connection::open(&sqlite_path).unwrap();
    database
        .execute(
            "CREATE TABLE surveys(md_m REAL NOT NULL, inc_deg REAL NOT NULL)",
            [],
        )
        .unwrap();
    drop(database);
    let sqlite = extract_path(&sqlite_path).unwrap();
    assert_eq!(sqlite.family, ArtifactFamily::Database);
    assert!(sqlite.profiles.iter().any(|profile| profile.name == "surveys"));
}

#[test]
fn recognized_binary_formats_report_unsupported_instead_of_fabricating_content() {
    let root = tempdir().unwrap();
    for name in ["sample.dlis", "sample.duckdb"] {
        let path = root.path().join(name);
        fs::write(&path, b"not a real database").unwrap();
        let result = extract_path(&path).unwrap();
        assert_eq!(result.status, ExtractionStatus::Unsupported);
        assert!(result.text_sections.is_empty());
        assert!(!result.warnings.is_empty());
    }
}
