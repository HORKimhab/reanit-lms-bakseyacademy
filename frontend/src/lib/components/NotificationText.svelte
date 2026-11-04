<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import type { Notification } from '$lib/types';

	let { notification: n }: { notification: Notification } = $props();
	const order = $derived(n.order_id ? `#${n.order_id.slice(0, 8).toUpperCase()}` : '');
</script>

{#if n.kind === 'order_paid'}
	{m.notification_order_paid({ order })}
{:else}
	{m.notification_order_rejected({ order, note: n.note ?? '' })}
{/if}
