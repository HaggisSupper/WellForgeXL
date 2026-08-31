#![allow(missing_docs)]
use crate::validate;
use crate::{
    BhaAssembly, BhaComponentRecord, ComponentDetail, ComponentKind, InterchangeError, MotorDetail,
    RotarySteerableDetail, StabilizerDetail, StructuralNode, TubularSection,
};
use uuid::Uuid;

/// Project a sanitized structural tree into canonical neutral BHA models.
///
/// # Errors
///
/// Returns an error when required fields are absent or values violate geometry rules.
#[allow(clippy::too_many_lines)]
pub fn project_bha(tree: &StructuralNode) -> Result<BhaAssembly, InterchangeError> {
    if tree.name != "BHA" {
        return Err(InterchangeError::UnsupportedRoot(tree.name.clone()));
    }
    let name = text(tree, "Caption")?;
    let components_node =
        child(tree, "Components").ok_or(InterchangeError::MissingField("Components"))?;
    let assembly_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("bha:{name}").as_bytes());
    let mut components = Vec::new();
    for (index, node) in components_node
        .children
        .iter()
        .filter(|n| n.name == "Component")
        .enumerate()
    {
        let cname = text(node, "Caption")?;
        let count = text(node, "Count")?
            .parse::<u32>()
            .ok()
            .filter(|v| *v > 0)
            .ok_or_else(|| InterchangeError::InvalidField {
                field: "Count",
                value: text(node, "Count").unwrap_or_default(),
            })?;
        let kind = match text(node, "PartType")?.as_str() {
            "Common" => ComponentKind::Common,
            "MudMotor" => ComponentKind::MudMotor,
            "RSS" => ComponentKind::Rss,
            "Stabilizer" => ComponentKind::Stabilizer,
            other => ComponentKind::Other(other.to_owned()),
        };
        let sections = child(node, "Sections")
            .map(|s| {
                s.children
                    .iter()
                    .filter(|n| n.name == "Section")
                    .map(parse_section)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        for s in &sections {
            validate::section(s)?;
        }
        let id = Uuid::new_v5(
            &assembly_id,
            format!("component:{index}:{cname}").as_bytes(),
        );
        let detail_nodes = ["MotorDetail", "RotarySteerableDetail", "StabilizerDetail"];
        let present_count: usize = detail_nodes
            .iter()
            .map(|name| node.children.iter().filter(|n| n.name == **name).count())
            .sum();
        let present: Vec<_> = detail_nodes
            .iter()
            .filter(|name| node.children.iter().any(|n| n.name == **name))
            .collect();
        if present_count > 1 || present.len() > 1 {
            return Err(InterchangeError::InvalidField {
                field: "ComponentDetail",
                value: "multiple detail blocks".into(),
            });
        }
        let detail = if let Some(d) = child(node, "MotorDetail") {
            reject_duplicate_scalars(d, &["Geometry", "BendAngleDeg", "LobeCount", "LobeRatio"])?;
            if kind != ComponentKind::MudMotor {
                return Err(InterchangeError::InvalidField {
                    field: "MotorDetail",
                    value: "component kind conflict".into(),
                });
            }
            let sections = parse_subsections(d)?;
            for s in &sections {
                validate::section(s)?;
            }
            ComponentDetail::Motor(MotorDetail {
                geometry: optional_text(d, "Geometry"),
                bend_angle_deg: optional_number(d, "BendAngleDeg")?,
                lobe_count: optional_u32(d, "LobeCount")?,
                lobe_ratio: optional_text(d, "LobeRatio"),
                subassembly_sections: sections,
            })
        } else if let Some(d) = child(node, "RotarySteerableDetail") {
            reject_duplicate_scalars(
                d,
                &[
                    "CollarOD",
                    "CollarID",
                    "Length",
                    "PadCount",
                    "PadDistanceFromBit",
                    "SteeringMode",
                    "PushTheBit",
                ],
            )?;
            if kind != ComponentKind::Rss {
                return Err(InterchangeError::InvalidField {
                    field: "RotarySteerableDetail",
                    value: "component kind conflict".into(),
                });
            }
            ComponentDetail::RotarySteerable(RotarySteerableDetail {
                collar_od_m: optional_number(d, "CollarOD")?,
                collar_id_m: optional_number(d, "CollarID")?,
                length_m: optional_number(d, "Length")?,
                pad_count: optional_u32(d, "PadCount")?,
                pad_distance_from_bit_m: optional_number(d, "PadDistanceFromBit")?,
                steering_mode: optional_text(d, "SteeringMode"),
                push_the_bit: optional_bool(d, "PushTheBit")?.unwrap_or(false),
            })
        } else if let Some(d) = child(node, "StabilizerDetail") {
            reject_duplicate_scalars(d, &["OD", "ID", "GaugeDiameter", "BladeCount"])?;
            if kind != ComponentKind::Stabilizer {
                return Err(InterchangeError::InvalidField {
                    field: "StabilizerDetail",
                    value: "component kind conflict".into(),
                });
            }
            ComponentDetail::Stabilizer(StabilizerDetail {
                od_m: optional_number(d, "OD")?,
                id_m: optional_number(d, "ID")?,
                gauge_diameter_m: optional_number(d, "GaugeDiameter")?,
                blade_count: optional_u32(d, "BladeCount")?,
                sub_lengths_m: optional_numbers(d, "SubLength")?,
            })
        } else if sections.is_empty() {
            ComponentDetail::Generic
        } else {
            ComponentDetail::Tubular {
                sections: sections.clone(),
            }
        };
        if let ComponentDetail::RotarySteerable(ref d) = detail {
            validate_rss(d)?;
        }
        if let ComponentDetail::Stabilizer(ref d) = detail {
            validate_stabilizer(d)?;
        }
        components.push(BhaComponentRecord {
            id,
            name: cname,
            count,
            kind,
            detail,
            sections,
        });
    }
    Ok(BhaAssembly {
        id: assembly_id,
        name,
        components,
    })
}
fn optional_text(node: &StructuralNode, name: &'static str) -> Option<String> {
    if node.children.iter().filter(|n| n.name == name).count() > 1 {
        return None;
    }
    child(node, name)
        .and_then(|n| n.text.clone())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}
fn reject_duplicate_scalars(
    node: &StructuralNode,
    fields: &[&'static str],
) -> Result<(), InterchangeError> {
    for field in fields {
        if node.children.iter().filter(|n| n.name == *field).count() > 1 {
            return Err(InterchangeError::InvalidField {
                field,
                value: "duplicate field".into(),
            });
        }
    }
    Ok(())
}
fn optional_bool(
    node: &StructuralNode,
    field: &'static str,
) -> Result<Option<bool>, InterchangeError> {
    if node.children.iter().filter(|n| n.name == field).count() > 1 {
        return Err(InterchangeError::InvalidField {
            field,
            value: "duplicate field".into(),
        });
    }
    match optional_text(node, field) {
        None => Ok(None),
        Some(v) if v.eq_ignore_ascii_case("true") => Ok(Some(true)),
        Some(v) if v.eq_ignore_ascii_case("false") => Ok(Some(false)),
        Some(v) => Err(InterchangeError::InvalidField { field, value: v }),
    }
}
fn optional_number(
    node: &StructuralNode,
    field: &'static str,
) -> Result<Option<f64>, InterchangeError> {
    if node.children.iter().filter(|n| n.name == field).count() > 1 {
        return Err(InterchangeError::InvalidField {
            field,
            value: "duplicate field".into(),
        });
    }
    optional_text(node, field)
        .map(|v| {
            v.parse::<f64>()
                .ok()
                .filter(|x| x.is_finite())
                .ok_or(InterchangeError::InvalidField { field, value: v })
        })
        .transpose()
}
fn optional_u32(
    node: &StructuralNode,
    field: &'static str,
) -> Result<Option<u32>, InterchangeError> {
    if node.children.iter().filter(|n| n.name == field).count() > 1 {
        return Err(InterchangeError::InvalidField {
            field,
            value: "duplicate field".into(),
        });
    }
    optional_text(node, field)
        .map(|v| {
            v.parse::<u32>()
                .ok()
                .filter(|x| *x > 0)
                .ok_or(InterchangeError::InvalidField { field, value: v })
        })
        .transpose()
}
fn validate_rss(d: &RotarySteerableDetail) -> Result<(), InterchangeError> {
    for v in [
        d.collar_od_m,
        d.collar_id_m,
        d.length_m,
        d.pad_distance_from_bit_m,
    ]
    .into_iter()
    .flatten()
    {
        if !v.is_finite() || v < 0.0 {
            return Err(InterchangeError::InvalidGeometry(
                "RSS dimensions must be finite and non-negative".into(),
            ));
        }
    }
    if let (Some(od), Some(id)) = (d.collar_od_m, d.collar_id_m)
        && od <= id
    {
        return Err(InterchangeError::InvalidGeometry(
            "RSS OD must exceed ID".into(),
        ));
    }
    Ok(())
}
fn validate_stabilizer(d: &StabilizerDetail) -> Result<(), InterchangeError> {
    for v in [d.od_m, d.id_m, d.gauge_diameter_m].into_iter().flatten() {
        if !v.is_finite() || v < 0.0 {
            return Err(InterchangeError::InvalidGeometry(
                "stabilizer dimensions must be finite and non-negative".into(),
            ));
        }
    }
    if let (Some(od), Some(id)) = (d.od_m, d.id_m)
        && od <= id
    {
        return Err(InterchangeError::InvalidGeometry(
            "stabilizer OD must exceed ID".into(),
        ));
    }
    if d.sub_lengths_m.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err(InterchangeError::InvalidGeometry(
            "stabilizer lengths must be positive".into(),
        ));
    }
    Ok(())
}
fn optional_numbers(
    node: &StructuralNode,
    field: &'static str,
) -> Result<Vec<f64>, InterchangeError> {
    node.children
        .iter()
        .filter(|n| n.name == field)
        .map(|n| {
            let value = n
                .text
                .clone()
                .ok_or(InterchangeError::MissingField(field))?;
            value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or(InterchangeError::InvalidField { field, value })
        })
        .collect()
}
fn parse_subsections(node: &StructuralNode) -> Result<Vec<TubularSection>, InterchangeError> {
    match child(node, "Sections") {
        None => Ok(Vec::new()),
        Some(s) => s
            .children
            .iter()
            .filter(|n| n.name == "Section")
            .map(parse_section)
            .collect(),
    }
}

fn parse_section(node: &StructuralNode) -> Result<TubularSection, InterchangeError> {
    Ok(TubularSection {
        kind: text(node, "SectionType")?,
        od_m: number(node, "OD")?,
        id_m: number(node, "ID")?,
        length_m: number(node, "Length")?,
        mass_kg: child(node, "Mass")
            .map(|_| number(node, "Mass"))
            .transpose()?,
    })
}
fn number(node: &StructuralNode, field: &'static str) -> Result<f64, InterchangeError> {
    text(node, field)?
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| InterchangeError::InvalidField {
            field,
            value: text(node, field).unwrap_or_default(),
        })
}
fn child<'a>(node: &'a StructuralNode, name: &str) -> Option<&'a StructuralNode> {
    node.children.iter().find(|n| n.name == name)
}
fn text(node: &StructuralNode, name: &'static str) -> Result<String, InterchangeError> {
    child(node, name)
        .and_then(|n| n.text.clone())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .ok_or(InterchangeError::MissingField(name))
}
