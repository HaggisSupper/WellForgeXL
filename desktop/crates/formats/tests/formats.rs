use wellforge_formats::{
    ParseError, ParseOptions, WriteMode, parse_bha, parse_project, validate_project,
};

#[test]
fn parses_project_and_preserves_original_bytes() {
    let source = include_bytes!("fixtures/minimal-project.xml");
    let document =
        parse_project(source, ParseOptions::default()).expect("synthetic project parses");
    assert_eq!(document.root.name, "DrillProject");
    assert_eq!(document.caption.as_deref(), Some("North Field"));
    assert_eq!(document.surveys.len(), 2);
    assert_eq!(document.surveys[1].measured_depth, Some(125.5));
    assert_eq!(
        document.write(WriteMode::PreserveUnchanged).expect("write"),
        source
    );
    assert!(validate_project(&document).is_empty());
}

#[test]
fn preserves_component_order() {
    let document = parse_bha(
        include_bytes!("fixtures/bha-order.xml"),
        ParseOptions::default(),
    )
    .expect("synthetic BHA parses");
    let labels: Vec<_> = document
        .components
        .iter()
        .map(|component| component.caption.as_str())
        .collect();
    assert_eq!(labels, ["Bit", "Motor", "Sensor"]);
}

#[test]
fn reports_unknown_nodes_in_encounter_order() {
    let document = parse_project(
        include_bytes!("fixtures/unknown-node-order.xml"),
        ParseOptions::default(),
    )
    .expect("synthetic project parses");
    let names: Vec<_> = document
        .unknown_nodes
        .iter()
        .map(|node| node.name.as_str())
        .collect();
    assert_eq!(names, ["CustomOne", "VendorNode", "CustomTwo"]);
}

#[test]
fn projects_only_the_root_caption() {
    let document = parse_project(
        include_bytes!("fixtures/caption-scope.xml"),
        ParseOptions::default(),
    )
    .expect("caption scope fixture parses");
    assert_eq!(document.caption.as_deref(), Some("Root Caption"));
}

#[test]
fn supports_declared_single_byte_encoding_with_whitespace() {
    let source = b"<?xml version = '1.0' encoding = 'windows-1251'?>\
<DrillProject><Caption>\xcf\xf0\xe8\xec\xe5\xf0</Caption></DrillProject>";
    let document = parse_project(source, ParseOptions::default()).expect("single-byte XML parses");
    assert_eq!(
        document.caption.as_deref(),
        Some("\u{041f}\u{0440}\u{0438}\u{043c}\u{0435}\u{0440}")
    );
    assert_eq!(
        document.write(WriteMode::PreserveUnchanged).expect("write"),
        source
    );
}

#[test]
fn rejects_forbidden_markup_after_single_byte_decoding() {
    let source = b"<?xml version = '1.0' encoding = 'windows-1251'?>\
<!DOCTYPE DrillProject [<!ENTITY value 'x'>]><DrillProject/>";
    assert!(matches!(
        parse_project(source, ParseOptions::default()),
        Err(ParseError::ForbiddenMarkup)
    ));
}

#[test]
fn rejects_utf_16_boms_and_declarations_explicitly() {
    let bom = b"\xff\xfe<\0D\0r\0i\0l\0l\0P\0r\0o\0j\0e\0c\0t\0/\0>\0";
    assert!(matches!(
        parse_project(bom, ParseOptions::default()),
        Err(ParseError::UnsupportedEncoding { ref label }) if label == "UTF-16"
    ));

    let declaration = b"<?xml version='1.0' encoding='UTF-16'?><DrillProject/>";
    assert!(matches!(
        parse_project(declaration, ParseOptions::default()),
        Err(ParseError::UnsupportedEncoding { ref label }) if label == "UTF-16"
    ));
}

#[test]
fn distinguishes_invalid_survey_numbers_from_absent_values() {
    let document = parse_project(
        include_bytes!("fixtures/invalid-survey-values.xml"),
        ParseOptions::default(),
    )
    .expect("invalid numeric values do not prevent structural parsing");
    assert_eq!(document.surveys[0].measured_depth, None);
    let diagnostics = document.validate();
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert_eq!(
        codes,
        [
            "INVALID_SURVEY_MEASURED_DEPTH",
            "INVALID_SURVEY_INCLINATION",
            "INVALID_SURVEY_INCLINATION"
        ]
    );
}

#[test]
fn rejects_malformed_and_wrong_roots() {
    assert!(matches!(
        parse_project(
            include_bytes!("fixtures/malformed.xml"),
            ParseOptions::default()
        ),
        Err(ParseError::Malformed(_)) | Err(ParseError::Xml(_))
    ));
    assert!(matches!(
        parse_project(
            include_bytes!("fixtures/wrong-root.xml"),
            ParseOptions::default()
        ),
        Err(ParseError::UnexpectedRoot { .. })
    ));
}

#[test]
fn rejects_dtd_and_enforces_byte_limit() {
    let dtd = b"<!DOCTYPE DrillProject [<!ENTITY x 'x'>]><DrillProject/>";
    assert!(matches!(
        parse_project(dtd, ParseOptions::default()),
        Err(ParseError::ForbiddenMarkup)
    ));
    let options = ParseOptions {
        max_bytes: 4,
        ..ParseOptions::default()
    };
    assert!(matches!(
        parse_project(include_bytes!("fixtures/minimal-project.xml"), options),
        Err(ParseError::ByteLimitExceeded { .. })
    ));
}

#[test]
fn enforces_depth_and_node_limits() {
    let deeply_nested = b"<DrillProject><A><B/></A></DrillProject>";
    let depth_options = ParseOptions {
        max_depth: 2,
        ..ParseOptions::default()
    };
    assert!(matches!(
        parse_project(deeply_nested, depth_options),
        Err(ParseError::DepthLimitExceeded { .. })
    ));

    let node_options = ParseOptions {
        max_nodes: 2,
        ..ParseOptions::default()
    };
    assert!(matches!(
        parse_project(include_bytes!("fixtures/minimal-project.xml"), node_options),
        Err(ParseError::NodeLimitExceeded { .. })
    ));
}
