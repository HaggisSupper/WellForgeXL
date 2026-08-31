//! Structural checks for the public interchange API.

use std::collections::BTreeSet;
use sha2::{Digest, Sha256};

fn digest_for_test(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}

#[test]
fn sanitizer_removes_matching_element_attribute_and_value() {
    let policy = wellforge_bha_interchange::SanitizationPolicy::new(BTreeSet::from([
        digest_for_test("restrictedtoken"),
    ]));
    let tree = wellforge_bha_interchange::parse_xml(
        "<BHA source=\"restrictedtoken\"><Caption>Neutral</Caption><restrictedtoken>restrictedtoken</restrictedtoken></BHA>",
    )
    .unwrap();
    let (sanitized, report) = wellforge_bha_interchange::sanitize_tree(tree, &policy).unwrap();
    assert!(sanitized.attributes.is_empty());
    assert_eq!(sanitized.children.len(), 1);
    assert_eq!(report.removed_elements, 1);
    assert_eq!(report.removed_attributes, 1);
    assert_eq!(report.removed_values, 0);
}

#[test]
fn structural_json_retains_child_order() {
    let tree = wellforge_bha_interchange::parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap();
    let json = wellforge_bha_interchange::structural_json::to_value(&tree);
    assert_eq!(json["children"][0]["name"], "Components");
    assert_eq!(json["children"][0]["children"][0]["name"], "Component");
    assert_eq!(json["children"][0]["children"][1]["name"], "Component");
}

#[test]
fn crate_exposes_a_typed_error() {
    let error = wellforge_bha_interchange::InterchangeError::UnsupportedRoot("Other".into());
    assert_eq!(error.to_string(), "unsupported BHA XML root: Other");
}

#[test]
fn parser_preserves_repeated_component_order() {
    let tree =
        wellforge_bha_interchange::parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap();
    let components = tree
        .children
        .iter()
        .find(|node| node.name == "Components")
        .unwrap();
    let captions: Vec<_> = components
        .children
        .iter()
        .map(|component| {
            component
                .children
                .iter()
                .find(|node| node.name == "Caption")
                .unwrap()
                .text
                .as_deref()
        })
        .collect();
    assert_eq!(captions, vec![Some("Neutral bit"), Some("Neutral tubular")]);
}

#[test]
fn parser_rejects_a_doctype() {
    let xml = "<!DOCTYPE BHA [<!ENTITY sample SYSTEM 'file:///not-used'>]><BHA/>";
    assert_eq!(
        wellforge_bha_interchange::parse_xml(xml).unwrap_err(),
        wellforge_bha_interchange::InterchangeError::ProhibitedXmlConstruct
    );
}

#[test]
fn parser_rejects_whitespace_variant_constructs_and_outside_text() {
    for xml in [
        "< !DOCTYPE BHA><BHA/>",
        "<! ENTITY x 'y'><BHA/>",
        "<BHA/><?custom x?>",
        "x<BHA/>",
        "<BHA/>x",
    ] {
        assert!(wellforge_bha_interchange::parse_xml(xml).is_err());
    }
}

#[test]
fn parser_preserves_cdata_text() {
    let tree =
        wellforge_bha_interchange::parse_xml("<BHA><Note><![CDATA[a < b]]></Note></BHA>").unwrap();
    assert_eq!(tree.children[0].text.as_deref(), Some("a < b"));
}

#[test]
fn parser_preserves_literal_declaration_markers_in_content() {
    for xml in [
        "<BHA><![CDATA[<!DOCTYPE]]></BHA>",
        "<BHA><!-- <!ENTITY --> </BHA>",
        "<BHA><![CDATA[<?custom]]></BHA>",
    ] {
        assert!(wellforge_bha_interchange::parse_xml(xml).is_ok());
    }
}

#[test]
fn parser_rejects_misplaced_xml_declaration() {
    assert!(wellforge_bha_interchange::parse_xml("<BHA/><?xml version='1.0'?>").is_err());
    assert!(
        wellforge_bha_interchange::parse_xml("<!--comment--><?xml version='1.0'?><BHA/>").is_err()
    );
    assert!(
        wellforge_bha_interchange::parse_xml("<![CDATA[x]]><?xml version='1.0'?><BHA/>").is_err()
    );
}
