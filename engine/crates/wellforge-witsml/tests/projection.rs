//! Offline WITSML identity projection tests.

use wellforge_witsml::{IdentityError, ProjectionError, project_source_identity};

#[test]
fn preserves_tubular_uuid_name_and_source_hash() {
    let xml = r#"<witsml:Tubular xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="12f55e30-2dc0-4aea-8a05-0af73d68d844"><name>Synthetic BHA</name></witsml:Tubular>"#;
    let source = project_source_identity(xml, "unit-test").unwrap();
    assert_eq!(
        source.uuid.to_string(),
        "12f55e30-2dc0-4aea-8a05-0af73d68d844"
    );
    assert_eq!(source.citation_name, "Synthetic BHA");
    assert!(source.content_hash.starts_with("sha256:"));
}

#[test]
fn rejects_unsupported_object_root() {
    let xml = r#"<witsml:MudLog xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="12f55e30-2dc0-4aea-8a05-0af73d68d844"/>"#;
    assert!(project_source_identity(xml, "unit-test").is_err());
}

#[test]
fn rejects_nil_uuid_from_witsml_xml() {
    let xml = r#"<witsml:Tubular xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="00000000-0000-0000-0000-000000000000"><name>Synthetic BHA</name></witsml:Tubular>"#;
    assert!(matches!(
        project_source_identity(xml, "unit-test"),
        Err(ProjectionError::Identity(IdentityError::NilUuid))
    ));
}

#[test]
fn rejects_witsml_xml_with_blank_name() {
    let xml = r#"<witsml:Tubular xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="12f55e30-2dc0-4aea-8a05-0af73d68d844"><name>  </name></witsml:Tubular>"#;
    assert!(matches!(
        project_source_identity(xml, "unit-test"),
        Err(ProjectionError::Identity(IdentityError::BlankCitationName))
    ));
}

#[test]
fn rejects_witsml_xml_with_missing_name() {
    let xml = r#"<witsml:Tubular xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="12f55e30-2dc0-4aea-8a05-0af73d68d844"></witsml:Tubular>"#;
    assert!(matches!(
        project_source_identity(xml, "unit-test"),
        Err(ProjectionError::Identity(IdentityError::BlankCitationName))
    ));
}

#[test]
fn rejects_witsml_xml_with_blank_source_system() {
    let xml = r#"<witsml:Tubular xmlns:witsml="http://www.energistics.org/energyml/data/witsmlv2" uuid="12f55e30-2dc0-4aea-8a05-0af73d68d844"><name>Synthetic BHA</name></witsml:Tubular>"#;
    assert!(matches!(
        project_source_identity(xml, " \t"),
        Err(ProjectionError::Identity(IdentityError::BlankSourceSystem))
    ));
}
