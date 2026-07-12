use axum::{Router, response::Html};
use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}

async fn login_page() -> Html<&'static str> {
    todo!()
}