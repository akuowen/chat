mod sse;
use axum::{routing::get, Router};
pub(crate) use sse::sse_handler;

pub fn get_router() -> Router {
    Router::new().route("/sse", get(sse_handler))
}
