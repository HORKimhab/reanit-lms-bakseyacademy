<script lang="ts">
	import { enhance } from '$app/forms';
	import { errorMessage } from '$lib/errors';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const status = (name: 'profile' | 'password') => {
		if (form?.form !== name) return null;
		return form.code ? { ok: false, text: errorMessage(form.code) } : { ok: true, text: 'បានរក្សាទុក' };
	};

	const input =
		'w-full rounded-lg border border-slate-200 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20';
	const button = 'rounded-lg bg-brand-600 px-5 py-2 font-semibold text-white hover:bg-brand-700';
</script>

<svelte:head><title>គណនីរបស់ខ្ញុំ · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">គណនីរបស់ខ្ញុំ</h1>
<nav class="mt-4 flex gap-4 text-sm">
	<a href="/account/library" class="text-brand-600 hover:underline">បណ្ណាល័យរបស់ខ្ញុំ</a>
	<a href="/account/orders" class="text-brand-600 hover:underline">ការបញ្ជាទិញរបស់ខ្ញុំ</a>
</nav>

<div class="mt-8 grid gap-6 lg:grid-cols-2">
	<section class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
		<h2 class="text-xl font-semibold">ព័ត៌មានផ្ទាល់ខ្លួន</h2>
		{@render notice(status('profile'))}
		<form method="POST" action="?/profile" use:enhance={() => async ({ update }) => update({ reset: false })} class="mt-4 space-y-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium">ឈ្មោះពេញ</span>
				<input name="full_name" required maxlength="100" autocomplete="name" value={data.user.full_name} class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">លេខទូរស័ព្ទ</span>
				<input value={data.user.phone ?? ''} disabled class="{input} bg-slate-50 text-slate-500" />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">អ៊ីមែល <span class="text-slate-400">(ស្រេចចិត្ត)</span></span>
				<input name="email" type="email" autocomplete="email" value={data.user.email ?? ''} class={input} />
			</label>
			<button class={button}>រក្សាទុក</button>
		</form>
	</section>

	<section class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
		<h2 class="text-xl font-semibold">ប្តូរពាក្យសម្ងាត់</h2>
		<p class="mt-1 text-sm text-slate-500">ឧបករណ៍ផ្សេងទៀតនឹងត្រូវចាកចេញ។</p>
		{@render notice(status('password'))}
		<form method="POST" action="?/password" use:enhance class="mt-4 space-y-4">
			<label class="block">
				<span class="mb-1 block text-sm font-medium">ពាក្យសម្ងាត់បច្ចុប្បន្ន</span>
				<input name="current_password" type="password" required autocomplete="current-password" class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">ពាក្យសម្ងាត់ថ្មី</span>
				<input name="new_password" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<label class="block">
				<span class="mb-1 block text-sm font-medium">បញ្ជាក់ពាក្យសម្ងាត់ថ្មី</span>
				<input name="new_password_confirm" type="password" required minlength="8" autocomplete="new-password" class={input} />
			</label>
			<button class={button}>ប្តូរពាក្យសម្ងាត់</button>
		</form>
	</section>
</div>

<form method="POST" action="/auth?/logout" class="mt-8">
	<button class="text-sm text-red-600 hover:underline">ចាកចេញ</button>
</form>

{#snippet notice(s: { ok: boolean; text: string } | null)}
	{#if s}
		<p class="mt-4 rounded-lg px-3 py-2 text-sm {s.ok ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}" role="status">
			{s.text}
		</p>
	{/if}
{/snippet}
