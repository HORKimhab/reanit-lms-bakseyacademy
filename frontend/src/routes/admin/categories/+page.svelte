<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>ប្រភេទ · Admin</title></svelte:head>

<h1 class="text-2xl font-bold">ប្រភេទ</h1>
<p class="text-sm text-slate-500">លុបប្រភេទ → វគ្គសិក្សា និងសៀវភៅក្នុងនោះក្លាយជា "គ្មានប្រភេទ"។ លេខលំដាប់តូចបង្ហាញមុន។</p>

<ul class="mt-4 space-y-2">
	{#each data.categories as c (c.id)}
		<li class="rounded-xl border border-slate-200 bg-white p-3">
			<div class="flex flex-wrap items-center gap-2">
				<form method="POST" action="?/update" use:enhance={keep} class="flex flex-1 flex-wrap gap-2">
					<input type="hidden" name="id" value={c.id} />
					<input name="name_km" required value={c.name_km} placeholder="ឈ្មោះ (ខ្មែរ)" class="{inputClass} w-48" />
					<input name="name_en" required value={c.name_en} placeholder="Name (English)" class="{inputClass} w-48" />
					<input name="slug" required value={c.slug} placeholder="slug" class="{inputClass} w-40 font-mono" />
					<input name="position" value={c.position} inputmode="numeric" class="{inputClass} w-16" aria-label="លំដាប់" />
					<button class={secondaryButton}>រក្សាទុក</button>
				</form>
				<form method="POST" action="?/delete" use:enhance>
					<input type="hidden" name="id" value={c.id} />
					<button class={dangerButton} onclick={confirmClick(`លុប "${c.name_km}"?`)}>លុប</button>
				</form>
			</div>
			<RowStatus {form} id={c.id} />
		</li>
	{/each}
</ul>

<h2 class="mt-8 font-semibold">ប្រភេទថ្មី</h2>
<form method="POST" action="?/create" use:enhance class="mt-2 flex flex-wrap gap-2">
	<input name="name_km" required placeholder="ឈ្មោះ (ខ្មែរ)" class="{inputClass} w-48" />
	<input name="name_en" required placeholder="Name (English)" class="{inputClass} w-48" />
	<input name="slug" required placeholder="slug" class="{inputClass} w-40 font-mono" />
	<input name="position" placeholder="0" inputmode="numeric" class="{inputClass} w-16" aria-label="លំដាប់" />
	<button class={primaryButton}>+ បន្ថែម</button>
</form>
<RowStatus {form} id="new" />
