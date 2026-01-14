/**
 * Bridge for communication between Frontend (JS/TS) and Backend (Rust)
 * Updated to match Rust GameState struct exactly.
 */

export interface Ball {
    screen_w: number;
    screen_h: number;
    radius: number;
    x: number;
    y: number;
    vx: number;
    vy: number;
}

export interface GameState {
    ball: Ball;
    stage: number;
    stage_elapsed: number;
    paused: boolean;
    is_transitioning: boolean;
    transition_timer: number;
    is_game_over: boolean;
    is_start_screen: boolean;
    stage5_paused: boolean;
    stage5_pause_elapsed: number;
}

export interface Color {
    r: number;
    g: number;
    b: number;
}

export interface Theme {
    background: {
        grid_color_dark: Color;
        grid_color_light: Color;
        grid_size: number;
    };
    ball: {
        gradient_start: Color;
        gradient_end: Color;
        outline_color: Color;
        radius_ratio: number;
    };
    ui: {
        title_color: Color;
        subtitle_color: Color;
        stats_color: Color;
        button_color: Color;
        button_hover_color: Color;
        background_color: Color;
    };
}

export type GameEvent = 
    | { type: 'BallBounced' }
    | { type: 'StageChanged', from: number, to: number }
    | { type: 'StageCompleted', stage: number }
    | { type: 'GameOver' };

declare global {
    interface Window {
        __TAURI__: {
            core: {
                invoke: (command: string, args?: any) => Promise<any>;
            }
        }
    }
}

export class Bridge {
    public static async invoke<T>(command: string, args: any = {}): Promise<T | null> {
        try {
            // @ts-ignore
            if (window.__TAURI__) {
                // @ts-ignore
                return await window.__TAURI__.core.invoke(command, args);
            }
            console.warn(`Bridge: Tauri not available for command ${command}`);
            return null;
        } catch (e) {
            console.error(`Bridge: Command ${command} failed`, e);
            return null;
        }
    }

    static async tick(dt: number): Promise<[GameState, GameEvent[]] | null> {
        return await this.invoke<[GameState, GameEvent[]]>('tick', { dt });
    }

    static async showMainWindow(): Promise<void> {
        await this.invoke('show_main_window');
    }

    static async toggleFullscreen(): Promise<void> {
        await this.invoke('toggle_fullscreen');
    }

    static async minimizeWindow(): Promise<void> {
        await this.invoke('minimize_window');
    }

    static async closeWindow(): Promise<void> {
        await this.invoke('close_window');
    }

    static async togglePause(): Promise<void> {
        await this.invoke('toggle_pause');
    }

    static async resizeGame(w: number, h: number): Promise<void> {
        await this.invoke('resize_game', { w, h });
    }

    static async nextStage(): Promise<GameState | null> {
        return await this.invoke<GameState>('next_stage');
    }

    static async prevStage(): Promise<GameState | null> {
        return await this.invoke<GameState>('prev_stage');
    }

    static async startGame(): Promise<void> {
        await this.invoke('start_game');
    }

    static async resetGame(w: number, h: number): Promise<GameState | null> {
        return await this.invoke<GameState>('reset_game', { w, h });
    }

    static async exitApp(): Promise<void> {
        await this.invoke('exit_app');
    }

    static async getTheme(): Promise<Theme | null> {
        return await this.invoke<Theme>('get_theme');
    }

    static async getLanguage(): Promise<string | null> {
        return await this.invoke<string>('get_language');
    }

    static async setLanguage(language: string): Promise<void> {
        await this.invoke('set_language', { language });
    }
}
