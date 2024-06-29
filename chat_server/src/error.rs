use axum::http::StatusCode;
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::FORBIDDEN,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jwt_simple::algorithms::{RS384KeyPair, RS384PublicKey, RSAKeyPairLike, RSAPublicKeyLike};
    use jwt_simple::claims::Claims;
    use jwt_simple::prelude::Duration;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct MyAdditionalData {
        user_is_admin: bool,
        user_country: String,
    }

    #[test]
    fn test_encode_and_decode_should_work() {
        let decoding_pem = include_str!("../keys/public.pem");
        let encoding_pem = include_str!("../keys/private.pem");
        let key_pair = RS384KeyPair::from_pem(encoding_pem).unwrap();
        let public_key = RS384PublicKey::from_pem(decoding_pem).unwrap();

        let my_additional_data = MyAdditionalData {
            user_is_admin: false,
            user_country: "FR".to_string(),
        };

        let claims = Claims::with_custom_claims(my_additional_data, Duration::from_secs(1000));
        let encode = key_pair.sign(claims.clone()).unwrap();
        println!("{}", encode);
        let claims = public_key
            .verify_token::<MyAdditionalData>(&encode, None)
            .unwrap();
        println!("{:?}", claims);
        assert!(!claims.custom.user_is_admin)
    }
}
