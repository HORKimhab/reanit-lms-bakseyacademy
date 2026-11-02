<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();
	const links = [{ href: '/admin/orders', label: 'ការបញ្ជាទិញ' }];
</script>

<div class="mb-6 flex items-center gap-4 border-b border-slate-200 pb-3">
	<span class="rounded bg-slate-900 px-2 py-0.5 text-xs font-semibold text-white">ADMIN</span>
	{#each links as l (l.href)}
		<a href={l.href} class="text-sm font-medium {page.url.pathname.startsWith(l.href) ? 'text-brand-600' : 'text-slate-600 hover:text-brand-600'}">
			{l.label}
		</a>
	{/each}
</div>

{@render children()}
