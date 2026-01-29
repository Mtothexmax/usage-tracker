<script lang="ts">
    import { usageState } from '$lib/usage.svelte';
    import { Button } from '$lib/components/ui/button';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { getCurrentWindow, LogicalSize, PhysicalPosition } from '@tauri-apps/api/window';

    let { onBreakStarted } = $props<{ onBreakStarted: () => void }>();

    let timeLeft = $state(30);
    let timer: any;

    async function closeWindow() {
        try {
            await invoke('close_window');
        } catch (e) {
            window.close();
        }
    }

    async function positionAtTop() {
        try {
            const win = getCurrentWindow();
            await win.setFullscreen(false);
            await win.setSize(new LogicalSize(400, 200));
            const monitor = await win.currentMonitor();
            if (monitor) {
                const scale = monitor.scaleFactor;
                const x = (monitor.size.width - (400 * scale)) / 2;
                await win.setPosition(new PhysicalPosition(x, 20 * scale));
            }
        } catch (e) {
            console.error('Positioning error', e);
        }
    }

    onMount(() => {
        positionAtTop();
        timer = setInterval(() => {
            if (timeLeft > 0) {
                timeLeft -= 1;
            } else {
                if (!usageState.isActive) {
                    onBreakStarted();
                } else {
                    // User stayed active, dismiss reminder until next cycle
                    closeWindow();
                }
            }
        }, 1000);

        return () => clearInterval(timer);
    });

    $effect(() => {
        if (!usageState.isActive && timeLeft < 25) {
            onBreakStarted();
        }
    });

    function formatTime(s: number) {
        const m = Math.floor(s / 60);
        const secs = s % 60;
        return `${m}:${secs.toString().padStart(2, '0')}`;
    }
</script>

<main class="h-screen w-screen flex flex-col items-center justify-center bg-red-500/80 backdrop-blur-xl text-white border-4 rounded-[32px] border-white/20 select-none p-4">
    <div class="text-center space-y-3">
        <h1 class="text-2xl font-bold tracking-tight text-white">Take a Break!</h1>
        <div class="text-5xl font-mono font-bold tabular-nums text-white">{formatTime(timeLeft)}</div>
        <p class="text-white/90 text-[13px] max-w-[300px] mx-auto leading-tight">
            Stand up and stretch. If you stop moving now, a 5 minute break timer will start.
        </p>
        <div class="pt-2 flex gap-3 justify-center">
            <Button variant="outline" class="bg-transparent border-white text-white hover:bg-white/20 h-8" onclick={closeWindow}>
                Skip
            </Button>
        </div>
    </div>
</main>