<script lang="ts">
	import { enhance } from '$app/forms';
	import { dollarsInput, inputClass, labelClass, primaryButton } from '$lib/admin-ui';
	import type { AdminCategory, AdminCourse, AdminInstructor } from '$lib/types';

	let {
		course = null,
		instructors,
		categories,
		action
	}: {
		course?: AdminCourse | null;
		instructors: AdminInstructor[];
		categories: AdminCategory[];
		action: string;
	} = $props();

	const pricePattern = '\\d+(\\.\\d{1,2})?';
</script>

<form method="POST" {action} use:enhance={() => ({ update }) => update({ reset: false })} class="grid gap-4 md:grid-cols-2">
	<label class="md:col-span-2">
		<span class={labelClass}>ចំណងជើង *</span>
		<input name="title" required maxlength="200" value={course?.title ?? ''} class={inputClass} />
	</label>
	<label>
		<span class={labelClass}>Slug (URL) *</span>
		<input
			name="slug"
			required
			maxlength="100"
			pattern="[a-z0-9]+(-[a-z0-9]+)*"
			value={course?.slug ?? ''}
			class="{inputClass} font-mono"
		/>
	</label>
	<label>
		<span class={labelClass}>រូបភាពគម្រប (URL)</span>
		<input name="cover_url" value={course?.cover_url ?? ''} placeholder="https://… ឬ /images/…" class={inputClass} />
	</label>
	<label>
		<span class={labelClass}>គ្រូ *</span>
		<select name="instructor_id" required class={inputClass}>
			<option value="">— ជ្រើសរើស —</option>
			{#each instructors as i (i.id)}
				<option value={i.id} selected={course?.instructor_id === i.id}>{i.name}</option>
			{/each}
		</select>
	</label>
	<label>
		<span class={labelClass}>ប្រភេទ</span>
		<select name="category_id" class={inputClass}>
			<option value="">— គ្មាន —</option>
			{#each categories as c (c.id)}
				<option value={c.id} selected={course?.category_id === c.id}>{c.name_km}</option>
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
			value={dollarsInput(course?.price_cents)}
			placeholder="30.00"
			class={inputClass}
		/>
	</label>
	<label>
		<span class={labelClass}>តម្លៃបញ្ចុះ (USD)</span>
		<input
			name="sale_price"
			inputmode="decimal"
			pattern={pricePattern}
			value={dollarsInput(course?.sale_price_cents)}
			placeholder="ទុកទទេ = គ្មាន"
			class={inputClass}
		/>
	</label>
	<label class="md:col-span-2">
		<span class={labelClass}>សេចក្តីសង្ខេប</span>
		<input name="summary" maxlength="500" value={course?.summary ?? ''} class={inputClass} />
	</label>
	<label class="md:col-span-2">
		<span class={labelClass}>ការពិពណ៌នា</span>
		<textarea name="description" rows="6" class={inputClass}>{course?.description ?? ''}</textarea>
	</label>
	<label class="flex items-center gap-2 text-sm">
		<input type="checkbox" name="published" checked={course?.published ?? false} />
		ផ្សាយជាសាធារណៈ
	</label>
	<div class="md:text-right">
		<button class={primaryButton}>{course ? 'រក្សាទុក' : 'បង្កើត'}</button>
	</div>
</form>
