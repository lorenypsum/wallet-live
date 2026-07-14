use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use jwt_simple::{
    algorithms::MACLike,
    prelude::{Claims, Duration, HS256Key},
};
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState, error::app_error::AppError, repository::repository_manager::Repository,
};

//TODO: procurar um método mais elaborado para um projeto real emde proteção de rotas, talvez com JWT
const SECRET_KEY: &[u8] = b"supersecretkeyyoushouldnotcommit";
pub struct UnauthenticatedUser {
    pub username: String,
    pub password: String,
}

impl UnauthenticatedUser {
    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = match repository.get_user(&self.username).await? {
            Some(user_record) => user_record,
            None => return Err(AppError::InvalidCredentials),
        };

        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(()) => Ok(User::new(user_record.id, user_record.username)),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Hashing algorithm failed: {err:?}"),
        }
    }

    pub async fn register(&self, repository: &Repository) -> Result<User, AppError> {
        validate_registration_credentials(&self.username, &self.password)?;

        let password_hash = password_auth::generate_hash(&self.password);
        let user_record = match repository.add_user(&self.username, &password_hash).await {
            Ok(user_record) => user_record,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(AppError::UsernameTaken);
            }
            Err(err) => return Err(AppError::Database(err)),
        };

        Ok(User::new(user_record.id, user_record.username))
    }

    pub fn new(username: String, password: String) -> Self {
        Self {
            username: username,
            password: password,
        }
    }
}

pub fn validate_registration_credentials(username: &str, password: &str) -> Result<(), AppError> {
    let trimmed_username = username.trim();
    if trimmed_username.chars().count() < 3 {
        return Err(AppError::Validation(
            "Username deve ter no mínimo 3 caracteres.".to_string(),
        ));
    }

    if password.chars().count() < 9 {
        return Err(AppError::Validation(
            "Senha deve ter no mínimo 9 caracteres.".to_string(),
        ));
    }

    if !password.chars().any(|ch| ch.is_ascii_digit()) {
        return Err(AppError::Validation(
            "Senha deve conter pelo menos um número.".to_string(),
        ));
    }

    if !password
        .chars()
        .any(|ch| !ch.is_alphanumeric() && !ch.is_whitespace())
    {
        return Err(AppError::Validation(
            "Senha deve conter pelo menos um símbolo.".to_string(),
        ));
    }

    Ok(())
}

pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub const fn id(&self) -> i64 {
        self.id
    }

    pub const fn username(&self) -> &String {
        &self.username
    }

    pub fn auth_token(self) -> Result<String, AppError> {
        let key = HS256Key::from_bytes(SECRET_KEY);
        //TODO: adicionar alguma informação para o usuário refazer a autenticação. (refresh token)
        let claims = Claims::with_custom_claims(UserClaims::from(self), Duration::from_mins(10));
        let token = key.authenticate(claims)?;
        Ok(token)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let key = HS256Key::from_bytes(SECRET_KEY);
        let claims: UserClaims = key.verify_token(token, None)?.custom;
        Ok(Self::new(claims.id, claims.username))
    }
}

#[derive(Serialize, Deserialize)]
struct UserClaims {
    id: i64,
    username: String,
}

impl From<User> for UserClaims {
    // Não entendi bem essa sintaxe.
    fn from(User { id, username }: User) -> Self {
        Self { id, username }
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = match jar.get("token") {
            Some(token) => token.value(),
            None => return Err(AppError::MissingAuthorization),
        };
        User::from_auth_token(token)
    }
}

impl FromRequestParts<AppState> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, _state).await.ok())
    }
}

#[cfg(test)]
mod tests {
    use super::validate_registration_credentials;
    use crate::error::app_error::AppError;

    #[test]
    fn register_validation_accepts_valid_credentials() {
        assert!(validate_registration_credentials("loren", "Abcdef1!2").is_ok());
    }

    #[test]
    fn register_validation_rejects_short_username() {
        let err = validate_registration_credentials("ab", "Abcdef1!2").expect_err("must fail");
        match err {
            AppError::Validation(message) => assert!(message.contains("mínimo 3")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn register_validation_rejects_password_without_number_or_symbol() {
        let err = validate_registration_credentials("loren", "abcdefghi").expect_err("must fail");
        match err {
            AppError::Validation(message) => assert!(
                message.contains("número")
                    || message.contains("símbolo")
                    || message.contains("mínimo 9")
            ),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
