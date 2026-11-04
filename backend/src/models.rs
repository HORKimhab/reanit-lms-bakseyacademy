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
    /// `course`, `ebook` or `bundle`
    pub kind: String,
    pub item_id: Uuid,
    pub slug: String,
    pub title: String,
    pub cover_url: Option<String>,
    /// Instructor (course), author (ebook) or the titles inside (bundle)
    pub byline: String,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
}

impl CartItem {
    /// `(course_id, ebook_id, bundle_id)`: exactly one is set, by `kind`.
    pub fn ids(&self) -> (Option<Uuid>, Option<Uuid>, Option<Uuid>) {
        let id = Some(self.item_id);
        match self.kind.as_str() {
            "course" => (id, None, None),
            "ebook" => (None, id, None),
            _ => (None, None, id),
        }
    }

    /// What the buyer pays: the sale price when there is one.
    pub fn effective_price_cents(&self) -> i32 {
        self.sale_price_cents.unwrap_or(self.price_cents)
    }
}

/// A discount code applied to a cart.
#[derive(Serialize)]
pub struct AppliedCoupon {
    pub code: String,
    pub percent_off: Option<i32>,
    pub amount_off_cents: Option<i32>,
    pub discount_cents: i64,
}

#[derive(Serialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
    /// Sum of the items' effective prices.
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    /// What checkout charges: subtotal minus the coupon's discount.
    pub total_cents: i64,
    pub coupon: Option<AppliedCoupon>,
}

impl Cart {
    pub fn new(items: Vec<CartItem>) -> Self {
        let subtotal_cents = items.iter().map(|i| i64::from(i.effective_price_cents())).sum();
        Self { items, subtotal_cents, discount_cents: 0, total_cents: subtotal_cents, coupon: None }
    }

    pub fn with_coupon(mut self, coupon: AppliedCoupon) -> Self {
        self.discount_cents = coupon.discount_cents;
        self.total_cents = self.subtotal_cents - coupon.discount_cents;
        self.coupon = Some(coupon);
        self
    }
}

/// A discount code as admins manage it, with how often it was used.
#[derive(Serialize, FromRow)]
pub struct AdminCoupon {
    pub id: Uuid,
    pub code: String,
    pub percent_off: Option<i32>,
    pub amount_off_cents: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    /// Non-cancelled orders that used it.
    pub uses: i64,
    /// Total discount on paid orders.
    pub discount_given_cents: i64,
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
    pub bundle_id: Option<Uuid>,
    pub title: String,
    pub price_cents: i32,
}

