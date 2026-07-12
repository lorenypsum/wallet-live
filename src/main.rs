mod app;
mod auth;
mod error;
mod models;
mod routes;
mod repository; 

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    app::App::start().await?;
    Ok(())
}
