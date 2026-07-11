use axum::{Json, Router, extract::State, routing::get};
use serde::Deserialize;

use crate::{app::AppState, models::Asset};

#[tracing::instrument(skip_all)]
pub fn router() -> Router<AppState> {
    Router::new().route("/assets", 
    get(list_assets).post(create_asset))
}

#[tracing::instrument(skip_all)]
async fn list_assets(state: State<AppState>) -> Json<Vec<Asset>> {
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
    state: State<AppState>,
    Json(request): Json<CreateAssetRequest>,
) -> Json<Asset> {
    let mut assets = state.assets.lock().await;

    let id = assets
        .iter()
        .map(|asset| asset.id)
        .max()
        .unwrap_or_default()
        + 1;

    let new_asset = Asset {
        id: id,
        name: request.name,
        unit_value: request.unit_value,
    };
    assets.push(new_asset.clone());
    Json(new_asset)
}
