export interface Category {
	id: string;
	slug: string;
	name_km: string;
	name_en: string;
}

interface ProductBase {
	id: string;
	slug: string;
	title: string;
	summary: string | null;
	cover_url: string | null;
	price_cents: number;
	sale_price_cents: number | null;
	category_slug: string | null;
	category_name_km: string | null;
	category_name_en: string | null;
	rating_avg: number;
	rating_count: number;
}

export interface Review {
	id: string;
	user_name: string;
	rating: number;
	comment: string | null;
	updated_at: string;
}

interface Reviewed {
	/** Latest reviews, commented ones first (at most 10). */
	reviews: Review[];
	my_review: Review | null;
}

export interface CourseCard extends ProductBase {
	instructor_name: string;
}

export interface Lesson {
	id: string;
	title: string;
	duration_seconds: number;
	is_preview: boolean;
	preview_video_id: string | null;
}

/** `GET /api/courses/{slug}/lessons/{id}`: a lesson the caller may watch. */
export interface LessonPlayback {
	id: string;
	section_id: string;
	title: string;
	duration_seconds: number;
	is_preview: boolean;
	video_url: string | null;
	/** `embed`: Bunny iframe URL; `file`: a `/media/…` stream for a native `<video>` (Telegram). */
	video_type: 'embed' | 'file';
	expires_at: string | null;
	completed: boolean;
}

export interface Section {
	id: string;
	title: string;
	lessons: Lesson[];
}

export interface CourseDetail extends CourseCard, Reviewed {
	description: string;
	instructor_bio: string | null;
	instructor_avatar_url: string | null;
	sections: Section[];
	lesson_count: number;
	total_duration_seconds: number;
	owned: boolean;
	/** In curriculum order; empty when signed out. */
	completed_lesson_ids: string[];
}

export interface EbookCard extends ProductBase {
	author_name: string;
}

export interface EbookDetail extends EbookCard, Reviewed {
	description: string;
	page_count: number | null;
	owned: boolean;
}

export interface EbookFile {
	url: string;
	expires_at: string;
}

export interface Paged<T> {
	items: T[];
	total: number;
	page: number;
	per_page: number;
}

export interface Banner {
	id: string;
	title: string;
	subtitle: string | null;
	image_url: string | null;
	link_url: string | null;
}

export interface Home {
	banners: Banner[];
	new_courses: CourseCard[];
	new_ebooks: EbookCard[];
}

export interface User {
	id: string;
	full_name: string;
	phone: string | null;
	email: string | null;
	role: 'student' | 'instructor' | 'admin';
}

export interface AuthResponse {
	token: string;
	max_age: number;
	user: User;
}

export interface CartItem {
	id: string;
	kind: 'course' | 'ebook';
	item_id: string;
	slug: string;
	title: string;
	cover_url: string | null;
	byline: string;
	price_cents: number;
	sale_price_cents: number | null;
}

export interface AppliedCoupon {
	code: string;
	percent_off: number | null;
	amount_off_cents: number | null;
	discount_cents: number;
}

export interface Cart {
	items: CartItem[];
	subtotal_cents: number;
	discount_cents: number;
	/** Subtotal minus the coupon's discount: what checkout charges. */
	total_cents: number;
	coupon: AppliedCoupon | null;
}

/** `GET /api/admin/coupons`. */
export interface AdminCoupon {
	id: string;
	code: string;
	percent_off: number | null;
	amount_off_cents: number | null;
	expires_at: string | null;
	max_uses: number | null;
	active: boolean;
	created_at: string;
	uses: number;
	discount_given_cents: number;
}

export interface PaymentMethod {
	code: string;
	name_km: string;
	name_en: string;
	account_name: string;
	account_number: string;
	qr_image_url: string | null;
}

export type OrderStatus = 'pending_payment' | 'awaiting_approval' | 'paid' | 'rejected' | 'cancelled';

export interface OrderItem {
	kind: 'course' | 'ebook';
	course_id: string | null;
	ebook_id: string | null;
	title: string;
	price_cents: number;
}

export interface OrderSummary {
	id: string;
	status: OrderStatus;
	total_cents: number;
	/** Coupon discount already taken off `total_cents`. */
	discount_cents: number;
	coupon_code: string | null;
	payment_method_code: string | null;
	payment_ref: string | null;
	admin_note: string | null;
	created_at: string;
	submitted_at: string | null;
	paid_at: string | null;
	/** Signed link (1 h) to the attached payment screenshot. */
	payment_proof_url: string | null;
}

export interface OrderDetail extends OrderSummary {
	payment_method: PaymentMethod | null;
	items: OrderItem[];
}

export interface AdminOrder extends OrderSummary {
	user_full_name: string;
	user_phone: string | null;
	items: OrderItem[];
}

/** `GET /api/me/notifications`: in-app notice about one of the user's orders. */
export interface Notification {
	id: string;
	kind: 'order_paid' | 'order_rejected';
	order_id: string | null;
	/** The admin's rejection note. */
	note: string | null;
	created_at: string;
	read_at: string | null;
}

export interface Notifications {
	items: Notification[];
	unread_count: number;
}

export interface OrderWithItems extends OrderSummary {
	items: OrderItem[];
}

