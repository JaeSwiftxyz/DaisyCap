<script lang="ts" module>
	import { wallpaper, type WallpaperId } from './wallpaper.js';

	const idle = (run: () => void) =>
		'requestIdleCallback' in window ? requestIdleCallback(run, { timeout: 400 }) : setTimeout(run, 1);

	/** Paints a wallpaper into the cache during idle time, so switching to it later is instant. */
	export function prewarm(ids: WallpaperId[], width: number, height: number) {
		const next = () => {
			const id = ids.shift();
			if (!id) return;
			wallpaper(id, width, height);
			idle(next);
		};
		idle(next);
	}
</script>

<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import { fallback } from './wallpaper.js';

	/** `width`×`height` is the paint resolution; the canvas covers its container at any size. */
	let { id, width, height }: { id: WallpaperId; width: number; height: number } = $props();

	let painted = $state(false);
	/** Set once the canvas has been near the viewport; later wallpaper changes paint right away. */
	let seen = false;

	const draw: Attachment<HTMLCanvasElement> = (canvas) => {
		const current = id;
		const paint = () => {
			canvas.getContext('2d')?.putImageData(wallpaper(current, width, height), 0, 0);
			seen = true;
			painted = true;
		};
		if (seen) {
			paint();
			return;
		}
		let handle: ReturnType<typeof idle> | undefined;
		const observer = new IntersectionObserver(
			([entry]) => {
				if (!entry.isIntersecting) return;
				observer.disconnect();
				handle = idle(paint);
			},
			{ rootMargin: '400px' }
		);
		observer.observe(canvas);
		return () => {
			observer.disconnect();
			if (handle === undefined) return;
			if ('cancelIdleCallback' in window) cancelIdleCallback(handle as number);
			else clearTimeout(handle);
		};
	};
</script>

<canvas {width} {height} style:background={painted ? null : fallback(id)} {@attach draw}></canvas>

<style>
	canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
</style>
