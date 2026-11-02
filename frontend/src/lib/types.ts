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

export interface Cart {
	items: CartItem[];
	total_cents: number;
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
	payment_method_code: string | null;
	payment_ref: string | null;
	admin_note: string | null;
	created_at: string;
	submitted_at: string | null;
	paid_at: string | null;
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
}

export interface Library {
	courses: LibraryCourse[];
	ebooks: EbookCard[];
}
