use std::convert::Infallible;
use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState, models::{Asset, UserRecord},
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

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash) 
            VALUES ($1, $2)
            RETURNING id, username, password_hash;",
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash 
            FROM users 
            WHERE username = $1",
            username
        )
        .fetch_optional(&self.db)
        .await
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
    use crate::{auth::admin::Admin, routes::api::{UpdateAssetRequest, list_assets, update_asset}};
    use crate::routes::api::CreateAssetRequest;
    use crate::routes::api::create_asset;

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let request = CreateAssetRequest {
            name: "Bitcoin".to_string(),
            unit_value: 100.0,
        };
        let Json(new_asset) = create_asset(Admin, db.into(), Json(request)).await.expect("Success");
        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Bitcoin");
        assert_eq!(new_asset.unit_value, 100.0);

        // cargo insta review --accept
        insta::assert_json_snapshot!(new_asset);
        
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
        async fn test_list_assets(db: PgPool) { 
        let Json(assets) = list_assets(db.into()).await.expect("Success");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Bitcoin");
        
        insta::assert_json_snapshot!(assets);
    }

     #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_asset(db: PgPool) {
        let request = UpdateAssetRequest {
            id: 1,
            name: Some("Ethereum".to_string()),
            unit_value: Some(20.0),
        };

        let Json(updated_asset) = update_asset(Admin, db.into(), Json(request))
        .await
        .expect("Success");

        assert_eq!(updated_asset.id, 1);
        assert_eq!(updated_asset.name, "Ethereum");
        assert_eq!(updated_asset.unit_value, 20.0);

        // cargo insta review --accept
        insta::assert_json_snapshot!(updated_asset);
        
    }
}