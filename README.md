# Usage Tracker & Break Reminder

A specialized desktop application built with **Tauri 2**, **Svelte 5 (Runes)**, and **shadcn-svelte**. It monitors your computer usage, provides a detailed activity heatmap, and reminds you to take breaks using a smart tracking system.

## Features

- **Activity Tracking**: Automatically records active sessions by monitoring mouse/keyboard input and media playback.
- **Smart Break Reminders**: 
  - Monitors continuous activity and triggers a subtle reminder after a configurable threshold (default 55 minutes).
  - **Dynamic Phases**: Starts with a small, transparent countdown at the top of the screen. If you stop moving, it transitions to a full "Break" mode.
  - **Auto-Dismiss**: If you continue working through the reminder countdown, it automatically dismisses itself until your next work cycle.
- **Visual Heatmap**: View your usage history over the last year with a GitHub-style activity heatmap.
- **Session Details**: Click any day to see a breakdown of active sessions and break durations.
- **Customizable Settings**:
  - Toggle break reminders on/off.
  - Set custom work thresholds and idle timeouts.
  - **Fullscreen Overlay**: Optional fullscreen SVG background (using a custom background asset) to make breaks more immersive.
  - **Debug View**: Real-time technical stats for tracking idle gaps and activity status.
- **Tray-First Design**: Starts minimized to the system tray to stay out of your way.

## Tech Stack

- **Frontend**: Svelte 5 (Runes), Tailwind CSS (v4), shadcn-svelte.
- **Backend**: Rust, Tauri 2, SQLite (via `rusqlite`) for local data persistence.
- **UI/UX**: Material Design principles, Lucide icons, and Framer Motion-like animations via Svelte's native transitions.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v18+)
- Standard Tauri build dependencies for your OS.

### Installation

1. Clone the repository.
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

## Configuration

Settings are stored locally in your application data directory (`config.json`). You can modify these directly through the in-app Settings menu (accessible via the gear icon in the top-right corner).

## License

MIT