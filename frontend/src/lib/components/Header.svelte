<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { page } from '$app/state';
	import { formatNumber } from '$lib/format';
	import { getLocale, setLocale } from '$lib/paraglide/runtime';

	const links = [
		{ href: '/', label: m.nav_home() },
		{ href: '/courses', label: m.nav_courses() },
		{ href: '/ebooks', label: m.nav_ebooks() },
		{ href: '/for-you', label: m.nav_for_you() }
	];

	let menuOpen = $state(false);
	let searchInput: HTMLInputElement | undefined = $state();

	const user = $derived(page.data.user ?? null);
	const cartCount = $derived(page.data.cartCount ?? 0);
	const unread = $derived(page.data.notifications?.unread_count ?? 0);
	const loginHref = $derived(
		page.url.pathname.startsWith('/auth')
			? '/auth'
			: `/auth?redirectTo=${encodeURIComponent(page.url.pathname + page.url.search)}`
	);

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

		<form action="/search" method="get" class="relative ml-auto hidden w-full max-w-xs sm:block">
			<input
				bind:this={searchInput}
				name="q"
				value={page.url.pathname === '/search' ? (page.url.searchParams.get('q') ?? '') : ''}
				type="search"
				placeholder={m.search_placeholder()}
				class="w-full rounded-lg border border-slate-200 bg-slate-50 py-1.5 pr-12 pl-3 text-sm outline-none focus:border-brand-500 focus:bg-white"
			/>
			<kbd class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border border-slate-200 px-1.5 text-xs text-slate-400">⌘K</kbd>
		</form>

		{#if user}
			<a href="/cart" class="relative ml-auto rounded-lg p-2 hover:bg-slate-100 sm:ml-0" aria-label={m.nav_cart()}>
				<svg viewBox="0 0 24 24" class="h-6 w-6" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
					<path d="M3 4h2l2.4 11.2a1 1 0 0 0 1 .8h9.2a1 1 0 0 0 1-.8L20.5 8H6.2" stroke-linecap="round" stroke-linejoin="round" />
					<circle cx="9" cy="20" r="1.3" /><circle cx="17" cy="20" r="1.3" />
				</svg>
				{#if cartCount}
					<span class="absolute -top-0.5 -right-0.5 grid h-5 min-w-5 place-items-center rounded-full bg-red-600 px-1 text-xs text-white">
						{formatNumber(cartCount)}
					</span>
				{/if}
			</a>
		{/if}

		{#if user}
			<a href="/account/notifications" class="relative rounded-lg p-2 hover:bg-slate-100" aria-label={m.notifications_title()}>
				<svg viewBox="0 0 24 24" class="h-6 w-6" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
					<path d="M6 16V11a6 6 0 1 1 12 0v5l1.5 2h-15L6 16Z" stroke-linejoin="round" />
					<path d="M10 20a2 2 0 0 0 4 0" stroke-linecap="round" />
				</svg>
				{#if unread}
					<span class="absolute -top-0.5 -right-0.5 grid h-5 min-w-5 place-items-center rounded-full bg-red-600 px-1 text-xs text-white">
						{formatNumber(unread)}
					</span>
				{/if}
			</a>
		{/if}

		<button
			onclick={() => setLocale(getLocale() === 'km' ? 'en' : 'km')}
			class="rounded-lg px-2 py-1.5 text-sm font-medium text-slate-600 hover:bg-slate-100 {user ? '' : 'ml-auto sm:ml-0'}"
			lang={getLocale() === 'km' ? 'en' : 'km'}
			aria-label={m.switch_language_label()}
		>
			{getLocale() === 'km' ? 'EN' : 'ខ្មែរ'}
		</button>

		<div class="hidden md:block">
			{#if user}
				<details class="relative">
					<summary class="flex cursor-pointer list-none items-center gap-2 rounded-lg px-3 py-1.5 text-sm font-medium hover:bg-slate-100">
						<span class="grid h-7 w-7 place-items-center rounded-full bg-brand-600 text-xs text-white">
							{user.full_name.slice(0, 1)}
						</span>
						<span class="max-w-32 truncate">{user.full_name}</span>
					</summary>
					<div class="absolute right-0 mt-2 w-48 rounded-xl border border-slate-200 bg-white p-1 text-sm shadow-lg">
						<a href="/account/library" class="block rounded-lg px-3 py-2 hover:bg-slate-100">{m.nav_library()}</a>
						<a href="/account/orders" class="block rounded-lg px-3 py-2 hover:bg-slate-100">{m.nav_orders()}</a>
						<a href="/account/notifications" class="block rounded-lg px-3 py-2 hover:bg-slate-100">{m.notifications_title()}</a>
						<a href="/account" class="block rounded-lg px-3 py-2 hover:bg-slate-100">{m.nav_account()}</a>
						{#if user.role === 'admin'}
							<a href="/admin/orders" class="block rounded-lg px-3 py-2 hover:bg-slate-100">{m.nav_admin()}</a>
						{/if}
						<form method="POST" action="/auth?/logout">
							<button class="w-full rounded-lg px-3 py-2 text-left text-red-600 hover:bg-red-50">{m.nav_logout()}</button>
						</form>
					</div>
				</details>
			{:else}
				<a href={loginHref} class="rounded-lg bg-brand-600 px-4 py-1.5 text-sm font-semibold whitespace-nowrap text-white hover:bg-brand-700">
					{m.nav_login()}
				</a>
			{/if}
		</div>

		<button
			class="rounded-lg p-2 hover:bg-slate-100 md:hidden"
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
			{#if user}
				<a href="/account/library" class="block py-2" onclick={() => (menuOpen = false)}>{m.nav_library()}</a>
				<a href="/account/orders" class="block py-2" onclick={() => (menuOpen = false)}>{m.nav_orders()}</a>
				<a href="/account/notifications" class="block py-2" onclick={() => (menuOpen = false)}>{m.notifications_title()}</a>
				<a href="/account" class="block py-2" onclick={() => (menuOpen = false)}>{m.nav_account()} ({user.full_name})</a>
				<form method="POST" action="/auth?/logout">
					<button class="block py-2 text-red-600">{m.nav_logout()}</button>
				</form>
			{:else}
				<a href={loginHref} class="block py-2 font-semibold text-brand-600" onclick={() => (menuOpen = false)}>{m.nav_login()}</a>
			{/if}
			<form action="/search" method="get" class="py-2 sm:hidden">
				<input name="q" type="search" placeholder={m.search_placeholder()} class="w-full rounded-lg border border-slate-200 px-3 py-1.5 text-sm" />
			</form>
		</nav>
	{/if}
</header>
