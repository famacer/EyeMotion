import { Renderer } from './renderer';
import { AudioPlayer } from './audio';
import { InputHandler } from './input';
import { VirtualControls, createLanguageMenu } from './virtual-controls';
import { setLocale } from './i18n';
import { Bridge, GameState, GameEvent } from './bridge';

export class Game {
    private renderer: Renderer;
    private audio: AudioPlayer;
    private input: InputHandler;
    private gameState: GameState | null = null;
    private mousePos: { x: number, y: number } = { x: 0, y: 0 };
    private isMouseDown: boolean = false;
    private lastMouseMoveTime: number = Date.now();
    private lastTime: number = 0;
    private isProcessing: boolean = false;
    private firstTick: boolean = true;

    constructor() {
        this.renderer = new Renderer('gameCanvas');
        this.audio = new AudioPlayer();
        this.input = new InputHandler();
        new VirtualControls();
        this.init();
    }
    
    private async init(): Promise<void> {
        console.log('Game: Initializing...');
        
        // Don't await forever, but try to load
        this.renderer.loadFonts();
        
        this.setupEventListeners();
        window.addEventListener('resize', () => this.handleResize());
        
        // Handle visibility change (background/foreground)
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                console.log('Game: App went to background');
                this.audio.stopBGM(); // Or suspend
                if (this.gameState && !this.gameState.paused && !this.gameState.is_start_screen && !this.gameState.is_game_over) {
                    this.togglePause();
                }
            } else {
                console.log('Game: App returned to foreground');
                // Optional: Resume BGM if it was playing? 
                // Better to leave it paused and let user resume via UI.
            }
        });

        createLanguageMenu();
        
        this.renderer.loadTheme();
        this.audio.init();
        
        try {
            const savedLang = await Bridge.getLanguage();
            if (savedLang) setLocale(savedLang);
        } catch (e) {
            console.warn('Game: Failed to load saved language:', e);
        }
        
        try {
            // 根据屏幕比例动态计算逻辑分辨率
            const { w, h } = this.getLogicalSize();
            this.renderer.setLogicalSize(w, h);
            
            const state = await Bridge.resetGame(w, h);
            this.gameState = state;
        } catch (e) {
            console.warn('Game: Failed to reset game:', e);
        }
        
        requestAnimationFrame((t) => this.gameLoop(t));
        (window as any).game = this;
    }

    private getLogicalSize(): { w: number, h: number } {
        const h = 1080;
        const w = Math.round(h * (window.innerWidth / window.innerHeight));
        return { w, h };
    }
    
    private setupEventListeners(): void {
        this.input.on('mousemove', (pos: { x: number, y: number }) => {
            // 只有当鼠标位置发生实际变化时才重置计时器，防止某些传感器抖动
            // 阈值提高到 2px，更彻底地解决微小抖动导致的计时器重置
            const dx = pos.x - this.mousePos.x;
            const dy = pos.y - this.mousePos.y;
            if (Math.abs(dx) > 2 || Math.abs(dy) > 2) {
                this.mousePos = pos;
                this.lastMouseMoveTime = Date.now();
            }
        });

        this.input.on('mousedown', (pos: { x: number, y: number }) => {
            this.isMouseDown = true;
            if (pos) {
                this.mousePos = pos;
                this.lastMouseMoveTime = Date.now();
            }
        });

        this.input.on('mouseup', () => {
            this.isMouseDown = false;
            this.lastMouseMoveTime = Date.now();
        });

        this.input.on('click', (pos: { x: number, y: number }) => {
            this.audio.resume();
            // 首先尝试处理渲染器中的按钮点击
            const handled = this.renderer.handlePoint(pos.x, pos.y);
            
            if (handled) {
                this.audio.playClick();
            }
            
            // 如果点击没有被按钮处理，且游戏正在进行，则执行暂停切换
            if (!handled && this.gameState && !this.gameState.is_start_screen && !this.gameState.is_game_over) {
                this.togglePause();
            }
        });

        this.input.on('debug-skip-stage', async () => {
            console.log('Game: Debug skip stage triggered');
            try {
                const result = await Bridge.nextStage();
                if (result) {
                    this.gameState = result;
                }
            } catch (e) {
                console.error('Game: Failed to skip stage:', e);
            }
        });

        this.input.on('debug-prev-stage', async () => {
            console.log('Game: Debug prev stage triggered');
            try {
                const result = await Bridge.prevStage();
                if (result) {
                    this.gameState = result;
                }
            } catch (e) {
                console.error('Game: Failed to go to prev stage:', e);
            }
        });

        this.input.on('window-control', async (btnId: string) => {
            console.log('Game: Window control clicked', btnId);
            if (btnId === 'ctrl-close') await this.exit();
            else if (btnId === 'ctrl-min') await Bridge.minimizeWindow();
            else if (btnId === 'ctrl-max') await Bridge.toggleFullscreen();
        });
    }
    
    private async gameLoop(timestamp: number): Promise<void> {
        if (!this.lastTime) this.lastTime = timestamp;
        const dt = (timestamp - this.lastTime) / 1000;
        this.lastTime = timestamp;

        if (this.isProcessing) {
            requestAnimationFrame((t) => this.gameLoop(t));
            return;
        }
        
        const clampedDt = Math.min(dt, 0.1);
        this.isProcessing = true;
        
        try {
            const result = await Bridge.tick(clampedDt);
            if (result) {
                const [state, events] = result;
                this.gameState = state;
                
                if (this.firstTick && state) {
                    this.firstTick = false;
                    console.log('Game: First valid state received, showing window');
                    setTimeout(() => {
                        Bridge.showMainWindow();
                    }, 100);
                }
            
                events.forEach(event => this.handleEvent(event));
                
                document.dispatchEvent(new CustomEvent('game-state-changed', { 
                    detail: this.gameState 
                }));
            }
        } catch (e) {
            console.error("Game loop error:", e);
        } finally {
            this.isProcessing = false;
        }
        
        try {
            // 如果处于游戏过程中（非开始屏幕、非结算屏幕），且 3 秒没动，则隐藏鼠标
            const now = Date.now();
            const isIdle = now - this.lastMouseMoveTime > 3000;
            const shouldHideCursor = isIdle && this.gameState && !this.gameState.is_start_screen && !this.gameState.is_game_over;

            this.renderer.render(this.gameState, this.mousePos, this.isMouseDown, !shouldHideCursor);
        } catch (e) {
            console.error("Rendering error:", e);
        }
        requestAnimationFrame((t) => this.gameLoop(t));
    }
    
    private handleEvent(event: GameEvent): void {
        if (event.type === 'BallBounced') {
            this.audio.playBounce();
        }
    }
    
    private async startGame(): Promise<void> {
        console.log('Game: Starting...');
        try {
            if (this.gameState) {
                this.gameState.is_start_screen = false;
                this.gameState.is_transitioning = true;
                this.gameState.transition_timer = 3.0; // 修改为 3.0s 以显示倒计时
            }
            await Bridge.startGame();
            this.audio.startBGM();
        } catch (e) {
            console.error('Game: Failed to start game:', e);
        }
    }

    public async quitGame(): Promise<void> {
        // 移动端：返回标题画面 (Start Screen)
        // 桌面端：退出应用 (Exit App)
        const isMobile = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
        if (isMobile) {
            console.log('Game: Mobile quit -> Returning to Start Screen');
            if (this.gameState) {
                this.gameState.is_start_screen = true;
                this.gameState.paused = false;
                this.gameState.is_game_over = false;
                this.audio.stopBGM();
                
                // CRITICAL FIX: Reset mouse down state to prevent immediate re-triggering of "Start" button
                // if the touch/click position overlaps with the Start button on the next frame.
                this.isMouseDown = false; 
                
                // 强制重绘一帧以更新 UI
                this.renderer.render(this.gameState, this.mousePos, this.isMouseDown, true);
            }
        } else {
            await this.exit();
        }
    }

    public async restartGame(): Promise<void> {
        console.log('Game: Restarting...');
        try {
            // 使用当前逻辑分辨率重置物理引擎
            const { w, h } = this.getLogicalSize();
            const state = await Bridge.resetGame(w, h);
            if (state) {
                this.gameState = state;
                this.gameState.is_start_screen = false; // Ensure it's false
            }
            await this.startGame();
        } catch (e) {
            console.error('Game: Failed to restart game:', e);
        }
    }
    
    private async togglePause(): Promise<void> {
        console.log('Game: Toggling pause...');
        try {
            if (this.gameState) {
                this.gameState.paused = !this.gameState.paused;
            }
            await Bridge.togglePause();
        } catch (e) {
            console.error('Game: Failed to toggle pause:', e);
        }
    }

    private async handleResize(): Promise<void> {
        this.renderer.resize();
        try {
            const { w, h } = this.getLogicalSize();
            this.renderer.setLogicalSize(w, h);
            await Bridge.resizeGame(w, h);
        } catch (e) {
            console.error('Game: Failed to resize game:', e);
        }
    }
    
    private async exit(): Promise<void> {
        console.log('Game: Exiting via command...');
        this.audio.stopBGM();
        try {
            await Bridge.exitApp();
        } catch (e) {
            console.error('Game: Failed to exit:', e);
            window.close();
        }
    }
}

// Start the game
new Game();
