use axum::{Json, extract::State};

use crate::{AppState, error::AppResult, models::Category};

pub async fn list(State(st): State<AppState>) -> AppResult<Json<Vec<Category>>> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, slug, name_km, name_en FROM categories ORDER BY position, name_en",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}
