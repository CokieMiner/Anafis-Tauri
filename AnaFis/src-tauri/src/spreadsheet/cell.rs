use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Uncertainty {
    pub value: f64,
    pub uncertainty: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "value")]
pub enum UnifiedCell {
    Text(String),
    Number(f64),
    Boolean(bool),
    Uncertainty(Uncertainty),
    Empty,
}
