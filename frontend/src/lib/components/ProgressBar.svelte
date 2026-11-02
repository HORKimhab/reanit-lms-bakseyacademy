<script lang="ts">
	import { khmerNumber } from '$lib/format';

	let { done, total }: { done: number; total: number } = $props();
	const percent = $derived(total > 0 ? Math.round((done / total) * 100) : 0);
</script>

<div class="flex items-center gap-2 text-xs text-slate-500">
	<div
		class="h-1.5 flex-1 overflow-hidden rounded-full bg-slate-200"
		role="progressbar"
		aria-valuemin={0}
		aria-valuemax={100}
		aria-valuenow={percent}
	>
		<div class="h-full rounded-full bg-green-600" style="width: {percent}%"></div>
	</div>
	<span>{khmerNumber(percent)}%</span>
</div>
