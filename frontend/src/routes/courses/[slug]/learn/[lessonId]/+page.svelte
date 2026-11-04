<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { enhance } from '$app/forms';
	import LessonQA from '$lib/components/LessonQA.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatClock, formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const course = $derived(data.course);
	const lesson = $derived(data.lesson);
	const lessons = $derived(course.sections.flatMap((s) => s.lessons));
	const index = $derived(lessons.findIndex((l) => l.id === lesson.id));
	const prev = $derived(index > 0 ? lessons[index - 1] : null);
	const next = $derived(index >= 0 && index < lessons.length - 1 ? lessons[index + 1] : null);
	const done = $derived(new Set(course.completed_lesson_ids));
	const canWatch = (l: { is_preview: boolean }) => course.owned || l.is_preview;
	const href = (id: string) => `/courses/${course.slug}/learn/${id}`;
</script>

<svelte:head><title>{lesson.title} · {course.title} · Reanit Academy</title></svelte:head>

<a href="/courses/{course.slug}" class="text-sm text-brand-600 hover:underline">← {course.title}</a>

<div class="mt-3 grid gap-6 lg:grid-cols-[1fr_340px]">
	<div>
		<div class="aspect-video overflow-hidden rounded-xl bg-slate-900">
			{#if lesson.video_url && lesson.video_type === 'file'}
				{#key lesson.video_url}
					<!-- svelte-ignore a11y_media_has_caption -->
					<video src={lesson.video_url} title={lesson.title} class="h-full w-full" controls preload="metadata" playsinline
					></video>
				{/key}
			{:else if lesson.video_url}
				{#key lesson.video_url}
					<iframe
						src={lesson.video_url}
						title={lesson.title}
						class="h-full w-full"
						loading="lazy"
						allow="accelerometer; gyroscope; autoplay; encrypted-media; picture-in-picture"
						allowfullscreen
					></iframe>
				{/key}
			{:else}
				<div class="grid h-full place-items-center p-6 text-center text-slate-300">{m.lesson_no_video()}</div>
			{/if}
		</div>

		<div class="mt-4 flex flex-wrap items-start justify-between gap-4">
			<div>
				<p class="text-sm text-slate-500">{m.lesson_position({ n: formatNumber(index + 1), total: formatNumber(lessons.length) })}</p>
				<h1 class="text-2xl font-bold">{lesson.title}</h1>
			</div>
			<div class="flex flex-wrap gap-2">
				{#if course.owned}
					<form method="POST" action={lesson.completed ? '?/uncomplete' : '?/complete'} use:enhance>
						{#if lesson.completed}
							<button class="rounded-lg border border-green-600 px-4 py-2 text-sm text-green-700 hover:bg-green-50">✓ {m.lesson_completed()}</button>
						{:else}
							{#if next}<input type="hidden" name="next" value={next.id} />{/if}
							<button class="rounded-lg bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-700">
								{next ? m.lesson_complete_and_next() + ' →' : m.lesson_mark_complete()}
							</button>
						{/if}
					</form>
				{/if}
				{#if prev && canWatch(prev)}
					<a href={href(prev.id)} class="rounded-lg border border-slate-200 px-4 py-2 text-sm hover:border-brand-500">← {m.carousel_prev()}</a>
				{/if}
				{#if next && canWatch(next)}
					<a href={href(next.id)} class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700">{m.carousel_next()} →</a>
				{/if}
			</div>
		</div>

		{#if form?.code && !('qaForm' in form)}
			<p class="mt-3 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
		{/if}

		{#if !course.owned}
			<p class="mt-6 rounded-lg bg-brand-50 px-4 py-3 text-sm text-brand-700">
				{m.lesson_preview_notice()} <a href="/courses/{course.slug}" class="font-semibold underline">{m.lesson_buy_course()}</a> {m.lesson_buy_course_suffix()}
			</p>
		{/if}

		{#if data.qa}
			<LessonQA qa={data.qa} canAsk={course.owned || data.qa.can_answer} form={form && 'qaForm' in form ? form : null} />
		{/if}
	</div>

	<aside class="lg:sticky lg:top-24 lg:max-h-[calc(100vh-7rem)] lg:self-start lg:overflow-y-auto">
		{#if course.owned}
			<div class="mb-3 rounded-xl border border-slate-200 bg-white px-4 py-3">
				<p class="mb-1 text-sm font-semibold">{m.lesson_progress({ done: formatNumber(done.size), total: formatNumber(lessons.length) })}</p>
				<ProgressBar done={done.size} total={lessons.length} />
			</div>
		{/if}
		<div class="divide-y divide-slate-200 overflow-hidden rounded-xl border border-slate-200 bg-white">
			{#each course.sections as section (section.id)}
				<div>
					<p class="bg-slate-50 px-4 py-2.5 text-sm font-semibold">{section.title}</p>
					<ul>
						{#each section.lessons as l (l.id)}
							{@const current = l.id === lesson.id}
							<li>
								{#if canWatch(l)}
									<a
										href={href(l.id)}
										aria-current={current ? 'page' : undefined}
										class="flex items-center justify-between gap-2 px-4 py-2 text-sm hover:bg-slate-50 {current
											? 'bg-brand-50 font-semibold text-brand-700'
											: ''}"
									>
										<span class="flex items-center gap-2">
											{#if done.has(l.id)}<span class="text-green-600" aria-label={m.lesson_completed()}>✓</span>{:else}<span class="text-slate-400">▶</span>{/if}
											{l.title}
										</span>
										<span class="text-slate-400">{formatClock(l.duration_seconds)}</span>
									</a>
								{:else}
									<span class="flex items-center justify-between gap-2 px-4 py-2 text-sm text-slate-400">
										<span class="flex items-center gap-2"><span>🔒</span>{l.title}</span>
										<span>{formatClock(l.duration_seconds)}</span>
									</span>
								{/if}
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>
	</aside>
</div>
