use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub slug: String,
    pub name_km: String,
    pub name_en: String,
}

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: String,
}

/// Returned by register/login. `token` goes into the frontend's HttpOnly cookie.
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub max_age: i64,
    pub user: User,
}

#[derive(Serialize, FromRow)]
pub struct Banner {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub image_url: Option<String>,
    pub link_url: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct CourseCard {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub category_slug: Option<String>,
    pub category_name_km: Option<String>,
    pub instructor_name: String,
    pub rating_avg: f64,
    pub rating_count: i64,
}

#[derive(Serialize, FromRow)]
pub struct CourseExtra {
    pub description: String,
    pub instructor_bio: Option<String>,
    pub instructor_avatar_url: Option<String>,
}

#[derive(FromRow)]
pub struct SectionRow {
    pub id: Uuid,
    pub title: String,
}

#[derive(FromRow)]
pub struct LessonRow {
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub duration_seconds: i32,
    pub is_preview: bool,
    pub video_id: Option<String>,
}

#[derive(Serialize)]
pub struct LessonOut {
    pub id: Uuid,
    pub title: String,
    pub duration_seconds: i32,
    pub is_preview: bool,
    /// Only exposed for free preview lessons; paid lessons need an enrollment check (todo).
    pub preview_video_id: Option<String>,
}

#[derive(Serialize)]
pub struct SectionOut {
    pub id: Uuid,
    pub title: String,
    pub lessons: Vec<LessonOut>,
}

#[derive(Serialize)]
pub struct CourseDetail {
    #[serde(flatten)]
    pub card: CourseCard,
    #[serde(flatten)]
    pub extra: CourseExtra,
    pub sections: Vec<SectionOut>,
    pub lesson_count: usize,
    pub total_duration_seconds: i64,
}

#[derive(Serialize, FromRow)]
pub struct EbookCard {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub author_name: String,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub category_slug: Option<String>,
    pub category_name_km: Option<String>,
    pub rating_avg: f64,
    pub rating_count: i64,
}

#[derive(Serialize, FromRow)]
pub struct EbookExtra {
    pub description: String,
    pub page_count: Option<i32>,
}

#[derive(Serialize)]
pub struct EbookDetail {
    #[serde(flatten)]
    pub card: EbookCard,
    #[serde(flatten)]
    pub extra: EbookExtra,
}

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Deserialize, Default)]
pub struct ListParams {
    pub q: Option<String>,
    pub category: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl ListParams {
    /// ILIKE pattern for the search term, with LIKE wildcards in user input escaped.
    pub fn search_pattern(&self) -> Option<String> {
        let q = self.q.as_deref()?.trim();
        if q.is_empty() {
            return None;
        }
        let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        Some(format!("%{escaped}%"))
    }

    pub fn category(&self) -> Option<&str> {
        self.category.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(12).clamp(1, 48)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_pattern_escapes_wildcards() {
        let p = ListParams { q: Some(" 50%_off ".into()), ..Default::default() };
        assert_eq!(p.search_pattern().as_deref(), Some("%50\\%\\_off%"));
    }

    #[test]
    fn blank_search_is_none() {
        let p = ListParams { q: Some("   ".into()), ..Default::default() };
        assert!(p.search_pattern().is_none());
    }

    #[test]
    fn pagination_is_clamped() {
        let p = ListParams { page: Some(0), per_page: Some(1000), ..Default::default() };
        assert_eq!((p.page(), p.per_page(), p.offset()), (1, 48, 0));
    }
}
