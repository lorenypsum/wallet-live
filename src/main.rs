mod app;
mod models;
mod routes;
mod auth;
mod error;


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    app::App::start().await?;
    Ok(())
}

