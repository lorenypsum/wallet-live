use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct  Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64
}