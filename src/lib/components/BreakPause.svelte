<script lang="ts">
    import { usageState } from '$lib/usage.svelte';
    import { Button } from '$lib/components/ui/button';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window';
    import bgSvg from '../../assets/background.svg?url';

    let timeLeft = $state(300); // 5 minutes
    let timer: any;

    async function closeWindow() {
        try {
            await invoke('close_window');
        } catch (e) {
            window.close();
        }
    }

    async function applyWindowSize() {
        try {
            const win = getCurrentWindow();
            await win.setResizable(true);
            
            if (usageState.config?.show_fullscreen_svg) {
                await win.setFullscreen(true);
            } else {
                // If we were in fullscreen, we need a moment to exit before positioning
                const wasFullscreen = await win.isFullscreen();
                if (wasFullscreen) {
                    await win.setFullscreen(false);
                    // Give the OS time to restore the window state
                    await new Promise(r => setTimeout(r, 150));
                }

                const width = 500;
                const height = 500;
                await win.setSize(new LogicalSize(width, height));
                
                // Use center() as a reliable baseline
                await win.center();
                
                // Now shift it to be at 20% from top
                const monitor = await win.currentMonitor();
                const pos = await win.outerPosition();
                if (monitor && pos) {
                    const scale = monitor.scaleFactor;
                    const targetY = Math.round(monitor.size.height * 0.2);
                    await win.setPosition(new PhysicalPosition(pos.x, targetY));
                }
            }
        } catch (e) {
            console.error('Failed to apply window size', e);
        }
    }

    async function toggleFullscreen() {
        if (!usageState.config) return;
        const newEnabled = !usageState.config.show_fullscreen_svg;
        const newConfig = { ...usageState.config, show_fullscreen_svg: newEnabled };
        
        try {
            await invoke('update_config', { config: newConfig });
            usageState.config = newConfig;
            await applyWindowSize();
        } catch (e) {
            console.error('Toggle error', e);
        }
    }

    onMount(() => {
        applyWindowSize();
        timer = setInterval(() => {
            if (timeLeft > 0) {
                timeLeft -= 1;
            } else {
                closeWindow();
            }
        }, 1000);
        return () => clearInterval(timer);
    });

    function formatTime(s: number) {
        const m = Math.floor(s / 60);
        const secs = s % 60;
        return `${m}:${secs.toString().padStart(2, '0')}`;
    }
</script>

<main 
    class="h-screen w-screen flex flex-col items-center justify-center backdrop-blur-xl text-white select-none relative
    {usageState.config?.show_fullscreen_svg ? 'bg-transparent' : 'bg-emerald-600/90 border-4 rounded-[32px] border-white/20 overflow-hidden'}"
>
    {#if usageState.config?.show_fullscreen_svg}
        <div class="absolute inset-0 -z-20 bg-slate-900"></div>
        <img 
            src={bgSvg} 
            alt="" 
            class="absolute inset-0 w-full h-full object-cover -z-10"
        />
        <div class="absolute inset-0 -z-10 bg-black/40 backdrop-blur-sm"></div>
    {/if}

    <button 
        class="absolute top-6 right-6 p-2 hover:bg-white/20 rounded-full transition-colors z-50 text-white/50 hover:text-white"
        onclick={toggleFullscreen}
        title="Toggle Fullscreen"
    >
        {#if usageState.config?.show_fullscreen_svg}
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/></svg>
        {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg>
        {/if}
    </button>

    <div class="text-center space-y-10 relative z-10 p-6">
        <h1 class="text-5xl font-bold tracking-tight text-white drop-shadow-md">Enjoy your break</h1>
        <div class="text-9xl font-mono font-bold tabular-nums text-white drop-shadow-lg">{formatTime(timeLeft)}</div>
        <p class="text-xl max-w-[400px] mx-auto leading-tight text-white/90 font-medium">
            Great! Rest your eyes and move around.
        </p>
        <div class="pt-6 flex justify-center">
             <Button variant="outline" class="bg-white text-slate-900 hover:bg-white/90 border-none px-16 py-8 text-2xl font-bold rounded-2xl h-auto shadow-xl" onclick={closeWindow}>
                Done
            </Button>
        </div>
    </div>
</main>

<style>
    :global(html), :global(body) {
        background: transparent !important;
        margin: 0;
        padding: 0;
    }
</style>