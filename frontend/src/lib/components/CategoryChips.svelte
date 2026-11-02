<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { localized } from '$lib/i18n';
	import type { Category } from '$lib/types';

	let { categories, active, basePath, q }: { categories: Category[]; active: string; basePath: string; q: string } =
		$props();

	function href(slug: string) {
		const p = new URLSearchParams();
		if (q) p.set('q', q);
		if (slug) p.set('category', slug);
		const s = p.toString();
		return s ? `${basePath}?${s}` : basePath;
	}
</script>

<div class="flex flex-wrap gap-2">
	{#each [{ slug: '', name_km: m.category_all(), name_en: m.category_all() }, ...categories] as c (c.slug)}
		<a
			href={href(c.slug)}
			class="rounded-full border px-3 py-1 text-sm transition {active === c.slug
				? 'border-brand-600 bg-brand-600 text-white'
				: 'border-slate-200 bg-white hover:border-brand-500'}"
		>
			{localized(c.name_km, c.name_en)}
		</a>
	{/each}
</div>
