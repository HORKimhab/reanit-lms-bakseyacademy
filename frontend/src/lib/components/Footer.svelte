<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { localized } from '$lib/i18n';
	import { site } from '$lib/site';

	const address = localized(site.address.km, site.address.en);

	const socials = [
		{ label: 'Facebook', href: site.social.facebook },
		{ label: 'Telegram', href: site.social.telegram },
		{ label: 'YouTube', href: site.social.youtube }
	].filter((s) => s.href);
</script>

<footer class="mt-16 border-t print:hidden border-slate-200 bg-white">
	<div class="mx-auto grid max-w-7xl gap-8 px-4 py-10 text-sm sm:grid-cols-2 lg:grid-cols-4">
		<div>
			<p class="text-lg font-bold text-brand-600">{site.name}</p>
			<p class="mt-2 text-slate-500">{localized(site.tagline.km, site.tagline.en)}</p>
		</div>
		<div>
			<p class="font-semibold">{m.footer_browse()}</p>
			<ul class="mt-2 space-y-1 text-slate-600">
				<li><a href="/courses" class="hover:text-brand-600">{m.nav_courses()}</a></li>
				<li><a href="/ebooks" class="hover:text-brand-600">{m.nav_ebooks()}</a></li>
			</ul>
		</div>
		<div>
			<p class="font-semibold">{m.footer_contact()}</p>
			<ul class="mt-2 space-y-1 text-slate-600">
				{#if address}
					<li>
						{#if site.mapUrl}
							<a href={site.mapUrl} target="_blank" rel="noopener" class="hover:text-brand-600">{address}</a>
						{:else}
							{address}
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
				<p class="font-semibold">{m.footer_follow()}</p>
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
