use anyhow::Result;
use notify_server::get_router;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let addr: String = format!("0.0.0.0:{}", "8888");
    let app = get_router();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("server is running on {}", addr);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
