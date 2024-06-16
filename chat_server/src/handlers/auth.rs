use axum::response::Html;

#[allow(unused)]
pub async fn auth() -> Html<&'static str> {
    Html("OK")
}
