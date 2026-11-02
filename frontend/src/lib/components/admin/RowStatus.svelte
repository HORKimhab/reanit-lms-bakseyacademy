<script lang="ts">
	import { errorMessage } from '$lib/errors';

	// Shows the result of the last action on the row with `id` ('new' for the create form).
	let { form, id }: { form: { id?: string; code?: string; saved?: boolean } | null | undefined; id: string } = $props();
</script>

{#if form?.id === id}
	{#if form.code}
		<p class="mt-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
	{:else if form.saved}
		<p class="mt-2 text-sm text-green-700" role="status">បានរក្សាទុក</p>
	{/if}
{/if}
