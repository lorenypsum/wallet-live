mod app;
mod auth;
mod error;
mod models;
mod repository;
mod routes;
mod services;

//model to use in other projects
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    app::App::start().await?;
    Ok(())
}
