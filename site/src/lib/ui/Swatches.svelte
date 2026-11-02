<script lang="ts">
	import Wallpaper from './Wallpaper.svelte';
	import { WALLPAPER_IDS, wallpaperName, type WallpaperId } from './wallpaper.js';

	/** The inspector's wallpaper grid, five to a row like the app. */
	let {
		selected,
		pressed = null,
		width = 52,
		height = 34,
		gap = 6
	}: { selected: WallpaperId; pressed?: WallpaperId | null; width?: number; height?: number; gap?: number } =
		$props();
</script>

<div class="swatches" style:grid-template-columns="repeat(5, {width}px)" style:gap="{gap}px">
	{#each WALLPAPER_IDS as id (id)}
		<span
			class={['swatch', { on: id === selected, pressed: id === pressed }]}
			style:height="{height}px"
			title={wallpaperName(id)}
			data-target="wp:{id}"
		>
			<Wallpaper {id} width={width * 2} height={height * 2} />
		</span>
	{/each}
</div>

<style>
	.swatches {
		display: grid;
	}

	.swatch {
		position: relative;
		overflow: hidden;
		border-radius: 6px;
		box-shadow: 0 0 0 1px var(--control-edge);
		transition:
			box-shadow 0.15s,
			scale 0.12s;
	}

	.on {
		box-shadow: 0 0 0 2.5px var(--accent);
	}

	.pressed {
		scale: 0.94;
	}
</style>
