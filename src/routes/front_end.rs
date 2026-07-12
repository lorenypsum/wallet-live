use crate::{
    app::AppState, auth::user::UnauthenticatedUser, error::app_error::AppError,
    repository::repository_manager::Repository,
};
use askama::Template;
use axum::{
    Form, Router,
    response::Html,
    routing::{get, post},
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage;

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

async fn register_page() -> Result<Html<String>, AppError> {
    let html = RegisterPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    Form(request): Form<LoginForm>,
) -> Result<Html<String>, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = unauth_user.authenticate(&repository).await?;
    Ok(Html(user.username().clone())) // Replace with actual HTML rendering
}

#[tracing::instrument(skip_all)]
async fn register(
    repository: Repository,
    Form(request): Form<LoginForm>,
) -> Result<Html<String>, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = unauth_user.register(&repository).await?;
    Ok(Html(format!(
        "Usuário {} registrado com sucesso.",
        user.username()
    )))
}
