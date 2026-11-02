<script lang="ts">
	import { easeOut, span } from '#lib/ease.js';
	import { loop } from '#lib/motion.svelte.js';
	import Dashboard from '#lib/ui/Dashboard.svelte';
	import RecordBar from '#lib/ui/RecordBar.svelte';
	import Segmented from '#lib/ui/Segmented.svelte';
	import Fit from './Fit.svelte';

	const SOURCES = ['Display', 'Window', 'Area', 'iPhone'];
	const STEP = 1800;

	const clock = loop(STEP * 3, 2600);
	let t = $derived(clock.t);
	let source = $derived(Math.floor(t / STEP));
	let shown = $derived(easeOut(span(t - source * STEP, 0, 260)));
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={304}>
		<div class="desktop">
			<div class="window"><Dashboard /></div>

			{#if source === 0}
				<div class="outline" style:opacity={shown}></div>
			{:else if source === 1}
				<div class="picked" style:opacity={shown}></div>
			{:else}
				<div class="dim" style:opacity={shown}></div>
				<div class="area" style:opacity={shown}><i></i><i></i><i></i><i></i></div>
			{/if}

			<div class="sources"><Segmented items={SOURCES} selected={source} accent /></div>
			<div class="bar"><RecordBar elapsed={4000 + t} /></div>
			<div class="camera"><span></span></div>
		</div>
	</Fit>
</div>

<style>
	.desktop {
		position: absolute;
		inset: 0;
		overflow: hidden;
		background:
			radial-gradient(circle farthest-corner at 50% 50%, rgba(64, 110, 255, 0.42), transparent 62%),
			radial-gradient(circle farthest-corner at 4% 112%, rgba(40, 200, 200, 0.26), transparent 60%),
			radial-gradient(circle farthest-corner at 104% 96%, rgba(130, 90, 255, 0.3), transparent 62%),
			linear-gradient(158deg, #121830 0%, #0a0d1a 46%, #0e1222 100%);
	}

	.window {
		position: absolute;
		top: 34px;
		left: 50px;
		width: 960px;
		height: 600px;
		overflow: hidden;
		border-radius: 18px;
		box-shadow: 0 30px 70px rgba(0, 0, 0, 0.6);
		transform: scale(0.42);
		transform-origin: 0 0;
	}

	.outline {
		position: absolute;
		inset: 4px;
		border: 2px solid var(--accent);
		border-radius: 10px;
	}

	.picked {
		position: absolute;
		top: 34px;
		left: 50px;
		width: 403px;
		height: 252px;
		border-radius: 8px;
		background: rgba(10, 132, 255, 0.16);
		box-shadow: inset 0 0 0 2.5px var(--accent);
	}

	.dim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		clip-path: polygon(
			evenodd,
			0 0,
			100% 0,
			100% 100%,
			0 100%,
			0 0,
			140px 70px,
			140px 222px,
			372px 222px,
			372px 70px,
			140px 70px
		);
	}

	.area {
		position: absolute;
		top: 70px;
		left: 140px;
		width: 232px;
		height: 152px;
		border: 1.5px dashed #fff;
	}

	.area i {
		position: absolute;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #fff;
		box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.3);
	}

	.area i:nth-child(1) {
		top: -5px;
		left: -5px;
	}

	.area i:nth-child(2) {
		top: -5px;
		right: -5px;
	}

	.area i:nth-child(3) {
		bottom: -5px;
		left: -5px;
	}

	.area i:nth-child(4) {
		right: -5px;
		bottom: -5px;
	}

	.sources {
		position: absolute;
		top: 14px;
		left: 50%;
		padding: 3px;
		border-radius: 10px;
		background: rgba(30, 30, 32, 0.82);
		backdrop-filter: blur(16px);
		box-shadow:
			inset 0 0 0 1px rgba(255, 255, 255, 0.12),
			0 10px 24px rgba(0, 0, 0, 0.4);
		translate: -50% 0;
		--field: transparent;
		--control-edge: transparent;
		--text: #ececee;
	}

	.bar {
		position: absolute;
		left: 26px;
		bottom: 20px;
	}

	.camera {
		position: absolute;
		right: 22px;
		bottom: 18px;
		display: grid;
		place-items: center;
		width: 112px;
		height: 92px;
		overflow: hidden;
		border-radius: 24px;
		background: linear-gradient(#3e4d6e, #74506f);
		box-shadow:
			0 0 0 1px rgba(255, 255, 255, 0.12),
			0 14px 30px rgba(0, 0, 0, 0.5);
	}

	.camera span {
		width: 46px;
		height: 56px;
		margin-top: 8px;
		border-radius: 50%;
		background: #e2b593;
	}
</style>
