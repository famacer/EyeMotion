class I18n {
    constructor() {
        this.currentLocale = this.detectLocale();
        this.translations = {};
        this.ready = false;
        
        this.loadTranslations(this.currentLocale);
    }

    detectLocale() {
        const browserLang = navigator.language || navigator.userLanguage;
        if (browserLang && browserLang.startsWith('zh')) {
            const lang = browserLang.toLowerCase();
            if (lang.includes('hans') || lang.includes('cn')) {
                return 'zh-Hans';
            } else if (lang.includes('hant') || lang.includes('tw') || lang.includes('hk')) {
                return 'zh-Hant';
            }
        }
        return 'en';
    }

    loadTranslations(locale) {
        fetch(`./locales/${locale}.json`)
            .then(response => response.json())
            .then(data => {
                this.translations = data;
                this.currentLocale = locale;
                this.ready = true;
                console.log(`I18n: Loaded locale "${locale}"`);
                document.dispatchEvent(new CustomEvent('i18n-ready'));
            })
            .catch(e => {
                console.error('I18n: Failed to load locale, using fallback', e);
                if (locale !== 'en') {
                    this.loadTranslations('en');
                } else {
                    this.translations = {};
                    this.ready = true;
                }
            });
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

    setLocale(locale) {
        this.currentLocale = locale;
        this.loadTranslations(locale);
        
        if (window.__TAURI__) {
            window.__TAURI__.core.invoke('set_language', { language: locale })
                .catch(e => console.error('I18n: Failed to save language', e));
        }
    }

    getLocale() {
        return this.currentLocale;
    }

    isReady() {
        return this.ready;
    }
}

const i18n = new I18n();
window.t = (key, params) => i18n.t(key, params);
window.setLocale = (locale) => i18n.setLocale(locale);
window.getLocale = () => i18n.getLocale();
window.i18nReady = () => i18n.isReady();
