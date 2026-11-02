<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>ផ្ទាំងផ្សាយ · Admin</title></svelte:head>

<h1 class="text-2xl font-bold">ផ្ទាំងផ្សាយ (ទំព័រដើម)</h1>
<p class="text-sm text-slate-500">គ្មានរូបភាព → ប្រើពណ៌ម៉ាក។ តំណ៖ ផ្លូវក្នុងគេហទំព័រ (/courses) ឬ https://…</p>

<ul class="mt-4 space-y-3">
	{#each data.banners as b (b.id)}
		<li class="rounded-xl border border-slate-200 bg-white p-3">
			<div class="flex gap-3">
				<div class="hidden h-20 w-36 shrink-0 overflow-hidden rounded-lg bg-gradient-to-br from-brand-500 to-sky-400 sm:block">
					{#if b.image_url}<img src={b.image_url} alt="" class="h-full w-full object-cover" />{/if}
				</div>
				<form method="POST" action="?/update" use:enhance={keep} class="grid flex-1 gap-2 md:grid-cols-2">
					<input type="hidden" name="id" value={b.id} />
					<input name="title" required value={b.title} placeholder="ចំណងជើង" class={inputClass} />
					<input name="subtitle" value={b.subtitle ?? ''} placeholder="ចំណងជើងរង" class={inputClass} />
					<input name="image_url" value={b.image_url ?? ''} placeholder="រូបភាព (URL)" class={inputClass} />
					<input name="link_url" value={b.link_url ?? ''} placeholder="តំណ" class={inputClass} />
					<div class="flex items-center gap-3 md:col-span-2">
						<label class="flex items-center gap-1 text-sm">លំដាប់ <input name="position" value={b.position} inputmode="numeric" class="{inputClass} w-16" /></label>
						<label class="flex items-center gap-1 text-sm"><input type="checkbox" name="published" checked={b.published} /> ផ្សាយ</label>
						<button class="{secondaryButton} ml-auto">រក្សាទុក</button>
					</div>
				</form>
			</div>
			<div class="mt-2 text-right">
				<form method="POST" action="?/delete" use:enhance>
					<input type="hidden" name="id" value={b.id} />
					<button class={dangerButton} onclick={confirmClick(`លុប "${b.title}"?`)}>លុប</button>
				</form>
			</div>
			<RowStatus {form} id={b.id} />
		</li>
	{/each}
</ul>

<h2 class="mt-8 font-semibold">ផ្ទាំងផ្សាយថ្មី</h2>
<form method="POST" action="?/create" use:enhance class="mt-2 grid gap-2 rounded-xl border border-slate-200 bg-white p-3 md:grid-cols-2">
	<input name="title" required placeholder="ចំណងជើង" class={inputClass} />
	<input name="subtitle" placeholder="ចំណងជើងរង" class={inputClass} />
	<input name="image_url" placeholder="រូបភាព (URL)" class={inputClass} />
	<input name="link_url" placeholder="តំណ" class={inputClass} />
	<div class="flex items-center gap-3 md:col-span-2">
		<label class="flex items-center gap-1 text-sm">លំដាប់ <input name="position" placeholder="0" inputmode="numeric" class="{inputClass} w-16" /></label>
		<label class="flex items-center gap-1 text-sm"><input type="checkbox" name="published" /> ផ្សាយ</label>
		<button class="{primaryButton} ml-auto">+ បន្ថែម</button>
	</div>
</form>
<RowStatus {form} id="new" />
