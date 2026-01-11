class InputHandler {
    constructor() {
        this.eventListeners = [];
        this.setup();
    }

    setup() {
        this.handleKeydown = this.handleKeydown.bind(this);
        this.handleTouchstart = this.handleTouchstart.bind(this);
        this.handleWindowControlClick = this.handleWindowControlClick.bind(this);
        
        document.addEventListener('keydown', this.handleKeydown);
        document.addEventListener('touchstart', this.handleTouchstart);
        
        const minBtn = document.getElementById('ctrl-min');
        const maxBtn = document.getElementById('ctrl-max');
        const closeBtn = document.getElementById('ctrl-close');
        
        if (minBtn) minBtn.addEventListener('click', this.handleWindowControlClick);
        if (maxBtn) maxBtn.addEventListener('click', this.handleWindowControlClick);
        if (closeBtn) closeBtn.addEventListener('click', this.handleWindowControlClick);
        
        this.hideWindowControlsOnMobile();
        
        window.addEventListener('resize', () => this.hideWindowControlsOnMobile());
    }

    hideWindowControlsOnMobile() {
        const isMobile = window.innerWidth <= 768;
        const windowControls = document.getElementById('window-controls');
        if (windowControls) {
            windowControls.style.display = isMobile ? 'none' : 'flex';
        }
    }

    on(event, callback) {
        if (!this.eventListeners[event]) {
            this.eventListeners[event] = [];
        }
        this.eventListeners[event].push(callback);
    }

    handleKeydown(e) {
        const key = e.code;
        this.emit('keydown', key);
    }

    handleTouchstart(e) {
        this.emit('touch', e);
    }

    handleWindowControlClick(e) {
        const btnId = e.target.id;
        console.log('Window control click:', btnId);
        this.emit('window-control', btnId);
    }

    emit(event, data) {
        const callbacks = this.eventListeners[event];
        if (callbacks) {
            callbacks.forEach(callback => callback(data));
        }
    }

    destroy() {
        document.removeEventListener('keydown', this.handleKeydown);
        document.removeEventListener('touchstart', this.handleTouchstart);
        
        const minBtn = document.getElementById('ctrl-min');
        const maxBtn = document.getElementById('ctrl-max');
        const closeBtn = document.getElementById('ctrl-close');
        
        if (minBtn) minBtn.removeEventListener('click', this.handleWindowControlClick);
        if (maxBtn) maxBtn.removeEventListener('click', this.handleWindowControlClick);
        if (closeBtn) closeBtn.removeEventListener('click', this.handleWindowControlClick);
    }
}
