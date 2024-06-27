mod config;
mod error;
mod handlers;
mod models;

use anyhow::Ok;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::ops::Deref;
use std::sync::Arc;

use crate::handlers::{auth, chat, message};
pub use models::User;

pub use config::AppConfig;
pub use error::{AppError, ErrorOutput};

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
    pub(crate) _pool: PgPool,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let pool = config
            .database
            .create_pool()
            .await
            .expect("Failed to create connection pool");
        Self {
            inner: Arc::new(AppStateInner {
                _config: config,
                _pool: pool,
            }),
        }
    }
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config).await;
    Ok(Router::new()
        .route("/", get(index_handler))
        .route("/message", get(message))
        .route("/auth", get(auth))
        .route("/chat", get(chat).patch(chat).delete(chat))
        .with_state(state))
}

pub async fn index_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
