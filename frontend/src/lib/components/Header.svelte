<script lang="ts">
	import { page } from '$app/state';

	const links = [
		{ href: '/', label: 'ទំព័រដើម' },
		{ href: '/courses', label: 'វគ្គសិក្សា' },
		{ href: '/ebooks', label: 'សៀវភៅ' }
	];

	let menuOpen = $state(false);
	let searchInput: HTMLInputElement | undefined = $state();

	const isActive = (href: string) =>
		href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href);

	function onKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
			e.preventDefault();
			searchInput?.focus();
		}
	}
</script>

<svelte:window onkeydown={onKeydown} />

<header class="sticky top-0 z-30 border-b border-slate-200 bg-white/90 backdrop-blur">
	<div class="mx-auto flex h-16 max-w-7xl items-center gap-4 px-4">
		<a href="/" class="text-lg font-bold text-brand-600">Reanit Academy</a>

		<nav class="hidden gap-1 md:flex">
			{#each links as link}
				<a
					href={link.href}
					class="rounded-lg px-3 py-1.5 text-sm font-medium transition hover:bg-slate-100"
					class:text-brand-600={isActive(link.href)}
				>
					{link.label}
				</a>
			{/each}
		</nav>

		<form action="/courses" method="get" class="relative ml-auto hidden w-full max-w-xs sm:block">
			<input
				bind:this={searchInput}
				name="q"
				type="search"
				placeholder="ស្វែងរកវគ្គសិក្សា..."
				class="w-full rounded-lg border border-slate-200 bg-slate-50 py-1.5 pr-12 pl-3 text-sm outline-none focus:border-brand-500 focus:bg-white"
			/>
			<kbd class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border border-slate-200 px-1.5 text-xs text-slate-400">⌘K</kbd>
		</form>

		<!-- TODO: cart + account once auth/orders land -->

		<button
			class="ml-auto rounded-lg p-2 hover:bg-slate-100 sm:ml-0 md:hidden"
			aria-label="Menu"
			aria-expanded={menuOpen}
			onclick={() => (menuOpen = !menuOpen)}
		>
			☰
		</button>
	</div>

	{#if menuOpen}
		<nav class="border-t border-slate-200 px-4 py-2 md:hidden">
			{#each links as link}
				<a href={link.href} class="block py-2" onclick={() => (menuOpen = false)}>{link.label}</a>
			{/each}
			<form action="/courses" method="get" class="py-2 sm:hidden">
				<input name="q" type="search" placeholder="ស្វែងរកវគ្គសិក្សា..." class="w-full rounded-lg border border-slate-200 px-3 py-1.5 text-sm" />
			</form>
		</nav>
	{/if}
</header>
