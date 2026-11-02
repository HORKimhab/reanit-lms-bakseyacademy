<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const messages: Record<string, string> = {
		invalid_credentials: 'លេខទូរស័ព្ទ ឬពាក្យសម្ងាត់មិនត្រឹមត្រូវ',
		phone_taken: 'លេខទូរស័ព្ទនេះបានចុះឈ្មោះរួចហើយ',
		invalid_phone: 'លេខទូរស័ព្ទមិនត្រឹមត្រូវ',
		invalid_name: 'សូមបញ្ចូលឈ្មោះពេញ',
		weak_password: 'ពាក្យសម្ងាត់ត្រូវមានយ៉ាងហោចណាស់ ៨ តួអក្សរ',
		password_mismatch: 'ពាក្យសម្ងាត់មិនដូចគ្នាទេ',
		unavailable: 'មិនអាចភ្ជាប់ទៅម៉ាស៊ីនមេបានទេ'
	};
	const message = (code: string) => messages[code] ?? 'មានបញ្ហា សូមព្យាយាមម្តងទៀត';

	// After a failed submit, stay on the tab that was submitted.
	const tab = $derived(form?.form ?? data.tab);
	const error = $derived(form?.form === tab ? message(form.code) : null);
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

<svelte:head><title>{tab === 'register' ? 'បង្កើតគណនី' : 'ចូលគណនី'} · Reanit Academy</title></svelte:head>

<div class="mx-auto max-w-md">
	<div class="grid grid-cols-2 rounded-xl bg-slate-100 p-1 text-center text-sm font-medium">
		<a href={tabHref('login')} class="rounded-lg py-2 {tab === 'login' ? 'bg-white shadow-sm' : 'text-slate-500'}">ចូលគណនី</a>
		<a href={tabHref('register')} class="rounded-lg py-2 {tab === 'register' ? 'bg-white shadow-sm' : 'text-slate-500'}">
			បង្កើតគណនី
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
					<span class="mb-1 block text-sm font-medium">ពាក្យសម្ងាត់</span>
					<input name="password" type="password" required autocomplete="current-password" class={input} />
				</label>
				<button disabled={pending} class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700 disabled:opacity-60">
					ចូល
				</button>
			</form>
		{:else}
			<form method="POST" action={action('register')} use:enhance={submit} class="space-y-4">
				<label class="block">
					<span class="mb-1 block text-sm font-medium">ឈ្មោះពេញ</span>
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
					<span class="mb-1 block text-sm font-medium">ពាក្យសម្ងាត់</span>
					<input name="password" type="password" required minlength="8" autocomplete="new-password" class={input} />
				</label>
				<label class="block">
					<span class="mb-1 block text-sm font-medium">បញ្ជាក់ពាក្យសម្ងាត់</span>
					<input name="password_confirm" type="password" required minlength="8" autocomplete="new-password" class={input} />
				</label>
				<button disabled={pending} class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700 disabled:opacity-60">
					បង្កើតគណនី
				</button>
			</form>
		{/if}
	</div>

	<p class="mt-6 text-center text-sm"><a href="/" class="text-slate-500 hover:text-brand-600">← ត្រឡប់ទៅទំព័រដើម</a></p>
</div>

{#snippet phoneField(value: string)}
	<label class="block">
		<span class="mb-1 block text-sm font-medium">លេខទូរស័ព្ទ</span>
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
