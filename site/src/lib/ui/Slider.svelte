<script lang="ts">
	/** An inspector row: label, value field and slider. `value` runs from 0 to 1. */
	let { label, value, text, active = false }: { label: string; value: number; text: string; active?: boolean } =
		$props();
</script>

<div class="row">
	<span class="label">{label}</span>
	<span class="field">{text}</span>
	<span class="slider">
		<span class="rail"></span>
		<span class="fill" style:width="{value * 100}%"></span>
		<span class={['knob', { active }]} style:left="calc({value} * (100% - 15px))"></span>
	</span>
</div>

<style>
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 22px;
	}

	.label {
		flex: 1;
		color: var(--text);
		font-size: 13px;
	}

	.field {
		width: 50px;
		padding: 2px 0;
		border-radius: 5px;
		background: var(--field);
		box-shadow: inset 0 0 0 0.5px var(--control-edge);
		color: var(--text);
		font-size: 12px;
		line-height: 15px;
		text-align: center;
		font-variant-numeric: tabular-nums;
	}

	.slider {
		position: relative;
		width: 96px;
		height: 15px;
	}

	.rail,
	.fill {
		position: absolute;
		left: 0;
		top: 6px;
		height: 3px;
		border-radius: 2px;
	}

	.rail {
		right: 0;
		background: var(--separator);
	}

	.fill {
		background: var(--accent);
	}

	.knob {
		position: absolute;
		top: 0;
		width: 15px;
		height: 15px;
		border-radius: 50%;
		background: var(--knob);
		box-shadow:
			0 1px 2px rgba(0, 0, 0, 0.4),
			0 0 0 0.5px rgba(0, 0, 0, 0.12);
		transition: scale 0.15s;
	}

	.knob.active {
		scale: 1.12;
	}
</style>
