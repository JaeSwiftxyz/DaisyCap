import { dmg } from '#lib/site.js';
import type { RequestHandler } from './$types';

export const prerender = false;

export const GET: RequestHandler = () =>
	new Response(null, { status: 302, headers: { location: dmg, 'cache-control': 'no-store' } });
