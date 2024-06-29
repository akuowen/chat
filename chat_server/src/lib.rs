use std::ops::Deref;
use std::sync::Arc;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::StatusCode;
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::headers::authorization::Bearer;
use axum_extra::{headers::Authorization, TypedHeader};
use jwt_simple::algorithms::{RS384PublicKey, RSAPublicKeyLike};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub use config::AppConfig;
pub use error::{AppError, ErrorOutput};
pub use models::User;

use crate::handlers::{auth, chat, message};

mod config;
mod error;
mod handlers;
mod models;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
    pub async fn create_pool(&self) -> anyhow::Result<sqlx::PgPool> {
        // Ok(sqlx::PgPool::connect(&self.connection_string())
        //     .await?)
        Ok(PgPool::connect(&self.connection_string()).await?)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) _config: AppConfig,
    // pub(crate) _pool: PgPool,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        // let pool = config
        //     .database
        //     .create_pool()
        //     .await
        //     .expect("Failed to create connection pool");
        Self {
            inner: Arc::new(AppStateInner {
                _config: config,
                // _pool: pool,
            }),
        }
    }
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config).await;
    anyhow::Ok(
        Router::new()
            .route("/", get(index_handler))
            .route("/message", get(message))
            .route("/auth", get(auth))
            .route("/chat", get(chat).patch(chat).delete(chat))
            .layer(from_fn_with_state(state.clone(), my_middleware))
            .with_state(state),
    )
}

pub async fn index_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn my_middleware(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(token))) => {
                let bearer = token.token();
                match token_is_valid(bearer) {
                    Ok(my_additional_data) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(my_additional_data);
                        req
                    }
                    Err(_) => return StatusCode::FORBIDDEN.into_response(),
                }
            }
            Err(_) => return StatusCode::FORBIDDEN.into_response(),
        };

    next.run(req).await
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MyAdditionalData {
    user_is_admin: bool,
    user_country: String,
}

fn token_is_valid(token: &str) -> Result<MyAdditionalData, AppError> {
    // get the public key from the file every request . shou be cached by appState
    let decoding_pem = include_str!("../keys/public.pem");
    let decode = RS384PublicKey::from_pem(decoding_pem)?;
    let result = decode.verify_token::<MyAdditionalData>(token, None)?;
    Ok(result.custom)
}
