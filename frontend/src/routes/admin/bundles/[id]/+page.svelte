<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, dollarsInput, inputClass, labelClass, primaryButton } from '$lib/admin-ui';
	import { formatPrice } from '$lib/format';
	import type { ItemOption } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const b = $derived(data.bundle);
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
	const items = $derived([
		...b.courses.map((c) => ({ kind: 'courses' as const, id: c.id, slug: c.slug, title: c.title, price: c.sale_price_cents ?? c.price_cents })),
		...b.ebooks.map((e) => ({ kind: 'ebooks' as const, id: e.id, slug: e.slug, title: e.title, price: e.sale_price_cents ?? e.price_cents }))
	]);
	const value = $derived(items.reduce((sum, i) => sum + i.price, 0));
</script>

<svelte:head><title>{b.title} · Admin</title></svelte:head>

<a href="/admin/bundles" class="text-sm text-brand-600 hover:underline">← កញ្ចប់</a>
<div class="mt-2 flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">{b.title}</h1>
	{#if b.published && items.length}<a href="/bundles/{b.slug}" class="text-sm text-brand-600 hover:underline">មើលទំព័រ ↗</a>{/if}
</div>

<form method="POST" action="?/save" use:enhance={keep} class="mt-4 grid gap-4 rounded-xl border border-slate-200 bg-white p-4 md:grid-cols-2">
	<label class={labelClass}>ចំណងជើង <input name="title" required value={b.title} class="{inputClass} mt-1" /></label>
	<label class={labelClass}>Slug <input name="slug" required value={b.slug} class="{inputClass} mt-1 font-mono" /></label>
	<label class="{labelClass} md:col-span-2">សេចក្តីសង្ខេប <input name="summary" value={b.summary ?? ''} class="{inputClass} mt-1" /></label>
	<label class="{labelClass} md:col-span-2">ពិពណ៌នា <textarea name="description" rows="5" class="{inputClass} mt-1">{b.description}</textarea></label>
	<label class="{labelClass} md:col-span-2">រូបក្រប (URL) <input name="cover_url" value={b.cover_url ?? ''} class="{inputClass} mt-1" /></label>
	<label class={labelClass}>តម្លៃ ($) <input name="price" required inputmode="decimal" value={dollarsInput(b.price_cents)} class="{inputClass} mt-1" /></label>
	<label class={labelClass}>តម្លៃបញ្ចុះ ($) <input name="sale_price" inputmode="decimal" value={dollarsInput(b.sale_price_cents)} class="{inputClass} mt-1" /></label>
	<label class="flex items-center gap-2 text-sm"><input type="checkbox" name="published" checked={b.published} class="accent-brand-600" /> ផ្សាយ</label>
	<div class="flex items-center justify-end gap-2">
		<button class={primaryButton}>រក្សាទុក</button>
	</div>
	<div class="md:col-span-2"><RowStatus {form} id="fields" /></div>
</form>

<section class="mt-6 rounded-xl border border-slate-200 bg-white p-4">
	<h2 class="font-semibold">មាតិកា</h2>
	<p class="text-sm text-slate-500">តម្លៃទិញដាច់ដោយឡែក {formatPrice(value)} · អ្នកទិញថ្មីទទួលបានមាតិកាបច្ចុប្បន្ន; អ្នកទិញមុនរក្សាអ្វីដែលបានទទួលរួច។</p>
	{#if items.length === 0}
		<p class="mt-3 text-sm text-amber-700">កញ្ចប់ទទេ — បន្ថែមវគ្គ ឬសៀវភៅ ដើម្បីលក់។</p>
	{:else}
		<ul class="mt-3 divide-y divide-slate-100">
			{#each items as item (item.id)}
				<li class="py-2">
					<div class="flex items-center justify-between gap-3">
						<span><span class="text-xs text-slate-400">{item.kind === 'courses' ? 'វគ្គ' : 'សៀវភៅ'} ·</span> {item.title} <span class="text-slate-400">({formatPrice(item.price)})</span></span>
						<form method="POST" action="?/removeItem" use:enhance>
							<input type="hidden" name="kind" value={item.kind} />
							<input type="hidden" name="item" value={item.id} />
							<button class={dangerButton}>ដកចេញ</button>
						</form>
					</div>
					<RowStatus {form} id={item.id} />
				</li>
			{/each}
		</ul>
	{/if}

	{#snippet picker(kind: 'courses' | 'ebooks', label: string, options: ItemOption[])}
		{#if options.length}
			<form method="POST" action="?/addItem" use:enhance class="mt-3 flex flex-wrap gap-2">
				<input type="hidden" name="kind" value={kind} />
				<select name="item" required class="{inputClass} max-w-md flex-1">
					{#each options as o (o.id)}
						<option value={o.id}>{o.title}{o.published ? '' : ' (មិនទាន់ផ្សាយ)'}</option>
					{/each}
				</select>
				<button class={primaryButton}>+ {label}</button>
			</form>
			<RowStatus {form} id="add-{kind}" />
		{/if}
	{/snippet}
	<div class="mt-2 border-t border-slate-100 pt-1">
		{@render picker('courses', 'វគ្គ', b.other_courses)}
		{@render picker('ebooks', 'សៀវភៅ', b.other_ebooks)}
	</div>
</section>

<form method="POST" action="?/delete" use:enhance class="mt-6">
	<button class={dangerButton} onclick={confirmClick(`លុបកញ្ចប់ "${b.title}"?`)}>លុបកញ្ចប់</button>
</form>
