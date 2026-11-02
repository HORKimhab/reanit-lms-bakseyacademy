<script lang="ts">
	import ProgressBar from '$lib/components/ProgressBar.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const { courses, ebooks } = $derived(data.library);
</script>

<svelte:head><title>បណ្ណាល័យរបស់ខ្ញុំ · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">បណ្ណាល័យរបស់ខ្ញុំ</h1>

{#if courses.length === 0 && ebooks.length === 0}
	<p class="mt-12 text-center text-slate-500">
		អ្នកមិនទាន់មានវគ្គសិក្សា ឬសៀវភៅនៅឡើយទេ។ <a href="/courses" class="text-brand-600 hover:underline">ស្វែងរកវគ្គសិក្សា</a>
	</p>
{/if}

{#if courses.length}
	<h2 class="mt-8 text-xl font-bold">វគ្គសិក្សារបស់ខ្ញុំ</h2>
	<div class="mt-4 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each courses as c (c.id)}
			<a href="/courses/{c.slug}/learn" class="group overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm hover:shadow-md">
				<div class="aspect-video bg-gradient-to-br from-brand-500 to-sky-400">
					{#if c.cover_url}<img src={c.cover_url} alt="" class="h-full w-full object-cover" />{/if}
				</div>
				<div class="p-4">
					<h3 class="line-clamp-2 font-semibold group-hover:text-brand-600">{c.title}</h3>
					<p class="text-sm text-slate-500">{c.instructor_name}</p>
					<div class="mt-2"><ProgressBar done={c.completed_lessons} total={c.lesson_count} /></div>
				</div>
			</a>
		{/each}
	</div>
{/if}

{#if ebooks.length}
	<h2 class="mt-10 text-xl font-bold">សៀវភៅរបស់ខ្ញុំ</h2>
	<div class="mt-4 grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each ebooks as e (e.id)}
			<a href="/ebooks/{e.slug}" class="group overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm hover:shadow-md">
				<div class="aspect-[3/4] bg-gradient-to-br from-brand-500 to-sky-400">
					{#if e.cover_url}<img src={e.cover_url} alt="" class="h-full w-full object-cover" />{/if}
				</div>
				<div class="p-3">
					<h3 class="line-clamp-2 text-sm font-semibold group-hover:text-brand-600">{e.title}</h3>
					<p class="text-xs text-slate-500">{e.author_name}</p>
				</div>
			</a>
		{/each}
	</div>
{/if}
