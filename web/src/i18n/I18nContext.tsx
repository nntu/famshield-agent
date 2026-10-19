import React, { createContext, useContext, useState, useEffect } from 'react';
import { vi, TranslationKeys } from './locales/vi';
import { en } from './locales/en';

export type SupportedLocale = 'vi' | 'en';

interface I18nContextProps {
  locale: SupportedLocale;
  setLocale: (locale: SupportedLocale) => void;
  t: (key: TranslationKeys, params?: Record<string, string | number>) => string;
}

const translations: Record<SupportedLocale, Record<TranslationKeys, string>> = {
  vi,
  en,
};

const I18nContext = createContext<I18nContextProps | undefined>(undefined);

const STORAGE_KEY = 'famshield_locale';

export const I18nProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // Default language is Vietnamese ('vi')
  const [locale, setLocaleState] = useState<SupportedLocale>(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === 'en' || saved === 'vi') {
      return saved;
    }
    return 'vi'; // Default to Tiếng Việt
  });

  const setLocale = (newLocale: SupportedLocale) => {
    setLocaleState(newLocale);
    localStorage.setItem(STORAGE_KEY, newLocale);
    document.documentElement.lang = newLocale;
  };

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const t = (key: TranslationKeys, params?: Record<string, string | number>): string => {
    const dict = translations[locale] || translations.vi;
    let text = dict[key] || translations.vi[key] || key;

    if (params) {
      Object.entries(params).forEach(([paramKey, paramVal]) => {
        text = text.replace(new RegExp(`\\{${paramKey}\\}`, 'g'), String(paramVal));
      });
    }

    return text;
  };

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  );
};

export const useI18n = (): I18nContextProps => {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within an I18nProvider');
  }
  return context;
};
