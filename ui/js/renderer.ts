import { Bridge, GameState, Theme, Color } from './bridge';

export class Renderer {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private theme: Theme;
    private buttons: Map<string, { x: number, y: number, w: number, h: number, callback: () => void }> = new Map();
    private buttonHoverFactors: Map<string, number> = new Map();

    constructor(canvasId: string) {
        const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
        if (!canvas) throw new Error(`Canvas with id ${canvasId} not found`);
        this.canvas = canvas;
        // 尝试开启 alpha 以获取更好的文字子像素抗锯齿效果
        this.ctx = this.canvas.getContext('2d', { 
            alpha: true,
            desynchronized: true
        })!;
        
        // 强制开启抗锯齿
        this.ctx.imageSmoothingEnabled = true;
        this.ctx.imageSmoothingQuality = 'high';

        this.theme = this.getDefaultTheme();
        this.resize();
        this.loadFonts();
    }

    public async loadTheme(): Promise<void> {
        try {
            const theme = await Bridge.getTheme();
            if (theme) {
                this.theme = theme;
            }
        } catch (e) {
            console.warn('Renderer: Failed to load theme, using default', e);
        }
    }

    private getDefaultTheme(): Theme {
        return {
            background: {
                grid_color_dark: { r: 1, g: 19, b: 104 }, // #011368
                grid_color_light: { r: 0, g: 49, b: 159 }, // #00319F
                grid_size: 80
            },
            ball: {
                gradient_start: { r: 252, g: 97, b: 112 }, // #FC6170
                gradient_end: { r: 230, g: 37, b: 56 }, // #E62538
                outline_color: { r: 255, g: 255, b: 255 },
                radius_ratio: 0.05
            },
            ui: {
                title_color: { r: 235, g: 191, b: 66 },
                subtitle_color: { r: 255, g: 255, b: 255 },
                stats_color: { r: 64, g: 197, b: 239 },
                button_color: { r: 252, g: 97, b: 112 },
                button_hover_color: { r: 230, g: 37, b: 56 },
                background_color: { r: 1, g: 19, b: 104 }
            }
        };
    }

    public resize(): void {
        const dpr = window.devicePixelRatio || 1;
        this.canvas.width = window.innerWidth * dpr;
        this.canvas.height = window.innerHeight * dpr;
        this.canvas.style.width = `${window.innerWidth}px`;
        this.canvas.style.height = `${window.innerHeight}px`;
        this.ctx.scale(dpr, dpr);
    }

    public async loadFonts(): Promise<void> {
        try {
            await document.fonts.ready;
        } catch (e) {
            console.warn('Renderer: Font loading failed', e);
        }
    }

    private colorToCSS(color: Color): string {
        return `rgb(${color.r}, ${color.g}, ${color.b})`;
    }

    // 获取相对于 1920x1080 设计稿的比例
    private getScale(): number {
        const h = window.innerHeight;
        // 始终基于高度缩放，以确保纵向比例一致
        return h / 1080;
    }

    // 将设计稿坐标转换为实际屏幕坐标
    private scalePos(x: number, y: number): { x: number, y: number } {
        const scale = this.getScale();
        const w = window.innerWidth;
        const h = window.innerHeight;
        
        // 计算 1920x1080 逻辑区域在实际屏幕上的偏移，使其完全居中
        const offsetX = (w - 1920 * scale) / 2;
        const offsetY = (h - 1080 * scale) / 2;
        
        return {
            x: x * scale + offsetX,
            y: y * scale + offsetY
        };
    }

    // 将设计稿大小转换为实际屏幕大小
    private scaleSize(size: number): number {
        return size * this.getScale();
    }

    private drawText(text: string, x: number, y: number, size: number, color: string | CanvasGradient, align: CanvasTextAlign = 'center', fontStack: string = "'AlumniSans', system-ui, sans-serif", opacity: number = 1): void {
        const pos = this.scalePos(x, y);
        const finalSize = Math.round(this.scaleSize(size)); // 整数化字号，减少渲染模糊
        this.ctx.save();
        this.ctx.globalAlpha = opacity;
        
        // 增加绘制提示
        this.ctx.imageSmoothingEnabled = true;
        this.ctx.imageSmoothingQuality = 'high';

        this.ctx.font = `${finalSize}px ${fontStack}`;
        this.ctx.fillStyle = color;
        this.ctx.textAlign = align;
        this.ctx.textBaseline = 'middle'; 
        
        if (/\d/.test(text) && align === 'center') {
            this.ctx.font = `tabular-nums ${finalSize}px ${fontStack}`;
        }
        
        // 使用微小的偏移或四舍五入来对齐像素，减少边缘模糊
        const drawX = Math.floor(pos.x) + 0.5;
        const drawY = Math.floor(pos.y) + 0.5;
        
        // 回归简洁渲染，移除多层叠加算法
        this.ctx.fillText(text, drawX, drawY);

        this.ctx.restore();
    }

