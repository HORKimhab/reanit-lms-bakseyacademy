<script lang="ts">
	let { action, q, category, placeholder }: { action: string; q: string; category: string; placeholder: string } =
		$props();
</script>

<form {action} method="get" class="flex gap-2">
	<input
		name="q"
		type="search"
		value={q}
		{placeholder}
		class="w-full max-w-md rounded-lg border border-slate-200 bg-white px-3 py-2 outline-none focus:border-brand-500"
	/>
	{#if category}<input type="hidden" name="category" value={category} />{/if}
	<button class="rounded-lg bg-brand-600 px-4 py-2 font-medium text-white hover:bg-brand-700">ស្វែងរក</button>
</form>
