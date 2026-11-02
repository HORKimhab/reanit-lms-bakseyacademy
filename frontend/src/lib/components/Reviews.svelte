<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { enhance } from '$app/forms';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime, formatNumber } from '$lib/format';
	import type { Review } from '$lib/types';

	let {
		reviews,
		myReview,
		owned,
		form
	}: {
		reviews: Review[];
		myReview: Review | null;
		owned: boolean;
		form: { reviewCode?: string; reviewSaved?: boolean; reviewDeleted?: boolean } | null | undefined;
	} = $props();

	let editing = $state(false);
	const showForm = $derived(owned && (!myReview || editing));
	const stars = (n: number) => '★'.repeat(n) + '☆'.repeat(5 - n);
</script>

<section class="mt-10">
	<h2 class="text-xl font-bold">{m.reviews_title()}</h2>

	{#if form?.reviewSaved}
		<p class="mt-3 text-sm text-green-700" role="status">{m.reviews_thanks()}</p>
	{:else if form?.reviewDeleted}
		<p class="mt-3 text-sm text-slate-600" role="status">{m.reviews_deleted()}</p>
	{:else if form?.reviewCode}
		<p class="mt-3 text-sm text-red-700" role="alert">{errorMessage(form.reviewCode)}</p>
	{/if}

	{#if showForm}
		<form
			method="POST"
			action="?/review"
			use:enhance={() =>
				async ({ update }) => {
					await update({ reset: false });
					editing = false;
				}}
			class="mt-4 space-y-3 rounded-xl border border-slate-200 bg-white p-4"
		>
			<fieldset>
				<legend class="text-sm font-semibold">{m.reviews_your_rating()}</legend>
				<div class="mt-1 flex flex-row-reverse justify-end gap-1 text-2xl">
					{#each [5, 4, 3, 2, 1] as n (n)}
						<input
							type="radio"
							name="rating"
							value={n}
							id="rating-{n}"
							required
							checked={myReview?.rating === n}
							class="peer sr-only"
						/>
						<label
							for="rating-{n}"
							class="cursor-pointer text-slate-300 peer-checked:text-amber-400 hover:text-amber-400"
							title={m.stars_count({ count: n, n: formatNumber(n) })}>★</label
						>
					{/each}
				</div>
			</fieldset>
			<textarea
				name="comment"
				rows="3"
				maxlength="2000"
				placeholder={m.reviews_comment_placeholder()}
				class="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm outline-none focus:border-brand-500"
				>{myReview?.comment ?? ''}</textarea
			>
			<div class="flex gap-2">
				<button class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700">{m.save()}</button>
				{#if editing}
					<button type="button" onclick={() => (editing = false)} class="rounded-lg px-4 py-2 text-sm text-slate-600 hover:bg-slate-100">
						{m.cancel()}
					</button>
				{/if}
			</div>
		</form>
	{:else if myReview}
		<div class="mt-4 flex flex-wrap items-center gap-3 rounded-xl border border-brand-100 bg-brand-50 px-4 py-3 text-sm">
			<span>{m.reviews_yours()} <span class="text-amber-500">{stars(myReview.rating)}</span></span>
			<button onclick={() => (editing = true)} class="text-brand-700 underline">{m.edit()}</button>
			<form method="POST" action="?/deleteReview" use:enhance>
				<button class="text-red-600 underline">{m.remove()}</button>
			</form>
		</div>
	{/if}

	{#if reviews.length === 0}
		<p class="mt-4 text-sm text-slate-500">{m.reviews_none()}</p>
	{:else}
		<ul class="mt-4 space-y-4">
			{#each reviews as r (r.id)}
				<li class="border-b border-slate-100 pb-4">
					<div class="flex flex-wrap items-center gap-x-3 text-sm">
						<strong>{r.user_name}</strong>
						<span class="text-amber-500" aria-label={m.stars_count({ count: r.rating, n: formatNumber(r.rating) })}>{stars(r.rating)}</span>
						<span class="text-xs text-slate-400">{formatDateTime(r.updated_at)}</span>
					</div>
					{#if r.comment}<p class="mt-1 whitespace-pre-line text-slate-700">{r.comment}</p>{/if}
				</li>
			{/each}
		</ul>
	{/if}
</section>
