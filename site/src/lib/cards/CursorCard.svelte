<script lang="ts">
	import { span } from '#lib/ease.js';
	import { loop } from '#lib/motion.svelte.js';
	import Canvas, { type Trail } from '#lib/ui/Canvas.svelte';
	import Chips from '#lib/ui/Chips.svelte';
	import { Recording } from '#lib/ui/recording.js';
	import { button, type Point } from '#lib/ui/screen.js';
	import Fit from './Fit.svelte';

	const A = button(0, 0);
	const B = button(1, 1);
	const REC = new Recording(1700, A, [{ start: 300, end: 1300, to: B }], [1500], [], 13);
	/** Recording time where each pass starts, raw first. */
	const RAW = 400;
	const SMOOTH = 4000;
	const FOCUS = { x: (A.x + B.x) / 2, y: (A.y + B.y) / 2 + 10 };

	const clock = loop(7400, 6800);
	let t = $derived(clock.t);
	let smooth = $derived(t >= SMOOTH - 400);
	let r = $derived(Math.min(REC.duration, Math.max(0, t - (smooth ? SMOOTH : RAW))));

	const trace = (at: (r: number) => Point, until: number) =>
		Array.from({ length: Math.floor(until / 16) + 1 }, (_, i) => at(Math.min(until, i * 16)));
	const RAW_PATH = trace((r) => REC.raw(r), REC.duration);

	let trails = $derived<Trail[]>(
		smooth
			? [
					{ points: RAW_PATH, kind: 'raw', opacity: 0.5 },
					{ points: trace((r) => REC.smooth(r), r), kind: 'smooth' }
				]
			: [{ points: trace((r) => REC.raw(r), r), kind: 'raw' }]
	);
	let label = $derived(span(t, (smooth ? SMOOTH : RAW) + 900, (smooth ? SMOOTH : RAW) + 1200));
</script>

<div class="card-visual app" {@attach clock.observe}>
	<Fit width={506} height={380}>
		<div class="canvas">
			<Canvas
				width={466}
				height={300}
				wallpaper="graphite"
				padding={3.3}
				zoom={1}
				level={1.7}
				focus={FOCUS}
				cursor={smooth ? REC.smooth(r) : REC.raw(r)}
				cursorSize={30}
				cursorScale={REC.pressing(r) ? 0.8 : 1}
				cursorTilt={smooth ? REC.tilt(r) : 0}
				pressed={r >= 1500 && r < 1660 ? '1,1' : null}
				{trails}
			/>
			<span class={['tag', smooth ? 'smooth' : 'raw']} style:opacity={label}>{smooth ? 'Smoothed' : 'Recorded'}</span>
		</div>
		<div class="row">
			<span>Smoothing</span>
			<Chips items={['Smooth', 'Medium', 'Rapid', 'Off']} selected={smooth ? 'Smooth' : 'Off'} />
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
		left: 20px;
		overflow: hidden;
		border-radius: 8px;
		box-shadow: 0 0 0 1px var(--line);
	}

	.tag {
		position: absolute;
		top: 160px;
		left: 228px;
		padding: 3px 8px;
		border-radius: 6px;
		color: #fff;
		font-size: 11px;
		font-weight: 600;
	}

	.raw {
		background: #636366;
	}

	.smooth {
		background: #0a84ff;
	}

	.row {
		position: absolute;
		top: 338px;
		left: 20px;
		right: 20px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		color: var(--muted);
		font-size: 13px;
	}
</style>
