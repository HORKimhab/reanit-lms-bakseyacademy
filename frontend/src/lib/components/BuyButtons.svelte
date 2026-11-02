<script lang="ts">
	import { enhance } from '$app/forms';
	import { errorMessage } from '$lib/errors';

	let { form, full = false }: { form: { code?: string; added?: boolean } | null | undefined; full?: boolean } = $props();
</script>

<form method="POST" action="?/addToCart" use:enhance class="flex flex-wrap gap-2 {full ? 'flex-col' : ''}">
	<button name="next" value="cart" class="rounded-lg bg-brand-600 px-6 py-2.5 font-semibold text-white hover:bg-brand-700">
		ទិញឥឡូវ
	</button>
	<button class="rounded-lg border border-brand-600 px-6 py-2.5 font-semibold text-brand-600 hover:bg-brand-50">
		បន្ថែមទៅកន្ត្រក
	</button>
</form>
{#if form?.added}
	<p class="mt-2 text-sm text-green-700" role="status">បានបន្ថែមទៅកន្ត្រក · <a href="/cart" class="underline">មើលកន្ត្រក</a></p>
{:else if form?.code}
	<p class="mt-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}
