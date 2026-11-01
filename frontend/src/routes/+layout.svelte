<script lang="ts">
	import '../app.css';
	import Footer from '$lib/components/Footer.svelte';
	import Header from '$lib/components/Header.svelte';

	let { children } = $props();
</script>

<Header />
<main class="mx-auto min-h-[60vh] max-w-7xl px-4 py-8">
	{@render children()}
</main>
<Footer />
