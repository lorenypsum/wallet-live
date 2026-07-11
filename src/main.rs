mod app;
mod models;
mod routes;


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    app::App::start().await?;
    Ok(())
}