/// One order row; `OrderDetail` adds its items and payment method.
#[derive(Serialize, FromRow)]
pub struct OrderSummary {
    pub id: Uuid,
    pub status: String,
    pub total_cents: i32,
    /// Coupon discount already taken off `total_cents`.
    pub discount_cents: i32,
    pub coupon_code: Option<String>,
    pub payment_method_code: Option<String>,
    pub payment_ref: Option<String>,
    pub admin_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub payment_proof_media_id: Option<uuid::Uuid>,
    /// Signed `/media/…` link to the attached payment screenshot (1 h); set by the handlers.
    #[sqlx(skip)]
    pub payment_proof_url: Option<String>,
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

/// An in-app notice (`kind`: `order_paid` / `order_rejected` with the admin's `note`).
#[derive(Serialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub kind: String,
    pub order_id: Option<Uuid>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct Notifications {
    pub items: Vec<Notification>,
    pub unread_count: i64,
}

/// An enrolled course with the learner's progress.
#[derive(Serialize)]
pub struct LibraryCourse {
    #[serde(flatten)]
    pub card: CourseCard,
    pub lesson_count: i64,
    pub completed_lessons: i64,
    /// Issued once every lesson is complete.
    pub certificate_id: Option<Uuid>,
}

/// A completion certificate (public, by id). Names are snapshots at issue time.
#[derive(Serialize, FromRow)]
pub struct Certificate {
    pub id: Uuid,
    pub full_name: String,
    pub course_title: String,
    pub instructor_name: String,
    pub issued_at: DateTime<Utc>,
    pub course_slug: String,
}

#[derive(Serialize)]
pub struct Recommendations {
    pub courses: Vec<CourseCard>,
    pub ebooks: Vec<EbookCard>,
    /// Ranked from the user's own purchases; false → popular items (signed out or nothing owned yet).
    pub personalized: bool,
}

#[derive(Serialize)]
pub struct Wishlist {
    pub courses: Vec<CourseCard>,
    pub ebooks: Vec<EbookCard>,
}

#[derive(Serialize)]
pub struct WishlistIds {
    pub course_ids: Vec<Uuid>,
    pub ebook_ids: Vec<Uuid>,
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

// ---- Lesson Q&A ----------------------------------------------------------------------------

#[derive(Serialize, FromRow)]
pub struct QuestionRow {
    pub id: Uuid,
    pub user_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    /// Asked by the viewer.
    pub mine: bool,
}

#[derive(Serialize, FromRow)]
pub struct Answer {
    pub id: Uuid,
    #[serde(skip)]
    pub question_id: Uuid,
    pub user_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub mine: bool,
}

#[derive(Serialize)]
pub struct Question {
    #[serde(flatten)]
    pub row: QuestionRow,
    pub answers: Vec<Answer>,
}

#[derive(Serialize)]
pub struct LessonQuestions {
    pub questions: Vec<Question>,
    /// The viewer is staff for this course (admin or its instructor): may answer and delete any.
    pub can_answer: bool,
}

/// A question in the admin answer queue.
#[derive(Serialize, FromRow)]
pub struct AdminQuestion {
    pub id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub user_name: String,
    pub course_slug: String,
    pub course_title: String,
    pub lesson_id: Uuid,
    pub lesson_title: String,
    pub answer_count: i64,
}

// ---- Bundles -------------------------------------------------------------------------------

#[derive(Serialize, FromRow)]
pub struct BundleCard {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub item_count: i64,
    /// What the items cost bought one by one.
    pub value_cents: i64,
}

#[derive(Serialize, FromRow)]
pub struct BundleExtra {
    pub description: String,
}

#[derive(Serialize)]
pub struct BundleDetail {
    #[serde(flatten)]
    pub card: BundleCard,
    #[serde(flatten)]
    pub extra: BundleExtra,
    /// The signed-in user already has everything in it.
    pub owned: bool,
    pub courses: Vec<CourseCard>,
    pub ebooks: Vec<EbookCard>,
}

#[derive(Serialize, FromRow)]
pub struct AdminBundleRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub published: bool,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub updated_at: DateTime<Utc>,
    pub item_count: i64,
    pub sales: i64,
}

#[derive(Serialize, FromRow)]
pub struct AdminBundleFields {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub description: String,
    pub cover_url: Option<String>,
    pub price_cents: i32,
    pub sale_price_cents: Option<i32>,
    pub published: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminBundle {
    #[serde(flatten)]
    pub fields: AdminBundleFields,
    pub courses: Vec<CourseCard>,
    pub ebooks: Vec<EbookCard>,
    /// Items not in the bundle yet (for the add pickers).
    pub other_courses: Vec<ItemOption>,
    pub other_ebooks: Vec<ItemOption>,
}

// ---- Admin users ---------------------------------------------------------------------------

#[derive(Serialize, FromRow)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub course_count: i64,
    pub ebook_count: i64,
}

/// A course / ebook a user has access to; `order_id` is `None` for manual grants.
#[derive(Serialize, FromRow)]
pub struct AdminOwnedItem {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub granted_at: DateTime<Utc>,
    pub order_id: Option<Uuid>,
}

#[derive(Serialize, FromRow)]
pub struct ItemOption {
    pub id: Uuid,
    pub title: String,
    pub published: bool,
}

#[derive(Serialize)]
pub struct AdminUserDetail {
    #[serde(flatten)]
    pub user: AdminUserRow,
    pub courses: Vec<AdminOwnedItem>,
    pub ebooks: Vec<AdminOwnedItem>,
    /// Items the user doesn't have yet (for the grant pickers).
    pub grantable_courses: Vec<ItemOption>,
    pub grantable_ebooks: Vec<ItemOption>,
}

// ---- Admin dashboard -----------------------------------------------------------------------

#[derive(Serialize, FromRow)]
pub struct OrderCounts {
    pub awaiting_approval: i64,
    pub pending_payment: i64,
    pub revenue_cents_all_time: i64,
    pub users: i64,
    pub new_users: i64,
}

/// Paid (non-free) orders on one Phnom Penh calendar day.
#[derive(Serialize, FromRow)]
pub struct DailyRevenue {
    pub day: chrono::NaiveDate,
    pub revenue_cents: i64,
    pub orders: i64,
}

#[derive(Serialize, FromRow)]
pub struct TopItem {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub sales: i64,
    pub revenue_cents: i64,
}

#[derive(Serialize)]
pub struct AdminStats {
    pub days: i32,
    /// Paid revenue / paid orders in the period.
    pub revenue_cents: i64,
    pub paid_orders: i64,
    #[serde(flatten)]
    pub counts: OrderCounts,
    pub daily: Vec<DailyRevenue>,
    pub top_courses: Vec<TopItem>,
    pub top_ebooks: Vec<TopItem>,
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

/// An uploaded media file (admin view; Telegram ids stay server-side).
#[derive(Serialize, FromRow)]
pub struct AdminMedia {
    pub id: Uuid,
    pub provider: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    /// Signed `/media/…` link (1 h) so admins can open the file; set by the media routes.
    #[sqlx(default)]
    pub preview_url: Option<String>,
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
    pub media_id: Option<Uuid>,
    pub media_file_name: Option<String>,
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
    pub media_id: Option<Uuid>,
    pub media_file_name: Option<String>,
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
    pub category_name_en: Option<String>,
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
    pub media_id: Option<Uuid>,
    pub course_id: Uuid,
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
    /// `embed` (Bunny iframe) or `file` (a `/media/…` stream for a native `<video>`).
    pub video_type: &'static str,
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
    /// The lesson the enrolled user opened last ("Continue learning" resumes there).
    pub last_lesson_id: Option<Uuid>,
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
    pub category_name_en: Option<String>,
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

#[derive(Serialize, FromRow)]
pub struct SitemapEntry {
    pub slug: String,
    pub updated_at: DateTime<Utc>,
}

/// Published catalog URLs (`GET /api/sitemap`).
#[derive(Serialize)]
pub struct Sitemap {
    pub courses: Vec<SitemapEntry>,
    pub ebooks: Vec<SitemapEntry>,
    pub bundles: Vec<SitemapEntry>,
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
