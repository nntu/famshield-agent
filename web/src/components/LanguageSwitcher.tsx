import React from 'react';
import { useI18n } from '../i18n/I18nContext';

export const LanguageSwitcher: React.FC = () => {
  const { locale, setLocale, t } = useI18n();

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
      <span style={{ fontSize: '0.85rem', color: '#94a3b8' }}>🌐 {t('language')}:</span>
      <div style={{ display: 'inline-flex', background: '#1e293b', borderRadius: '8px', padding: '3px', border: '1px solid #334155' }}>
        <button
          onClick={() => setLocale('vi')}
          style={{
            padding: '4px 10px',
            borderRadius: '6px',
            border: 'none',
            fontSize: '0.8rem',
            fontWeight: locale === 'vi' ? 600 : 400,
            background: locale === 'vi' ? '#38bdf8' : 'transparent',
            color: locale === 'vi' ? '#0f172a' : '#94a3b8',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            transition: 'all 0.15s ease',
          }}
          title="Tiếng Việt (Ngôn ngữ mặc định)"
        >
          <span>🇻🇳</span> Tiếng Việt
        </button>
        <button
          onClick={() => setLocale('en')}
          style={{
            padding: '4px 10px',
            borderRadius: '6px',
            border: 'none',
            fontSize: '0.8rem',
            fontWeight: locale === 'en' ? 600 : 400,
            background: locale === 'en' ? '#38bdf8' : 'transparent',
            color: locale === 'en' ? '#0f172a' : '#94a3b8',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            transition: 'all 0.15s ease',
          }}
          title="English"
        >
          <span>🇺🇸</span> English
        </button>
      </div>
    </div>
  );
};
