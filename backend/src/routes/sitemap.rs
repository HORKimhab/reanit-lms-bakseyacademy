//! Published catalog URLs for the storefront's `sitemap.xml`.

use axum::{Json, extract::State};

use crate::{
    AppState,
    error::AppResult,
    models::{Sitemap, SitemapEntry},
};

/// Every published course, ebook and bundle slug with its last change, newest first.
pub async fn sitemap(State(st): State<AppState>) -> AppResult<Json<Sitemap>> {
    let (courses, ebooks, bundles) = tokio::try_join!(
        sqlx::query_as::<_, SitemapEntry>(
            "SELECT slug, updated_at FROM courses WHERE published ORDER BY updated_at DESC",
        )
        .fetch_all(&st.db),
        sqlx::query_as::<_, SitemapEntry>(
            "SELECT slug, updated_at FROM ebooks WHERE published ORDER BY updated_at DESC",
        )
        .fetch_all(&st.db),
        sqlx::query_as::<_, SitemapEntry>(
            "SELECT slug, updated_at FROM bundles b WHERE published \
             AND EXISTS (SELECT 1 FROM bundle_items bi WHERE bi.bundle_id = b.id) ORDER BY updated_at DESC",
        )
        .fetch_all(&st.db),
    )?;
    Ok(Json(Sitemap { courses, ebooks, bundles }))
}
