<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="py-20 text-center">
	<p class="text-5xl font-bold text-brand-600">{page.status}</p>
	<p class="mt-4 text-slate-600">{page.error?.message}</p>
	<a href="/" class="mt-6 inline-block text-brand-600 hover:underline">← ត្រឡប់ទៅទំព័រដើម</a>
</div>
