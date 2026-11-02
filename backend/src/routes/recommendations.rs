//! "For you": unowned published items ranked by co-purchase (people who own what you own
//! also own…), then shared category, then overall popularity. Signed out → popularity only.

use axum::{Json, extract::State};

use super::{courses, ebooks};
use crate::{
    AppState,
    auth::CurrentUser,
    error::AppResult,
    models::{CourseCard, EbookCard, Recommendations},
};

/// `$1` = user id (NULL when signed out). Defines `mine_c`, `mine_e`, `peers`, `my_cats`.
const CONTEXT: &str = r#"
WITH mine_c AS (SELECT course_id FROM enrollments WHERE user_id = $1),
     mine_e AS (SELECT ebook_id FROM ebook_purchases WHERE user_id = $1),
     peers AS (
         SELECT user_id FROM enrollments WHERE course_id IN (SELECT course_id FROM mine_c) AND user_id <> $1
         UNION
         SELECT user_id FROM ebook_purchases WHERE ebook_id IN (SELECT ebook_id FROM mine_e) AND user_id <> $1
     ),
     my_cats AS (
         SELECT category_id FROM courses WHERE id IN (SELECT course_id FROM mine_c)
         UNION
         SELECT category_id FROM ebooks WHERE id IN (SELECT ebook_id FROM mine_e)
     )
"#;

const COURSE_SCORES: &str = r#"
, scores AS (
    SELECT x.id,
           2 * (SELECT COUNT(*) FROM enrollments en WHERE en.course_id = x.id AND en.user_id IN (SELECT user_id FROM peers))
           + CASE WHEN x.category_id IN (SELECT category_id FROM my_cats) THEN 1 ELSE 0 END AS score,
           (SELECT COUNT(*) FROM enrollments en WHERE en.course_id = x.id) AS popularity
    FROM courses x
    WHERE x.published AND x.id NOT IN (SELECT course_id FROM mine_c)
)
"#;

const EBOOK_SCORES: &str = r#"
, scores AS (
    SELECT x.id,
           2 * (SELECT COUNT(*) FROM ebook_purchases p WHERE p.ebook_id = x.id AND p.user_id IN (SELECT user_id FROM peers))
           + CASE WHEN x.category_id IN (SELECT category_id FROM my_cats) THEN 1 ELSE 0 END AS score,
           (SELECT COUNT(*) FROM ebook_purchases p WHERE p.ebook_id = x.id) AS popularity
    FROM ebooks x
    WHERE x.published AND x.id NOT IN (SELECT ebook_id FROM mine_e)
)
"#;

const LIMIT: i64 = 8;

pub async fn recommendations(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
) -> AppResult<Json<Recommendations>> {
    let user_id = current.as_ref().map(|c| c.user.id);
    let course_sql = format!(
        "{CONTEXT} {COURSE_SCORES} {} JOIN scores sc ON sc.id = c.id \
         ORDER BY sc.score DESC, sc.popularity DESC, rating_avg DESC, c.created_at DESC LIMIT $2",
        courses::CARD_SELECT
    );
    let ebook_sql = format!(
        "{CONTEXT} {EBOOK_SCORES} {} JOIN scores sc ON sc.id = e.id \
         ORDER BY sc.score DESC, sc.popularity DESC, rating_avg DESC, e.created_at DESC LIMIT $2",
        ebooks::CARD_SELECT
    );
    let (courses, ebooks, personalized) = tokio::try_join!(
        sqlx::query_as::<_, CourseCard>(&course_sql).bind(user_id).bind(LIMIT).fetch_all(&st.db),
        sqlx::query_as::<_, EbookCard>(&ebook_sql).bind(user_id).bind(LIMIT).fetch_all(&st.db),
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM enrollments WHERE user_id = $1)
                 OR EXISTS (SELECT 1 FROM ebook_purchases WHERE user_id = $1)",
        )
        .bind(user_id)
        .fetch_one(&st.db),
    )?;
    Ok(Json(Recommendations { courses, ebooks, personalized }))
}
