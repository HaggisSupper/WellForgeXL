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

#[test]
fn detail_validation_rejects_invalid_boolean_and_duplicate_blocks() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>R</Caption><Count>1</Count><PartType>RSS</PartType><RotarySteerableDetail><PushTheBit>maybe</PushTheBit></RotarySteerableDetail><RotarySteerableDetail/></Component></Components></BHA>";
    assert!(project_bha(&parse_xml(xml).unwrap()).is_err());
}

#[test]
fn detail_duplicate_scalar_is_rejected() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>M</Caption><Count>1</Count><PartType>MudMotor</PartType><MotorDetail><LobeCount>2</LobeCount><LobeCount>3</LobeCount></MotorDetail></Component></Components></BHA>";
    assert!(matches!(
        project_bha(&parse_xml(xml).unwrap()),
        Err(InterchangeError::InvalidField {
            field: "LobeCount",
            ..
        })
    ));
}

#[test]
fn detail_absent_optionals_are_accepted() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>S</Caption><Count>1</Count><PartType>Stabilizer</PartType><StabilizerDetail/></Component></Components></BHA>";
    assert!(project_bha(&parse_xml(xml).unwrap()).is_ok());
}

#[test]
fn detail_invalid_geometry_is_rejected() {
    for detail in [
        "<CollarOD>0</CollarOD>",
        "<CollarOD>-1</CollarOD>",
        "<CollarOD>1</CollarOD><CollarID>1</CollarID>",
        "<Length>0</Length>",
    ] {
        let xml = format!(
            "<BHA><Caption>A</Caption><Components><Component><Caption>R</Caption><Count>1</Count><PartType>RSS</PartType><RotarySteerableDetail>{detail}</RotarySteerableDetail></Component></Components></BHA>"
        );
        assert!(matches!(
            project_bha(&parse_xml(&xml).unwrap()),
            Err(InterchangeError::InvalidGeometry(_))
        ));
    }
}

#[test]
fn detail_distinct_kind_conflict_is_rejected() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>M</Caption><Count>1</Count><PartType>MudMotor</PartType><MotorDetail/><StabilizerDetail/></Component></Components></BHA>";
    assert!(matches!(
        project_bha(&parse_xml(xml).unwrap()),
        Err(InterchangeError::InvalidField {
            field: "ComponentDetail",
            ..
        })
    ));
}

#[test]
fn stabilizer_fields_project_exactly() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>S</Caption><Count>1</Count><PartType>Stabilizer</PartType><StabilizerDetail><OD>0.3</OD><ID>0.1</ID><GaugeDiameter>0.31</GaugeDiameter><BladeCount>4</BladeCount><SubLength>1.2</SubLength><SubLength>0.8</SubLength></StabilizerDetail></Component></Components></BHA>";
    let a = project_bha(&parse_xml(xml).unwrap()).unwrap();
    if let ComponentDetail::Stabilizer(d) = &a.components[0].detail {
        assert_eq!(
            (
                d.od_m,
                d.id_m,
                d.gauge_diameter_m,
                d.blade_count,
                &d.sub_lengths_m
            ),
            (Some(0.3), Some(0.1), Some(0.31), Some(4), &vec![1.2, 0.8])
        );
    } else {
        panic!();
    }
}

#[test]
fn motor_nested_invalid_geometry_is_rejected() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>M</Caption><Count>1</Count><PartType>MudMotor</PartType><MotorDetail><Sections><Section><SectionType>X</SectionType><OD>0.1</OD><ID>0.2</ID><Length>1</Length></Section></Sections></MotorDetail></Component></Components></BHA>";
    assert!(matches!(
        project_bha(&parse_xml(xml).unwrap()),
        Err(InterchangeError::InvalidGeometry(_))
    ));
}

#[test]
fn rss_optional_fields_project_exactly() {
    let xml = "<BHA><Caption>A</Caption><Components><Component><Caption>R</Caption><Count>1</Count><PartType>RSS</PartType><RotarySteerableDetail><CollarOD>0.2</CollarOD><CollarID>0.1</CollarID><Length>3</Length><PadCount>3</PadCount><PadDistanceFromBit>1.5</PadDistanceFromBit><SteeringMode>push</SteeringMode><PushTheBit>false</PushTheBit></RotarySteerableDetail></Component></Components></BHA>";
    let a = project_bha(&parse_xml(xml).unwrap()).unwrap();
    if let ComponentDetail::RotarySteerable(d) = &a.components[0].detail {
        assert_eq!(
            (
                d.collar_od_m,
                d.collar_id_m,
                d.length_m,
                d.pad_count,
                d.pad_distance_from_bit_m,
                d.steering_mode.as_deref(),
                d.push_the_bit
            ),
            (
                Some(0.2),
                Some(0.1),
                Some(3.0),
                Some(3),
                Some(1.5),
                Some("push"),
                false
            )
        );
    } else {
        panic!();
    }
}
