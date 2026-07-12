use crate::routes::api::CreateAssetRequest;
use crate::routes::api::create_asset;
use std::convert::Infallible;
use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::{AppState},
    models::Asset,
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(Asset, "SELECT id, name, unit_value FROM assets")
            .fetch_all(&self.db)
            .await
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value) VALUES ($1, $2) RETURNING id, name, unit_value",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(&self, 
        asset_id: i64, 
        name: Option<String>, 
        unit_value: Option<f64>) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets SET name = COALESCE($2, name), unit_value = COALESCE($3, unit_value) WHERE id = $1 RETURNING id, name, unit_value",
            asset_id,
            name,
            unit_value,
            
        )
        .fetch_optional(&self.db).await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}

#[cfg(test)]
mod tests {

    use axum::Json;
    use sqlx::PgPool;
    use crate::auth::admin::Admin;

use super::*;

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let request = CreateAssetRequest {
            name: "Test Asset".to_string(),
            unit_value: 100.0,
        };
        let Json(new_asset) = create_asset(Admin, db.into(), Json(request)).await.expect("Success");
        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Test Asset");
        assert_eq!(new_asset.unit_value, 100.0);

        insta::assert_json_snapshot!(new_asset);
        // cargo insta review --accept
    }
}