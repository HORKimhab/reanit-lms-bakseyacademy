<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { enhance } from '$app/forms';
	import { errorMessage } from '$lib/errors';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const status = (name: 'profile' | 'password' | 'google') => {
		if (form?.form !== name) return null;
		return form.code ? { ok: false, text: errorMessage(form.code) } : { ok: true, text: m.saved() };
	};
	const googleStatus = $derived(
		status('google') ??
			(data.oauthError ? { ok: false, text: errorMessage(data.oauthError) } : data.googleLinked ? { ok: true, text: m.account_google_linked_done() } : null)
	);

	const input =
		'w-full rounded-lg border border-slate-200 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20';
	const button = 'rounded-lg bg-brand-600 px-5 py-2 font-semibold text-white hover:bg-brand-700';
</script>

<svelte:head><title>{m.nav_account()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_account()}</h1>
<nav class="mt-4 flex gap-4 text-sm">
	<a href="/account/library" class="text-brand-600 hover:underline">{m.nav_library()}</a>
	<a href="/account/orders" class="text-brand-600 hover:underline">{m.nav_orders()}</a>
</nav>

<div class="mt-8 grid gap-6 lg:grid-cols-2">
	<section class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
		<h2 class="text-xl font-semibold">{m.account_profile()}</h2>
		{@render notice(status('profile'))}
		<form method="POST" action="?/profile" use:enhance={() => async ({ update }) => update({ reset: false })} class="mt-4 space-y-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_full_name()}</span>
				<input name="full_name" required maxlength="100" autocomplete="name" value={data.user.full_name} class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_phone()}</span>
				<input value={data.user.phone ?? ''} disabled class="{input} bg-slate-50 text-slate-500" />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_email()} <span class="text-slate-400">({m.optional()})</span></span>
				<input name="email" type="email" autocomplete="email" value={data.user.email ?? ''} class={input} />
			</label>
			<button class={button}>{m.save()}</button>
		</form>
	</section>

	<section class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
		<h2 class="text-xl font-semibold">{m.account_change_password()}</h2>
		<p class="mt-1 text-sm text-slate-500">{m.account_password_hint()}</p>
		{@render notice(status('password'))}
		<form method="POST" action="?/password" use:enhance class="mt-4 space-y-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_current_password()}</span>
				<input name="current_password" type="password" required autocomplete="current-password" class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_new_password()}</span>
				<input name="new_password" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_confirm_new_password()}</span>
				<input name="new_password_confirm" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<button class={button}>{m.account_change_password()}</button>
		</form>
	</section>
</div>

{#if data.google}
	<section class="mt-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm lg:max-w-xl">
		<h2 class="text-xl font-semibold">Google</h2>
		{@render notice(googleStatus)}
		{#if data.user.google_linked}
			<p class="mt-2 text-sm text-slate-600">✓ {m.account_google_linked()}</p>
			<form method="POST" action="?/unlinkGoogle" use:enhance class="mt-4">
				<button class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium hover:bg-slate-50">{m.account_google_unlink()}</button>
			</form>
		{:else}
			<p class="mt-2 text-sm text-slate-600">{m.account_google_hint()}</p>
			<a href="/auth/google?mode=link" data-sveltekit-reload class="mt-4 inline-block rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium hover:bg-slate-50">
				{m.account_google_link()}
			</a>
		{/if}
	</section>
{/if}

<form method="POST" action="/auth?/logout" class="mt-8">
	<button class="text-sm text-red-600 hover:underline">{m.nav_logout()}</button>
</form>

{#snippet notice(s: { ok: boolean; text: string } | null)}
	{#if s}
		<p class="mt-4 rounded-lg px-3 py-2 text-sm {s.ok ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}" role="status">
			{s.text}
		</p>
	{/if}
{/snippet}
