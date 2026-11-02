// Shared Tailwind classes for admin forms.
export const inputClass =
	'w-full rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-sm outline-none focus:border-brand-500';
export const labelClass = 'block text-sm font-medium text-slate-700';
export const primaryButton = 'rounded-lg bg-brand-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-brand-700';
export const secondaryButton = 'rounded-lg border border-slate-200 px-3 py-1.5 text-sm hover:border-brand-500';
export const dangerButton = 'rounded-lg border border-red-200 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50';

/** `onclick` for destructive submit buttons. */
export const confirmClick = (message: string) => (e: MouseEvent) => {
	if (!confirm(message)) e.preventDefault();
};

/** Cents → a dollar amount for a price `<input>` value: 1550 → "15.50", null → "". */
export const dollarsInput = (cents: number | null | undefined) => (cents == null ? '' : (cents / 100).toFixed(2));
