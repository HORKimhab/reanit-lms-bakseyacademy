<script lang="ts">
	import { formatPrice } from '$lib/format';

	let { price, sale, size = 'md' }: { price: number; sale: number | null; size?: 'md' | 'lg' } = $props();
</script>

<span class="inline-flex items-baseline gap-2">
	<span class="font-bold text-brand-600" class:text-2xl={size === 'lg'}>{formatPrice(sale ?? price)}</span>
	{#if sale !== null}
		<span class="text-sm text-slate-400 line-through">{formatPrice(price)}</span>
	{/if}
</span>
