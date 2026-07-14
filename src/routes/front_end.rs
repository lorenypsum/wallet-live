use std::collections::{BTreeSet, HashMap};

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::app_error::AppError,
    models::OwnedAsset,
    repository::repository_manager::Repository,
    services::brapi::{BrapiQuote, PRESET_ASSETS, fetch_quotes},
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use time::{PrimitiveDateTime, format_description::FormatItem, macros::format_description};
use tokio::try_join;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/assets", get(assets).post(create_position))
        .route("/assets/update", axum::routing::post(update_owned_asset))
        .route("/assets/delete", axum::routing::post(delete_owned_asset))
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

#[derive(Clone)]
struct PresetAssetView {
    symbol: String,
    name: String,
    current_price: f64,
}

#[derive(Clone)]
struct PortfolioItem {
    asset_id: i64,
    name: String,
    symbol: String,
    quantity_owned: f64,
    bought_for: f64,
    bought_at_input: String,
    current_price: f64,
    current_value: f64,
    result_value: f64,
    result_percent: f64,
    is_profit: bool,
}

#[derive(Template)]
#[template(path = "assets.html")]
#[allow(dead_code)]
pub struct AssetsPage {
    username: String,
    portfolio: Vec<PortfolioItem>,
    preset_assets: Vec<PresetAssetView>,
    total_positions: usize,
    total_invested: f64,
    total_current_value: f64,
    total_delta: f64,
    total_return_percent: f64,
    error: Option<String>,
    success: Option<String>,
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

pub async fn assets(
    State(state): State<AppState>,
    repository: Repository,
    user: User,
    Query(query): Query<FlashQuery>,
) -> Result<Html<String>, AppError> {
    let (owned_assets, _) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets()
    )?;

    let symbols = collect_symbols(&owned_assets);
    let quotes = fetch_quotes(&state.http_client, &state.brapi_token, &symbols)
        .await
        .unwrap_or_default();

    let portfolio = build_portfolio(owned_assets, &quotes);
    let preset_assets = build_presets(&quotes);

    let total_positions = portfolio.len();
    let total_invested = portfolio
        .iter()
        .map(|asset| asset.bought_for * asset.quantity_owned)
        .sum();
    let total_current_value = portfolio.iter().map(|asset| asset.current_value).sum();
    let total_delta = portfolio.iter().map(|asset| asset.result_value).sum();
    let total_return_percent = if total_invested > 0.0 {
        total_delta / total_invested * 100.0
    } else {
        0.0
    };

