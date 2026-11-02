import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		// Khmer unless the visitor picked English (PARAGLIDE_LOCALE cookie, set by the header switcher).
		paraglideVitePlugin({ project: './project.inlang', outdir: './src/lib/paraglide', strategy: ['cookie', 'baseLocale'] })
	]
});
