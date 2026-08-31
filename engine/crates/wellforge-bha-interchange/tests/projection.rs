#![allow(missing_docs)]
use wellforge_bha_interchange::{
    ComponentDetail, ComponentKind, InterchangeError, parse_xml, project_bha,
};

#[test]
fn projection_maps_supported_tool_details() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>M</Caption><Count>1</Count><PartType>MudMotor</PartType><MotorDetail><BendAngleDeg>1.25</BendAngleDeg></MotorDetail></Component><Component><Caption>R</Caption><Count>1</Count><PartType>RSS</PartType><RotarySteerableDetail><PushTheBit>true</PushTheBit></RotarySteerableDetail></Component></Components></BHA>";
    let assembly = project_bha(&parse_xml(xml).unwrap()).unwrap();
    let motor = assembly
        .components
        .iter()
        .find(|item| item.kind == ComponentKind::MudMotor)
        .unwrap();
    assert!(
        matches!(motor.detail, ComponentDetail::Motor(ref detail) if detail.bend_angle_deg == Some(1.25))
    );
    let rss = assembly
        .components
        .iter()
        .find(|item| item.kind == ComponentKind::Rss)
        .unwrap();
    assert!(
        matches!(rss.detail, ComponentDetail::RotarySteerable(ref detail) if detail.push_the_bit)
    );
}

#[test]
fn projection_retains_component_and_section_order() {
    let assembly =
        project_bha(&parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap()).unwrap();
    assert_eq!(assembly.components[0].name, "Neutral bit");
    assert_eq!(assembly.components[1].sections[0].kind, "Tool joint top");
    assert_eq!(assembly.components[1].sections[1].kind, "Tube");
    assert_eq!(assembly.components[1].kind, ComponentKind::Stabilizer);
}

#[test]
fn projection_uuid_is_stable_and_unknown_kind_is_preserved() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>X</Caption><Count>1</Count><PartType>Novel</PartType></Component></Components></BHA>";
    let first = project_bha(&parse_xml(xml).unwrap()).unwrap();
    let second = project_bha(&parse_xml(xml).unwrap()).unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(first.components[0].id, second.components[0].id);
    assert_eq!(
        first.components[0].kind,
        ComponentKind::Other("Novel".into())
    );
}

#[test]
fn projection_rejects_non_finite_numbers() {
    for value in ["NaN", "inf", "-inf"] {
        let xml = format!(
            "<BHA><Caption>A</Caption><Components><Component><Caption>X</Caption><Count>1</Count><PartType>Common</PartType><Sections><Section><SectionType>Tube</SectionType><OD>{value}</OD><ID>0.1</ID><Length>1</Length></Section></Sections></Component></Components></BHA>"
        );
        assert!(matches!(
            project_bha(&parse_xml(&xml).unwrap()),
            Err(InterchangeError::InvalidField { field: "OD", .. })
        ));
    }
}

#[test]
fn projection_rejects_section_with_od_not_greater_than_id() {
    let tree = parse_xml("<BHA><Caption>Neutral</Caption><Components><Component><Caption>Pipe</Caption><Count>1</Count><PartType>Common</PartType><Sections><Section><SectionType>Tube</SectionType><OD>0.1</OD><ID>0.1</ID><Length>1.0</Length></Section></Sections></Component></Components></BHA>").unwrap();
    assert!(matches!(
        project_bha(&tree),
        Err(InterchangeError::InvalidGeometry(_))
    ));
}
