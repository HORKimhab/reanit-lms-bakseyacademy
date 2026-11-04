<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages.js';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime } from '$lib/format';
	import type { LessonQuestions } from '$lib/types';

	let {
		qa,
		canAsk,
		form
	}: { qa: LessonQuestions; canAsk: boolean; form: { qaForm?: string; code?: string; saved?: boolean } | null | undefined } = $props();

	const error = (id: string) => (form?.qaForm === id && form.code ? errorMessage(form.code) : null);
	const confirmDelete = (e: MouseEvent) => {
		if (!confirm(m.qa_delete_confirm())) e.preventDefault();
	};
</script>

<section class="mt-10">
	<h2 class="text-xl font-bold">{m.qa_title()}</h2>

	{#if canAsk}
		<form method="POST" action="?/ask" use:enhance class="mt-3 space-y-2">
			<textarea name="body" required maxlength="2000" rows="3" placeholder={m.qa_ask_placeholder()} class="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm outline-none focus:border-brand-500"></textarea>
			{#if error('ask')}<p class="text-sm text-red-700" role="alert">{error('ask')}</p>{/if}
			<button class="rounded-lg bg-brand-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-brand-700">{m.qa_ask()}</button>
		</form>
	{/if}
	{#if error('delete')}<p class="mt-2 text-sm text-red-700" role="alert">{error('delete')}</p>{/if}

	{#if qa.questions.length === 0}
		<p class="mt-4 text-sm text-slate-500">{m.qa_empty()}</p>
	{:else}
		<ul class="mt-4 space-y-4">
			{#each qa.questions as q (q.id)}
				<li class="rounded-xl border border-slate-200 bg-white p-4">
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0">
							<p class="text-sm"><strong>{q.user_name}</strong> <span class="text-xs text-slate-400">· {formatDateTime(q.created_at)}</span></p>
							<p class="mt-1 whitespace-pre-line text-slate-800">{q.body}</p>
						</div>
						{#if q.mine || qa.can_answer}
							<form method="POST" action="?/deleteQuestion" use:enhance>
								<input type="hidden" name="id" value={q.id} />
								<button class="text-xs text-red-600 hover:underline" onclick={confirmDelete}>{m.qa_delete()}</button>
							</form>
						{/if}
					</div>
					{#if q.answers.length}
						<ul class="mt-3 space-y-3 border-l-2 border-brand-200 pl-4">
							{#each q.answers as a (a.id)}
								<li class="flex items-start justify-between gap-3">
									<div class="min-w-0">
										<p class="text-sm">
											<strong>{a.user_name}</strong>
											<span class="ml-1 rounded bg-brand-50 px-1.5 text-xs text-brand-700">{m.qa_staff()}</span>
											<span class="text-xs text-slate-400">· {formatDateTime(a.created_at)}</span>
										</p>
										<p class="mt-1 whitespace-pre-line text-slate-700">{a.body}</p>
									</div>
									{#if a.mine || qa.can_answer}
										<form method="POST" action="?/deleteAnswer" use:enhance>
											<input type="hidden" name="id" value={a.id} />
											<button class="text-xs text-red-600 hover:underline" onclick={confirmDelete}>{m.qa_delete()}</button>
										</form>
									{/if}
								</li>
							{/each}
						</ul>
					{/if}
					{#if qa.can_answer}
						<form method="POST" action="?/answer" use:enhance class="mt-3 flex gap-2">
							<input type="hidden" name="id" value={q.id} />
							<input name="body" required maxlength="4000" placeholder={m.qa_answer_placeholder()} class="min-w-0 flex-1 rounded-lg border border-slate-200 px-3 py-1.5 text-sm outline-none focus:border-brand-500" />
							<button class="rounded-lg border border-brand-600 px-3 py-1.5 text-sm font-semibold text-brand-600 hover:bg-brand-50">{m.qa_answer()}</button>
						</form>
						{#if error(q.id)}<p class="mt-1 text-sm text-red-700" role="alert">{error(q.id)}</p>{/if}
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</section>
