import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface UsageSession {
    start: string;
    end: string;
    start_ts: number;
    end_ts: number;
    duration_minutes: number;
}

export interface UsageDay {
    date: string;
    total_minutes: number;
    sessions: UsageSession[];
}

export interface AppConfig {
    break_threshold_seconds: number;
    show_debug_view: boolean;
    break_reminder_threshold_minutes: number;
    break_reminder_enabled: boolean;
    show_fullscreen_svg: boolean;
}

export class UsageState {
    data = $state<UsageDay[]>([]);
    loading = $state(true);
    error = $state<string | null>(null);
    isActive = $state(false);
    isMediaPlaying = $state(false);
    config = $state<AppConfig | null>(null);
    now = $state(Date.now());

    constructor() {
        this.fetchConfig();
        this.fetchData();
        this.setupListener();
        // Regular refresh every 5 minutes as a fallback
        setInterval(() => this.fetchData(), 300000);
        // Update now every second for real-time break display
        setInterval(() => {
            this.now = Date.now();
        }, 1000);
    }

    async fetchConfig() {
        try {
            this.config = await invoke<AppConfig>('get_config');
        } catch (e) {
            console.error('Failed to fetch config', e);
        }
    }

    async fetchData() {
        try {
            const result = await invoke<UsageDay[]>('get_usage_data', { days: 365 });
            this.data = result;
            this.loading = false;
        } catch (e) {
            this.error = String(e);
            this.loading = false;
        }
    }

    async setupListener() {
        await listen<[boolean, boolean, number]>('usage-update', (event) => {
            const [active, mediaPlaying, serverTime] = event.payload;
            const wasActive = this.isActive;
            this.isActive = active;
            this.isMediaPlaying = mediaPlaying;
            this.now = serverTime * 1000;
            
            if (active || wasActive) {
                this.fetchData();
            }
        });
    }
}

export const usageState = new UsageState();