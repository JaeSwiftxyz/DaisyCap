<script lang="ts">
	import { loop } from '#lib/motion.svelte.js';
	import Canvas from '#lib/ui/Canvas.svelte';
	import { Recording } from '#lib/ui/recording.js';
	import { button } from '#lib/ui/screen.js';
	import Fit from './Fit.svelte';

	const HOME = { x: 700, y: 470 };
	const A = button(0, 0);
	const B = button(1, 2);
	const REC = new Recording(
		5400,
		HOME,
		[
			{ start: 200, end: 900, to: A },
			{ start: 2500, end: 3300, to: B },
			{ start: 4700, end: 5300, to: HOME }
		],
		[1050, 3450],
		[
			{ start: 750, end: 2300, at: A, level: 1.8 },
			{ start: 3100, end: 4600, at: B, level: 1.7 }
		]
	);
	const TRACK = 440;

	const clock = loop(6000, 1500);
	let r = $derived(Math.min(clock.t, REC.duration));
	let camera = $derived(REC.camera(r));
	let pressed = $derived.by(() => {
		const click = REC.clicks.findIndex((c) => r >= c.t && r < c.t + 160);
		return click === 0 ? '0,0' : click === 1 ? '1,2' : null;
	});
	const x = (ms: number) => (ms / REC.duration) * TRACK;
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={380}>
		<div class="canvas">
			<Canvas
				width={440}
				height={300}
				wallpaper="ocean"
				padding={11}
				zoom={camera.zoom}
				level={camera.level}
				focus={camera.focus}
				blur={camera.blur}
				cursor={REC.smooth(r)}
				cursorSize={34}
				cursorScale={REC.pressing(r) ? 0.8 : 1}
				cursorTilt={REC.tilt(r)}
				circles={REC.circles(r)}
				{pressed}
			/>
		</div>
		<div class="track">
			{#each REC.zooms as zoom (zoom.start)}
				<span class="segment" style:left="{x(zoom.start) + 1}px" style:width="{x(zoom.end) - x(zoom.start) - 2}px">
					{zoom.level}× Auto
				</span>
			{/each}
			<span class="playhead" style:left="{x(r)}px"></span>
		</div>
	</Fit>
</div>

<style>
	.card-visual {
		background: radial-gradient(ellipse 70% 60% at 50% 0%, rgba(110, 150, 255, 0.08), transparent);
	}

	.canvas {
		position: absolute;
		top: 24px;
		left: 33px;
		overflow: hidden;
		border-radius: 8px;
		box-shadow: 0 0 0 1px var(--line);
	}

	.track {
		position: absolute;
		top: 338px;
		left: 33px;
		width: 440px;
		height: 26px;
		border-radius: 6px;
		background: var(--track);
	}

	.segment {
		position: absolute;
		top: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		padding-left: 8px;
		border-radius: 6px;
		background: var(--zoom);
		color: #fff;
		font-size: 10px;
		font-weight: 600;
		white-space: nowrap;
	}

	.playhead {
		position: absolute;
		top: -4px;
		bottom: -4px;
		width: 2px;
		margin-left: -1px;
		background: var(--accent);
	}
</style>
