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
    Motor(MotorDetail),
    RotarySteerable(RotarySteerableDetail),
    Stabilizer(StabilizerDetail),
}
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct MotorDetail {
    pub geometry: Option<String>,
    pub bend_angle_deg: Option<f64>,
    pub lobe_count: Option<u32>,
    pub lobe_ratio: Option<String>,
    pub subassembly_sections: Vec<TubularSection>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct RotarySteerableDetail {
    pub collar_od_m: Option<f64>,
    pub collar_id_m: Option<f64>,
    pub length_m: Option<f64>,
    pub pad_count: Option<u32>,
    pub pad_distance_from_bit_m: Option<f64>,
    pub steering_mode: Option<String>,
    pub push_the_bit: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct StabilizerDetail {
    pub od_m: Option<f64>,
    pub id_m: Option<f64>,
    pub gauge_diameter_m: Option<f64>,
    pub blade_count: Option<u32>,
    pub sub_lengths_m: Vec<f64>,
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
