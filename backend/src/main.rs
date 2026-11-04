mod auth;
mod cleanup;
mod config;
mod error;
mod http_range;
mod media_token;
mod models;
mod payments;
mod routes;
mod storage;
mod throttle;
mod telegram;
mod validate;
mod video;

#[cfg(test)]
mod tests;

use sqlx::postgres::PgPoolOptions;
use tower_http::{
    compression::{
        CompressionLayer,
        predicate::{DefaultPredicate, Predicate},
    },
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub video: Option<video::BunnyStream>,
    pub storage: Option<storage::ObjectStorage>,
    pub telegram: Option<telegram::Telegram>,
    pub media_key: std::sync::Arc<[u8]>,
    pub login_throttle: std::sync::Arc<throttle::LoginThrottle>,
    pub payments: payments::Payments,
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
    if config.payments.bakong.is_none() {
        tracing::warn!("BAKONG_TOKEN not set: Bakong KHQR payments need admin approval");
    }
    if config.payments.payway.is_none() {
        tracing::warn!("PAYWAY_MERCHANT_ID not set: ABA PayWay payments need admin approval");
    }
    if config.telegram.is_none() {
        tracing::warn!("TELEGRAM_BOT_TOKEN not set: Telegram channel storage is off");
    } else if config.telegram.as_ref().is_some_and(|tg| tg.proofs_go_to_storage_channel()) {
        tracing::warn!("TELEGRAM_ADMIN_CHAT_ID not set: payment screenshots go to the storage channel");
    }

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    cleanup::spawn(db.clone(), config.order_abandon_days);

    let state = AppState {
        db,
        video: config.video,
        storage: config.storage,
        telegram: config.telegram,
        media_key: config.media_key.into(),
        login_throttle: Default::default(),
        payments: config.payments,
    };
    routes::spawn_payment_poller(state.clone());
    let app = routes::router(state)
        // Ranged media bodies (Accept-Ranges) must keep their exact bytes and Content-Length.
        .layer(CompressionLayer::new().compress_when(DefaultPredicate::new().and(
            |_: axum::http::StatusCode, _: axum::http::Version, h: &axum::http::HeaderMap, _: &axum::http::Extensions| {
                !h.contains_key(axum::http::header::ACCEPT_RANGES)
            },
        )))
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
