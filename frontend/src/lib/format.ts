import { m } from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime';

const KHMER_DIGITS = ['០', '១', '២', '៣', '៤', '៥', '៦', '៧', '៨', '៩'];

/** Digits in the current locale's script: Khmer digits in km, ASCII in en. */
export function formatNumber(n: number | string): string {
	if (getLocale() === 'en') return String(n);
	return String(n).replace(/\d/g, (d) => KHMER_DIGITS[Number(d)]);
}

export function formatPrice(cents: number): string {
	if (cents === 0) return m.price_free();
	const dollars = cents / 100;
	return `$${Number.isInteger(dollars) ? dollars : dollars.toFixed(2)}`;
}

export function formatDuration(seconds: number): string {
	const h = Math.floor(seconds / 3600);
	const min = Math.round((seconds % 3600) / 60);
	if (h === 0) return m.duration_minutes({ minutes: formatNumber(min) });
	return min === 0
		? m.duration_hours({ hours: formatNumber(h) })
		: m.duration_hours_minutes({ hours: formatNumber(h), minutes: formatNumber(min) });
}

export function formatClock(seconds: number): string {
	const mm = Math.floor(seconds / 60);
	const s = seconds % 60;
	return `${mm}:${String(s).padStart(2, '0')}`;
}

const KHMER_MONTHS = ['មករា', 'កុម្ភៈ', 'មីនា', 'មេសា', 'ឧសភា', 'មិថុនា', 'កក្កដា', 'សីហា', 'កញ្ញា', 'តុលា', 'វិច្ឆិកា', 'ធ្នូ'];

/**
 * Date + time in Phnom Penh time: "២ វិច្ឆិកា ២០២៦ ១៤:០៥" (Khmer month names and digits) or
 * "2 Nov 2026, 14:05" in English.
 */
export function formatDateTime(iso: string): string {
	const d = new Date(iso);
	const options: Intl.DateTimeFormatOptions = {
		timeZone: 'Asia/Phnom_Penh',
		day: 'numeric',
		month: getLocale() === 'en' ? 'short' : 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false
	};
	if (getLocale() === 'en') return new Intl.DateTimeFormat('en-GB', options).format(d);
	const parts = new Intl.DateTimeFormat('en-GB', options).formatToParts(d);
	const get = (t: string) => parts.find((p) => p.type === t)?.value ?? '';
	const month = KHMER_MONTHS[Number(get('month')) - 1];
	return formatNumber(`${Number(get('day'))} ${month} ${get('year')} ${get('hour')}:${get('minute')}`);
}