    let html = AssetsPage {
        username: user.username().to_owned(),
        portfolio,
        preset_assets,
        total_positions,
        total_invested,
        total_current_value,
        total_delta,
        total_return_percent,
        error: query.error,
        success: query.success,
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct CreatePositionForm {
    asset_symbol: String,
    asset_name: String,
    bought_for: String,
    current_price: String,
    quantity: String,
    bought_at: String,
}

#[derive(Deserialize)]
struct UpdateOwnedAssetForm {
    asset_id: String,
    asset_name: String,
    bought_for: String,
    current_price: String,
    quantity: String,
    bought_at: String,
}

#[derive(Deserialize)]
struct DeleteOwnedAssetForm {
    asset_id: String,
}

async fn create_position(
    repository: Repository,
    user: User,
    Form(request): Form<CreatePositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let symbol = normalize_symbol(&request.asset_symbol)?;
    let name = normalize_name(&request.asset_name, &symbol)?;
    let bought_at = normalize_bought_at(&request.bought_at)?;
    let quantity = parse_positive("Quantidade", &request.quantity)?;
    let bought_for = parse_positive("Preço de compra", &request.bought_for)?;
    let current_price = parse_positive("Preço atual", &request.current_price)?;

    let asset = match repository.find_asset_by_symbol(&symbol).await? {
        Some(asset) => {
            repository
                .update_asset(asset.id, None, None, Some(current_price))
                .await?;
            asset
        }
        None => repository.create_asset(name, symbol, current_price).await?,
    };

    repository
        .insert_owned_asset(user.id(), asset.id, quantity, bought_for, bought_at)
        .await?;

    Ok(flash_redirect(
        "/assets",
        "success",
        "Ativo cadastrado na carteira com sucesso.",
    ))
}

async fn update_owned_asset(
    repository: Repository,
    user: User,
    Form(request): Form<UpdateOwnedAssetForm>,
) -> Result<impl IntoResponse, AppError> {
    let asset_id = parse_asset_id(&request.asset_id)?;
    let asset_name = normalize_portfolio_name(&request.asset_name)?;
    let quantity = parse_positive("Quantidade", &request.quantity)?;
    let bought_for = parse_positive("Preço de compra", &request.bought_for)?;
    let current_price = parse_positive("Preço atual", &request.current_price)?;
    let bought_at = normalize_bought_at(&request.bought_at)?;

    repository
        .update_asset(asset_id, Some(asset_name), None, Some(current_price))
        .await?;

    match repository
        .update_owned_asset(user.id(), asset_id, quantity, bought_for, bought_at)
        .await?
    {
        true => Ok(flash_redirect(
            "/assets",
            "success",
            "Ativo atualizado com sucesso.",
        )),
        false => Ok(flash_redirect(
            "/assets",
            "error",
            "Ativo não encontrado para edição.",
        )),
    }
}

async fn delete_owned_asset(
    repository: Repository,
    user: User,
    Form(request): Form<DeleteOwnedAssetForm>,
) -> Result<impl IntoResponse, AppError> {
    let asset_id = parse_asset_id(&request.asset_id)?;

    match repository.delete_owned_asset(user.id(), asset_id).await? {
        true => Ok(flash_redirect(
            "/assets",
            "success",
            "Ativo removido da carteira com sucesso.",
        )),
        false => Ok(flash_redirect(
            "/assets",
            "error",
            "Ativo não encontrado para exclusão.",
        )),
    }
}

fn build_presets(quotes: &HashMap<String, BrapiQuote>) -> Vec<PresetAssetView> {
    PRESET_ASSETS
        .iter()
        .map(|preset| {
            let quote = quotes.get(preset.symbol);
            PresetAssetView {
                symbol: preset.symbol.to_string(),
                name: quote
                    .and_then(|item| {
                        item.data.as_ref().and_then(|data| {
                            data.short_name.clone().or_else(|| data.long_name.clone())
                        })
                    })
                    .unwrap_or_else(|| preset.name.to_string()),
                current_price: quote
                    .and_then(|item| {
                        item.data
                            .as_ref()
                            .and_then(|data| data.regular_market_price)
                    })
                    .unwrap_or(0.0),
            }
        })
        .collect()
}

fn build_portfolio(
    owned_assets: Vec<OwnedAsset>,
    quotes: &HashMap<String, BrapiQuote>,
) -> Vec<PortfolioItem> {
    owned_assets
        .into_iter()
        .map(|asset| {
            let current_price = quotes
                .get(&asset.symbol)
                .and_then(|quote| {
                    quote
                        .data
                        .as_ref()
                        .and_then(|data| data.regular_market_price)
                })
                .unwrap_or(asset.unit_value);
            let current_value = current_price * asset.quantity_owned;
            let result_value = (current_price - asset.bought_for) * asset.quantity_owned;
            let invested = asset.bought_for * asset.quantity_owned;
            let result_percent = if invested > 0.0 {
                result_value / invested * 100.0
            } else {
                0.0
            };

            PortfolioItem {
                asset_id: asset.id,
                name: asset.name,
                symbol: asset.symbol,
                quantity_owned: asset.quantity_owned,
                bought_for: asset.bought_for,
                bought_at_input: format_datetime_local(&asset.last_bought_at),
                current_price,
                current_value,
                result_value,
                result_percent,
                is_profit: result_value >= 0.0,
            }
        })
        .collect()
}

fn collect_symbols(owned_assets: &[OwnedAsset]) -> Vec<String> {
    let mut symbols = BTreeSet::new();
    for preset in PRESET_ASSETS {
        symbols.insert(preset.symbol.to_string());
    }
    for asset in owned_assets {
        symbols.insert(asset.symbol.clone());
    }
    symbols.into_iter().collect()
}

fn format_datetime_local(input: &str) -> String {
    let normalized = input.replace(' ', "T");
    normalized.chars().take(16).collect()
}

fn normalize_bought_at(value: &str) -> Result<PrimitiveDateTime, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Informe a data e hora da compra.".to_string(),
        ));
    }

    const DATETIME_LOCAL_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]");

    PrimitiveDateTime::parse(trimmed, DATETIME_LOCAL_FORMAT).map_err(|_| {
        AppError::Validation("A data da compra precisa estar no formato correto.".to_string())
    })
}

fn normalize_symbol(value: &str) -> Result<String, AppError> {
    let normalized = value.trim().to_uppercase();
    if normalized.is_empty() || !normalized.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(AppError::Validation(
            "Selecione um ativo válido da lista da BRAPI.".to_string(),
        ));
    }
    Ok(normalized)
}

fn normalize_name(value: &str, symbol: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(symbol.to_string());
    }
    Ok(trimmed.to_string())
}

fn normalize_portfolio_name(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Nome do ativo é obrigatório.".to_string(),
        ));
    }

    Ok(trimmed.to_string())
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

fn parse_positive(name: &str, value: &str) -> Result<f64, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{name} é obrigatório.")));
    }

    let parsed = trimmed
        .parse::<f64>()
        .map_err(|_| AppError::Validation(format!("{name} precisa ser um número válido.")))?;
    validate_positive(name, parsed)?;
    Ok(parsed)
}

fn parse_asset_id(value: &str) -> Result<i64, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Selecione um ativo válido antes de continuar.".to_string(),
        ));
    }

    let parsed = trimmed.parse::<i64>().map_err(|_| {
        AppError::Validation("Selecione um ativo válido antes de continuar.".to_string())
    })?;
    validate_asset_id(parsed)?;
    Ok(parsed)
}

pub mod filters {
    use askama;

    #[askama::filter_fn]
    pub fn brl(value: &f64, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("R$ {:.2}", value))
    }

    #[askama::filter_fn]
    pub fn percent(value: &f64, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("{value:.2}%"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_bought_at, normalize_portfolio_name, parse_positive, validate_asset_id,
        validate_positive,
    };
    use crate::error::app_error::AppError;
    use time::macros::datetime;

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

    #[test]
    fn normalize_bought_at_formats_datetime_local() {
        let value = normalize_bought_at("2026-07-14T15:45").expect("valid datetime");
        assert_eq!(value, datetime!(2026-07-14 15:45:00));
    }

    #[test]
    fn parse_positive_rejects_empty_values() {
        let err = parse_positive("Quantidade", "").expect_err("must fail");
        match err {
            AppError::Validation(message) => {
                assert!(message.contains("Quantidade"));
                assert!(message.contains("obrigatório"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn normalize_portfolio_name_rejects_empty_values() {
        let err = normalize_portfolio_name("   ").expect_err("must fail");
        match err {
            AppError::Validation(message) => {
                assert!(message.contains("Nome do ativo"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
