const KHMER_DIGITS = ['០', '១', '២', '៣', '៤', '៥', '៦', '៧', '៨', '៩'];

export function khmerNumber(n: number | string): string {
	return String(n).replace(/\d/g, (d) => KHMER_DIGITS[Number(d)]);
}

export function formatPrice(cents: number): string {
	if (cents === 0) return 'ឥតគិតថ្លៃ';
	const dollars = cents / 100;
	return `$${Number.isInteger(dollars) ? dollars : dollars.toFixed(2)}`;
}

export function formatDuration(seconds: number): string {
	const h = Math.floor(seconds / 3600);
	const m = Math.round((seconds % 3600) / 60);
	if (h === 0) return `${khmerNumber(m)} នាទី`;
	return m === 0 ? `${khmerNumber(h)} ម៉ោង` : `${khmerNumber(h)} ម៉ោង ${khmerNumber(m)} នាទី`;
}

export function formatClock(seconds: number): string {
	const m = Math.floor(seconds / 60);
	const s = seconds % 60;
	return `${m}:${String(s).padStart(2, '0')}`;
}

/** Date + time in Phnom Penh time with Khmer month names and digits, e.g. "២ វិច្ឆិកា ២០២៦ ១៤:០៥". */
export function formatDateTime(iso: string): string {
	const d = new Date(iso);
	const parts = new Intl.DateTimeFormat('en-GB', {
		timeZone: 'Asia/Phnom_Penh',
		day: 'numeric',
		month: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false
	}).formatToParts(d);
	const get = (t: string) => parts.find((p) => p.type === t)?.value ?? '';
	const month = KHMER_MONTHS[Number(get('month')) - 1];
	return khmerNumber(`${Number(get('day'))} ${month} ${get('year')} ${get('hour')}:${get('minute')}`);
}

const KHMER_MONTHS = ['មករា', 'កុម្ភៈ', 'មីនា', 'មេសា', 'ឧសភា', 'មិថុនា', 'កក្កដា', 'សីហា', 'កញ្ញា', 'តុលា', 'វិច្ឆិកា', 'ធ្នូ'];
