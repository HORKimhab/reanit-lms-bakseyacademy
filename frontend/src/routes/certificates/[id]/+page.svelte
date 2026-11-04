<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import Seo from '$lib/components/Seo.svelte';
	import { formatDate } from '$lib/format';
	import { site } from '$lib/site';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const c = $derived(data.certificate);
</script>

<Seo title={m.certificate_page_title({ name: c.full_name, course: c.course_title })} />

<div class="mb-4 flex flex-wrap items-center justify-between gap-3 print:hidden">
	<p class="text-sm text-slate-500">{m.certificate_print_hint()}</p>
	<button onclick={() => window.print()} class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700">
		{m.certificate_print()}
	</button>
</div>

<article class="certificate mx-auto aspect-[1.414] w-full max-w-4xl bg-white p-3 shadow-lg print:shadow-none">
	<div class="flex h-full flex-col items-center justify-center border-4 border-double border-brand-600 px-8 py-6 text-center">
		<p class="text-sm font-semibold tracking-widest text-brand-600 uppercase">{site.name}</p>
		<h1 class="mt-4 text-3xl font-bold sm:text-4xl">{m.certificate_heading()}</h1>
		<p class="mt-6 text-slate-600">{m.certificate_presented_to()}</p>
		<p class="mt-2 border-b border-slate-300 px-8 pb-2 text-3xl font-semibold sm:text-4xl">{c.full_name}</p>
		<p class="mt-6 text-slate-600">{m.certificate_for_completing()}</p>
		<p class="mt-2 text-2xl font-bold text-brand-700">{c.course_title}</p>
		<div class="mt-10 grid w-full max-w-xl grid-cols-2 gap-8 text-sm">
			<div>
				<p class="border-t border-slate-400 pt-2 font-semibold">{c.instructor_name}</p>
				<p class="text-slate-500">{m.course_instructor_label()}</p>
			</div>
			<div>
				<p class="border-t border-slate-400 pt-2 font-semibold">{formatDate(c.issued_at)}</p>
				<p class="text-slate-500">{m.certificate_issued_on()}</p>
			</div>
		</div>
		<p class="mt-8 font-mono text-xs text-slate-400">{m.certificate_id_label()} {c.id}</p>
	</div>
</article>

<p class="mt-6 text-center text-sm print:hidden">
	<a href="/courses/{c.course_slug}" class="text-brand-600 hover:underline">{c.course_title}</a>
</p>

<style>
	@media print {
		@page {
			size: A4 landscape;
			margin: 0;
		}
		:global(main) {
			padding: 0 !important;
			max-width: none !important;
		}
		.certificate {
			max-width: none;
			width: 297mm;
			height: 210mm;
			aspect-ratio: auto;
			-webkit-print-color-adjust: exact;
			print-color-adjust: exact;
		}
	}
</style>
