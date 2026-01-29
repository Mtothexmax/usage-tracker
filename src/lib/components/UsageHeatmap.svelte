<script lang="ts">
    import { usageState, type UsageDay } from '$lib/usage.svelte';
    import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
    import { Badge } from '$lib/components/ui/badge';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import SettingsDialog from './SettingsDialog.svelte';

    let selectedDay = $state<UsageDay | null>(null);
    let scrollContainer = $state<HTMLDivElement | null>(null);
    let showSettings = $state(false);

    onMount(() => {
        if (scrollContainer) {
            scrollContainer.scrollLeft = scrollContainer.scrollWidth;
        }
    });

    $effect(() => {
        if (usageState.data.length > 0) {
            const today = todayStr;
            
            if (!selectedDay) {
                const foundToday = usageState.data.find(d => d.date === today);
                if (foundToday) {
                    selectedDay = foundToday;
                } else {
                    selectedDay = usageState.data[usageState.data.length - 1];
                }
            } else {
                const updated = usageState.data.find(d => d.date === selectedDay?.date);
                if (updated) {
                    selectedDay = updated;
                }
            }
        }
    });

    $effect(() => {
        if (scrollContainer && months) {
            requestAnimationFrame(() => {
                if (scrollContainer) {
                    scrollContainer.scrollLeft = scrollContainer.scrollWidth;
                }
            });
        }
    });

    function getUsageLevel(minutes: number) {
        if (minutes === 0) return 'bg-muted/30'; 
        if (minutes <= 60) return 'bg-emerald-900/40';
        if (minutes <= 180) return 'bg-emerald-700/60';
        if (minutes <= 360) return 'bg-emerald-500/80';
        return 'bg-emerald-400'; 
    }

    function formatTime(minutes: number) {
        if (minutes === 0) return '0m';
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return h > 0 ? `${h}h ${m}m` : `${m}m`;
    }

    function getBreaks(sessions: any[]) {
        const breaks = [];
        for (let i = 0; i < sessions.length - 1; i++) {
            const endTs = sessions[i].end_ts;
            const nextStartTs = sessions[i+1].start_ts;
            const diffSeconds = nextStartTs - endTs;
            
            if (diffSeconds > 0) {
                breaks.push({
                    start: sessions[i].end,
                    end: sessions[i+1].start,
                    duration: Math.floor(diffSeconds / 60)
                });
            }
        }
        return breaks;
    }

    const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    let months = $derived.by(() => {
        const grouped: Record<string, UsageDay[]> = {};
        const now = new Date();
        for (let i = 0; i < 365; i++) {
            const d = new Date();
            d.setDate(now.getDate() - i);
            const year = d.getFullYear();
            const month = String(d.getMonth() + 1).padStart(2, '0');
            const day = String(d.getDate()).padStart(2, '0');
            const dateStr = `${year}-${month}-${day}`;
            const monthStr = dateStr.substring(0, 7);
            if (!grouped[monthStr]) grouped[monthStr] = [];
            
            const dayData = usageState.data.find(d => d.date === dateStr) || {
                date: dateStr,
                total_minutes: 0,
                sessions: []
            };
            grouped[monthStr].push(dayData);
        }

        return Object.entries(grouped)
            .sort((a, b) => a[0].localeCompare(b[0]))
            .map(([month, days]) => {
                days.sort((a, b) => a.date.localeCompare(b.date));
                const firstDayDate = new Date(days[0].date);
                const startPadding = (firstDayDate.getDay() + 6) % 7;
                
                const weeks: (UsageDay | null)[][] = [];
                let currentWeek: (UsageDay | null)[] = [];
                
                for (let i = 0; i < startPadding; i++) {
                    currentWeek.push(null);
                }
                
                for (let i = 0; i < days.length; i++) {
                    currentWeek.push(days[i]);
                    if (currentWeek.length === 7) {
                        weeks.push(currentWeek);
                        currentWeek = [];
                    }
                }
                
                if (currentWeek.length > 0) {
                    while (currentWeek.length < 7) {
                        currentWeek.push(null);
                    }
                    weeks.push(currentWeek);
                }
                
                return { month, weeks };
            });
    });

    const todayStr = $derived.by(() => {
        const now = new Date();
        const year = now.getFullYear();
        const month = String(now.getMonth() + 1).padStart(2, '0');
        const day = String(now.getDate()).padStart(2, '0');
        return `${year}-${month}-${day}`;
    });
    
    const todayUsageData = $derived(usageState.data.find(d => d.date === todayStr));
    const todayUsage = $derived(todayUsageData?.total_minutes || 0);

    let showDetailsPopup = $state(false);

    function formatDisplayDate(dateStr: string) {
        const date = new Date(dateStr + 'T00:00:00');
        const day = date.getDate();
        const suffix = (day % 10 === 1 && day !== 11) ? 'st' :
                       (day % 10 === 2 && day !== 12) ? 'nd' :
                       (day % 10 === 3 && day !== 13) ? 'rd' : 'th';
        
        return date.toLocaleDateString('en-US', { 
            weekday: 'long', 
            month: 'long', 
            day: 'numeric' 
        }) + suffix;
    }

    const isOngoingBreak = $derived.by(() => {
        if (!selectedDay || !todayUsageData || selectedDay.date !== todayUsageData.date) return null;
        if (!todayUsageData.sessions || todayUsageData.sessions.length === 0) return null;
        
        const lastSession = todayUsageData.sessions[todayUsageData.sessions.length - 1];
        const lastPingTs = lastSession.end_ts;
        const nowTs = Math.floor(usageState.now / 1000);
        
        const diffSeconds = nowTs - lastPingTs;
        const thresholdSeconds = usageState.config?.break_threshold_seconds || 120;

        if (diffSeconds >= thresholdSeconds && !usageState.isActive) {
            return {
                start: lastSession.end,
                duration: Math.floor(diffSeconds / 60)
            };
        }
        return null;
    });

    const sessionsAndBreaks = $derived.by(() => {
        if (!selectedDay) return [];
        const items = [];
        const breaks = getBreaks(selectedDay.sessions);
        for (let i = 0; i < selectedDay.sessions.length; i++) {
            items.push({ type: 'session', ...selectedDay.sessions[i] });
            if (breaks[i]) {
                items.push({ type: 'break', ...breaks[i] });
            }
        }
        
        if (isOngoingBreak) {
            items.push({ 
                type: 'ongoing-break', 
                start: isOngoingBreak.start, 
                end: 'Now', 
                duration: isOngoingBreak.duration 
            });
        }
        return items;
    });
