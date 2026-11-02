<script lang="ts">
	import { inputClass, secondaryButton } from '$lib/admin-ui';

	// Narrows the "ជ្រើសពីបណ្ណាល័យ" pickers on this page: reloads it with ?mediaq= (see mediaPickerPath).
	let { q }: { q: string } = $props();
</script>

<form method="GET" class="mb-3 flex items-center gap-2 text-sm" data-sveltekit-keepfocus data-sveltekit-noscroll>
	<input name="mediaq" type="search" value={q} placeholder="ស្វែងរកឯកសារក្នុងបណ្ណាល័យ" class="{inputClass} w-64" />
	<button class={secondaryButton}>ស្វែងរក</button>
	{#if q}<a href="?" class="text-slate-500 hover:underline">សម្អាត</a>{/if}
</form>
