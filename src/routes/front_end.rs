use crate::{app::{App, AppState}, error::app_error::AppError};
use askama::Template;
use axum::{Router, response::Html};

pub fn router() -> Router<AppState> {
    Router::new()
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}
