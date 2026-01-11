class I18n {
    constructor() {
        this.currentLocale = this.detectLocale();
        this.translations = {};
        this.loadTranslations(this.currentLocale);
    }

    detectLocale() {
        const browserLang = navigator.language || navigator.userLanguage;
        if (browserLang.startsWith('zh')) {
            if (browserLang.includes('Hans') || browserLang.includes('CN')) {
                return 'zh-Hans';
            } else if (browserLang.includes('Hant') || browserLang.includes('TW') || browserLang.includes('HK')) {
                return 'zh-Hant';
            }
        }
        return 'en';
    }

    async loadTranslations(locale) {
        try {
            const response = await fetch(`./locales/${locale}.json`);
            this.translations = await response.json();
            this.currentLocale = locale;
        } catch (e) {
            console.error(`Failed to load locale ${locale}:`, e);
            if (locale !== 'en') {
                await this.loadTranslations('en');
            }
        }
    }

    t(key, params = {}) {
        const keys = key.split('.');
        let value = this.translations;
        
        for (const k of keys) {
            if (value && typeof value === 'object') {
                value = value[k];
            } else {
                return key;
            }
        }
        
        if (typeof value === 'string') {
            return value.replace(/\{(\w+)\}/g, (_, param) => params[param] || '');
        }
        
        return key;
    }

    async setLocale(locale) {
        await this.loadTranslations(locale);
        if (window.__TAURI__) {
            await window.__TAURI__.core.invoke('set_language', { language: locale });
        }
        document.dispatchEvent(new CustomEvent('locale-changed', { detail: { locale } }));
    }

    getLocale() {
        return this.currentLocale;
    }
}

const i18n = new I18n();
window.t = (key, params) => i18n.t(key, params);
window.setLocale = (locale) => i18n.setLocale(locale);
window.getLocale = () => i18n.getLocale();
