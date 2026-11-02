<script lang="ts">
	import { errorMessage } from '$lib/errors';
	import { formatClock } from '$lib/format';
	import type { AdminMedia } from '$lib/types';

	// Hidden `media_id` input + "upload to Telegram" button with progress. A video upload also fills
	// the surrounding form's `duration` field when it is still empty.
	let {
		mediaId = null,
		mediaName = null,
		accept = ''
	}: { mediaId?: string | null; mediaName?: string | null; accept?: string } = $props();

	// Follows the saved value from the server; uploads / "remove" override it until the next save.
	let id = $derived(mediaId);
	let name = $derived(mediaName);
	let progress: number | null = $state(null);
	let failure = $state('');
	let fileInput: HTMLInputElement | undefined = $state();

	function upload() {
		const file = fileInput?.files?.[0];
		if (!file) return;
		const body = new FormData();
		body.append('size', String(file.size)); // must precede the file: the API streams it to Telegram
		body.append('file', file);
		const xhr = new XMLHttpRequest();
		xhr.open('POST', '/admin/media/upload');
		xhr.upload.onprogress = (e) => (progress = e.lengthComputable ? Math.round((e.loaded / e.total) * 100) : null);
		xhr.onload = () => {
			progress = null;
			const res = JSON.parse(xhr.responseText || '{}');
			if (xhr.status === 201) {
				const media = res as AdminMedia;
				id = media.id;
				name = media.file_name;
				failure = '';
				const duration = fileInput?.form?.elements.namedItem('duration');
				if (media.duration_seconds && duration instanceof HTMLInputElement && (!duration.value || duration.value === '0:00')) {
					duration.value = formatClock(media.duration_seconds);
				}
			} else {
				failure = errorMessage(res.error ?? 'unknown');
			}
			if (fileInput) fileInput.value = '';
		};
		xhr.onerror = () => {
			progress = null;
			failure = errorMessage('unavailable');
		};
		progress = 0;
		failure = '';
		xhr.send(body);
	}
</script>

<span class="inline-flex flex-wrap items-center gap-2 text-xs">
	<input type="hidden" name="media_id" value={id ?? ''} />
	{#if id}
		<span class="rounded bg-sky-50 px-2 py-0.5 text-sky-700" title="Telegram">✈ {name ?? 'file'}</span>
		<button type="button" class="text-red-600 hover:underline" onclick={() => ((id = null), (name = null))}>ដកចេញ</button>
	{/if}
	{#if progress !== null}
		<span class="w-24 overflow-hidden rounded-full bg-slate-200" role="progressbar" aria-valuenow={progress}>
			<span class="block h-1.5 bg-sky-600" style="width: {progress}%"></span>
		</span>
		<span>{progress}%</span>
	{:else}
		<label class="cursor-pointer rounded border border-slate-200 px-2 py-0.5 hover:border-brand-500">
			{id ? 'ប្តូរឯកសារ' : 'ផ្ទុកឡើង Telegram'}
			<input bind:this={fileInput} type="file" {accept} class="sr-only" onchange={upload} />
		</label>
	{/if}
	{#if failure}<span class="text-red-600" role="alert">{failure}</span>{/if}
</span>
