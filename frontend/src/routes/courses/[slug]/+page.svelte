<script lang="ts">
	import BuyButtons from '$lib/components/BuyButtons.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';
	import Price from '$lib/components/Price.svelte';
	import Rating from '$lib/components/Rating.svelte';
	import Reviews from '$lib/components/Reviews.svelte';
	import { formatClock, formatDuration, khmerNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const course = $derived(data.course);
</script>

<svelte:head>
	<title>{course.title} · Reanit Academy</title>
	{#if course.summary}<meta name="description" content={course.summary} />{/if}
</svelte:head>

<div class="grid gap-8 lg:grid-cols-[1fr_340px]">
	<div>
		{#if course.category_name_km}
			<a href="/courses?category={course.category_slug}" class="text-sm text-brand-600">#{course.category_name_km}</a>
		{/if}
		<h1 class="mt-1 text-3xl font-bold">{course.title}</h1>
		{#if course.summary}<p class="mt-2 text-lg text-slate-600">{course.summary}</p>{/if}
		<div class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-slate-600">
			<Rating avg={course.rating_avg} count={course.rating_count} />
			<span>គ្រូ៖ <strong>{course.instructor_name}</strong></span>
			<span>{khmerNumber(course.lesson_count)} មេរៀន · {formatDuration(course.total_duration_seconds)}</span>
		</div>

		{#if course.description}
			<h2 class="mt-10 text-xl font-bold">អំពីវគ្គសិក្សា</h2>
			<p class="mt-3 whitespace-pre-line text-slate-700">{course.description}</p>
		{/if}

		<h2 class="mt-10 text-xl font-bold">មាតិកាវគ្គសិក្សា</h2>
		<div class="mt-3 divide-y divide-slate-200 overflow-hidden rounded-xl border border-slate-200 bg-white">
			{#each course.sections as section, i (section.id)}
				<details open={i === 0}>
					<summary class="flex cursor-pointer items-center justify-between bg-slate-50 px-4 py-3 font-semibold">
						{section.title}
						<span class="text-sm font-normal text-slate-500">{khmerNumber(section.lessons.length)} មេរៀន</span>
					</summary>
					<ul>
						{#each section.lessons as lesson (lesson.id)}
							{@const watchable = course.owned || lesson.is_preview}
							<li>
								<svelte:element
									this={watchable ? 'a' : 'div'}
									href={watchable ? `/courses/${course.slug}/learn/${lesson.id}` : undefined}
									class="flex items-center justify-between px-4 py-2.5 text-sm {watchable ? 'hover:bg-slate-50' : ''}"
								>
									<span class="flex items-center gap-2">
										{#if course.completed_lesson_ids.includes(lesson.id)}
											<span class="text-green-600" aria-label="បានបញ្ចប់">✓</span>
										{:else}
											<span class="text-slate-400">{watchable ? '▶' : '🔒'}</span>
										{/if}
										{lesson.title}
										{#if lesson.is_preview && !course.owned}
											<span class="rounded bg-brand-50 px-1.5 text-xs text-brand-700">មើលសាកល្បង</span>
										{/if}
									</span>
									<span class="text-slate-400">{formatClock(lesson.duration_seconds)}</span>
								</svelte:element>
							</li>
						{/each}
					</ul>
				</details>
			{/each}
		</div>

		{#if course.instructor_bio}
			<h2 class="mt-10 text-xl font-bold">អំពីគ្រូ</h2>
			<div class="mt-3 flex gap-4">
				{#if course.instructor_avatar_url}
					<img src={course.instructor_avatar_url} alt={course.instructor_name} class="h-16 w-16 rounded-full object-cover" />
				{/if}
				<div>
					<p class="font-semibold">{course.instructor_name}</p>
					<p class="text-sm text-slate-600">{course.instructor_bio}</p>
				</div>
			</div>
		{/if}

		<Reviews reviews={course.reviews} myReview={course.my_review} owned={course.owned} {form} />
	</div>

	<aside class="lg:sticky lg:top-24 lg:self-start">
		<div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
			<div class="aspect-video bg-gradient-to-br from-brand-500 to-sky-400">
				{#if course.cover_url}<img src={course.cover_url} alt={course.title} class="h-full w-full object-cover" />{/if}
			</div>
			<div class="space-y-4 p-5">
				<Price price={course.price_cents} sale={course.sale_price_cents} size="lg" />
				{#if course.owned}
					<p class="rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700">✓ អ្នកបានចុះឈ្មោះក្នុងវគ្គនេះរួចហើយ</p>
					<ProgressBar done={course.completed_lesson_ids.length} total={course.lesson_count} />
					<a
						href="/courses/{course.slug}/learn"
						class="block rounded-lg bg-brand-600 px-6 py-2.5 text-center font-semibold text-white hover:bg-brand-700"
					>
						បន្តការសិក្សា
					</a>
				{:else}
					<BuyButtons {form} full />
				{/if}
			</div>
		</div>
	</aside>
</div>
