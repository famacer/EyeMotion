import { Bridge } from './bridge';

export class I18n {
    private currentLocale: string;
    private translations: any = {};
    private ready: boolean = false;

    constructor() {
        this.currentLocale = this.detectLocale();
        this.loadTranslations(this.currentLocale);
    }

    private detectLocale(): string {
        const browserLang = navigator.language || (navigator as any).userLanguage || 'en';
        if (browserLang && browserLang.toLowerCase().startsWith('zh')) {
            const lang = browserLang.toLowerCase();
            if (lang.includes('hans') || lang.includes('cn')) {
                return 'zh-Hans';
            } else if (lang.includes('hant') || lang.includes('tw') || lang.includes('hk')) {
                return 'zh-Hant';
            }
        }
        return 'en';
    }

    private loadTranslations(locale: string): void {
        const url = `./locales/${locale}.json`;
        fetch(url)
            .then(response => response.json())
            .then(data => {
                this.translations = data;
                this.currentLocale = locale;
                this.ready = true;
                document.dispatchEvent(new CustomEvent('i18n-ready', { detail: { locale } }));
                document.dispatchEvent(new CustomEvent('locale-changed', { detail: { locale } }));
            })
            .catch(() => {
                if (locale !== 'en') this.loadTranslations('en');
                else {
                    this.translations = {};
                    this.ready = true;
                }
            });
    }

    public t(key: string, params: { [key: string]: any } = {}): string {
        const keys = key.split('.');
        let value = this.translations;
        
        for (const k of keys) {
            if (value && typeof value === 'object') value = value[k];
            else return key;
        }
        
        if (typeof value === 'string') {
            return value.replace(/\{(\w+)\}/g, (_, param) => params[param] || '');
        }
        return key;
    }

    public async setLocale(locale: string): Promise<void> {
        this.currentLocale = locale;
        this.loadTranslations(locale);
        try {
            await Bridge.invoke('set_language', { language: locale });
        } catch (e) {
            console.error('I18n: Failed to save language:', e);
        }
    }

    public getLocale(): string {
        return this.currentLocale;
    }
}

export const i18n = new I18n();
export const t = (key: string, params?: any) => i18n.t(key, params);
export const setLocale = (locale: string) => i18n.setLocale(locale);
