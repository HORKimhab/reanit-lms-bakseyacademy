import { cents, checked, optInt, text } from './admin';

/** Course form fields → `CourseInput` body. */
export function courseBody(d: FormData) {
	return {
		slug: text(d, 'slug'),
		title: text(d, 'title'),
		summary: text(d, 'summary'),
		description: text(d, 'description'),
		cover_url: text(d, 'cover_url'),
		instructor_id: text(d, 'instructor_id'),
		category_id: text(d, 'category_id') || null,
		price_cents: cents(d, 'price') ?? 0,
		sale_price_cents: cents(d, 'sale_price'),
		published: checked(d, 'published')
	};
}

/** Ebook form fields → `EbookInput` body. A blank file key keeps the current file. */
export function ebookBody(d: FormData) {
	return {
		slug: text(d, 'slug'),
		title: text(d, 'title'),
		author_name: text(d, 'author_name'),
		summary: text(d, 'summary'),
		description: text(d, 'description'),
		cover_url: text(d, 'cover_url'),
		page_count: optInt(d, 'page_count'),
		category_id: text(d, 'category_id') || null,
		price_cents: cents(d, 'price') ?? 0,
		sale_price_cents: cents(d, 'sale_price'),
		published: checked(d, 'published'),
		file_key: text(d, 'file_key'),
		remove_file: checked(d, 'remove_file')
	};
}
