class Game {
    constructor() {
        this.canvas = document.getElementById('gameCanvas');
        this.renderer = new Renderer('gameCanvas');
        this.audio = new AudioPlayer();
        this.input = new InputHandler();
        this.virtualControls = new VirtualControls();
        
        this.gameState = null;
        this.lastTime = 0;
        this.isProcessing = false;
        this.tickCount = 0;
        this.firstTick = true;
        this.tickTimeout = null;
        
        this.init();
    }
    
    async init() {
        console.log('Game: Initializing...');
        
        this.setupEventListeners();
        createLanguageMenu();
        
        this.renderer.loadTheme();
        this.audio.init();
        
        if (window.__TAURI__) {
            try {
                const savedLang = await window.__TAURI__.core.invoke('get_language');
                if (savedLang) {
                    setLocale(savedLang);
                }
            } catch (e) {
                console.warn('Game: Failed to load saved language', e);
            }
        }
        
        requestAnimationFrame((t) => this.gameLoop(t));
        
        window.game = this;
        console.log('Game: Initialization complete');
    }
    
    setupEventListeners() {
        this.input.on('keydown', (key) => {
            console.log('Game: Keydown', key);
            if (key === 'Space') {
                if (this.gameState?.is_start_screen) {
                    this.startGame();
                } else {
                    this.togglePause();
                }
            } else if (key === 'KeyR') {
                this.restart();
            } else if (key === 'Escape') {
                this.exit();
            } else if (key === 'KeyP') {
                this.audio.toggleMusic();
            }
        });
        
        this.input.on('touch', (e) => {
            this.audio.resume();
            if (this.gameState?.is_start_screen) {
                this.startGame();
            } else {
                this.togglePause();
            }
        });

        this.input.on('window-control', (btnId) => {
            console.log('Game: Window control:', btnId);
            if (window.__TAURI__) {
                if (btnId === 'ctrl-min') {
                    window.__TAURI__.core.invoke('plugin:window|set_minimized', { minimized: true });
                } else if (btnId === 'ctrl-max') {
                    window.__TAURI__.core.invoke('plugin:window|toggle_maximize');
                } else if (btnId === 'ctrl-close') {
                    window.__TAURI__.core.invoke('exit_app');
                }
            }
        });
    }
    
    async gameLoop(timestamp) {
        if (!this.lastTime) this.lastTime = timestamp;
        const dt = (timestamp - this.lastTime) / 1000;
        this.lastTime = timestamp;
        
        if (!this.isProcessing && dt > 0 && dt < 0.2) {
            this.isProcessing = true;
            
            try {
                if (window.__TAURI__) {
                    const [state, events] = await this.safeInvoke('tick', { dt });
                    
                    if (state) {
                        this.gameState = state;
                        window.gameState = state;
                        this.tickCount++;
                        
                        if (this.firstTick && this.tickCount >= 1) {
                            this.firstTick = false;
                            const loadingOverlay = document.getElementById('loading-overlay');
                            if (loadingOverlay) {
                                loadingOverlay.style.display = 'none';
                                console.log('Game: Loading overlay hidden');
                            }
                        }
                        
                        for (const event of events) {
                            this.handleEvent(event);
                        }
                        
                        document.dispatchEvent(new CustomEvent('game-state-changed', { 
                            detail: state 
                        }));
                    }
                }
            } catch (e) {
                console.error("Game loop error:", e);
            } finally {
                this.isProcessing = false;
            }
        }
        
        this.renderer.render(this.gameState);
        
        requestAnimationFrame((t) => this.gameLoop(t));
    }
    
    async safeInvoke(command, params) {
        const timeoutPromise = new Promise((_, reject) => {
            this.tickTimeout = setTimeout(() => reject(new Error('Timeout')), 5000);
        });
        
        try {
            const result = await Promise.race([
                window.__TAURI__.core.invoke(command, params),
                timeoutPromise
            ]);
            
            if (this.tickTimeout) {
                clearTimeout(this.tickTimeout);
                this.tickTimeout = null;
            }
            
            return result;
        } catch (e) {
            if (this.tickTimeout) {
                clearTimeout(this.tickTimeout);
                this.tickTimeout = null;
            }
            console.error(`Game: ${command} failed:`, e);
            return null;
        }
    }
    
    handleEvent(event) {
        if (event === 'BallBounced') {
            this.audio.playBounce();
        }
    }
    
    async startGame() {
        if (window.__TAURI__) {
            await this.safeInvoke('start_game', {});
        }
    }
    
    async togglePause() {
        if (window.__TAURI__) {
            await this.safeInvoke('toggle_pause', {});
        }
    }
    
    async restart() {
        if (window.__TAURI__) {
            await this.safeInvoke('reset_game', {
                w: window.innerWidth,
                h: window.innerHeight
            });
        }
    }
    
    exit() {
        if (window.__TAURI__) {
            window.__TAURI__.core.invoke('exit_app');
        }
    }
}

window.addEventListener('DOMContentLoaded', () => {
    console.log('DOM Content Loaded');
    setTimeout(() => {
        new Game();
    }, 100);
});
