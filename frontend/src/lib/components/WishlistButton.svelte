<script lang="ts">
	// Heart toggle for a course / ebook. Posts to the wishlist page's action, which redirects back;
	// with JS the redirect is skipped and page data refreshed in place.
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages.js';

	let { kind, id, overlay = false }: { kind: 'course' | 'ebook'; id: string; overlay?: boolean } = $props();

	const here = $derived(page.url.pathname + page.url.search);
	const saved = $derived(
		(kind === 'course' ? page.data.wishlist?.course_ids : page.data.wishlist?.ebook_ids)?.includes(id) ?? false
	);
	const label = $derived(saved ? m.wishlist_remove() : m.wishlist_add());
	const classes = $derived(
		overlay
			? 'grid h-9 w-9 place-items-center rounded-full bg-white/90 shadow hover:bg-white'
			: 'flex items-center gap-2 rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium hover:bg-slate-50'
	);
</script>

{#snippet heart()}
	<svg viewBox="0 0 24 24" class="h-5 w-5 {saved ? 'fill-red-500 text-red-500' : 'fill-none text-slate-600'}" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
		<path d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10Z" stroke-linejoin="round" />
	</svg>
{/snippet}

{#if page.data.user}
	<form
		method="POST"
		action="/account/wishlist?/toggle"
		use:enhance={() => async ({ result, update }) => {
			if (result.type === 'redirect') await invalidateAll();
			else await update();
		}}
	>
		<input type="hidden" name="kind" value={kind} />
		<input type="hidden" name="id" value={id} />
		<input type="hidden" name="saved" value={saved ? '1' : ''} />
		<input type="hidden" name="redirectTo" value={here} />
		<button class={classes} aria-label={label} aria-pressed={saved} title={label}>
			{@render heart()}
			{#if !overlay}<span>{label}</span>{/if}
		</button>
	</form>
{:else}
	<a href="/auth?redirectTo={encodeURIComponent(here)}" class={classes} aria-label={m.wishlist_add()} title={m.wishlist_add()}>
		{@render heart()}
		{#if !overlay}<span>{m.wishlist_add()}</span>{/if}
	</a>
{/if}
