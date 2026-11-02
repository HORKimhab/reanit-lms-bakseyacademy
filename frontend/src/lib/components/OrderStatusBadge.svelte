<script lang="ts">
	import type { OrderStatus } from '$lib/types';

	let { status }: { status: OrderStatus } = $props();

	const styles: Record<OrderStatus, [string, string]> = {
		pending_payment: ['រង់ចាំការបង់ប្រាក់', 'bg-amber-50 text-amber-700'],
		awaiting_approval: ['រង់ចាំការអនុម័ត', 'bg-sky-50 text-sky-700'],
		paid: ['បានបង់ប្រាក់', 'bg-green-50 text-green-700'],
		rejected: ['ត្រូវបានបដិសេធ', 'bg-red-50 text-red-700'],
		cancelled: ['បានបោះបង់', 'bg-slate-100 text-slate-500']
	};
	const [label, cls] = $derived(styles[status]);
</script>

<span class="inline-block rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap {cls}">{label}</span>
