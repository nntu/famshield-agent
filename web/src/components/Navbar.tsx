import React from 'react';
import { Shield, LogOut, CheckCircle2 } from 'lucide-react';
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
    <header className="bg-white/95 backdrop-blur-md border-b border-slate-200 sticky top-0 z-40 shadow-xs">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-3 flex items-center justify-between flex-wrap gap-3">
        {/* Brand */}
        <div className="flex items-center gap-3.5">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-sky-500 to-blue-600 flex items-center justify-center shadow-md shadow-sky-500/20 text-white">
            <Shield size={22} strokeWidth={2.5} />
          </div>
          <div>
            <div className="text-xl font-extrabold tracking-tight text-slate-900 leading-none">
              FamShield
            </div>
            <div className="text-xs text-slate-500 mt-1 font-medium">{t.app_subtitle}</div>
          </div>

          <span
            className={`px-2.5 py-0.5 rounded-full text-xs font-semibold flex items-center gap-1.5 border ml-1 ${
              operatingMode === 'StandaloneFree'
                ? 'bg-sky-50 text-sky-700 border-sky-200'
                : 'bg-purple-50 text-purple-700 border-purple-200'
            }`}
          >
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                operatingMode === 'StandaloneFree' ? 'bg-sky-500' : 'bg-purple-500'
              }`}
            />
            {operatingMode}
          </span>
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-3">
          {/* Language Selector */}
          <div className="inline-flex bg-slate-100 rounded-lg p-0.5 border border-slate-200">
            <button
              onClick={() => onLangChange('vi')}
              className={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all ${
                lang === 'vi'
                  ? 'bg-white text-sky-600 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              🇻🇳 VN
            </button>
            <button
              onClick={() => onLangChange('en')}
              className={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all ${
                lang === 'en'
                  ? 'bg-white text-sky-600 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              🇺🇸 EN
            </button>
          </div>

          {isAuthenticated ? (
            <>
              <div className="flex items-center gap-1.5 bg-emerald-50 text-emerald-700 border border-emerald-200 px-3 py-1.5 rounded-lg text-xs font-semibold">
                <CheckCircle2 size={14} />
                <span>{t.status_authenticated}</span>
              </div>
              <button
                onClick={onLogout}
                className="flex items-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-300 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors"
              >
                <LogOut size={14} />
                <span>{t.btn_logout}</span>
              </button>
            </>
          ) : null}
        </div>
      </div>

      {/* Tabs navigation */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 flex gap-1 overflow-x-auto border-t border-slate-100">
        {tabs.map((tab) => {
          const isActive = currentTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`flex items-center gap-2 px-4 py-3 text-sm font-semibold border-b-2 transition-all whitespace-nowrap cursor-pointer ${
                isActive
                  ? 'border-sky-600 text-sky-600 bg-sky-50/50'
                  : 'border-transparent text-slate-600 hover:text-slate-900 hover:bg-slate-50'
              }`}
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
