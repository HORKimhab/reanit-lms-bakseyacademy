<script lang="ts">
	import { page as pageState } from '$app/state';

	let { page, perPage, total }: { page: number; perPage: number; total: number } = $props();

	const pages = $derived(Math.max(1, Math.ceil(total / perPage)));

	function href(n: number) {
		const p = new URLSearchParams(pageState.url.searchParams);
		if (n === 1) p.delete('page');
		else p.set('page', String(n));
		const s = p.toString();
		return s ? `?${s}` : pageState.url.pathname;
	}
</script>

{#if pages > 1}
	<nav class="mt-8 flex justify-center gap-1" aria-label="Pagination">
		{#each Array.from({ length: pages }, (_, i) => i + 1) as n}
			<a
				href={href(n)}
				aria-current={n === page ? 'page' : undefined}
				class="grid h-9 min-w-9 place-items-center rounded-lg border px-2 text-sm {n === page
					? 'border-brand-600 bg-brand-600 text-white'
					: 'border-slate-200 bg-white hover:border-brand-500'}"
			>
				{n}
			</a>
		{/each}
	</nav>
{/if}
