//! Admin user management: search, roles, and manual grants / revocations (refunds, support).

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminOwnedItem, AdminUserDetail, AdminUserRow, ItemOption, ListParams, Page},
};

const USER_COLUMNS: &str = "u.id, u.full_name, u.phone, u.email, u.role, u.created_at,
     (SELECT COUNT(*) FROM enrollments en WHERE en.user_id = u.id) AS course_count,
     (SELECT COUNT(*) FROM ebook_purchases ep WHERE ep.user_id = u.id) AS ebook_count";

const USER_FILTER: &str = "FROM users u
     WHERE ($1::text IS NULL OR u.full_name ILIKE $1 OR u.phone ILIKE $1 OR u.email ILIKE $1 OR u.id::text ILIKE $1)
       AND ($2::text IS NULL OR u.role = $2)";

#[derive(Deserialize)]
pub struct RoleParam {
    role: Option<String>,
}

const ROLES: [&str; 3] = ["student", "instructor", "admin"];

/// `GET /api/admin/users?q=&role=&page=`: newest first; `q` matches name, phone, email or id.
pub async fn list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
    Query(rp): Query<RoleParam>,
) -> AppResult<Json<Page<AdminUserRow>>> {
    let role = rp.role.filter(|r| !r.is_empty());
    if role.as_deref().is_some_and(|r| !ROLES.contains(&r)) {
        return Err(AppError::Validation("invalid_role"));
    }
    let pattern = p.search_pattern();
    let list_sql = format!("SELECT {USER_COLUMNS} {USER_FILTER} ORDER BY u.created_at DESC LIMIT $3 OFFSET $4");
    let count_sql = format!("SELECT COUNT(*) {USER_FILTER}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminUserRow>(&list_sql)
            .bind(pattern.as_deref())
            .bind(role.as_deref())
            .bind(p.per_page())
            .bind(p.offset())
            .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(pattern.as_deref())
            .bind(role.as_deref())
            .fetch_one(&st.db),
    )?;
    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

/// A user with everything they own and what could still be granted.
pub async fn detail(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminUserDetail>> {
    let user = sqlx::query_as::<_, AdminUserRow>(&format!("SELECT {USER_COLUMNS} FROM users u WHERE u.id = $1"))
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let (courses, ebooks, grantable_courses, grantable_ebooks) = tokio::try_join!(
        sqlx::query_as::<_, AdminOwnedItem>(
            "SELECT c.id, c.slug, c.title, en.created_at AS granted_at, en.order_id
             FROM enrollments en JOIN courses c ON c.id = en.course_id
             WHERE en.user_id = $1 ORDER BY en.created_at DESC",
        )
        .bind(id)
        .fetch_all(&st.db),
        sqlx::query_as::<_, AdminOwnedItem>(
            "SELECT e.id, e.slug, e.title, ep.created_at AS granted_at, ep.order_id
             FROM ebook_purchases ep JOIN ebooks e ON e.id = ep.ebook_id
             WHERE ep.user_id = $1 ORDER BY ep.created_at DESC",
        )
        .bind(id)
        .fetch_all(&st.db),
        sqlx::query_as::<_, ItemOption>(
            "SELECT id, title, published FROM courses c
             WHERE NOT EXISTS (SELECT 1 FROM enrollments en WHERE en.course_id = c.id AND en.user_id = $1)
             ORDER BY title",
        )
        .bind(id)
        .fetch_all(&st.db),
        sqlx::query_as::<_, ItemOption>(
            "SELECT id, title, published FROM ebooks e
             WHERE NOT EXISTS (SELECT 1 FROM ebook_purchases ep WHERE ep.ebook_id = e.id AND ep.user_id = $1)
             ORDER BY title",
        )
        .bind(id)
        .fetch_all(&st.db),
    )?;
    Ok(Json(AdminUserDetail { user, courses, ebooks, grantable_courses, grantable_ebooks }))
}

#[derive(Deserialize)]
pub struct RoleInput {
    role: String,
}

/// Sets `student` / `instructor` / `admin`. Admins can't change their own role (so the last admin
/// can't lock everyone out by accident): 409 `own_role`.
pub async fn set_role(
    State(st): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<RoleInput>,
) -> AppResult<StatusCode> {
    if !ROLES.contains(&input.role.as_str()) {
        return Err(AppError::Validation("invalid_role"));
    }
    if id == admin.id {
        return Err(AppError::Conflict("own_role"));
    }
    let updated = sqlx::query("UPDATE users SET role = $2 WHERE id = $1")
        .bind(id)
        .bind(&input.role)
        .execute(&st.db)
        .await?
        .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `courses` / `ebooks` → (grant table, item column, item table).
fn grant_table(kind: &str) -> AppResult<(&'static str, &'static str, &'static str)> {
    match kind {
        "courses" => Ok(("enrollments", "course_id", "courses")),
        "ebooks" => Ok(("ebook_purchases", "ebook_id", "ebooks")),
        _ => Err(AppError::NotFound),
    }
}

/// `PUT /api/admin/users/{id}/{courses|ebooks}/{item}`: grants without an order (idempotent; also
/// unpublished items) and drops it from the user's cart.
pub async fn grant(
    State(st): State<AppState>,
    _: AdminUser,
    Path((id, kind, item)): Path<(Uuid, String, Uuid)>,
) -> AppResult<StatusCode> {
    let (table, col, items) = grant_table(&kind)?;
    let mut tx = st.db.begin().await?;
    let found: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE id = $1) AND EXISTS (SELECT 1 FROM {items} WHERE id = $2)"
    ))
    .bind(id)
    .bind(item)
    .fetch_one(&mut *tx)
    .await?;
    if !found {
        return Err(AppError::NotFound);
    }
    sqlx::query(&format!("INSERT INTO {table} (user_id, {col}) VALUES ($1, $2) ON CONFLICT DO NOTHING"))
        .bind(id)
        .bind(item)
        .execute(&mut *tx)
        .await?;
    sqlx::query(&format!("DELETE FROM cart_items WHERE user_id = $1 AND {col} = $2"))
        .bind(id)
        .bind(item)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Revokes access (e.g. after a refund). Orders stay as they are; progress and reviews are kept in
/// case access is granted again. Idempotent.
pub async fn revoke(
    State(st): State<AppState>,
    _: AdminUser,
    Path((id, kind, item)): Path<(Uuid, String, Uuid)>,
) -> AppResult<StatusCode> {
    let (table, col, _) = grant_table(&kind)?;
    sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1 AND {col} = $2"))
        .bind(id)
        .bind(item)
        .execute(&st.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
