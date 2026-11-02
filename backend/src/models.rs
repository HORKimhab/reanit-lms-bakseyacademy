use chrono::{DateTime, Utc};
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
pub struct CartItem {
    pub id: Uuid,
    /// `course` or `ebook`
    pub kind: String,
    pub item_id: Uuid,
    pub slug: String,
    pub title: String,
    pub cover_url: Option<String>,
    /// Instructor (course) or author (ebook)
    pub byline: String,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
}

impl CartItem {
    /// What the buyer pays: the sale price when there is one.
    pub fn effective_price_cents(&self) -> i32 {
        self.sale_price_cents.unwrap_or(self.price_cents)
    }
}

#[derive(Serialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
    pub total_cents: i64,
}

impl Cart {
    pub fn new(items: Vec<CartItem>) -> Self {
        let total_cents = items.iter().map(|i| i64::from(i.effective_price_cents())).sum();
        Self { items, total_cents }
    }
}

#[derive(Serialize, FromRow)]
pub struct PaymentMethod {
    pub code: String,
    pub name_km: String,
    pub name_en: String,
    pub account_name: String,
    pub account_number: String,
    pub qr_image_url: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct OrderItem {
    #[serde(skip)]
    #[sqlx(default)]
    pub order_id: Option<Uuid>,
    pub kind: String,
    pub course_id: Option<Uuid>,
    pub ebook_id: Option<Uuid>,
    pub title: String,
    pub price_cents: i32,
}

/// One order row; `OrderDetail` adds its items and payment method.
#[derive(Serialize, FromRow)]
pub struct OrderSummary {
    pub id: Uuid,
    pub status: String,
    pub total_cents: i32,
    pub payment_method_code: Option<String>,
    pub payment_ref: Option<String>,
    pub admin_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct OrderDetail {
    #[serde(flatten)]
    pub order: OrderSummary,
    pub payment_method: Option<PaymentMethod>,
    pub items: Vec<OrderItem>,
}

#[derive(FromRow)]
pub struct AdminOrderRow {
    #[sqlx(flatten)]
    pub order: OrderSummary,
    pub user_full_name: String,
    pub user_phone: Option<String>,
}

/// An order with its items, for the buyer's order history.
#[derive(Serialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: OrderSummary,
    pub items: Vec<OrderItem>,
}

/// An enrolled course with the learner's progress.
#[derive(Serialize)]
pub struct LibraryCourse {
    #[serde(flatten)]
    pub card: CourseCard,
    pub lesson_count: i64,
    pub completed_lessons: i64,
}

#[derive(Serialize)]
pub struct Recommendations {
    pub courses: Vec<CourseCard>,
    pub ebooks: Vec<EbookCard>,
    /// Ranked from the user's own purchases; false → popular items (signed out or nothing owned yet).
    pub personalized: bool,
}

#[derive(Serialize)]
pub struct Library {
    pub courses: Vec<LibraryCourse>,
    pub ebooks: Vec<EbookCard>,
}

/// An order as the admin review queue sees it: who ordered, what, and the reference to check.
#[derive(Serialize)]
pub struct AdminOrder {
    #[serde(flatten)]
    pub order: OrderSummary,
    pub user_full_name: String,
    pub user_phone: Option<String>,
    pub items: Vec<OrderItem>,
}

// ---- Admin catalog -------------------------------------------------------------------------

#[derive(Serialize, FromRow)]
pub struct AdminCategory {
    pub id: Uuid,
    pub slug: String,
    pub name_km: String,
    pub name_en: String,
    pub position: i32,
}

#[derive(Serialize, FromRow)]
pub struct AdminInstructor {
    pub id: Uuid,
    pub name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub course_count: i64,
}

#[derive(Serialize, FromRow)]
pub struct AdminBanner {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub image_url: Option<String>,
    pub link_url: Option<String>,
    pub position: i32,
    pub published: bool,
}

#[derive(Serialize, FromRow)]
pub struct AdminCourseRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub published: bool,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub instructor_name: String,
    pub category_name_km: Option<String>,
    pub lesson_count: i64,
    pub enrollment_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, FromRow)]
