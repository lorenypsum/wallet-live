mod app;
mod models;


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    app::App::start().await?;
    Ok(())
}

