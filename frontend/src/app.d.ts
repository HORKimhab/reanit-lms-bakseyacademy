// See https://svelte.dev/docs/kit/types#app.d.ts
import type { User } from '$lib/types';

declare global {
	namespace App {
		interface Locals {
			/** Set by hooks.server.ts when the session cookie is valid. */
			user: User | null;
			token: string | null;
		}
	}
}
export {};
