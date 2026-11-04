import { m } from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime';
import type { ItemKind } from '$lib/types';

/** Picks the API's `*_km` / `*_en` field for the current locale (Khmer when English is missing). */
export function localized(km: string, en: string | null | undefined): string {
	return getLocale() === 'en' && en ? en : km;
}

/** A card's category name in the current locale (`null` when uncategorized). */
export function categoryName(item: { category_name_km: string | null; category_name_en: string | null }): string | null {
	return item.category_name_km === null ? null : localized(item.category_name_km, item.category_name_en);
}

/** "Course" / "Ebook" / "Bundle" in the current locale. */
export const kindLabel = (kind: ItemKind) =>
	kind === 'course' ? m.kind_course() : kind === 'ebook' ? m.kind_ebook() : m.kind_bundle();

/** Storefront path segment for an item kind: `/courses/…`, `/ebooks/…`, `/bundles/…`. */
export const kindPath = (kind: ItemKind) => (kind === 'course' ? 'courses' : kind === 'ebook' ? 'ebooks' : 'bundles');
