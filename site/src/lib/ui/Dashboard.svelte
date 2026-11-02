<script lang="ts">
	import { CARD, SCREEN, card } from './screen.js';

	/** Which button is held down, as `col,row`. */
	let { pressed = null }: { pressed?: string | null } = $props();

	const CARDS = [0, 1, 2].flatMap((row) => [0, 1].map((col) => ({ col, row, ...card(col, row) })));
	const LINES = [
		[132, 150, 166, 184, 132],
		[150, 176, 120, 160, 168],
		[184, 132, 150, 166, 184],
		[132, 168, 184, 150, 120]
	];
</script>

<div class="screen" style:width="{SCREEN.w}px" style:height="{SCREEN.h}px" aria-hidden="true">
	<div class="toolbar">
		<span style:background="#ff5f57"></span><span style:background="#febc2e"></span><span style:background="#28c840"></span>
	</div>
	<div class="sidebar">
		{#each { length: 12 }, i (i)}
			<span class={{ on: i === 1 }}></span>
		{/each}
	</div>
	{#each CARDS as c, i (i)}
		<div class="card" style:left="{c.x}px" style:top="{c.y}px" style:width="{CARD.w}px" style:height="{CARD.h}px">
			<span class="title"></span>
			{#each LINES[i % LINES.length] as width, j (j)}
				<span class="line" style:width="{width}px"></span>
			{/each}
			<span class={['button', { pressed: pressed === `${c.col},${c.row}` }]}></span>
		</div>
	{/each}
	<span class="progress"></span>
</div>

<style>
	.screen {
		position: relative;
		overflow: hidden;
		background: #f6f6f8;
	}

	.toolbar {
		position: absolute;
		inset: 0 0 auto;
		display: flex;
		gap: 7px;
		align-items: center;
		height: 28px;
		padding-left: 12px;
		background: #e8e8ec;
	}

	.toolbar span {
		width: 9px;
		height: 9px;
		border-radius: 50%;
	}

	.sidebar {
		position: absolute;
		top: 28px;
		bottom: 0;
		left: 0;
		display: flex;
		flex-direction: column;
		gap: 13px;
		width: 150px;
		padding: 24px 0 0 14px;
		background: #ececf0;
	}

	.sidebar span {
		width: 110px;
		height: 10px;
		background: #c8c8d0;
	}

	.sidebar .on {
		background: #007aff;
	}

	.card {
		position: absolute;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 16px;
		background: #fff;
	}

	.title {
		width: 110px;
		height: 12px;
		margin-bottom: 8px;
		background: #282830;
	}

	.line {
		height: 6px;
		background: #aaaab4;
	}

	.button {
		width: 56px;
		height: 16px;
		margin-top: auto;
		background: #007aff;
		transition: filter 0.08s;
	}

	.button.pressed {
		filter: brightness(0.78);
	}

	.progress {
		position: absolute;
		left: 182px;
		top: 566px;
		width: 120px;
		height: 4px;
		background: #34c759;
	}
</style>
