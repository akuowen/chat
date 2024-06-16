use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("pass hash error: {0}")]
    PassHashError(#[from] argon2::password_hash::Error),
}
