class VirtualControls {
    constructor() {
        this.container = document.createElement('div');
        this.container.id = 'virtual-controls';
        this.container.className = 'mobile-only';
        document.body.appendChild(this.container);
        
        this.createButtons();
        this.bindEvents();
        
        this.isMobile = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
        if (!this.isMobile) {
            this.container.style.display = 'none';
        }
    }
    
    createButtons() {
        const mainRow = document.createElement('div');
        mainRow.className = 'controls-row main-row';
        
        this.pauseButton = this.createButton('pause_button', () => this.togglePause());
        this.restartButton = this.createButton('restart_button', () => this.restart());
        
        mainRow.appendChild(this.pauseButton);
        mainRow.appendChild(this.restartButton);
        
        const secondaryRow = document.createElement('div');
        secondaryRow.className = 'controls-row';
        
        this.languageButton = this.createButton('language', () => this.showLanguageMenu());
        
        secondaryRow.appendChild(this.languageButton);
        
        this.container.appendChild(mainRow);
        this.container.appendChild(secondaryRow);
    }
    
    createButton(textKey, onClick) {
        const button = document.createElement('button');
        button.className = 'virtual-btn';
        button.textContent = t(textKey);
        button.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            onClick();
        });
        
        button.addEventListener('touchstart', (e) => {
            e.preventDefault();
            button.classList.add('active');
        });
        button.addEventListener('touchend', () => {
            button.classList.remove('active');
        });
        
        return button;
    }
    
    bindEvents() {
        document.addEventListener('locale-changed', () => this.updateText());
        document.addEventListener('game-state-changed', (e) => {
            const state = e.detail;
            if (state.paused) {
                this.pauseButton.textContent = t('resume_button');
            } else {
                this.pauseButton.textContent = t('pause_button');
            }
        });
    }
    
    updateText() {
        if (window.gameState) {
            if (window.gameState.paused) {
                this.pauseButton.textContent = t('resume_button');
            } else {
                this.pauseButton.textContent = t('pause_button');
            }
        } else {
            this.pauseButton.textContent = t('pause_button');
        }
        this.restartButton.textContent = t('restart_button');
        this.languageButton.textContent = t('language');
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
    
    showLanguageMenu() {
        const menu = document.getElementById('language-menu');
        if (menu) {
            menu.classList.toggle('visible');
        }
    }
}

function createLanguageMenu() {
    const menu = document.createElement('div');
    menu.id = 'language-menu';
    menu.className = 'language-menu';
    
    const locales = [
        { code: 'en', name: 'English' },
        { code: 'zh-Hans', name: '简体中文' },
        { code: 'zh-Hant', name: '繁體中文' }
    ];
    
    locales.forEach(locale => {
        const option = document.createElement('div');
        option.className = 'language-option';
        option.textContent = locale.name;
        option.addEventListener('click', () => {
            setLocale(locale.code);
            menu.classList.remove('visible');
        });
        menu.appendChild(option);
    });
    
    const closeButton = document.createElement('button');
    closeButton.textContent = t('close');
    closeButton.addEventListener('click', () => {
        menu.classList.remove('visible');
    });
    
    const footer = document.createElement('div');
    footer.className = 'language-menu-footer';
    footer.appendChild(closeButton);
    
    menu.appendChild(footer);
    document.body.appendChild(menu);
    
    menu.addEventListener('click', (e) => {
        if (e.target === menu) {
            menu.classList.remove('visible');
        }
    });
}
