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
    extract::Query,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use tokio::try_join;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/assets", get(assets).post(purchase_asset))
        .route("/assets/update", axum::routing::post(update_owned_asset))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/register", get(register_page).post(register))
}

#[derive(Default, Deserialize)]
pub(crate) struct FlashQuery {
    error: Option<String>,
    success: Option<String>,
}

#[derive(Template)]
#[template(path = "home.html")]
#[allow(dead_code)]
struct HomePage {
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage {
    error: Option<String>,
}

async fn login_page(Query(query): Query<FlashQuery>) -> Result<Html<String>, AppError> {
    let html = LoginPage { error: query.error }.render()?;
    Ok(Html(html))
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove("token"), Redirect::to("/login"))
}

async fn register_page(Query(query): Query<FlashQuery>) -> Result<Html<String>, AppError> {
    let html = RegisterPage { error: query.error }.render()?;
    Ok(Html(html))
}

async fn home(maybe_user: Option<User>) -> Result<Html<String>, AppError> {
    let (logged_in, username) = match maybe_user {
        Some(user) => (true, user.username().to_owned()),
        None => (false, String::new()),
    };

    let html = HomePage {
        logged_in,
        username,
    }
    .render()?;
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
        Err(AppError::InvalidCredentials | AppError::UserNotFound) => {
            return Ok((
                jar,
                flash_redirect("/login", "error", "Usuário ou senha inválidos."),
            ));
        }
        Err(other_err) => return Err(other_err),
    };
    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).path("/").http_only(true);

    Ok((
        jar.add(cookie),
        flash_redirect("/assets", "success", "Bem-vindo ao dashboard."),
    ))
}

#[tracing::instrument(skip_all)]
async fn register(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.register(&repository).await {
        Ok(user) => user,
        Err(AppError::UsernameTaken) => {
            return Ok((
                jar,
                flash_redirect("/register", "error", "Esse usuário já está em uso."),
            ));
        }
        Err(other_err) => return Err(other_err),
    };
    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).path("/").http_only(true);
    Ok((
        jar.add(cookie),
        flash_redirect("/assets", "success", "Conta criada com sucesso."),
    ))
}

#[derive(Template)]
#[template(path = "assets.html")]
#[allow(dead_code)]
pub struct AssetsPage {
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    user: User,
    username: String,
    total_positions: usize,
    total_invested: f64,
    total_current_value: f64,
    total_delta: f64,
    total_return_percent: f64,
    error: Option<String>,
    success: Option<String>,
}

pub async fn assets(
    repository: Repository,
    user: User,
    Query(query): Query<FlashQuery>,
) -> Result<Html<String>, AppError> {
    let (owned_assets, available_assets) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets()
    )?;

    let total_positions = owned_assets.len();
    let total_invested = owned_assets
        .iter()
        .map(|asset| asset.quantity_owned * asset.bought_for)
        .sum::<f64>();
    let total_current_value = owned_assets
        .iter()
        .map(|asset| asset.quantity_owned * asset.unit_value)
        .sum();
    let total_delta = owned_assets.iter().map(|asset| asset.value_delta).sum();
    let total_return_percent = if total_invested > 0.0 {
        total_delta / total_invested * 100.0
    } else {
        0.0
    };

    let html = AssetsPage {
        owned_assets,
        available_assets,
        username: user.username().to_owned(),
        total_positions,
        total_invested,
        total_current_value,
        total_delta,
        total_return_percent,
        error: query.error,
        success: query.success,
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

#[derive(Deserialize)]
struct UpdateOwnedAssetForm {
    asset_id: i64,
    unit_value: f64,
    quantity: f64,
}

fn flash_redirect(path: &str, key: &str, message: &str) -> Redirect {
    let encoded = urlencoding::encode(message);
    Redirect::to(&format!("{}?{}={}", path, key, encoded))
}

fn validate_positive(name: &str, value: f64) -> Result<(), AppError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(AppError::Validation(format!(
            "{name} precisa ser maior que zero."
        )));
    }

    Ok(())
}

fn validate_asset_id(asset_id: i64) -> Result<(), AppError> {
    if asset_id <= 0 {
        return Err(AppError::Validation(
            "Selecione um ativo válido antes de continuar.".to_string(),
        ));
    }

    Ok(())
}

async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(request): Form<PurchaseAssetForm>,
) -> Result<impl IntoResponse, AppError> {
    validate_asset_id(request.asset_id)?;
    validate_positive("Quantidade", request.quantity)?;
    validate_positive("Valor unitário", request.unit_value)?;

    repository
        .insert_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        )
        .await?;

    Ok(flash_redirect(
        "/assets",
        "success",
        "Investimento cadastrado com sucesso.",
    ))
}

async fn update_owned_asset(
    repository: Repository,
    user: User,
    Form(request): Form<UpdateOwnedAssetForm>,
) -> Result<impl IntoResponse, AppError> {
    validate_asset_id(request.asset_id)?;
    validate_positive("Quantidade", request.quantity)?;
    validate_positive("Valor unitário", request.unit_value)?;

    match repository
        .update_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        )
        .await?
    {
        true => Ok(flash_redirect(
            "/assets",
            "success",
            "Posição atualizada com sucesso.",
        )),
        false => Ok(flash_redirect(
            "/assets",
            "error",
            "Posição não encontrada para edição.",
        )),
    }
}

pub mod filters {
    use askama;

    #[askama::filter_fn]
    pub fn brl(value: &f64, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("R$ {:.2}", value))
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_asset_id, validate_positive};
    use crate::error::app_error::AppError;

    #[test]
    fn validate_positive_accepts_positive_values() {
        assert!(validate_positive("Quantidade", 1.0).is_ok());
    }

    #[test]
    fn validate_positive_rejects_zero_or_negative_values() {
        let err = validate_positive("Quantidade", 0.0).expect_err("must fail");
        match err {
            AppError::Validation(message) => {
                assert!(message.contains("Quantidade"));
                assert!(message.contains("maior que zero"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_asset_id_rejects_invalid_ids() {
        let err = validate_asset_id(0).expect_err("must fail");
        match err {
            AppError::Validation(message) => {
                assert!(message.contains("ativo válido"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
