use axum::{Json, extract::State};
use serde::Serialize;

use super::{courses, ebooks};
use crate::{
    AppState,
    error::AppResult,
    models::{CourseCard, EbookCard},
};

#[derive(Serialize)]
pub struct Home {
    new_courses: Vec<CourseCard>,
    new_ebooks: Vec<EbookCard>,
}

pub async fn home(State(st): State<AppState>) -> AppResult<Json<Home>> {
    let (new_courses, new_ebooks) = tokio::try_join!(
        courses::query_cards(&st.db, None, None, 8, 0),
        ebooks::query_cards(&st.db, None, None, 8, 0),
    )?;
    Ok(Json(Home { new_courses, new_ebooks }))
}
