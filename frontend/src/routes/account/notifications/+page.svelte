<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages.js';
	import NotificationText from '$lib/components/NotificationText.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>{m.notifications_title()} · Reanit Academy</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-3xl font-bold">{m.notifications_title()}</h1>
	{#if data.list.unread_count > 0}
		<form method="POST" action="?/read" use:enhance>
			<button class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium hover:bg-slate-100">{m.notifications_mark_read()}</button>
		</form>
	{/if}
</div>
{#if form?.code}<p class="mt-3 text-sm text-red-600">{errorMessage(form.code)}</p>{/if}

{#if data.list.items.length === 0}
	<p class="mt-12 text-center text-slate-500">{m.notifications_empty()}</p>
{:else}
	<ul class="mt-6 divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white">
		{#each data.list.items as n (n.id)}
			<li>
				<a href={n.order_id ? `/orders/${n.order_id}` : '/account/orders'} class="flex gap-3 p-4 hover:bg-slate-50">
					<span class="mt-2 h-2 w-2 shrink-0 rounded-full {n.read_at ? 'bg-transparent' : 'bg-brand-600'}"></span>
					<span class="min-w-0">
						<span class="block text-sm {n.read_at ? 'text-slate-600' : 'font-medium'}"><NotificationText notification={n} /></span>
						<span class="text-xs text-slate-400">{formatDateTime(n.created_at)}</span>
					</span>
				</a>
			</li>
		{/each}
	</ul>
{/if}
