//! Structural checks for the public interchange API.

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
