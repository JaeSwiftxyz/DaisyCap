<script lang="ts">
	import { span } from '#lib/ease.js';
	import { loop } from '#lib/motion.svelte.js';
	import Canvas from '#lib/ui/Canvas.svelte';
	import Segmented from '#lib/ui/Segmented.svelte';
	import Slider from '#lib/ui/Slider.svelte';
	import Swatches from '#lib/ui/Swatches.svelte';
	import type { WallpaperId } from '#lib/ui/wallpaper.js';
	import Fit from './Fit.svelte';

	const LOOKS: { wallpaper: WallpaperId; padding: number; roundness: number }[] = [
		{ wallpaper: 'grape', padding: 13, roundness: 26 },
		{ wallpaper: 'ocean', padding: 8, roundness: 16 },
		{ wallpaper: 'lagoon', padding: 16, roundness: 36 },
		{ wallpaper: 'aurora', padding: 11, roundness: 22 },
		{ wallpaper: 'graphite', padding: 6, roundness: 12 },
		{ wallpaper: 'mint', padding: 14, roundness: 30 }
	];
	const STEP = 1700;

	const clock = loop(STEP * LOOKS.length, 1200);
	let t = $derived(clock.t);
	let i = $derived(Math.floor(t / STEP));
	let look = $derived(LOOKS[i]);
	let before = $derived(LOOKS[(i + LOOKS.length - 1) % LOOKS.length]);
	let p = $derived(span(t - i * STEP, 0, 420));
	let ease = $derived(1 - (1 - p) ** 3);
	let padding = $derived(before.padding + (look.padding - before.padding) * ease);
	let roundness = $derived(before.roundness + (look.roundness - before.roundness) * ease);
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={304}>
		<div class="canvas">
			<Canvas
				width={290}
				height={202}
				wallpaper={look.wallpaper}
				from={before.wallpaper}
				fade={p}
				resolution={[480, 320]}
				{padding}
				{roundness}
			/>
		</div>
		<div class="panel">
			<h5>Background</h5>
			<Segmented items={['Wallpaper', 'Gradient', 'Color', 'Image']} selected={0} />
			<div class="swatches">
				<Swatches selected={look.wallpaper} width={34} height={22} gap={6} />
			</div>
			<Slider label="Padding" value={padding / 40} text="{padding.toFixed(1)}%" />
			<Slider label="Roundness" value={roundness / 60} text="{Math.round(roundness)} px" />
		</div>
	</Fit>
</div>

<style>
	.card-visual {
		background: radial-gradient(ellipse 70% 60% at 50% 0%, rgba(110, 150, 255, 0.08), transparent);
	}

	.canvas {
		position: absolute;
		top: 51px;
		left: 22px;
		overflow: hidden;
		border-radius: 8px;
		box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
	}

	.panel {
		position: absolute;
		top: 58px;
		right: 20px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		width: 256px;
		padding: 10px 12px 12px;
		border-radius: 12px;
		background: var(--inspector);
		box-shadow:
			0 0 0 1px var(--control-edge),
			0 20px 50px var(--drop);
		color: var(--text);
	}

	.panel > :global(.segmented) {
		align-self: flex-start;
	}

	.panel > :global(.segmented span) {
		padding-inline: 6px;
	}

	h5 {
		margin: 0;
		color: var(--secondary);
		font-size: 12px;
		font-weight: 600;
	}

	.swatches {
		margin: 2px 0 4px;
	}

	.panel :global(.slider) {
		width: 72px;
	}

	.panel :global(.label) {
		font-size: 12px;
	}
</style>
