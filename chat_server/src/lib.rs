mod config;
mod error;
mod handlers;
mod models;

use axum::response::Html;
use axum::routing::get;
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;

use crate::handlers::{auth, chat, message};
pub use config::AppConfig;
pub use error::AppError;
pub use models::User;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    _config: AppConfig,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { _config: config }),
        }
    }
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);
    Router::new()
        .route("/", get(index_handler))
        .route("/message", get(message))
        .route("/auth", get(auth))
        .route("/chat", get(chat).patch(chat).delete(chat))
        .with_state(state)
}

pub async fn index_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