    private drawButton(id: string, text: string, x: number, y: number, fontSize: number, paddingX: number, paddingY: number, gradientColors: string[], callback: () => void, mousePos?: { x: number, y: number }, isMouseDown?: boolean): void {
        const finalFontSize = this.scaleSize(fontSize);
        const finalPaddingX = this.scaleSize(paddingX);
        const finalPaddingY = this.scaleSize(paddingY);
        
        this.ctx.font = `${finalFontSize}px 'AlumniSans'`;
        const textMetrics = this.ctx.measureText(text);
        const textWidth = textMetrics.width;
        
        const btnW = Math.round(textWidth + finalPaddingX * 2);
        const btnH = Math.round((finalFontSize * 0.7) + finalPaddingY * 2); 
        
        const pos = this.scalePos(x, y);
        const btnX = Math.round(pos.x - btnW / 2);
        const btnY = Math.round(pos.y - btnH / 2);
        
        // 记录按钮区域用于点击检测
        this.buttons.set(id, { x: btnX, y: btnY, w: btnW, h: btnH, callback });
        
        // 状态检测
        const isHover = mousePos && mousePos.x >= btnX && mousePos.x <= btnX + btnW && mousePos.y >= btnY && mousePos.y <= btnY + btnH;
        const isClick = isHover && isMouseDown;

        // 动画逻辑：平滑更新 hoverFactor
        let factor = this.buttonHoverFactors.get(id) || 0;
        const targetFactor = isHover ? 1 : 0;
        // 动画速度：每帧变化 0.05，约 20 帧 (330ms) 完成过渡，比之前更慢一些
        if (factor < targetFactor) factor = Math.min(targetFactor, factor + 0.05);
        else if (factor > targetFactor) factor = Math.max(targetFactor, factor - 0.05);
        this.buttonHoverFactors.set(id, factor);

        this.ctx.save();
        
        // 应用动画缩放 (基于 factor)
        if (isClick) {
            this.ctx.translate(pos.x, pos.y);
            this.ctx.scale(0.95, 0.95);
            this.ctx.translate(-pos.x, -pos.y);
        } else if (factor > 0) {
            this.ctx.translate(pos.x, pos.y);
            const scale = 1 + (0.05 * factor); // 0 -> 1 对应 1.0 -> 1.05
            this.ctx.scale(scale, scale);
            this.ctx.translate(-pos.x, -pos.y);
        }

        // 绘制阴影 (基于 factor)
        if (factor > 0 && !isClick) {
            this.ctx.shadowBlur = 15 * factor;
            this.ctx.shadowColor = `rgba(0, 0, 0, ${0.3 * factor})`;
            this.ctx.shadowOffsetY = 5 * factor;
        }

        const gradient = this.ctx.createLinearGradient(btnX, btnY, btnX, btnY + btnH);
        if (isClick) {
            gradient.addColorStop(0, gradientColors[1]);
            gradient.addColorStop(1, gradientColors[1]);
        } else {
            gradient.addColorStop(0, gradientColors[0]);
            gradient.addColorStop(1, gradientColors[1]);
        }
        
        const radius = this.scaleSize(48);
        this.ctx.fillStyle = gradient;
        
        // 1. 填充主体
        this.drawSquircle(btnX, btnY, btnW, btnH, radius);
        this.ctx.fill();

        // 2. Hover 亮度遮罩 (基于 factor)
        if (factor > 0 && !isClick) {
            this.ctx.fillStyle = `rgba(255, 255, 255, ${0.1 * factor})`;
            this.drawSquircle(btnX, btnY, btnW, btnH, radius);
            this.ctx.fill();
        }

        // 3. 加上描边 (Inside 描边实现)
        this.ctx.save();
        this.drawSquircle(btnX, btnY, btnW, btnH, radius);
        this.ctx.clip();
        this.ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)'; // 描边透明度改回 30%
        this.ctx.lineWidth = this.scaleSize(3) * 2; // 2倍线宽，因为 clip 掉了一半
        this.ctx.stroke();
        this.ctx.restore();

        // 4. 抗锯齿补强：在边缘再次绘制一层极细的描边，平滑 clip 导致的锯齿
        this.ctx.save();
        this.ctx.strokeStyle = 'rgba(255, 255, 255, 0.15)'; // 极低透明度
        this.ctx.lineWidth = 1; // 物理 1px
        this.drawSquircle(btnX, btnY, btnW, btnH, radius);
        this.ctx.stroke();
        this.ctx.restore();
        
        // 绘制文字：确保文字本身没有阴影效果
        this.ctx.shadowBlur = 0;
        this.ctx.shadowColor = 'transparent';
        this.ctx.shadowOffsetY = 0;
        
