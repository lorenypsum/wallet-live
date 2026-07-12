use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct  Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64
}

//Todo: alterar para usar email com validação de email.
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}