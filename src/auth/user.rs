use std::f32::consts::E;

use password_auth::VerifyError;
use jwt_simple::{algorithms::MACLike, claims, prelude::{Claims, Duration, HS256Key, NoCustomClaims}};
use serde::{Deserialize, Serialize};

use crate::{app::App, auth::user, error::app_error::AppError, repository::repository_manager::Repository};

//TODO: procurar um método mais elaborado para um projeto real emde proteção de rotas, talvez com JWT
const SECRET_KEY: &[u8] = b"supersecretkeyyoushouldnotcommit";
pub struct UnauthenticatedUser {
    pub username: String,
    pub password: String,
}

impl UnauthenticatedUser {
    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = match repository.get_user(&self.username).await?{
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
        Self { username: username, password: password }
    }
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
    fn from(User {id, username}: User) -> Self {
        Self {
            id, username
        }
    }
}