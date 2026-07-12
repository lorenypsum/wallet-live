use std::f32::consts::E;

use password_auth::VerifyError;

use crate::{auth::user, error::app_error::AppError, repository::repository_manager::Repository};

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
}

pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }
}