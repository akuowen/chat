use axum::response::Html;

pub async fn message() -> Html<&'static str> {
    Html("message")
}
