use crate::{auth::admin::Admin, error::app_error::AppError};
use axum::{Json, Router, extract::State, routing::get};
use serde::Deserialize;
use std::collections::HashMap;
use crate::{app::AppState, models::Asset};

#[tracing::instrument(skip_all)]
pub fn router() -> Router<AppState> {
    // TODO: não tem como melhorar isso?
    Router::new().route("/assets", get(list_assets).post(create_asset).patch(update_asset))
}

#[tracing::instrument(skip_all)]
async fn list_assets(state: State<AppState>) -> Json<HashMap<i64, Asset>> {
    let assets = state.assets.lock().await;
    Json(assets.clone())
}

#[derive(Deserialize)]
struct CreateAssetRequest {
    pub name: String,
    pub unit_value: i32,
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    _admin: Admin,
    state: State<AppState>,
    Json(request): Json<CreateAssetRequest>,
) -> Json<Asset> {
    let mut assets = state.assets.lock().await;

    let id = assets
        .values()
        .map(|asset| asset.id)
        .max()
        .unwrap_or_default()
        + 1;

    let new_asset = Asset {
        id: id,
        name: request.name,
        unit_value: request.unit_value,
    };
    assets.insert(id, new_asset.clone());
    Json(new_asset)
}

#[derive(Deserialize)]
struct UpdateAssetRequest {
    id: i64,
    pub name: Option<String>,
    pub unit_value: Option<i32>,
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    _admin: Admin,
    state: State<AppState>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let mut assets = state.assets.lock().await;
    let Some(existing_asset) = assets.get_mut(&request.id) else {
        return Err(AppError::AssetNotFound);
    };

    if let Some(new_name) = request.name {
        existing_asset.name = new_name;
    }

    if let Some(new_unit_value) = request.unit_value {
        existing_asset.unit_value = new_unit_value;
    }

    Ok(Json(existing_asset.clone()))
}