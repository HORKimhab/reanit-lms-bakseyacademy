<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const ebook = $derived(data.ebook);
</script>

<svelte:head><title>{ebook.title} · Reanit Academy</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-3">
	<div>
		<a href="/ebooks/{ebook.slug}" class="text-sm text-brand-600 hover:underline">← ត្រឡប់ក្រោយ</a>
		<h1 class="text-2xl font-bold">{ebook.title}</h1>
	</div>
	{#if data.fileUrl}
		<a
			href="/ebooks/{ebook.slug}/download"
			data-sveltekit-reload
			class="rounded-lg border border-brand-600 px-4 py-2 text-sm font-semibold text-brand-600 hover:bg-brand-50"
		>
			ទាញយក
		</a>
	{/if}
</div>

{#if data.fileUrl}
	<iframe src={data.fileUrl} title={ebook.title} class="mt-4 h-[80vh] w-full rounded-xl border border-slate-200 bg-white"></iframe>
{:else}
	<p class="mt-10 rounded-xl border border-slate-200 bg-white p-8 text-center text-slate-500">
		ឯកសារសៀវភៅនេះមិនទាន់អាចប្រើបានទេ។ សូមទាក់ទងក្រុមការងារ។
	</p>
{/if}
