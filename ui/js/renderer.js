class Renderer {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.theme = this.getDefaultTheme();
        this.ready = true;
    }

    getDefaultTheme() {
        return {
            background: {
                grid_color_dark: { r: 0, g: 31, b: 86 },
                grid_color_light: { r: 0, g: 48, b: 130 },
                grid_size: 80.0
            },
            ball: {
                gradient_start: { r: 16, g: 180, b: 195 },
                gradient_end: { r: 17, g: 197, b: 140 },
                outline_color: { r: 70, g: 226, b: 213 },
                radius_ratio: 1.0 / 40.0
            },
            ui: {
                title_color: { r: 255, g: 215, b: 0 },
                subtitle_color: { r: 255, g: 105, b: 120 },
                stats_color: { r: 135, g: 206, b: 235 }
            }
        };
    }

    async loadTheme() {
        this.ready = false;
        
        if (window.__TAURI__) {
            try {
                console.log('Renderer: Loading theme from backend...');
                this.theme = await window.__TAURI__.core.invoke('get_theme');
                console.log('Renderer: Theme loaded');
            } catch (e) {
                console.error('Renderer: Failed to load theme, using default', e);
                this.theme = this.getDefaultTheme();
            }
        }
        
        this.ready = true;
    }

    resize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }

    drawBackground() {
        const { grid_color_dark, grid_color_light, grid_size } = this.theme.background;
        const cols = Math.ceil(this.canvas.width / grid_size);
        const rows = Math.ceil(this.canvas.height / grid_size);

        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                const color = (r + c) % 2 === 0 
                    ? this.hexToRgba(grid_color_dark)
                    : this.hexToRgba(grid_color_light);
                this.ctx.fillStyle = color;
                this.ctx.fillRect(c * grid_size, r * grid_size, grid_size, grid_size);
            }
        }
    }

    drawBall(ball) {
        if (!ball) return;
        
        const { pos, radius } = ball;
        const [x, y] = pos;
        
        const gradient = this.ctx.createRadialGradient(x, y, 0, x, y, radius);
        gradient.addColorStop(0, this.hexToRgba(this.theme.ball.gradient_start));
        gradient.addColorStop(1, this.hexToRgba(this.theme.ball.gradient_end));

        this.ctx.beginPath();
        this.ctx.arc(x, y, radius, 0, Math.PI * 2);
        this.ctx.fillStyle = gradient;
        this.ctx.fill();

        this.ctx.strokeStyle = this.hexToRgba(this.theme.ball.outline_color);
        this.ctx.lineWidth = 2;
        this.ctx.stroke();
        this.ctx.closePath();
    }

    drawText(text, x, y, size, color) {
        this.ctx.font = `bold ${size}px sans-serif`;
        this.ctx.fillStyle = this.hexToRgba(color);
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';
        this.ctx.fillText(text, x, y);
    }

    hexToRgba(color) {
        return `rgba(${color.r}, ${color.g}, ${color.b}, 1)`;
    }

    render(gameState) {
        if (!this.ready) {
            return;
        }
        
        this.resize();
        this.drawBackground();

        if (!gameState) {
            this.ctx.fillStyle = "white";
            this.ctx.font = "24px sans-serif";
            this.ctx.textAlign = "center";
            this.ctx.fillText("Loading Backend...", this.canvas.width/2, this.canvas.height/2);
            return;
        }

        if (!gameState.is_start_screen && !gameState.is_game_over && !gameState.is_transitioning) {
            this.drawBall(gameState.ball);
        }

        this.drawUI(gameState);
    }

    drawUI(gameState) {
        if (!gameState) return;
        
        if (gameState.is_start_screen) {
            const sizeTitle = this.canvas.width / 8;
            const sizeStart = this.canvas.width / 20;
            const sizeFooter = this.canvas.width / 24;
            
            const yTitle = this.canvas.height * 0.35;
            const yStart = yTitle + sizeTitle * 0.5;
            const yFooter = this.canvas.height - sizeFooter;

            this.drawText(t('title'), this.canvas.width/2, yTitle, sizeTitle, this.theme.ui.title_color);
            this.drawText(t('press_space'), this.canvas.width/2, yStart, sizeStart, this.theme.ui.subtitle_color);
            
            const bgmTxt = window.audioPlayer?.bgmEnabled ? t('controls.on') : t('controls.off');
            const footerText = `${t('controls.exit')} - ${t('controls.space')} - ${t('controls.music')} (${bgmTxt})`;
            this.drawText(footerText, this.canvas.width/2, yFooter, sizeFooter, this.theme.ui.stats_color);
        } else if (gameState.is_game_over) {
            const sizeGo = this.canvas.width / 8;
            const sizeRestart = this.canvas.width / 20;
            
            const yGo = this.canvas.height * 0.35;
            const yRestart = yGo + sizeGo * 0.8;

            this.drawText(t('game_over'), this.canvas.width/2, yGo, sizeGo, this.theme.ui.title_color);
            this.drawText(t('restart'), this.canvas.width/2, yRestart, sizeRestart, this.theme.ui.stats_color);
        } else if (gameState.is_transitioning) {
            const text = gameState.stage === 5 ? t('stage_circular') : `${t('stage')} ${gameState.stage}`;
            const sizeLevel = this.canvas.width / 8;
            
            this.drawText(text, this.canvas.width/2, this.canvas.height/2, sizeLevel, this.theme.ui.title_color);
        } else {
            const rem = Math.max(0, 45 - gameState.stage_elapsed).toFixed(0);
            const txt = `${t('time')}: ${rem}`;
            const sizeTime = this.canvas.width / 30;
            
            this.drawText(txt, this.canvas.width/2, this.canvas.height - 80, sizeTime, this.theme.ui.stats_color);
            
            if (gameState.paused) {
                const sizePause = this.canvas.width / 8;
                this.drawText(t('paused'), this.canvas.width/2, this.canvas.height/2, sizePause, this.theme.ui.title_color);
            }
        }
    }
}
