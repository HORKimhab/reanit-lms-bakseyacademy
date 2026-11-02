<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { page } from '$app/state';
</script>

<div class="py-20 text-center">
	<p class="text-5xl font-bold text-brand-600">{page.status}</p>
	<p class="mt-4 text-slate-600">{page.error?.message}</p>
	<a href="/" class="mt-6 inline-block text-brand-600 hover:underline">← {m.error_back_home()}</a>
</div>
