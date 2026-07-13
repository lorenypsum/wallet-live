use crate::{
    app::AppState, auth::user::{UnauthenticatedUser, User}, error::app_error::AppError, repository::repository_manager::Repository,
};
use askama::Template;
use axum::{
    Form, Router, response::{Html, IntoResponse, Redirect, Response}, routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::{Cookie}};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
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
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserNotFound) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };
    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true);

    Ok((jar.add(cookie), Redirect::to("/"))) // Replace with actual HTML rendering

}

#[tracing::instrument(skip_all)]
async fn register(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = unauth_user.register(&repository).await?;
     let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);
    Ok((jar.add(cookie), Redirect::to("/"))) // Replace with actual HTML rendering
}

async fn index(maybe_user: Option<User>) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => {
            let html = format!("<h1>Welcome, {}!</h1>", user.username()).into_response();
            Ok(html)
        }
        None => {
            Ok(Redirect::to("/login").into_response())
        }
    }
}

