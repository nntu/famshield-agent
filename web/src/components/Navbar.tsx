import React from 'react';
import { Shield, Lock, LogOut, CheckCircle2, Globe } from 'lucide-react';
import { Language, TabType } from '../types';
import { translations } from '../i18n';

interface NavbarProps {
  currentTab: TabType;
  onTabChange: (tab: TabType) => void;
  lang: Language;
  onLangChange: (lang: Language) => void;
  operatingMode: string;
  onLogout: () => void;
  isAuthenticated: boolean;
}

export const Navbar: React.FC<NavbarProps> = ({
  currentTab,
  onTabChange,
  lang,
  onLangChange,
  operatingMode,
  onLogout,
  isAuthenticated,
}) => {
  const t = translations[lang];

  const tabs: { id: TabType; label: string; icon: string }[] = [
    { id: 'dashboard', label: t.nav_dashboard, icon: '📊' },
    { id: 'activity', label: t.nav_activity, icon: '⏱️' },
    { id: 'rules', label: t.nav_rules, icon: '🛡️' },
    { id: 'logs', label: t.nav_logs, icon: '📋' },
    { id: 'evidence', label: t.nav_evidence, icon: '🖼️' },
    { id: 'security', label: t.nav_security, icon: '⚙️' },
  ];

  return (
    <header style={{
      background: 'rgba(15, 23, 42, 0.95)',
      backdropFilter: 'blur(12px)',
      borderBottom: '1px solid #1e293b',
      position: 'sticky',
      top: 0,
      zIndex: 50,
      boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
    }}>
      <div style={{
        maxWidth: 1400,
        margin: '0 auto',
        padding: '12px 24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        flexWrap: 'wrap',
        gap: 12,
      }}>
        {/* Brand */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
          <div style={{
            width: 40,
            height: 40,
            borderRadius: 10,
            background: 'linear-gradient(135deg, #0284c7, #38bdf8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 0 20px rgba(56, 189, 248, 0.4)',
          }}>
            <Shield size={22} color="#ffffff" strokeWidth={2.5} />
          </div>
          <div>
            <div style={{ fontSize: '1.25rem', fontWeight: 800, letterSpacing: '-0.02em', color: '#f8fafc' }}>
              FamShield
            </div>
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>{t.app_subtitle}</div>
          </div>

          <span style={{
            background: operatingMode === 'StandaloneFree' ? 'rgba(56, 189, 248, 0.12)' : 'rgba(168, 85, 247, 0.15)',
            color: operatingMode === 'StandaloneFree' ? '#38bdf8' : '#c084fc',
            border: `1px solid ${operatingMode === 'StandaloneFree' ? 'rgba(56, 189, 248, 0.3)' : 'rgba(168, 85, 247, 0.4)'}`,
            padding: '3px 10px',
            borderRadius: 9999,
            fontSize: '0.75rem',
            fontWeight: 700,
            display: 'flex',
            alignItems: 'center',
            gap: 6,
          }}>
            <span style={{
              width: 6,
              height: 6,
              borderRadius: '50%',
              background: operatingMode === 'StandaloneFree' ? '#38bdf8' : '#c084fc',
              boxShadow: `0 0 8px ${operatingMode === 'StandaloneFree' ? '#38bdf8' : '#c084fc'}`,
            }} />
            {operatingMode}
          </span>
        </div>

        {/* Right Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          {/* Language Selector */}
          <div style={{
            display: 'inline-flex',
            background: '#090d16',
            borderRadius: 8,
            padding: 3,
            border: '1px solid #1e293b',
          }}>
            <button
              onClick={() => onLangChange('vi')}
              style={{
                background: lang === 'vi' ? '#38bdf8' : 'transparent',
                color: lang === 'vi' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: lang === 'vi' ? 700 : 500,
                cursor: 'pointer',
                transition: 'all 0.15s ease',
              }}
            >
              🇻🇳 VN
            </button>
            <button
              onClick={() => onLangChange('en')}
              style={{
                background: lang === 'en' ? '#38bdf8' : 'transparent',
                color: lang === 'en' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: lang === 'en' ? 700 : 500,
                cursor: 'pointer',
                transition: 'all 0.15s ease',
              }}
            >
              🇺🇸 EN
            </button>
          </div>

          {isAuthenticated ? (
            <>
              <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                background: 'rgba(52, 211, 153, 0.12)',
                color: '#34d399',
                border: '1px solid rgba(52, 211, 153, 0.3)',
                padding: '6px 12px',
                borderRadius: 8,
                fontSize: '0.8rem',
                fontWeight: 600,
              }}>
                <CheckCircle2 size={15} />
                <span>{t.status_authenticated}</span>
              </div>
              <button
                onClick={onLogout}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  background: '#1e293b',
                  color: '#e2e8f0',
                  border: '1px solid #334155',
                  padding: '6px 12px',
                  borderRadius: 8,
                  fontSize: '0.8rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'background 0.15s',
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = '#334155')}
                onMouseLeave={(e) => (e.currentTarget.style.background = '#1e293b')}
              >
                <LogOut size={14} />
                <span>{t.btn_logout}</span>
              </button>
            </>
          ) : null}
        </div>
      </div>

      {/* Tabs navigation */}
      <div style={{
        maxWidth: 1400,
        margin: '0 auto',
        padding: '0 24px',
        display: 'flex',
        gap: 4,
        overflowX: 'auto',
      }}>
        {tabs.map((tab) => {
          const isActive = currentTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              style={{
                background: 'transparent',
                border: 'none',
                color: isActive ? '#38bdf8' : '#94a3b8',
                padding: '12px 16px',
                fontSize: '0.9rem',
                fontWeight: 600,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                borderBottom: `3px solid ${isActive ? '#38bdf8' : 'transparent'}`,
                transition: 'all 0.15s ease',
                whiteSpace: 'nowrap',
              }}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          );
        })}
      </div>
    </header>
  );
};
