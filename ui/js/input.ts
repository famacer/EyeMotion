export class InputHandler {
    private listeners: Map<string, Array<(data?: any) => void>> = new Map();

    constructor() {
        this.init();
    }

    private init(): void {
        // Global click/touch/mouse move handler for game buttons and interaction
        window.addEventListener('mousedown', (e) => {
            this.emit('mousedown', { x: e.clientX, y: e.clientY });
            this.emit('click', { x: e.clientX, y: e.clientY });
        });

        window.addEventListener('mouseup', (e) => {
            this.emit('mouseup', { x: e.clientX, y: e.clientY });
        });

        window.addEventListener('mousemove', (e) => {
            this.emit('mousemove', { x: e.clientX, y: e.clientY });
        });

        // Touch handling
        let touchStartPos: { x: number, y: number } | null = null;
        let touchStartTime: number = 0;

        window.addEventListener('touchstart', (e) => {
            if (e.touches.length > 0) {
                const touch = e.touches[0];
                touchStartPos = { x: touch.clientX, y: touch.clientY };
                touchStartTime = Date.now();
                this.emit('mousedown', { x: touch.clientX, y: touch.clientY });
            }
        }, { passive: false });

        window.addEventListener('touchmove', (e) => {
            if (e.touches.length > 0) {
                const touch = e.touches[0];
                this.emit('mousemove', { x: touch.clientX, y: touch.clientY });
            }
        }, { passive: false });

        window.addEventListener('touchend', (e) => {
            this.emit('mouseup');
            
            if (touchStartPos) {
                const touch = e.changedTouches[0];
                const dx = touch.clientX - touchStartPos.x;
                const dy = touch.clientY - touchStartPos.y;
                const distance = Math.sqrt(dx * dx + dy * dy);
                const duration = Date.now() - touchStartTime;

                // Relaxed threshold for mobile: 30px distance, 800ms duration
                if (distance < 30 && duration < 800) {
                    // Prevent default to avoid double-firing with mouse events
                    if (e.cancelable) e.preventDefault();
                    this.emit('click', { x: touch.clientX, y: touch.clientY });
                }
                
                touchStartPos = null;
            }
        });

        // Keyboard events for system/debug shortcuts
        window.addEventListener('keydown', (e) => {
            // Debug skip stage shortcut: Ctrl + Shift + . (Period)
            // 确保在任何键盘布局下都能触发，检查 key 或 code
            if (e.ctrlKey && e.shiftKey && (e.key === '.' || e.key === '。' || e.code === 'Period')) {
                e.preventDefault();
                this.emit('debug-skip-stage');
                return;
            }

            // Debug prev stage shortcut: Ctrl + Shift + , (Comma)
            if (e.ctrlKey && e.shiftKey && (e.key === ',' || e.key === '，' || e.code === 'Comma')) {
                e.preventDefault();
                this.emit('debug-prev-stage');
                return;
            }
        });

        // Window control buttons (custom titlebar)
        document.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const btn = target.closest('.win-btn');
            if (btn) {
                this.emit('window-control', btn.id);
            }
        });
    }

    public isKeyDown(_code: string): boolean {
        return false;
    }

    public on(event: string, callback: (data?: any) => void): void {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event)?.push(callback);
    }

    private emit(event: string, data?: any): void {
        this.listeners.get(event)?.forEach(cb => cb(data));
    }
}
