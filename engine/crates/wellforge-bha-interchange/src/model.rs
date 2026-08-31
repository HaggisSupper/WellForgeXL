#![allow(missing_docs)]
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ComponentKind {
    Common,
    MudMotor,
    Rss,
    Stabilizer,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ComponentDetail {
    Generic,
    Tubular { sections: Vec<TubularSection> },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TubularSection {
    pub kind: String,
    pub od_m: f64,
    pub id_m: f64,
    pub length_m: f64,
    pub mass_kg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BhaComponentRecord {
    pub id: Uuid,
    pub name: String,
    pub count: u32,
    pub kind: ComponentKind,
    pub detail: ComponentDetail,
    pub sections: Vec<TubularSection>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BhaAssembly {
    pub id: Uuid,
    pub name: String,
    pub components: Vec<BhaComponentRecord>,
}
