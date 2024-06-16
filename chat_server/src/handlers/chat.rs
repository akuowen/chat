use axum::response::Html;

pub async fn chat() -> Html<&'static str> {
    Html("TEST")
}
