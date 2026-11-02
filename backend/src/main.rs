mod auth;
mod config;
mod error;
mod models;
mod routes;
mod storage;
mod validate;
mod video;

use sqlx::postgres::PgPoolOptions;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub video: Option<video::BunnyStream>,
    pub storage: Option<storage::ObjectStorage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let config = config::Config::from_env()?;
    match &config.video {
        None => tracing::warn!("BUNNY_LIBRARY_ID not set: lessons have no playable video"),
        Some(v) if v.token_key.is_none() => {
            tracing::warn!("BUNNY_TOKEN_KEY not set: lesson embed URLs are unsigned and never expire")
        }
        Some(_) => {}
    }
    if config.storage.is_none() {
        tracing::warn!("STORAGE_BUCKET not set: ebook downloads return 503");
    }

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let app = routes::router(AppState { db, video: config.video, storage: config.storage })
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on http://{}", config.bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
