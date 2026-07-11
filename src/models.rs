use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct  Asset {
    pub id: u32,
    pub name: String,
    pub unit_value: i32
}