<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import type { OrderStatus } from '$lib/types';

	let { status }: { status: OrderStatus } = $props();

	const styles: Record<OrderStatus, [string, string]> = {
		pending_payment: [m.order_status_pending_payment(), 'bg-amber-50 text-amber-700'],
		awaiting_approval: [m.order_status_awaiting_approval(), 'bg-sky-50 text-sky-700'],
		paid: [m.order_status_paid(), 'bg-green-50 text-green-700'],
		rejected: [m.order_status_rejected(), 'bg-red-50 text-red-700'],
		cancelled: [m.order_status_cancelled(), 'bg-slate-100 text-slate-500']
	};
	const [label, cls] = $derived(styles[status]);
</script>

<span class="inline-block rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap {cls}">{label}</span>
