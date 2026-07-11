use crate::models::Asset;
use std::sync::Arc;
use axum::{Json, Router, routing::get};
use axum::extract::State;
use color_eyre::eyre::Ok;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<Mutex<Vec<Asset>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .route("/", get(list_assets).post(create_asset))
            .with_state(AppState::new());
        
        info!("Server running on http://0.0.0.0:3000");
        axum::serve(listener, router).await?;
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn list_assets(state: State<AppState>) -> Json<Vec<Asset>> {
    let assets = state.assets.lock().await;
    Json(assets.clone())
}

#[tracing::instrument(skip_all)]
async fn create_asset(state: State<AppState>) -> Json<Asset> {
    todo!()
}