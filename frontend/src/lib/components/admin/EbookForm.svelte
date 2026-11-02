<script lang="ts">
	import { enhance } from '$app/forms';
	import { dollarsInput, inputClass, labelClass, primaryButton } from '$lib/admin-ui';
	import MediaField from './MediaField.svelte';
	import type { AdminCategory, AdminEbook, AdminMedia } from '$lib/types';

	let {
		ebook = null,
		categories,
		library = [],
		action
	}: {
		ebook?: AdminEbook | null;
		categories: AdminCategory[];
		library?: AdminMedia[];
		action: string;
	} = $props();

	const pricePattern = '\\d+(\\.\\d{1,2})?';
</script>

<form method="POST" {action} use:enhance={() => ({ update }) => update({ reset: false })} class="grid gap-4 md:grid-cols-2">
	<label class="md:col-span-2">
		<span class={labelClass}>ចំណងជើង *</span>
		<input name="title" required maxlength="200" value={ebook?.title ?? ''} class={inputClass} />
	</label>
	<label>
		<span class={labelClass}>Slug (URL) *</span>
		<input
			name="slug"
			required
			maxlength="100"
			pattern="[a-z0-9]+(-[a-z0-9]+)*"
			value={ebook?.slug ?? ''}
			class="{inputClass} font-mono"
		/>
	</label>
	<label>
		<span class={labelClass}>អ្នកនិពន្ធ *</span>
		<input name="author_name" required maxlength="100" value={ebook?.author_name ?? ''} class={inputClass} />
	</label>
	<label>
		<span class={labelClass}>រូបភាពគម្រប (URL)</span>
		<input name="cover_url" value={ebook?.cover_url ?? ''} placeholder="https://… ឬ /images/…" class={inputClass} />
	</label>
	<label>
		<span class={labelClass}>ប្រភេទ</span>
		<select name="category_id" class={inputClass}>
			<option value="">— គ្មាន —</option>
			{#each categories as c (c.id)}
				<option value={c.id} selected={ebook?.category_id === c.id}>{c.name_km}</option>
			{/each}
		</select>
	</label>
	<label>
		<span class={labelClass}>តម្លៃ (USD) *</span>
		<input
			name="price"
			required
			inputmode="decimal"
			pattern={pricePattern}
			value={dollarsInput(ebook?.price_cents)}
			placeholder="10.00"
			class={inputClass}
		/>
	</label>
	<label>
		<span class={labelClass}>តម្លៃបញ្ចុះ (USD)</span>
		<input
			name="sale_price"
			inputmode="decimal"
			pattern={pricePattern}
			value={dollarsInput(ebook?.sale_price_cents)}
			placeholder="ទុកទទេ = គ្មាន"
			class={inputClass}
		/>
	</label>
	<label>
		<span class={labelClass}>ចំនួនទំព័រ</span>
		<input name="page_count" inputmode="numeric" pattern="\d+" value={ebook?.page_count ?? ''} class={inputClass} />
	</label>
	<div>
		<label>
			<span class={labelClass}>File key (S3/R2)</span>
			<input
				name="file_key"
				placeholder={ebook?.has_file ? 'ទុកទទេ = រក្សាឯកសារបច្ចុប្បន្ន' : 'ឧ. ebooks/my-book.pdf'}
				class="{inputClass} font-mono"
			/>
		</label>
		<p class="mt-1"><MediaField mediaId={ebook?.media_id} mediaName={ebook?.media_file_name} accept=".pdf,.epub,application/pdf,application/epub+zip" {library} /></p>
		{#if ebook}
			<p class="mt-1 flex items-center gap-3 text-xs {ebook.has_file ? 'text-green-700' : 'text-amber-700'}">
				{ebook.has_file ? '✓ មានឯកសារ' : 'មិនទាន់មានឯកសារ'}
				{#if ebook.has_file}
					<label class="flex items-center gap-1 text-red-600"><input type="checkbox" name="remove_file" /> ដកឯកសារចេញ</label>
				{/if}
			</p>
		{/if}
	</div>
	<label class="md:col-span-2">
		<span class={labelClass}>សេចក្តីសង្ខេប</span>
		<input name="summary" maxlength="500" value={ebook?.summary ?? ''} class={inputClass} />
	</label>
	<label class="md:col-span-2">
		<span class={labelClass}>ការពិពណ៌នា</span>
		<textarea name="description" rows="6" class={inputClass}>{ebook?.description ?? ''}</textarea>
	</label>
	<label class="flex items-center gap-2 text-sm">
		<input type="checkbox" name="published" checked={ebook?.published ?? false} />
		ផ្សាយជាសាធារណៈ
	</label>
	<div class="md:text-right">
		<button class={primaryButton}>{ebook ? 'រក្សាទុក' : 'បង្កើត'}</button>
	</div>
</form>
