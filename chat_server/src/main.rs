mod config;

use anyhow::Result;
use axum::Router;
use chat_server::DatabaseConfig;
use chat_server::{get_router, AppConfig};
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let app_config = AppConfig::try_load()?;
    let addr: String = format!("0.0.0.0:{}", app_config.server.port);
    let app: Router = get_router(app_config).await?;
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("server is running on {}", addr);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
