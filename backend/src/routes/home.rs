use axum::{Json, extract::State};
use serde::Serialize;

use super::{courses, ebooks};
use crate::{
    AppState,
    error::AppResult,
    models::{Banner, CourseCard, EbookCard},
};

#[derive(Serialize)]
pub struct Home {
    banners: Vec<Banner>,
    new_courses: Vec<CourseCard>,
    new_ebooks: Vec<EbookCard>,
}

pub async fn home(State(st): State<AppState>) -> AppResult<Json<Home>> {
    let (banners, new_courses, new_ebooks) = tokio::try_join!(
        sqlx::query_as::<_, Banner>(
            "SELECT id, title, subtitle, image_url, link_url FROM banners \
             WHERE published ORDER BY position, created_at",
        )
        .fetch_all(&st.db),
        courses::query_cards(&st.db, None, None, 8, 0),
        ebooks::query_cards(&st.db, None, None, 8, 0),
    )?;
    Ok(Json(Home { banners, new_courses, new_ebooks }))
}
