class InputHandler {
    constructor() {
        this.eventListeners = [];
        this.setup();
    }

    setup() {
        this.handleKeydown = this.handleKeydown.bind(this);
        this.handleTouchstart = this.handleTouchstart.bind(this);
        
        document.addEventListener('keydown', this.handleKeydown);
        document.addEventListener('touchstart', this.handleTouchstart);
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

    emit(event, data) {
        const callbacks = this.eventListeners[event];
        if (callbacks) {
            callbacks.forEach(callback => callback(data));
        }
    }

    destroy() {
        document.removeEventListener('keydown', this.handleKeydown);
        document.removeEventListener('touchstart', this.handleTouchstart);
    }
}