</script>

<div class="flex flex-col gap-6 w-full max-w-6xl p-4 relative">
    <button 
        class="fixed top-1 right-1 p-2 hover:bg-muted rounded-full transition-colors text-muted-foreground hover:text-foreground z-50 bg-background/50 backdrop-blur-sm"
        onclick={() => showSettings = true}
        aria-label="Settings"
    >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path><circle cx="12" cy="12" r="3"></circle></svg>
    </button>

    {#if usageState.config?.show_debug_view}
        {@const lastPing = todayUsageData?.sessions?.[todayUsageData.sessions.length - 1]?.end_ts}
        <div class="text-[10px] text-muted-foreground/50 font-mono flex flex-wrap gap-x-4 gap-y-1 border-b pb-2">
            <span>Today: {todayStr}</span>
            <span>Sel: {selectedDay?.date}</span>
            <span>Thresh: {usageState.config?.break_threshold_seconds}s</span>
            <span>Now: {new Date(usageState.now).toLocaleTimeString()}</span>
            {#if lastPing}
                <span>Gap: {Math.floor(usageState.now / 1000) - lastPing}s</span>
                <span>LastPing: {lastPing}</span>
            {/if}
            <span>Active: {usageState.isActive}</span>
            <button 
                class="bg-primary/20 hover:bg-primary/30 px-2 rounded ml-auto text-[9px]"
                onclick={() => invoke('show_break_reminder')}
            >
                Trigger Break
            </button>
        </div>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
            <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0">
                <CardTitle class="text-sm font-medium text-muted-foreground">Today's Usage</CardTitle>
                <div class="flex items-center gap-2">
                    <span class="text-[10px] font-medium uppercase tracking-wider {usageState.isActive ? (usageState.isMediaPlaying ? 'text-blue-500' : 'text-emerald-500') : 'text-muted-foreground'}">
                        {usageState.isMediaPlaying ? 'Watching Video' : (usageState.isActive ? 'Active' : 'Idle')}
                    </span>
                    <div class="w-2 h-2 rounded-full {usageState.isActive ? (usageState.isMediaPlaying ? 'bg-blue-500 animate-pulse' : 'bg-emerald-500 animate-pulse') : 'bg-muted-foreground/30'}"></div>
                </div>
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">{formatTime(todayUsage)}</div>
            </CardContent>
        </Card>
        <Card>
            <CardHeader class="pb-2">
                <CardTitle class="text-sm font-medium text-muted-foreground">Weekly Average</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">
                    {formatTime(Math.round(usageState.data.reduce((acc, d) => acc + d.total_minutes, 0) / (usageState.data.length || 1)))}
                </div>
            </CardContent>
        </Card>
        <Card>
            <CardHeader class="pb-2">
                <CardTitle class="text-sm font-medium text-muted-foreground">Active Sessions</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">{todayUsageData?.sessions.length || 0}</div>
            </CardContent>
        </Card>
    </div>

    <Card class="w-full">
        <CardHeader>
            <CardTitle>Usage Heatmap</CardTitle>
        </CardHeader>
                <CardContent>
                    <div 
                        bind:this={scrollContainer}
                        class="flex overflow-x-auto pb-4 gap-8 scrollbar-thin scrollbar-thumb-muted-foreground/20 hover:scrollbar-thumb-muted-foreground/40 transition-colors"
                    >
                        {#each months as { month, weeks }}
                            <div class="flex flex-col gap-2 min-w-fit last:pr-12">
                                <span class="text-xs font-semibold text-muted-foreground/70 ml-1">
                                    {monthNames[parseInt(month.split('-')[1]) - 1]} {month.split('-')[0]}
                                </span>
                                <div class="flex flex-col">
                                    {#each weeks as week}
                                        <div class="flex">
                                            {#each week as day}
                                                {#if day}
                                                    <button 
                                                        class="group p-[3px] outline-none"
                                                        onclick={() => selectedDay = day}
                                                        title="{new Date(day.date).toLocaleDateString('en-US', { weekday: 'long' })}: {formatTime(day.total_minutes)}"
                                                        aria-label="Select {day.date}"
                                                    >
                                                        <div class="w-4 h-4 rounded-xs transition-all duration-200 
                                                               group-hover:scale-125 group-hover:z-10 group-hover:ring-2 group-hover:ring-primary/50
                                                               {getUsageLevel(day.total_minutes)} 
                                                               {selectedDay?.date === day.date ? 'ring-2 ring-primary ring-offset-2 ring-offset-background scale-110 z-10' : ''}">
                                                        </div>
                                                    </button>
                                                {:else}
                                                    <div class="p-[3px]">
                                                        <div class="w-4 h-4 rounded-xs bg-transparent"></div>
                                                    </div>
                                                {/if}
                                            {/each}
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/each}
                    </div>
                </CardContent>
        
    </Card>

    {#if selectedDay}
        <Card class="animate-in fade-in slide-in-from-bottom-4 duration-300">
            <CardHeader class="flex flex-row items-center justify-between">
                <div>
                    <CardTitle class="flex items-center gap-3">
                        Usage for {formatDisplayDate(selectedDay.date)}
                        {#if isOngoingBreak}
                            <Badge variant="secondary" class="bg-primary/10 text-primary border-primary/20 animate-pulse">
                                Currently in a Break ({formatTime(isOngoingBreak.duration)})
                            </Badge>
                        {/if}
                    </CardTitle>
                    <p class="text-2xl font-bold mt-2">{formatTime(selectedDay.total_minutes)}</p>
                </div>
                <button 
                    class="px-4 py-2 text-sm font-medium text-primary hover:underline"
                    onclick={() => showDetailsPopup = true}
                >
                    More details
                </button>
            </CardHeader>
        </Card>
    {/if}
</div>

{#if showDetailsPopup && selectedDay}
    <div 
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200"
        onclick={() => showDetailsPopup = false}
        role="presentation"
    >
        <div 
            class="bg-background border rounded-xl shadow-2xl max-w-lg w-full max-h-[80vh] flex flex-col animate-in zoom-in-95 duration-200"
            onclick={(e) => e.stopPropagation()}
            role="presentation"
        >
            <div class="p-6 border-b flex items-center justify-between">
                <h3 class="text-xl font-bold">Sessions & Breaks — {formatDisplayDate(selectedDay.date)}</h3>
                <button 
                    class="p-2 hover:bg-muted rounded-full"
                    onclick={() => showDetailsPopup = false}
                    aria-label="Close"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                </button>
            </div>
            <div class="p-6 overflow-y-auto">
                <div class="space-y-4">
                    {#if sessionsAndBreaks.length === 0}
                        <p class="text-muted-foreground text-center py-4">No usage recorded for this day.</p>
                    {:else}
                        <div class="grid gap-4">
                            {#each sessionsAndBreaks as item}
                                {#if item.type === 'session'}
                                    <div class="flex items-center justify-between p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                                        <div class="flex flex-col">
                                            <span class="text-sm font-medium">{item.start} — {item.end}</span>
                                            <span class="text-xs text-emerald-600 dark:text-emerald-400">Active Session</span>
                                        </div>
                                        <Badge variant="outline" class="border-emerald-500/30 text-emerald-600 dark:text-emerald-400">{formatTime(item.duration_minutes)}</Badge>
                                    </div>
                                {:else if item.type === 'break'}
                                    <div class="flex items-center justify-between p-2 ml-4 border-l-2 border-muted border-dashed">
                                        <div class="flex flex-col">
                                            <span class="text-xs text-muted-foreground">{item.start} — {item.end}</span>
                                            <span class="text-xs text-muted-foreground/60">Break</span>
                                        </div>
                                        <span class="text-xs text-muted-foreground/60">{formatTime(item.duration)}</span>
                                    </div>
                                {:else}
                                    <div class="flex items-center justify-between p-2 ml-4 border-l-2 border-primary border-dashed bg-primary/5 animate-pulse rounded-r-lg">
                                        <div class="flex flex-col">
                                            <span class="text-xs font-semibold text-primary">{item.start} — {item.end}</span>
                                            <span class="text-xs text-primary/70 font-medium">Ongoing Break</span>
                                        </div>
                                        <span class="text-xs text-primary/70 font-medium">{formatTime(item.duration)}</span>
                                    </div>
                                {/if}
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
            <div class="p-6 border-t flex justify-end">
                <button 
                    class="px-4 py-2 bg-primary text-primary-foreground rounded-md font-medium hover:bg-primary/90 transition-colors"
                    onclick={() => showDetailsPopup = false}
                >
                    Close
                </button>
            </div>
        </div>
    </div>
{/if}

{#if showSettings}
    <SettingsDialog onclose={() => showSettings = false} />
{/if}

<style>
    .scrollbar-thin::-webkit-scrollbar {
        height: 6px;
    }
    .scrollbar-thin::-webkit-scrollbar-track {
        background: var(--muted);
        border-radius: 3px;
    }
    .scrollbar-thin::-webkit-scrollbar-thumb {
        background: var(--muted-foreground);
        border-radius: 3px;
        opacity: 0.3;
    }
</style>