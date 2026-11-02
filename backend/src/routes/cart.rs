use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{Cart, CartItem},
};

/// The user's cart, published items only. Shared with checkout so both price the same rows.
const CART_SELECT: &str = r#"
SELECT ci.id, 'course' AS kind, c.id AS item_id, c.slug, c.title, c.cover_url,
       i.name AS byline, c.price_cents, c.sale_price_cents, ci.created_at
FROM cart_items ci
JOIN courses c ON c.id = ci.course_id
JOIN instructors i ON i.id = c.instructor_id
WHERE ci.user_id = $1 AND c.published
UNION ALL
SELECT ci.id, 'ebook' AS kind, e.id AS item_id, e.slug, e.title, e.cover_url,
       e.author_name AS byline, e.price_cents, e.sale_price_cents, ci.created_at
FROM cart_items ci
JOIN ebooks e ON e.id = ci.ebook_id
WHERE ci.user_id = $1 AND e.published
ORDER BY created_at
"#;

pub async fn load_cart(db: impl PgExecutor<'_>, user_id: Uuid) -> sqlx::Result<Cart> {
    let items = sqlx::query_as::<_, CartItem>(CART_SELECT)
        .bind(user_id)
        .fetch_all(db)
        .await?;
    Ok(Cart::new(items))
}

pub async fn get(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<Cart>> {
    Ok(Json(load_cart(&st.db, current.user.id).await?))
}

#[derive(Deserialize)]
pub struct AddBody {
    course_id: Option<Uuid>,
    ebook_id: Option<Uuid>,
}

/// Adds a published course or ebook. Adding an item already in the cart is a no-op;
/// adding one the user already owns is 409 `already_owned`.
pub async fn add(
    State(st): State<AppState>,
    current: CurrentUser,
    Json(body): Json<AddBody>,
) -> AppResult<Json<Cart>> {
    let user_id = current.user.id;
    let (exists_sql, owned_sql, id) = match (body.course_id, body.ebook_id) {
        (Some(id), None) => (
            "SELECT EXISTS (SELECT 1 FROM courses WHERE id = $1 AND published)",
            "SELECT EXISTS (SELECT 1 FROM enrollments WHERE course_id = $1 AND user_id = $2)",
            id,
        ),
        (None, Some(id)) => (
            "SELECT EXISTS (SELECT 1 FROM ebooks WHERE id = $1 AND published)",
            "SELECT EXISTS (SELECT 1 FROM ebook_purchases WHERE ebook_id = $1 AND user_id = $2)",
            id,
        ),
        _ => return Err(AppError::Validation("invalid_item")),
    };

    let (exists, owned) = tokio::try_join!(
        sqlx::query_scalar::<_, bool>(exists_sql)
            .bind(id)
            .fetch_one(&st.db),
        sqlx::query_scalar::<_, bool>(owned_sql)
            .bind(id)
            .bind(user_id)
            .fetch_one(&st.db),
    )?;
    if !exists {
        return Err(AppError::NotFound);
    }
    if owned {
        return Err(AppError::Conflict("already_owned"));
    }

    sqlx::query(
        "INSERT INTO cart_items (user_id, course_id, ebook_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(body.course_id)
    .bind(body.ebook_id)
    .execute(&st.db)
    .await?;
    Ok(Json(load_cart(&st.db, user_id).await?))
}

pub async fn remove(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Cart>> {
    sqlx::query("DELETE FROM cart_items WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(current.user.id)
        .execute(&st.db)
        .await?;
    Ok(Json(load_cart(&st.db, current.user.id).await?))
}
