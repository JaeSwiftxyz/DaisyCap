<script lang="ts">
	import { span } from '#lib/ease.js';
	import { loop } from '#lib/motion.svelte.js';
	import Canvas from '#lib/ui/Canvas.svelte';
	import ExportSheet from '#lib/ui/ExportSheet.svelte';
	import Fit from './Fit.svelte';

	const PRESS = 1200;
	const START = 1350;
	const DONE = 3300;

	const clock = loop(7200, 5000);
	let t = $derived(clock.t);
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={304}>
		<div class="canvas">
			<Canvas width={340} height={250} wallpaper="lagoon" padding={9} resolution={[480, 320]} />
		</div>
		<div class="sheet">
			<ExportSheet
				pressed={t >= PRESS && t < PRESS + 160}
				progress={t >= START ? span(t, START, DONE) : null}
				done={t >= DONE}
				seconds={12}
				estimate={7}
				size={6.2}
				speed={3.9}
			/>
		</div>
	</Fit>
</div>

<style>
	.card-visual {
		background: radial-gradient(ellipse 70% 60% at 50% 0%, rgba(110, 150, 255, 0.08), transparent);
	}

	.canvas {
		position: absolute;
		top: 27px;
		left: 18px;
		overflow: hidden;
		border-radius: 8px;
		box-shadow: 0 0 0 1px var(--line);
	}

	.sheet {
		position: absolute;
		top: 12px;
		right: 18px;
		transform: scale(0.72);
		transform-origin: 100% 0;
	}
</style>
