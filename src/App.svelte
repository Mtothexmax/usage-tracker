<script lang="ts">
	import UsageHeatmap from '$lib/components/UsageHeatmap.svelte';
	import BreakReminder from '$lib/components/BreakReminder.svelte';
	import BreakPause from '$lib/components/BreakPause.svelte';
	import { onMount } from 'svelte';
	import { usageState } from '$lib/usage.svelte';
	import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window';

	let isBreakWindow = $state(false);
	let breakPhase = $state<'reminder' | 'pause'>('reminder');
	let isReady = $state(false);

	async function startPausePhase() {
		breakPhase = 'pause';
		try {
			const win = getCurrentWindow();
			if (usageState.config?.show_fullscreen_svg) {
				await win.setFullscreen(true);
			} else {
				await win.setFullscreen(false);
				await win.setSize(new LogicalSize(450, 450));
				await win.center();
				
				const monitor = await win.currentMonitor();
				const pos = await win.outerPosition();
				if (monitor && pos) {
					const scale = monitor.scaleFactor;
					await win.setPosition(new PhysicalPosition(pos.x, pos.y - Math.round(100 * scale)));
				}
			}
		} catch (e) {
			console.error('Failed to transition window size', e);
		}
	}

	onMount(async () => {
		const params = new URLSearchParams(window.location.search);
		isBreakWindow = params.get('window') === 'break';
		
		if (isBreakWindow) {
			// Ensure we have config before showing anything
			try {
				await Promise.race([
					usageState.fetchConfig(),
					new Promise((_, reject) => setTimeout(() => reject('Timeout fetching config'), 2000))
				]);
			} catch (e) {
				console.error('Config fetch failed or timed out', e);
			}
		}
		isReady = true;
	});
</script>

{#if isBreakWindow}
	{#if isReady}
		{#if breakPhase === 'reminder'}
			<BreakReminder onBreakStarted={startPausePhase} />
		{:else}
			<BreakPause />
		{/if}
	{:else}
		<!-- Fallback while loading to avoid ghost window -->
		<div class="h-screen w-screen bg-black/10 backdrop-blur-sm"></div>
	{/if}
{:else}
	<main
		class="flex min-h-screen flex-col items-center bg-background text-foreground p-4 md:p-8 select-none"
	>
		<UsageHeatmap />
	</main>
{/if}

<style>
	:global(html), :global(body) {
		margin: 0;
		padding: 0;
		overflow: hidden;
		background-color: transparent !important;
	}
</style>