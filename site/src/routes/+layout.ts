import { dev } from '$app/env';
import { inject } from '@vercel/analytics';

export const prerender = true;

// `@vercel/analytics/sveltekit` subscribes to `$app/stores`, which throws in SvelteKit 3, so this
// passes the same Vercel build settings to `inject()` directly. The script tracks page views itself.
inject(
	{
		mode: dev ? 'development' : 'production',
		framework: 'sveltekit',
		basePath: import.meta.env.VITE_VERCEL_OBSERVABILITY_BASEPATH
	},
	import.meta.env.VITE_VERCEL_OBSERVABILITY_CLIENT_CONFIG
);
