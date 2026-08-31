#![allow(missing_docs)]
use crate::validate;
use crate::{
    BhaAssembly, BhaComponentRecord, ComponentDetail, ComponentKind, InterchangeError,
    StructuralNode, TubularSection,
};
use uuid::Uuid;

/// Project a sanitized structural tree into canonical neutral BHA models.
///
/// # Errors
///
/// Returns an error when required fields are absent or values violate geometry rules.
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
        let detail = if sections.is_empty() {
            ComponentDetail::Generic
        } else {
            ComponentDetail::Tubular {
                sections: sections.clone(),
            }
        };
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
