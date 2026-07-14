use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::app_error::AppError,
    models::{Asset, OwnedAsset},
    repository::repository_manager::Repository,
};
use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use tokio::try_join;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/assets", get(assets).post(purchase_asset))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
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

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove("token"), Redirect::to("/login"))
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
    let cookie = Cookie::build(("token", token)).path("/").http_only(true);

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

async fn index(maybe_user: Option<User>) -> Result<Redirect, AppError> {
    match maybe_user {
        Some(_) => Ok(Redirect::to("/assets")),
        None => Ok(Redirect::to("/login")),
    }
}

#[derive(Template)]
#[template(path = "assets.html")]
pub struct AssetsPage {
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    user: User,
}

pub async fn assets(repository: Repository, user: User) -> Result<Html<String>, AppError> {
    let (owned_assets, available_assets) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets()
    )?;

    let html = AssetsPage {
        owned_assets,
        available_assets,
        user,
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct PurchaseAssetForm {
    asset_id: i64,
    unit_value: f64,
    quantity: f64,
}

pub async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(request): Form<PurchaseAssetForm>,
) -> Result<Redirect, AppError> {
    repository
        .insert_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        )
        .await?;
    Ok(Redirect::to("/assets"))
}

pub mod filters {
    use askama;
    use time::{
        OffsetDateTime, format_description::StaticFormatDescription, macros::format_description,
    };

    #[askama::filter_fn]
    pub fn human_datetime(
        datetime: &OffsetDateTime,
        _env: &dyn askama::Values,
    ) -> askama::Result<String> {
        const HUMAN_READABLE_FORMAT: StaticFormatDescription = format_description!(
            "[year]-[month]-[day] [hour]:[minute]
            "
        );

        datetime
            .format(&HUMAN_READABLE_FORMAT)
            .map_err(askama::Error::custom)
    }
}
