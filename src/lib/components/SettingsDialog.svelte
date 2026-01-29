<script lang="ts">
    import { usageState } from '$lib/usage.svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from '$lib/components/ui/card';

    let { onclose } = $props<{ onclose: () => void }>();

    let config = $state({ ...usageState.config });
    let isSaving = $state(false);

    async function save() {
        if (!config) return;
        isSaving = true;
        try {
            await invoke('update_config', { config });
            usageState.config = { ...config } as any;
            onclose();
        } catch (e) {
            console.error('Failed to save config', e);
        } finally {
            isSaving = false;
        }
    }

    // Helper to handle numeric inputs
    function handleNumericInput(key: keyof typeof config, value: string) {
        const num = parseInt(value);
        if (!isNaN(num)) {
            (config as any)[key] = num;
        }
    }
</script>

<div 
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200"
    onmousedown={(e) => { if (e.target === e.currentTarget) onclose(); }}
    role="presentation"
>
    <div 
        class="max-w-md w-full animate-in zoom-in-95 duration-200"
        onmousedown={(e) => e.stopPropagation()}
        role="presentation"
    >
        <Card>
            <CardHeader>
                <CardTitle>Settings</CardTitle>
                <CardDescription>Configure your break reminders and tracking.</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                <!-- Break Reminder Toggle -->
                <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                        <label for="reminder-enabled" class="text-sm font-medium">Break Reminder</label>
                        <p class="text-xs text-muted-foreground">Show a popup when it's time to take a break.</p>
                    </div>
                    <button 
                        id="reminder-enabled"
                        class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 {config?.break_reminder_enabled ? 'bg-primary' : 'bg-input'}"
                        onclick={() => { if (config) config.break_reminder_enabled = !config.break_reminder_enabled; }}
                    >
                        <span class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform {config?.break_reminder_enabled ? 'translate-x-5' : 'translate-x-0'}"></span>
                    </button>
                </div>

                <!-- Break Reminder Threshold -->
                <div class="space-y-2">
                    <label for="reminder-threshold" class="text-sm font-medium">Reminder Every (minutes)</label>
                    <Input 
                        id="reminder-threshold"
                        type="number" 
                        value={config?.break_reminder_threshold_minutes}
                        oninput={(e) => handleNumericInput('break_reminder_threshold_minutes', e.currentTarget.value)}
                        placeholder="55"
                    />
                    <p class="text-[10px] text-muted-foreground">Minutes of active work before showing the reminder.</p>
                </div>

                <!-- Break Idle Threshold -->
                <div class="space-y-2">
                    <label for="idle-threshold" class="text-sm font-medium">Idle Timeout (seconds)</label>
                    <Input 
                        id="idle-threshold"
                        type="number" 
                        value={config?.break_threshold_seconds}
                        oninput={(e) => handleNumericInput('break_threshold_seconds', e.currentTarget.value)}
                        placeholder="120"
                    />
                    <p class="text-[10px] text-muted-foreground">Seconds of inactivity before a session is considered ended.</p>
                </div>

                <!-- Debug View Toggle -->
                <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                        <label for="debug-view" class="text-sm font-medium">Debug View</label>
                        <p class="text-xs text-muted-foreground">Show technical tracking information at the top.</p>
                    </div>
                    <button 
                        id="debug-view"
                        class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 {config?.show_debug_view ? 'bg-primary' : 'bg-input'}"
                        onclick={() => { if (config) config.show_debug_view = !config.show_debug_view; }}
                    >
                        <span class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform {config?.show_debug_view ? 'translate-x-5' : 'translate-x-0'}"></span>
                    </button>
                </div>

                <!-- Full Screen Overlay Toggle -->
                <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                        <label for="fullscreen-svg" class="text-sm font-medium">Full Screen Overlay</label>
                        <p class="text-xs text-muted-foreground">Show a full screen SVG during breaks.</p>
                    </div>
                    <button 
                        id="fullscreen-svg"
                        class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 {config?.show_fullscreen_svg ? 'bg-primary' : 'bg-input'}"
                        onclick={() => { if (config) config.show_fullscreen_svg = !config.show_fullscreen_svg; }}
                    >
                        <span class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform {config?.show_fullscreen_svg ? 'translate-x-5' : 'translate-x-0'}"></span>
                    </button>
                </div>
            </CardContent>
            <CardFooter class="flex justify-end gap-2">
                <Button variant="outline" onclick={onclose}>Cancel</Button>
                <Button onclick={save} disabled={isSaving}>
                    {isSaving ? 'Saving...' : 'Save Changes'}
                </Button>
            </CardFooter>
        </Card>
    </div>
</div>
