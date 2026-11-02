<script lang="ts">
	import { site } from '$lib/site';

	const socials = [
		{ label: 'Facebook', href: site.social.facebook },
		{ label: 'Telegram', href: site.social.telegram },
		{ label: 'YouTube', href: site.social.youtube }
	].filter((s) => s.href);
</script>

<footer class="mt-16 border-t border-slate-200 bg-white">
	<div class="mx-auto grid max-w-7xl gap-8 px-4 py-10 text-sm sm:grid-cols-2 lg:grid-cols-4">
		<div>
			<p class="text-lg font-bold text-brand-600">{site.name}</p>
			<p class="mt-2 text-slate-500">{site.tagline}</p>
		</div>
		<div>
			<p class="font-semibold">ប្រភេទ</p>
			<ul class="mt-2 space-y-1 text-slate-600">
				<li><a href="/courses" class="hover:text-brand-600">វគ្គសិក្សា</a></li>
				<li><a href="/ebooks" class="hover:text-brand-600">សៀវភៅ</a></li>
			</ul>
		</div>
		<div>
			<p class="font-semibold">ទំនាក់ទំនង</p>
			<ul class="mt-2 space-y-1 text-slate-600">
				{#if site.address}
					<li>
						{#if site.mapUrl}
							<a href={site.mapUrl} target="_blank" rel="noopener" class="hover:text-brand-600">{site.address}</a>
						{:else}
							{site.address}
						{/if}
					</li>
				{/if}
				{#each site.phones as phone (phone)}
					<li><a href="tel:{phone.replace(/\s/g, '')}" class="hover:text-brand-600">{phone}</a></li>
				{/each}
				{#if site.email}
					<li><a href="mailto:{site.email}" class="hover:text-brand-600">{site.email}</a></li>
				{/if}
			</ul>
		</div>
		{#if socials.length}
			<div>
				<p class="font-semibold">តាមដានយើង</p>
				<ul class="mt-2 space-y-1 text-slate-600">
					{#each socials as s (s.label)}
						<li><a href={s.href} target="_blank" rel="noopener" class="hover:text-brand-600">{s.label}</a></li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
	<p class="border-t border-slate-100 py-4 text-center text-xs text-slate-400">© {new Date().getFullYear()} {site.name}</p>
</footer>
