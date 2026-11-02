<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { khmerNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>គ្រូ · Admin</title></svelte:head>

<h1 class="text-2xl font-bold">គ្រូ</h1>

<ul class="mt-4 space-y-2">
	{#each data.instructors as i (i.id)}
		<li class="rounded-xl border border-slate-200 bg-white p-3">
			<form method="POST" action="?/update" use:enhance={keep} class="grid gap-2 md:grid-cols-[1fr_1fr_auto]">
				<input type="hidden" name="id" value={i.id} />
				<input name="name" required value={i.name} placeholder="ឈ្មោះ" class={inputClass} />
				<input name="avatar_url" value={i.avatar_url ?? ''} placeholder="រូបថត (URL)" class={inputClass} />
				<button class={secondaryButton}>រក្សាទុក</button>
				<textarea name="bio" rows="2" placeholder="ប្រវត្តិរូប" class="{inputClass} md:col-span-3">{i.bio ?? ''}</textarea>
			</form>
			<div class="mt-2 flex items-center justify-between text-xs text-slate-500">
				<span>{khmerNumber(i.course_count)} វគ្គសិក្សា</span>
				<form method="POST" action="?/delete" use:enhance>
					<input type="hidden" name="id" value={i.id} />
					<button class={dangerButton} onclick={confirmClick(`លុប "${i.name}"?`)}>លុប</button>
				</form>
			</div>
			<RowStatus {form} id={i.id} />
		</li>
	{/each}
</ul>

<h2 class="mt-8 font-semibold">គ្រូថ្មី</h2>
<form method="POST" action="?/create" use:enhance class="mt-2 grid gap-2 rounded-xl border border-slate-200 bg-white p-3 md:grid-cols-[1fr_1fr_auto]">
	<input name="name" required placeholder="ឈ្មោះ" class={inputClass} />
	<input name="avatar_url" placeholder="រូបថត (URL)" class={inputClass} />
	<button class={primaryButton}>+ បន្ថែម</button>
	<textarea name="bio" rows="2" placeholder="ប្រវត្តិរូប" class="{inputClass} md:col-span-3"></textarea>
</form>
<RowStatus {form} id="new" />
