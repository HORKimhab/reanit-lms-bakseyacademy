import { adminCall } from './admin';
import type { Actions } from '@sveltejs/kit';

/**
 * `create` / `update` / `delete` actions for a flat admin collection at `/api/admin/{resource}`.
 * Each form posts `id` (update/delete) and its fields; `body` maps the fields to the JSON input.
 */
export function crudActions(resource: string, body: (form: FormData) => unknown): Actions {
	const base = `/api/admin/${resource}`;
	const id = (form: FormData) => String(form.get('id') ?? '');
	return {
		create: async (event) => {
			const form = await event.request.formData();
			const res = await adminCall(event, base, { method: 'POST', body: body(form) }, { id: 'new' });
			return res.ok ? { id: 'new', saved: true } : res.failure;
		},
		update: async (event) => {
			const form = await event.request.formData();
			const res = await adminCall(event, `${base}/${encodeURIComponent(id(form))}`, { method: 'PUT', body: body(form) }, { id: id(form) });
			return res.ok ? { id: id(form), saved: true } : res.failure;
		},
		delete: async (event) => {
			const form = await event.request.formData();
			const res = await adminCall(event, `${base}/${encodeURIComponent(id(form))}`, { method: 'DELETE' }, { id: id(form) });
			return res.ok ? { id: id(form), deleted: true } : res.failure;
		}
	};
}