        this.ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';
        this.ctx.font = `${finalFontSize}px 'AlumniSans'`;
        this.ctx.fillText(text, Math.round(pos.x), Math.round(pos.y + finalFontSize * 0.05));
        this.ctx.restore();
    }

    /**
     * 绘制超椭圆 (Squircle)
     * 使用三次贝塞尔曲线近似超椭圆，解决普通圆角的“生硬”感，同时边缘更平滑
     */
    private drawSquircle(x: number, y: number, width: number, height: number, radius: number): void {
        this.ctx.beginPath();
        this.ctx.moveTo(x + radius, y);
        this.ctx.lineTo(x + width - radius, y);
        this.ctx.bezierCurveTo(x + width, y, x + width, y, x + width, y + radius);
        this.ctx.lineTo(x + width, y + height - radius);
        this.ctx.bezierCurveTo(x + width, y + height, x + width, y + height, x + width - radius, y + height);
        this.ctx.lineTo(x + radius, y + height);
        this.ctx.bezierCurveTo(x, y + height, x, y + height, x, y + height - radius);
        this.ctx.lineTo(x, y + radius);
        this.ctx.bezierCurveTo(x, y, x, y, x + radius, y);
        this.ctx.closePath();
    }

    public handlePoint(x: number, y: number): boolean {
        for (const btn of this.buttons.values()) {
            if (x >= btn.x && x <= btn.x + btn.w && y >= btn.y && y <= btn.y + btn.h) {
                btn.callback();
                return true;
            }
        }
        return false;
    }

    render(gameState: GameState | null, mousePos?: { x: number, y: number }, isMouseDown?: boolean, showCursor: boolean = true): void {
        this.ctx.imageSmoothingEnabled = true;
        this.ctx.imageSmoothingQuality = 'high';
        
        this.buttons.clear();
        this.drawBackground();

        if (!gameState) {
            this.drawText("CONNECTING...", 1920 / 2, 1080 / 2, 24, "#FFFFFF");
            return;
        }

        if (gameState.is_start_screen) {
            this.drawStartScreen(mousePos, isMouseDown);
        } else if (gameState.is_game_over) {
            this.drawGameOver(mousePos, isMouseDown);
        } else {
            // 倒计时期间（is_transitioning 为 true）隐藏小球
            if (!gameState.is_transitioning) {
                const ballPos = this.scalePos(gameState.ball.x, gameState.ball.y);
                const ballRadius = this.scaleSize(gameState.ball.radius);
                this.drawBallAt(ballPos.x, ballPos.y, ballRadius);
            }
            
            if (gameState.stage === 5 && gameState.stage5_paused) {
                this.drawStage5Pause();
            }
            
            this.drawUI(gameState);
        }

        // 重新检查鼠标是否在任何按钮上（此时 this.buttons 已经由 drawButton 填充完毕）
        let hoveringAny = false;
        if (mousePos) {
            for (const btn of this.buttons.values()) {
                if (mousePos.x >= btn.x && mousePos.x <= btn.x + btn.w && 
                    mousePos.y >= btn.y && mousePos.y <= btn.y + btn.h) {
                    hoveringAny = true;
                    break;
                }
            }
        }
        
        if (!showCursor) {
            this.canvas.style.cursor = 'none';
        } else {
            this.canvas.style.cursor = hoveringAny ? 'pointer' : 'default';
        }
    }

    private drawBackground(): void {
        const w = window.innerWidth;
        const h = window.innerHeight;
        const scale = this.getScale();
        
        // 强制背景网格为正方形，不随屏幕拉伸变形
        const gridSize = 80 * scale;
        
        const color1 = this.colorToCSS(this.theme.background.grid_color_dark);
        const color2 = this.colorToCSS(this.theme.background.grid_color_light);
        
        // 计算偏移量以使网格在屏幕中心对称
        const offsetX = (w % gridSize) / 2;
        const offsetY = (h % gridSize) / 2;
        
        for (let x = -gridSize; x < w + gridSize; x += gridSize) {
            for (let y = -gridSize; y < h + gridSize; y += gridSize) {
                // 使用固定的逻辑坐标计算颜色，确保网格大小一致且位置稳定
                const gridX = Math.round((x - offsetX) / gridSize);
                const gridY = Math.round((y - offsetY) / gridSize);
                const isEven = (gridX + gridY) % 2 === 0;
                
                this.ctx.fillStyle = isEven ? color1 : color2;
                // 使用 Math.floor 和 +1 这种方式，或者 Math.round，避免亚像素缝隙导致的锯齿感
                const drawX = Math.floor(x);
                const drawY = Math.floor(y);
                const drawSize = Math.ceil(gridSize + (x - drawX));
                this.ctx.fillRect(drawX, drawY, drawSize, drawSize);
            }
        }
    }

    private drawBallAt(x: number, y: number, radius: number): void {
        this.ctx.save();
        const gradient = this.ctx.createLinearGradient(x - radius, y - radius, x + radius, y + radius);
        gradient.addColorStop(0, this.colorToCSS(this.theme.ball.gradient_start));
        gradient.addColorStop(1, this.colorToCSS(this.theme.ball.gradient_end));
        
        this.ctx.beginPath();
        this.ctx.arc(x, y, radius, 0, Math.PI * 2);
        this.ctx.fillStyle = gradient;
        this.ctx.fill();
        
        this.ctx.strokeStyle = this.colorToCSS(this.theme.ball.outline_color);
        this.ctx.lineWidth = 2;
        this.ctx.stroke();
        this.ctx.restore();
    }

    private drawStartScreen(mousePos?: { x: number, y: number }, isMouseDown?: boolean): void {
        this.drawText('EYE MOTION', 1920 / 2, 400, 200, '#EBBF42'); 
        
        this.drawButton(
            'start-btn', 
            'START', 
            1920 / 2, 
            560, 
            64,  
            35,  
            16,  // 上下填充增加到 16px
            ['#FC6170', '#E62538'],
            () => (window as any).game.startGame(),
            mousePos,
            isMouseDown
        );
    }

    private drawGameOver(mousePos?: { x: number, y: number }, isMouseDown?: boolean): void {
        // "GAME OVER" 字号翻倍：120 -> 240，位置微调
        this.drawText("GAME OVER", 1920 / 2, 1080 / 2 - 250, 240, "#FC6170");
        
        // RESTART 按钮，位置下移：150 -> 200
        this.drawButton(
            'restart-btn', 
            'RESTART', 
            1920 / 2, 
            1080 / 2 + 200, 
            64, 
            35, 
            16,
            ['#EBBF42', '#E68325'],
            () => (window as any).game.restartGame(),
            mousePos,
            isMouseDown
        );

        // QUIT 按钮，距离 RESTART 120px (相对)
        this.drawButton(
            'quit-btn', 
            'QUIT', 
            1920 / 2, 
            1080 / 2 + 200 + 120, 
            64, 
            35, 
            16,
            ['#41C6F0', '#108FDF'],
            () => (window as any).game.quitGame(),
            mousePos,
            isMouseDown
        );
    }

    private drawUI(state: GameState): void {
        this.drawStats(state);
        
        if (state.paused) {
            this.ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
            this.ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
            // 暂停提示移至画面上方 1/3 处
            this.drawText('PAUSED', 1920 / 2, 1080 / 3, 100, '#EBBF42');
            
            this.buttons.set('resume-area', { 
                x: 0, y: 0, w: window.innerWidth, h: window.innerHeight, 
                callback: () => (window as any).game.togglePause() 
            });
        }

        if (state.is_transitioning) {
            const count = Math.ceil(state.transition_timer);
            // "STAGE *" 渐变色处理
            const stageText = `STAGE ${state.stage}`;
            const pos = this.scalePos(1920 / 2, 520);
            const size = 140;
            const finalSize = Math.round(this.scaleSize(size));
            
            // 创建线性渐变：从上到下 (#41C6F0 -> #108FDF)
            const gradient = this.ctx.createLinearGradient(
                pos.x, pos.y - finalSize / 2, 
                pos.x, pos.y + finalSize / 2
            );
            gradient.addColorStop(0, '#41C6F0');
            gradient.addColorStop(1, '#108FDF');
            
            this.drawText(stageText, 1920 / 2, 520, size, gradient);
            
            // Stage 倒计时数字保持原有黄色
            this.drawText(`${count}`, 1920 / 2, 700, 160, '#EBBF42');
        }
    }

    private drawStats(gameState: GameState): void {
        const color = '#40C5EF';
        const fontSize = 32;
        const padding = 60;
        
        const stageText = `STAGE: ${gameState.stage}`;
        const timeVal = Math.floor(gameState.stage_elapsed).toString().padStart(2, '0');
        
        this.drawText(stageText, padding, padding, fontSize, color, 'left');
        
        // "TIME: " 标签，右对齐在 1765
        this.drawText("TIME: ", 1765, padding, fontSize, color, 'right');
        
        // 数字紧跟在 "TIME: " 后面，极致缩小间距 (1765 -> 1768)
        this.drawText(timeVal, 1768, padding, fontSize, color, 'left');
        
        // 单位 "S" 紧随其后，数字 00 的宽度在 AlumniSans 下约 28-30px，所以 1798 比较紧凑
        this.drawText("S", 1798, padding, fontSize, color, 'left');
    }

    private drawStage5Pause(): void {
        return;
    }
}
