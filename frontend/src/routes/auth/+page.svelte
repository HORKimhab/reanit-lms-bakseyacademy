<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { enhance } from '$app/forms';
	import { errorMessage } from '$lib/errors';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// After a failed submit, stay on the tab that was submitted.
	const tab = $derived(form?.form ?? data.tab);
	const error = $derived(form?.form === tab ? errorMessage(form.code) : data.oauthError ? errorMessage(data.oauthError) : null);
	const query = $derived(data.redirectTo === '/' ? '' : `redirectTo=${encodeURIComponent(data.redirectTo)}`);
	const tabHref = (t: string) => `/auth?${[t === 'register' ? 'tab=register' : '', query].filter(Boolean).join('&')}`;
	const action = (name: string) => `?/${name}${query ? `&${query}` : ''}`;

	let pending = $state(false);
	const submit = () => {
		pending = true;
		return async ({ update }: { update: () => Promise<void> }) => {
			await update();
			pending = false;
		};
	};

	const input =
		'w-full rounded-lg border border-slate-200 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20';
</script>

<svelte:head><title>{tab === 'register' ? m.auth_register() : m.nav_login()} · Reanit Academy</title></svelte:head>

<div class="mx-auto max-w-md">
	<div class="grid grid-cols-2 rounded-xl bg-slate-100 p-1 text-center text-sm font-medium">
		<a href={tabHref('login')} class="rounded-lg py-2 {tab === 'login' ? 'bg-white shadow-sm' : 'text-slate-500'}">{m.nav_login()}</a>
		<a href={tabHref('register')} class="rounded-lg py-2 {tab === 'register' ? 'bg-white shadow-sm' : 'text-slate-500'}">
			{m.auth_register()}
		</a>
	</div>

	<div class="mt-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
		{#if error}
			<p class="mb-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{error}</p>
		{/if}

		{#if tab === 'login'}
			<form method="POST" action={action('login')} use:enhance={submit} class="space-y-4">
				{@render phoneField(form?.form === 'login' ? form.phone : '')}
				<label class="block">
					<span class="mb-1 block text-sm font-medium">{m.field_password()}</span>
					<input name="password" type="password" required autocomplete="current-password" class={input} />
				</label>
				{#if data.options.password_reset}
					<p class="-mt-2 text-right text-sm">
						<a href="/auth/forgot" class="text-brand-600 hover:underline">{m.auth_forgot_password()}</a>
					</p>
				{/if}
				<button disabled={pending} class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700 disabled:opacity-60">
					{m.auth_login_submit()}
				</button>
			</form>
		{:else}
			<form method="POST" action={action('register')} use:enhance={submit} class="space-y-4">
				<label class="block">
					<span class="mb-1 block text-sm font-medium">{m.field_full_name()}</span>
					<input
						name="full_name"
						required
						maxlength="100"
						autocomplete="name"
						value={form?.form === 'register' ? form.full_name : ''}
						class={input}
					/>
				</label>
				{@render phoneField(form?.form === 'register' ? form.phone : '')}
				<label class="block">
					<span class="mb-1 block text-sm font-medium">{m.field_password()}</span>
					<input name="password" type="password" required minlength="8" autocomplete="new-password" class={input} />
				</label>
				<label class="block">
					<span class="mb-1 block text-sm font-medium">{m.field_confirm_password()}</span>
					<input name="password_confirm" type="password" required minlength="8" autocomplete="new-password" class={input} />
				</label>
				<button disabled={pending} class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700 disabled:opacity-60">
					{m.auth_register()}
				</button>
			</form>
		{/if}

		{#if data.options.google}
			<div class="my-5 flex items-center gap-3 text-xs text-slate-400"><span class="h-px flex-1 bg-slate-200"></span>{m.auth_or()}<span class="h-px flex-1 bg-slate-200"></span></div>
			<a
				href="/auth/google{query ? `?${query}` : ''}"
				data-sveltekit-reload
				class="flex w-full items-center justify-center gap-2 rounded-lg border border-slate-300 py-2.5 font-semibold hover:bg-slate-50"
			>
				<svg viewBox="0 0 24 24" class="h-5 w-5" aria-hidden="true"
					><path fill="#4285F4" d="M23.5 12.3c0-.8-.1-1.6-.2-2.3H12v4.5h6.5a5.6 5.6 0 0 1-2.4 3.6v3h3.9c2.3-2.1 3.5-5.2 3.5-8.8Z" /><path
						fill="#34A853"
						d="M12 24c3.2 0 6-1.1 8-2.9l-3.9-3c-1.1.7-2.5 1.2-4.1 1.2-3.1 0-5.8-2.1-6.7-5H1.3v3.1A12 12 0 0 0 12 24Z"
					/><path fill="#FBBC05" d="M5.3 14.3a7.2 7.2 0 0 1 0-4.6V6.6h-4a12 12 0 0 0 0 10.8l4-3.1Z" /><path
						fill="#EA4335"
						d="M12 4.8c1.8 0 3.3.6 4.6 1.8l3.4-3.4A12 12 0 0 0 1.3 6.6l4 3.1c.9-2.9 3.6-4.9 6.7-4.9Z"
					/></svg
				>
				{m.auth_google()}
			</a>
		{/if}
	</div>

	<p class="mt-6 text-center text-sm"><a href="/" class="text-slate-500 hover:text-brand-600">← {m.error_back_home()}</a></p>
</div>

{#snippet phoneField(value: string)}
	<label class="block">
		<span class="mb-1 block text-sm font-medium">{m.field_phone()}</span>
		<div class="flex rounded-lg border border-slate-200 focus-within:border-brand-500 focus-within:ring-2 focus-within:ring-brand-500/20">
			<span class="grid place-items-center border-r border-slate-200 bg-slate-50 px-3 text-sm text-slate-500">+855</span>
			<input
				name="phone"
				type="tel"
				inputmode="tel"
				required
				autocomplete="tel-national"
				placeholder="12 345 678"
				{value}
				class="w-full rounded-r-lg px-3 py-2 outline-none"
			/>
		</div>
	</label>
{/snippet}
