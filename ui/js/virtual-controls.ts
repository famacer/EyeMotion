import { t, setLocale } from './i18n';
import { Bridge } from './bridge';

export class VirtualControls {
    private container: HTMLDivElement;
    private pauseButton!: HTMLButtonElement;
    private restartButton!: HTMLButtonElement;
    private languageButton!: HTMLButtonElement;
    private isMobile: boolean;

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
    
    private createButtons(): void {
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
    
    private createButton(textKey: string, onClick: () => void): HTMLButtonElement {
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
    
    private bindEvents(): void {
        document.addEventListener('locale-changed', () => this.updateText());
        document.addEventListener('game-state-changed', (e: any) => {
            const state = e.detail;
            if (this.pauseButton) {
                this.pauseButton.textContent = state.paused ? t('resume_button') : t('pause_button');
            }
        });
    }
    
    private updateText(): void {
        this.pauseButton.textContent = t('pause_button');
        this.restartButton.textContent = t('restart_button');
        this.languageButton.textContent = t('language');
    }
    
    private async togglePause(): Promise<void> {
        await Bridge.togglePause();
    }
    
    private async restart(): Promise<void> {
        await Bridge.resetGame(window.innerWidth, window.innerHeight);
    }
    
    private showLanguageMenu(): void {
        const menu = document.getElementById('language-menu');
        if (menu) menu.classList.toggle('visible');
    }
}

export function createLanguageMenu(): void {
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
    closeButton.textContent = t('close') || 'Close';
    closeButton.addEventListener('click', () => {
        menu.classList.remove('visible');
    });
    
    const footer = document.createElement('div');
    footer.className = 'language-menu-footer';
    footer.appendChild(closeButton);
    
    menu.appendChild(footer);
    document.body.appendChild(menu);
}
