use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub unit_value: f64,
}

//Todo: alterar para usar email com validação de email.
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct PurchasedHistory {
    pub bought_at: String,
    pub bought_for: f64,
    pub quantity_bought: f64,
    pub value_delta: f64,
}

#[derive(Serialize)]
pub struct OwnedAsset {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub unit_value: f64,
    pub bought_for: f64,
    pub value_delta: f64,
    pub quantity_owned: f64,
    pub last_bought_at: String,
    pub purchased_history: Json<Vec<PurchasedHistory>>,
}
