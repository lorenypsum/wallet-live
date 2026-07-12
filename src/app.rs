use axum::routing::Router;
use color_eyre::eyre::Ok;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    // TODO: escrever um sqlx error -> Result<Self, sqlx::Error>
    pub async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let db = PgPool::connect(&database_url).await?;
        Ok(Self {
            db,
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

        dotenvy::dotenv()?;
        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", crate::routes::api::router())
            .with_state(state);

        info!("Server running on http://0.0.0.0:3000");
        axum::serve(listener, router).await?;
        Ok(())
    }
}
