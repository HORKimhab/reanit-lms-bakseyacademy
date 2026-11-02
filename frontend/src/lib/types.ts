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

export interface Section {
	id: string;
	title: string;
	lessons: Lesson[];
}

export interface CourseDetail extends CourseCard {
	description: string;
	instructor_bio: string | null;
	instructor_avatar_url: string | null;
	sections: Section[];
	lesson_count: number;
	total_duration_seconds: number;
}

export interface EbookCard extends ProductBase {
	author_name: string;
}

export interface EbookDetail extends EbookCard {
	description: string;
	page_count: number | null;
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
