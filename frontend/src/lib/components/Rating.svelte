<script lang="ts">
	let { avg, count }: { avg: number; count: number } = $props();
</script>

{#if count > 0}
	<span class="inline-flex items-center gap-1 text-sm">
		<span class="text-amber-500">★</span>
		<span class="font-semibold">{avg.toFixed(1)}</span>
		<span class="text-slate-400">({count})</span>
	</span>
{:else}
	<span class="text-sm text-slate-400">មិនទាន់មានការវាយតម្លៃ</span>
{/if}
