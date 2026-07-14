use axum::routing::Router;
use color_eyre::eyre::Ok;
use reqwest::Client;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http_client: Client,
    pub brapi_token: String,
}

impl AppState {
    // TODO: escrever um sqlx error -> Result<Self, sqlx::Error>
    pub async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let brapi_token =
            std::env::var("BRAPI_TOKEN").unwrap_or_else(|_| "tKyN1YgZbjWSjgpH8r8y3m".to_string());
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(5);
        let db = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(15))
            .connect(&database_url)
            .await?;
        let http_client = Client::builder().build()?;
        Ok(Self {
            db,
            http_client,
            brapi_token,
        })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        let _ = dotenvy::dotenv();
        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", crate::routes::api::router())
            .merge(routes::front_end::router())
            .with_state(state);

        info!("Server running on http://0.0.0.0:3000");
        axum::serve(listener, router).await?;
        Ok(())
    }
}
