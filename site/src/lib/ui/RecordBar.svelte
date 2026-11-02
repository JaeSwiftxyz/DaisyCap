<script lang="ts">
	/** The floating recording control bar (`recbar.rs`): 286×44 points, laid out at the app's offsets. */
	let {
		countdown = 0,
		elapsed = 0,
		pressed = false
	}: { countdown?: number; elapsed?: number; pressed?: boolean } = $props();

	let clock = $derived(`00:${String(Math.floor(elapsed / 1000)).padStart(2, '0')}`);
	/** The dot pulses about once a second while recording. */
	let pulse = $derived(countdown > 0 ? 1 : 0.65 + 0.35 * Math.cos((elapsed / 850) * Math.PI));
</script>

<div class="bar">
	<span class={['dot', { waiting: countdown > 0 }]} style:opacity={pulse}></span>
	<span class={['timer', { waiting: countdown > 0 }]}>{countdown > 0 ? `Starts in ${countdown}` : clock}</span>
	<span class="sep" style:left="124px"></span>
	<span class="icon" style:left="132px">
		<svg viewBox="0 0 14 14"><path d="M3.5 3.5l7 7M10.5 3.5l-7 7" /></svg>
	</span>
	<span class="icon" style:left="164px">
		<svg viewBox="0 0 14 14"><path d="M3.6 4.1A4.6 4.6 0 1 1 2.4 8.2" /><path d="M2.6 1.8v2.8h2.8" /></svg>
	</span>
	<span class="icon" style:left="196px">
		<svg viewBox="0 0 14 14" class="solid"
			><rect x="3.2" y="2.5" width="2.6" height="9" rx="0.7" /><rect
				x="8.2"
				y="2.5"
				width="2.6"
				height="9"
				rx="0.7"
			/></svg
		>
	</span>
	<span class="sep" style:left="234px"></span>
	<span class={['stop', { pressed, waiting: countdown > 0 }]}><span></span></span>
</div>

<style>
	.bar {
		position: relative;
		width: 286px;
		height: 44px;
		border-radius: 22px;
		background: rgba(30, 30, 32, 0.78);
		backdrop-filter: blur(20px) saturate(160%);
		box-shadow:
			inset 0 0 0 1px rgba(255, 255, 255, 0.14),
			0 12px 32px rgba(0, 0, 0, 0.45);
		color: #fff;
	}

	.bar > * {
		position: absolute;
	}

	.dot {
		top: 17px;
		left: 18px;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: #ff453a;
	}

	.dot.waiting {
		background: #8e8e93;
	}

	.timer {
		top: 0;
		left: 36px;
		font: 600 15px/44px var(--ui);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.timer.waiting {
		font-size: 13px;
	}

	.sep {
		top: 12px;
		width: 1px;
		height: 20px;
		background: rgba(255, 255, 255, 0.18);
	}

	.icon {
		top: 7px;
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
	}

	svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: rgba(255, 255, 255, 0.92);
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	svg.solid {
		fill: rgba(255, 255, 255, 0.92);
		stroke: none;
	}

	.waiting ~ .icon svg {
		opacity: 0.4;
	}

	.stop {
		top: 7px;
		left: 244px;
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		background: #ff453a;
		transition: scale 0.1s;
	}

	.stop span {
		width: 10px;
		height: 10px;
		border-radius: 2px;
		background: #fff;
	}

	.stop.waiting {
		opacity: 0.45;
	}

	.stop.pressed {
		scale: 0.88;
	}
</style>