export interface Recommendations {
	courses: CourseCard[];
	ebooks: EbookCard[];
	/** false → popular items (signed out or nothing owned yet). */
	personalized: boolean;
}

export interface LibraryCourse extends CourseCard {
	lesson_count: number;
	completed_lessons: number;
	/** Issued once every lesson is complete. */
	certificate_id: string | null;
}

/** `GET /api/certificates/{id}` (public). Names are snapshots at issue time. */
export interface Certificate {
	id: string;
	full_name: string;
	course_title: string;
	instructor_name: string;
	issued_at: string;
	course_slug: string;
}

export interface Library {
	courses: LibraryCourse[];
	ebooks: EbookCard[];
}

// ---- Admin catalog ----

export interface AdminCategory {
	id: string;
	slug: string;
	name_km: string;
	name_en: string;
	position: number;
}

export interface AdminInstructor {
	id: string;
	name: string;
	bio: string | null;
	avatar_url: string | null;
	course_count: number;
}

export interface AdminBanner {
	id: string;
	title: string;
	subtitle: string | null;
	image_url: string | null;
	link_url: string | null;
	position: number;
	published: boolean;
}

export interface AdminCourseRow {
	id: string;
	slug: string;
	title: string;
	published: boolean;
	price_cents: number;
	sale_price_cents: number | null;
	instructor_name: string;
	category_name_km: string | null;
	lesson_count: number;
	enrollment_count: number;
	updated_at: string;
}

export interface AdminLesson {
	id: string;
	title: string;
	position: number;
	video_id: string | null;
	/** Uploaded (Telegram) media; wins over `video_id`. */
	media_id: string | null;
	media_file_name: string | null;
	duration_seconds: number;
	is_preview: boolean;
}

export interface AdminSection {
	id: string;
	title: string;
	position: number;
	lessons: AdminLesson[];
}

export interface AdminCourse {
	id: string;
	slug: string;
	title: string;
	summary: string | null;
	description: string;
	cover_url: string | null;
	instructor_id: string;
	category_id: string | null;
	price_cents: number;
	sale_price_cents: number | null;
	published: boolean;
	updated_at: string;
	sections: AdminSection[];
}

export interface AdminEbookRow {
	id: string;
	slug: string;
	title: string;
	author_name: string;
	published: boolean;
	price_cents: number;
	sale_price_cents: number | null;
	category_name_km: string | null;
	has_file: boolean;
	purchase_count: number;
	updated_at: string;
}

/** The file key is write-only; `has_file` says whether one is set. */
export interface AdminEbook {
	id: string;
	slug: string;
	title: string;
	author_name: string;
	summary: string | null;
	description: string;
	cover_url: string | null;
	page_count: number | null;
	category_id: string | null;
	price_cents: number;
	sale_price_cents: number | null;
	published: boolean;
	has_file: boolean;
	/** Uploaded (Telegram) media; wins over the S3 file key. */
	media_id: string | null;
	media_file_name: string | null;
	updated_at: string;
}

export interface AdminMedia {
	id: string;
	provider: 'bunny' | 's3' | 'telegram';
	file_name: string | null;
	mime_type: string | null;
	size_bytes: number | null;
	duration_seconds: number | null;
	created_at: string;
	/** Signed `/media/…` link (1 h) for admins to open the file. */
	preview_url: string | null;
}

/** `GET /api/sitemap`: published catalog slugs. */
export interface SitemapEntry {
	slug: string;
	updated_at: string;
}

export interface Sitemap {
	courses: SitemapEntry[];
	ebooks: SitemapEntry[];
}

/** `GET /api/admin/stats?days=`: dashboard numbers. */
export interface AdminTopItem {
	id: string;
	slug: string;
	title: string;
	sales: number;
	revenue_cents: number;
}

export interface AdminStats {
	days: number;
	revenue_cents: number;
	paid_orders: number;
	awaiting_approval: number;
	pending_payment: number;
	revenue_cents_all_time: number;
	users: number;
	new_users: number;
	/** One row per Phnom Penh day (`YYYY-MM-DD`), oldest first, zero-filled. */
	daily: { day: string; revenue_cents: number; orders: number }[];
	top_courses: AdminTopItem[];
	top_ebooks: AdminTopItem[];
}

/** `GET /api/me/wishlist`: saved items that are still published. */
export interface Wishlist {
	courses: CourseCard[];
	ebooks: EbookCard[];
}

/** `GET /api/me/wishlist/ids`. */
export interface WishlistIds {
	course_ids: string[];
	ebook_ids: string[];
}

/** Lesson Q&A (`GET /api/courses/{slug}/lessons/{id}/questions`). */
export interface Answer {
	id: string;
	user_name: string;
	body: string;
	created_at: string;
	mine: boolean;
}

export interface Question extends Answer {
	answers: Answer[];
}

export interface LessonQuestions {
	questions: Question[];
	/** Admin or the course's instructor: may answer and delete any. */
	can_answer: boolean;
}

/** `GET /api/admin/questions`: the answer queue. */
export interface AdminQuestion {
	id: string;
	body: string;
	created_at: string;
	user_name: string;
	course_slug: string;
	course_title: string;
	lesson_id: string;
	lesson_title: string;
	answer_count: number;
}
