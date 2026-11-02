<script lang="ts">
	import { enhance } from '$app/forms';
	import CourseForm from '$lib/components/admin/CourseForm.svelte';
	import MediaField from '$lib/components/admin/MediaField.svelte';
	import MediaSearch from '$lib/components/admin/MediaSearch.svelte';
	import { confirmClick, dangerButton, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { errorMessage } from '$lib/errors';
	import { formatClock, formatDateTime, formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const course = $derived(data.course);
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>{course.title} · Admin</title></svelte:head>

<div class="flex flex-wrap items-start justify-between gap-4">
	<div>
		<a href="/admin/courses" class="text-sm text-brand-600 hover:underline">← វគ្គសិក្សា</a>
		<h1 class="text-2xl font-bold">{course.title}</h1>
		<p class="text-xs text-slate-500">កែចុងក្រោយ {formatDateTime(course.updated_at)}</p>
	</div>
	<div class="flex gap-2">
		{#if course.published}
			<a href="/courses/{course.slug}" target="_blank" class={secondaryButton}>មើលទំព័រ ↗</a>
		{/if}
		<form method="POST" action="?/delete" use:enhance>
			<button class={dangerButton} onclick={confirmClick('លុបវគ្គសិក្សានេះ? មិនអាចត្រឡប់វិញបានទេ។')}>លុប</button>
		</form>
	</div>
</div>

<div class="sticky top-16 z-10 mt-3 empty:hidden">
	{#if form?.code}
		<p class="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700 shadow-sm" role="alert">{errorMessage(form.code)}</p>
	{:else if form?.saved}
		<p class="rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700 shadow-sm" role="status">បានរក្សាទុក</p>
	{/if}
</div>

<section class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
	<CourseForm {course} instructors={data.instructors} categories={data.categories} action="?/save" />
</section>

<section class="mt-8">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<h2 class="text-xl font-bold">មាតិកា</h2>
		<MediaSearch q={data.mediaq} />
	</div>
	<p class="text-sm text-slate-500">
		Video ID = Bunny Stream video GUID ឬ ផ្ទុកឡើង Telegram (ឈ្នះលើ Video ID) · រយៈពេលជា នាទី:វិនាទី (ឧ. 12:30) · "មើលសាកល្បង" អនុញ្ញាតឲ្យអ្នកមិនទាន់ទិញមើលបាន
	</p>

	<div class="mt-4 space-y-4">
		{#each course.sections as section, si (section.id)}
			<div class="rounded-xl border border-slate-200 bg-white">
				<div class="flex flex-wrap items-center gap-2 border-b border-slate-100 bg-slate-50 px-4 py-2">
					<span class="text-sm text-slate-400">{formatNumber(si + 1)}.</span>
					<form method="POST" action="?/renameSection" use:enhance={keep} class="flex flex-1 gap-2">
						<input type="hidden" name="id" value={section.id} />
						<input name="title" required value={section.title} class="{inputClass} font-semibold" />
						<button class={secondaryButton}>រក្សាទុក</button>
					</form>
					<form method="POST" action="?/moveSection" use:enhance class="flex gap-1">
						<input type="hidden" name="id" value={section.id} />
						<button name="direction" value="up" disabled={si === 0} class="{secondaryButton} disabled:opacity-30" aria-label="ឡើងលើ">↑</button>
						<button
							name="direction"
							value="down"
							disabled={si === course.sections.length - 1}
							class="{secondaryButton} disabled:opacity-30"
							aria-label="ចុះក្រោម">↓</button
						>
					</form>
					<form method="POST" action="?/deleteSection" use:enhance>
						<input type="hidden" name="id" value={section.id} />
						<button class={dangerButton} onclick={confirmClick('លុបផ្នែកនេះ និងមេរៀនទាំងអស់ក្នុងនោះ?')}>លុប</button>
					</form>
				</div>

				<ul class="divide-y divide-slate-100">
					{#each section.lessons as lesson, li (lesson.id)}
						<li class="flex flex-wrap items-center gap-2 px-4 py-2">
							<form method="POST" action="?/updateLesson" use:enhance={keep} class="flex flex-1 flex-wrap items-center gap-2">
								<input type="hidden" name="id" value={lesson.id} />
								<input name="title" required value={lesson.title} class="{inputClass} min-w-48 flex-1" aria-label="ចំណងជើងមេរៀន" />
								<input name="video_id" value={lesson.video_id ?? ''} placeholder="Video ID" class="{inputClass} w-56 font-mono" aria-label="Video ID" />
								<MediaField mediaId={lesson.media_id} mediaName={lesson.media_file_name} accept="video/*" library={data.videos} />
								<input name="duration" value={formatClock(lesson.duration_seconds)} class="{inputClass} w-20" aria-label="រយៈពេល" />
								<label class="flex items-center gap-1 text-xs whitespace-nowrap">
									<input type="checkbox" name="is_preview" checked={lesson.is_preview} /> មើលសាកល្បង
								</label>
								<button class={secondaryButton}>រក្សាទុក</button>
							</form>
							<form method="POST" action="?/moveLesson" use:enhance class="flex gap-1">
								<input type="hidden" name="id" value={lesson.id} />
								<button name="direction" value="up" disabled={li === 0} class="{secondaryButton} disabled:opacity-30" aria-label="ឡើងលើ">↑</button>
								<button
									name="direction"
									value="down"
									disabled={li === section.lessons.length - 1}
									class="{secondaryButton} disabled:opacity-30"
									aria-label="ចុះក្រោម">↓</button
								>
							</form>
							<form method="POST" action="?/deleteLesson" use:enhance>
								<input type="hidden" name="id" value={lesson.id} />
								<button class={dangerButton} onclick={confirmClick('លុបមេរៀននេះ?')}>លុប</button>
							</form>
						</li>
					{/each}
				</ul>

				<form method="POST" action="?/addLesson" use:enhance class="flex flex-wrap items-center gap-2 border-t border-slate-100 px-4 py-2">
					<input type="hidden" name="id" value={section.id} />
					<input name="title" required placeholder="មេរៀនថ្មី" class="{inputClass} min-w-48 flex-1" />
					<input name="video_id" placeholder="Video ID" class="{inputClass} w-56 font-mono" />
					<MediaField accept="video/*" library={data.videos} />
					<input name="duration" placeholder="0:00" class="{inputClass} w-20" />
					<label class="flex items-center gap-1 text-xs whitespace-nowrap"><input type="checkbox" name="is_preview" /> មើលសាកល្បង</label>
					<button class={primaryButton}>+ បន្ថែមមេរៀន</button>
				</form>
			</div>
		{/each}
	</div>

	<form method="POST" action="?/addSection" use:enhance class="mt-4 flex gap-2">
		<input name="title" required placeholder="ចំណងជើងផ្នែកថ្មី" class="{inputClass} max-w-md" />
		<button class={primaryButton}>+ បន្ថែមផ្នែក</button>
	</form>
</section>
