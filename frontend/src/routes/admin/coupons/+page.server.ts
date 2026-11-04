import { api } from '$lib/server/api';
import { cents, checked, optInt, text } from '$lib/server/admin';
import { crudActions } from '$lib/server/admin-crud';
import { requireToken } from '$lib/server/guard';
import type { AdminCoupon } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	return { coupons: await api<AdminCoupon[]>(fetch, '/api/admin/coupons', { token }) };
};

/** `kind` + `value` → `percent_off` or `amount_off_cents`; the expiry date ends at midnight, Phnom Penh time. */
export const actions: Actions = crudActions('coupons', (f) => {
	const percent = text(f, 'kind') === 'percent';
	const expires = text(f, 'expires');
	return {
		code: text(f, 'code'),
		percent_off: percent ? (optInt(f, 'value') ?? -1) : null,
		amount_off_cents: percent ? null : (cents(f, 'value') ?? -1),
		expires_at: /^\d{4}-\d{2}-\d{2}$/.test(expires) ? `${expires}T23:59:59+07:00` : null,
		max_uses: optInt(f, 'max_uses'),
		active: checked(f, 'active')
	};
});
