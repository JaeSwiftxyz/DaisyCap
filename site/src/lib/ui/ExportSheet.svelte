<script lang="ts">
	import Segmented from './Segmented.svelte';

	type Props = {
		progress?: number | null;
		done?: boolean;
		pressed?: boolean;
		seconds?: number;
		estimate?: number;
		size?: number;
		speed?: number;
	};

	/**
	 * The editor's Export window: settings, then progress, then the result. The defaults match a
	 * 1080p H.264 export of the synthetic demo, which PERFORMANCE.md measures at about 4× real time.
	 */
	let { progress = null, done = false, pressed = false, seconds = 3.3, estimate = 2, size = 1.7, speed = 3.9 }: Props =
		$props();
</script>

{#snippet menu(value: string, width: number)}
	<span class="menu" style:width="{width}px">
		{value}
		<svg viewBox="0 0 8 5"><path d="M0 0h8L4 5z" /></svg>
	</span>
{/snippet}

<div class="sheet">
	<div class="head">
		Export
		<svg class="close" viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" /></svg>
	</div>
	<div class="body">
		<div class="row"><span>Format</span><Segmented items={['GIF', 'MP4']} selected={1} /></div>
		<div class="row"><span>Resolution</span>{@render menu('1080p', 100)}</div>
		<div class="row"><span>Frame rate</span>{@render menu('60 fps', 100)}</div>
		<div class="row"><span>Quality</span>{@render menu('Social media', 128)}</div>
		<div class="row"><span>Codec</span><Segmented items={['H.264', 'HEVC', 'ProRes']} selected={0} /></div>
		<div class="row">
			<span>Fit to file size</span>
			<span class="toggle"><span></span></span>
		</div>
		<p class="caption">1604×1080 • {seconds.toFixed(1)} s • about {estimate} MB</p>

		{#if progress !== null && !done}
			<div class="progress">
				<span class="track"><span class="fill" style:width="{progress * 100}%"></span></span>
				<span class="percent">{Math.round(progress * 100)}%</span>
			</div>
			<p class="caption">
				{progress > 0.02 ? `${Math.max(1, Math.ceil(((1 - progress) * seconds) / speed))} s left` : '\u00a0'}
			</p>
			<div class="actions"><span class="button">Cancel</span></div>
		{:else}
			{#if done}
				<div class="done">
					<span class="ok">✓ Exported</span>
					<p class="caption">Product tour.mp4 • 1604×1080 • {size.toFixed(1)} MB • {speed.toFixed(1)}× real time</p>
					<div class="actions start">
						<span class="button">Show in Finder</span>
						<span class="button">Copy</span>
						<span class="button">Open</span>
					</div>
				</div>
			{/if}
			<div class="actions"><span class={['primary', { pressed }]} data-target="go">Export</span></div>
		{/if}
	</div>
</div>

<style>
	.sheet {
		width: 380px;
		overflow: hidden;
		border-radius: 12px;
		background: var(--popover);
		box-shadow:
			0 0 0 1px var(--control-edge),
			0 24px 60px var(--drop);
		color: var(--text);
		font-size: 13px;
	}

	.head {
		position: relative;
		display: grid;
		place-items: center;
		height: 34px;
		background: var(--popover-head);
		font-weight: 600;
	}

	.close {
		position: absolute;
		top: 12px;
		right: 13px;
		width: 10px;
		height: 10px;
		stroke: var(--text);
		stroke-width: 1.3;
		stroke-linecap: round;
	}

	.body {
		display: flex;
		flex-direction: column;
		gap: 5px;
		padding: 12px 14px 14px;
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 24px;
	}

	.menu {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 22px;
		padding: 0 9px;
		border-radius: 6px;
		background: var(--control);
		box-shadow: inset 0 0 0 0.5px var(--control-edge);
		font-size: 11.5px;
	}

	.menu svg {
		width: 8px;
		height: 5px;
		fill: var(--text);
	}

	.toggle {
		position: relative;
		width: 32px;
		height: 18px;
		border-radius: 9px;
		background: var(--separator);
	}

	.toggle span {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: #fff;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
	}

	.caption {
		margin: 0;
		color: var(--secondary);
		font-size: 11px;
		line-height: 15px;
	}

	.progress {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 10px;
	}

	.track {
		flex: 1;
		height: 16px;
		overflow: hidden;
		border-radius: 5px;
		background: var(--field);
		box-shadow: inset 0 0 0 0.5px var(--control-edge);
	}

	.fill {
		display: block;
		height: 100%;
		background: var(--accent);
	}

	.percent {
		width: 34px;
		color: var(--secondary);
		font-size: 11px;
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	.done {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 10px;
	}

	.ok {
		color: var(--done);
		font-weight: 600;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 6px;
		margin-top: 8px;
	}

	.actions.start {
		justify-content: flex-start;
		margin-top: 4px;
	}

	.button,
	.primary {
		padding: 3px 10px;
		border-radius: 6px;
		background: var(--control);
		box-shadow: inset 0 0 0 0.5px var(--control-edge);
		font-size: 11.5px;
		line-height: 16px;
		white-space: nowrap;
	}

	.primary {
		padding: 4px 16px;
		background: var(--accent);
		box-shadow: none;
		color: #fff;
		font-weight: 600;
		transition: filter 0.1s;
	}

	.primary.pressed {
		filter: brightness(0.82);
	}
</style>
