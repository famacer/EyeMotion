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
        
        this.init();
    }
    
    async init() {
        await this.renderer.loadTheme();
        this.setupEventListeners();
        
        if (window.__TAURI__) {
            const savedLang = await window.__TAURI__.core.invoke('get_language');
            if (savedLang) {
                await setLocale(savedLang);
            }
        }
        
        createLanguageMenu();
        
        requestAnimationFrame((t) => this.gameLoop(t));
        
        window.game = this;
    }
    
    setupEventListeners() {
        this.input.on('keydown', (key) => {
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
    }
    
    async gameLoop(timestamp) {
        if (!this.lastTime) this.lastTime = timestamp;
        const dt = (timestamp - this.lastTime) / 1000;
        this.lastTime = timestamp;
        
        if (!this.isProcessing && dt > 0 && dt < 0.2) {
            this.isProcessing = true;
            
            try {
                if (window.__TAURI__) {
                    const [state, events] = await window.__TAURI__.core.invoke('tick', { dt });
                    
                    this.gameState = state;
                    window.gameState = state;
                    
                    for (const event of events) {
                        this.handleEvent(event);
                    }
                    
                    document.dispatchEvent(new CustomEvent('game-state-changed', { 
                        detail: state 
                    }));
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
    
    handleEvent(event) {
        if (event === 'BallBounced') {
            this.audio.playBounce();
        }
    }
    
    async startGame() {
        if (window.__TAURI__) {
            await window.__TAURI__.core.invoke('start_game');
        }
    }
    
    async togglePause() {
        if (window.__TAURI__) {
            await window.__TAURI__.core.invoke('toggle_pause');
        }
    }
    
    async restart() {
        if (window.__TAURI__) {
            await window.__TAURI__.core.invoke('reset_game', {
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
    setTimeout(() => {
        new Game();
    }, 100);
});
