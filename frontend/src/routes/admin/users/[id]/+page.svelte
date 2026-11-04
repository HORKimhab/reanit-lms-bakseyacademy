<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { formatDateTime } from '$lib/format';
	import type { AdminOwnedItem, ItemOption } from '$lib/types';
	import { roleLabels } from '../roles';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const u = $derived(data.u);
	const isMe = $derived(data.user?.id === u.id);
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>{u.full_name} · Admin</title></svelte:head>

<a href="/admin/users" class="text-sm text-brand-600 hover:underline">← អ្នកប្រើប្រាស់</a>
<h1 class="mt-2 text-2xl font-bold">{u.full_name}</h1>
<p class="text-sm text-slate-600">
	{u.phone ?? ''}{u.phone && u.email ? ' · ' : ''}{u.email ?? ''} · ចុះឈ្មោះ {formatDateTime(u.created_at)}
	· <a href="/admin/orders?status=all&q={encodeURIComponent(u.phone ?? u.full_name)}" class="text-brand-600 hover:underline">ការបញ្ជាទិញ</a>
</p>

<section class="mt-6 rounded-xl border border-slate-200 bg-white p-4">
	<h2 class="font-semibold">តួនាទី</h2>
	<form method="POST" action="?/role" use:enhance={keep} class="mt-2 flex flex-wrap items-center gap-2">
		<select name="role" class="{inputClass} w-48" disabled={isMe}>
			{#each Object.entries(roleLabels) as [value, label] (value)}
				<option {value} selected={u.role === value}>{label}</option>
			{/each}
		</select>
		<button class={secondaryButton} disabled={isMe}>រក្សាទុក</button>
		{#if isMe}<span class="text-sm text-slate-500">អ្នកមិនអាចប្តូរតួនាទីខ្លួនឯងបានទេ</span>{/if}
	</form>
	<p class="mt-1 text-xs text-slate-500">«គ្រូ» ឆ្លើយសំណួរបាននៅវគ្គដែលគណនីនេះភ្ជាប់ជាគ្រូ (ភ្ជាប់នៅមូលដ្ឋានទិន្នន័យ `instructors.user_id`)។</p>
	<RowStatus {form} id="role" />
</section>

{#snippet owned(title: string, kind: 'courses' | 'ebooks', items: AdminOwnedItem[], options: ItemOption[])}
	<section class="mt-6 rounded-xl border border-slate-200 bg-white p-4">
		<h2 class="font-semibold">{title}</h2>
		{#if items.length === 0}
			<p class="mt-2 text-sm text-slate-500">គ្មាន</p>
		{:else}
			<ul class="mt-2 divide-y divide-slate-100">
				{#each items as item (item.id)}
					<li class="py-2">
						<div class="flex flex-wrap items-center justify-between gap-2">
							<div class="min-w-0">
								<a href="/{kind}/{item.slug}" class="font-medium hover:text-brand-600">{item.title}</a>
								<p class="text-xs text-slate-500">
									{formatDateTime(item.granted_at)} ·
									{#if item.order_id}ការបញ្ជាទិញ #{item.order_id.slice(0, 8).toUpperCase()}{:else}ផ្តល់ដោយអ្នកគ្រប់គ្រង{/if}
								</p>
							</div>
							<form method="POST" action="?/revoke" use:enhance>
								<input type="hidden" name="kind" value={kind} />
								<input type="hidden" name="item" value={item.id} />
								<button class={dangerButton} onclick={confirmClick(`ដកសិទ្ធិ "${item.title}"?`)}>ដកសិទ្ធិ</button>
							</form>
						</div>
						<RowStatus {form} id={item.id} />
					</li>
				{/each}
			</ul>
		{/if}
		{#if options.length}
			<form method="POST" action="?/grant" use:enhance class="mt-3 flex flex-wrap gap-2 border-t border-slate-100 pt-3">
				<input type="hidden" name="kind" value={kind} />
				<select name="item" required class="{inputClass} max-w-md flex-1">
					{#each options as o (o.id)}
						<option value={o.id}>{o.title}{o.published ? '' : ' (មិនទាន់ផ្សាយ)'}</option>
					{/each}
				</select>
				<button class={primaryButton}>+ ផ្តល់សិទ្ធិ</button>
			</form>
			<RowStatus {form} id="grant-{kind}" />
		{/if}
	</section>
{/snippet}

{@render owned('វគ្គសិក្សា', 'courses', u.courses, u.grantable_courses)}
{@render owned('សៀវភៅ', 'ebooks', u.ebooks, u.grantable_ebooks)}