pub struct AdminCourseFields {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub description: String,
    pub cover_url: Option<String>,
    pub instructor_id: Uuid,
    pub category_id: Option<Uuid>,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub published: bool,
    pub updated_at: DateTime<Utc>,
}

/// Lessons carry the raw `video_id` here: admin-only, so editors can see what is attached.
#[derive(Serialize, FromRow)]
pub struct AdminLesson {
    pub id: Uuid,
    #[serde(skip)]
    pub section_id: Uuid,
    pub title: String,
    pub position: i32,
    pub video_id: Option<String>,
    pub duration_seconds: i32,
    pub is_preview: bool,
}

#[derive(Serialize)]
pub struct AdminSection {
    pub id: Uuid,
    pub title: String,
    pub position: i32,
    pub lessons: Vec<AdminLesson>,
}

#[derive(Serialize)]
pub struct AdminCourse {
    #[serde(flatten)]
    pub fields: AdminCourseFields,
    pub sections: Vec<AdminSection>,
}

#[derive(Serialize, FromRow)]
pub struct AdminEbookRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub author_name: String,
    pub published: bool,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub category_name_km: Option<String>,
    pub has_file: bool,
    pub purchase_count: i64,
    pub updated_at: DateTime<Utc>,
}

/// `file_key` is write-only: admins see `has_file`, never the key.
#[derive(Serialize, FromRow)]
pub struct AdminEbook {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub author_name: String,
    pub summary: Option<String>,
    pub description: String,
    pub cover_url: Option<String>,
    pub page_count: Option<i32>,
    pub category_id: Option<Uuid>,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub published: bool,
    pub has_file: bool,
    pub updated_at: DateTime<Utc>,
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
pub struct Review {
    pub id: Uuid,
    pub user_name: String,
    pub rating: i16,
    pub comment: Option<String>,
    pub updated_at: DateTime<Utc>,
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
    /// Only exposed for free preview lessons; enrolled users get playback via `/lessons/{id}`.
    pub preview_video_id: Option<String>,
}

#[derive(FromRow)]
pub struct LessonAccessRow {
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub duration_seconds: i32,
    pub is_preview: bool,
    pub video_id: Option<String>,
    pub published: bool,
    pub enrolled: bool,
    pub completed: bool,
}

/// A lesson the caller may watch. `video_url` is a signed embed URL (`None` when the lesson has
/// no video or no provider is configured); `expires_at` is set when the URL is signed.
#[derive(Serialize)]
pub struct LessonPlayback {
    pub id: Uuid,
    pub section_id: Uuid,
    pub title: String,
    pub duration_seconds: i32,
    pub is_preview: bool,
    pub video_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    /// The caller marked it complete (false when signed out).
    pub completed: bool,
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
    /// The signed-in user is enrolled (false when signed out).
    pub owned: bool,
    /// Lessons the signed-in user completed, in curriculum order (empty when signed out).
    pub completed_lesson_ids: Vec<Uuid>,
    /// Latest reviews, commented ones first (at most 10).
    pub reviews: Vec<Review>,
    /// The signed-in user's own review.
    pub my_review: Option<Review>,
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
    /// The signed-in user bought it (false when signed out).
    pub owned: bool,
    /// Latest reviews, commented ones first (at most 10).
    pub reviews: Vec<Review>,
    /// The signed-in user's own review.
    pub my_review: Option<Review>,
}

/// A short-lived presigned link to an owned ebook's file.
#[derive(Serialize)]
pub struct EbookFile {
    pub url: String,
    pub expires_at: DateTime<Utc>,
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
    /// Trimmed search term; blank → `None`.
    pub fn query(&self) -> Option<&str> {
        self.q.as_deref().map(str::trim).filter(|q| !q.is_empty())
    }

    /// ILIKE pattern for the search term, with LIKE wildcards in user input escaped.
    pub fn search_pattern(&self) -> Option<String> {
        let q = self.query()?;
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
