use std::convert::Infallible;
use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState, models::{Asset, OwnedAsset, UserRecord},
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

    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        sqlx::query_as!(
            OwnedAsset,
            r#"
            SELECT 
                a.id, 
                a.name, 
                a.unit_value, 
                MAX(o.bought_for) AS "bought_for!",
                SUM((a.unit_value - o.bought_for) * o.quantity_owned) AS "value_delta!", 
                SUM(o.quantity_owned) AS "quantity_owned!", 
                JSON_AGG(
                JSON_BUILD_OBJECT(
                    'bought_at', o.timestamp::text, 
                    'bought_for', o.bought_for, 
                    'quantity_bought', o.quantity_owned, 
                    'value_delta', (a.unit_value - o.bought_for) * o.quantity_owned
        )
        ) AS "purchased_history!: _"
         FROM assets AS a
         JOIN owned_assets AS o 
         ON o.asset_id = a.id
         WHERE o.user_id = $1
         GROUP BY a.id;
         "#,
            user_id
        ).fetch_all(&self.db)
        .await
    }
    
    pub async fn insert_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity_owned: f64,
        bought_for: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO owned_assets (user_id, asset_id, bought_for, quantity_owned)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, asset_id)
            DO UPDATE SET
                quantity_owned = owned_assets.quantity_owned + EXCLUDED.quantity_owned,
                bought_for = (
                    (owned_assets.bought_for * owned_assets.quantity_owned)
                    + (EXCLUDED.bought_for * EXCLUDED.quantity_owned)
                ) / (owned_assets.quantity_owned + EXCLUDED.quantity_owned),
                timestamp = NOW()
            "#,
            user_id,
            asset_id,
            bought_for,
            quantity_owned
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity_owned: f64,
        bought_for: f64,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE owned_assets
            SET quantity_owned = $3,
                bought_for = $4,
                timestamp = NOW()
            WHERE user_id = $1 AND asset_id = $2
            "#,
            user_id,
            asset_id,
            quantity_owned,
            bought_for
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
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
    use crate::repository::repository_manager::Repository;
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

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_owned_asset(db: PgPool) {
        let user_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO users (username, password_hash)
            VALUES ('alice', 'password_hash')
            RETURNING id
            "#,
        )
        .fetch_one(&db)
        .await
        .expect("user inserted");

        let repository = Repository::from(db.clone());
        repository
            .insert_owned_asset(user_id, 1, 2.0, 100.0)
            .await
            .expect("investment inserted");

        let updated = repository
            .update_owned_asset(user_id, 1, 3.5, 125.0)
            .await
            .expect("update succeeds");

        assert!(updated);

        let positions = repository
            .list_owned_assets(user_id)
            .await
            .expect("positions listed");

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].quantity_owned, 3.5);
        assert_eq!(positions[0].bought_for, 125.0);
    }
}