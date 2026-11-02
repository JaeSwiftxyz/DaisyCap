<script lang="ts">
	import { loop } from '#lib/motion.svelte.js';
	import Timeline from '#lib/ui/Timeline.svelte';
	import Fit from './Fit.svelte';

	const DURATION = 12_000;

	const clock = loop(6000, 2700);
	let playhead = $derived(clock.t * 2);
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={304}>
		<div class="panel">
			<Timeline
				duration={DURATION}
				width={446}
				{playhead}
				step={2000}
				playing
				compact
				clips={[
					{ start: 0, end: 4200, label: 'Clip' },
					{ start: 4200, end: 5200, label: '2×', selected: true },
					{ start: 5200, end: DURATION, label: 'Clip' }
				]}
				zooms={[
					{ start: 1000, end: 3800, label: '1.8× Auto' },
					{ start: 7600, end: 10_000, label: '1.7× Auto' }
				]}
				masks={[{ start: 5600, end: 7800, label: 'Blur' }]}
				voiceovers={[{ start: 6000, end: 11_400 }]}
			/>
		</div>
	</Fit>
</div>

<style>
	.card-visual {
		background: radial-gradient(ellipse 70% 60% at 50% 0%, rgba(110, 150, 255, 0.08), transparent);
	}

	.panel {
		position: absolute;
		top: 34px;
		left: 20px;
		width: 466px;
		padding: 10px;
		border-radius: 10px;
		background: var(--sidebar);
		box-shadow: 0 0 0 1px var(--line);
	}
</style>
