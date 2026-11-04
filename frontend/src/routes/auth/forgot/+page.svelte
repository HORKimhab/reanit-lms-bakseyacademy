<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages.js';
	import { errorMessage } from '$lib/errors';
	import { site } from '$lib/site';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
	const step = $derived(form?.step ?? 'request');
	const input =
		'w-full rounded-lg border border-slate-200 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20';
</script>

<svelte:head><title>{m.reset_title()} · Reanit Academy</title></svelte:head>

<div class="mx-auto max-w-md rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
	<h1 class="text-2xl font-bold">{m.reset_title()}</h1>

	{#if form?.code}
		<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
	{/if}

	{#if step === 'request'}
		<p class="mt-2 text-sm text-slate-600">{m.reset_request_hint()}</p>
		<form method="POST" action="?/request" use:enhance class="mt-4 space-y-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.field_phone()}</span>
				<input name="phone" type="tel" required autocomplete="tel" inputmode="tel" placeholder="012 345 678" value={form?.phone ?? ''} class={input} />
			</label>
			<button class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700">{m.reset_send_code()}</button>
		</form>
	{:else}
		<p class="mt-2 text-sm text-slate-600" role="status">{m.reset_code_sent()}</p>
		<form method="POST" action="?/confirm" use:enhance class="mt-4 space-y-4">
			<input type="hidden" name="phone" value={form?.phone ?? ''} />
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.reset_code_label()}</span>
				<input name="code" required inputmode="numeric" autocomplete="one-time-code" pattern="[0-9]{6}" maxlength="6" class="{input} font-mono tracking-widest" />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.reset_new_password()}</span>
				<input name="new_password" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">{m.reset_new_password_confirm()}</span>
				<input name="new_password_confirm" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<button class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700">{m.reset_submit()}</button>
		</form>
		<form method="POST" action="?/request" use:enhance class="mt-3 text-center">
			<input type="hidden" name="phone" value={form?.phone ?? ''} />
			<button class="text-sm text-brand-600 hover:underline">{m.reset_resend()}</button>
		</form>
	{/if}

	<p class="mt-6 text-center text-sm text-slate-500">
		<a href="/auth" class="text-brand-600 hover:underline">{m.nav_login()}</a>
		{#if site.supportUrl}· <a href={site.supportUrl} target="_blank" rel="noopener" class="hover:underline">{m.reset_need_help()}</a>{/if}
	</p>
</div>
