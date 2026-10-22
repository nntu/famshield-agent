import { CheckCircle2, LogOut, Shield } from 'lucide-react';
import type React from 'react';
import { translations } from '../i18n';
import type { Language, TabType } from '../types';

interface NavbarProps {
  currentTab: TabType;
  onTabChange: (tab: TabType) => void;
  lang: Language;
  onLangChange: (lang: Language) => void;
  operatingMode: string;
  appVersion?: string;
  updateAvailable?: boolean;
  latestVersion?: string;
  onLogout: () => void;
  isAuthenticated: boolean;
}

export const Navbar: React.FC<NavbarProps> = ({
  currentTab,
  onTabChange,
  lang,
  onLangChange,
  operatingMode,
  appVersion,
  updateAvailable,
  latestVersion,
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
    <header className="w-full bg-white border-b border-slate-200/80 sticky top-0 z-40 shadow-xs">
      <div className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-3 flex items-center justify-between flex-wrap gap-3">
        {/* Brand */}
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg bg-sky-600 flex items-center justify-center text-white shadow-xs shrink-0">
            <Shield size={20} strokeWidth={2.5} />
          </div>
          <div>
            <div className="text-lg font-bold tracking-tight text-slate-900 leading-none">
              FamShield
            </div>
            <div className="text-2xs text-slate-500 mt-1 font-medium">{t.app_subtitle}</div>
          </div>

          <span
            className={`px-2.5 py-0.5 rounded-full text-xs font-semibold flex items-center gap-1.5 border ml-1.5 ${
              operatingMode === 'StandaloneFree'
                ? 'bg-sky-50 text-sky-700 border-sky-200/80'
                : 'bg-sky-100 text-sky-800 border-sky-200/80'
            }`}
          >
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                operatingMode === 'StandaloneFree' ? 'bg-sky-500' : 'bg-sky-700'
              }`}
            />
            {operatingMode}
          </span>

          <button
            type="button"
            onClick={() => onTabChange('security')}
            title={
              updateAvailable
                ? lang === 'vi'
                  ? `Có bản cập nhật mới v${latestVersion}! Bấm để mở Cài đặt & Cập nhật.`
                  : `New update v${latestVersion} available! Click to open Security & Updates.`
                : lang === 'vi'
                  ? `Phiên bản hiện tại: v${appVersion || '1.0.1'}. Bấm để kiểm tra bản mới.`
                  : `Current version: v${appVersion || '1.0.1'}. Click to check updates.`
            }
            className={`px-2.5 py-0.5 rounded-full text-xs font-mono font-medium flex items-center gap-1.5 transition-all cursor-pointer ${
              updateAvailable
                ? 'bg-amber-50 text-amber-900 border border-amber-300 hover:bg-amber-100 shadow-2xs animate-pulse font-semibold'
                : 'bg-slate-100 text-slate-600 hover:text-slate-900 hover:bg-slate-200/80 border border-slate-200/80'
            }`}
          >
            {updateAvailable ? (
              <>
                <span aria-hidden="true">🚀</span>
                <span>v{appVersion || '1.0.1'}</span>
                <span className="text-2xs font-sans font-semibold text-amber-700 ml-0.5">
                  ({lang === 'vi' ? 'Bản mới' : 'Update'} v{latestVersion})
                </span>
              </>
            ) : (
              <span>v{appVersion || '1.0.1'}</span>
            )}
          </button>
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-2.5">
          {/* Language Selector */}
          <div className="inline-flex bg-slate-100 rounded-lg p-0.5 border border-slate-200/80">
            <button
              onClick={() => onLangChange('vi')}
              aria-pressed={lang === 'vi'}
              aria-label="Tiếng Việt"
              className={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all cursor-pointer ${
                lang === 'vi'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-500 hover:text-slate-900'
              }`}
            >
              🇻🇳 VN
            </button>
            <button
              onClick={() => onLangChange('en')}
              aria-pressed={lang === 'en'}
              aria-label="English"
              className={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all cursor-pointer ${
                lang === 'en'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-500 hover:text-slate-900'
              }`}
            >
              🇺🇸 EN
            </button>
          </div>

          {isAuthenticated ? (
            <>
              <div className="flex items-center gap-1.5 bg-emerald-50 text-emerald-700 border border-emerald-200/80 px-2.5 py-1 rounded-lg text-xs font-semibold">
                <CheckCircle2 size={13} className="text-emerald-600" />
                <span>{t.status_authenticated}</span>
              </div>
              <button
                onClick={onLogout}
                className="flex items-center gap-1.5 bg-white hover:bg-slate-50 text-slate-600 hover:text-slate-900 border border-slate-200/80 px-2.5 py-1 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
              >
                <LogOut size={13} />
                <span>{t.btn_logout}</span>
              </button>
            </>
          ) : null}
        </div>
      </div>

      {/* Tabs navigation */}
      <div className="border-t border-slate-100 bg-white">
        <div
          role="tablist"
          aria-label={lang === 'vi' ? 'Điều hướng chính' : 'Primary navigation'}
          className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 flex gap-1 overflow-x-auto"
        >
          {tabs.map((tab) => {
            const isActive = currentTab === tab.id;
            return (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={isActive}
                onClick={() => onTabChange(tab.id)}
                className={`flex items-center gap-2 px-3.5 py-2.5 text-xs sm:text-sm font-semibold border-b-2 transition-all whitespace-nowrap cursor-pointer ${
                  isActive
                    ? 'border-sky-600 text-sky-700 bg-sky-50/50 font-bold'
                    : 'border-transparent text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                }`}
                style={{
                  borderTop: 'none',
                  borderLeft: 'none',
                  borderRight: 'none',
                }}
              >
                <span aria-hidden="true">{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            );
          })}
        </div>
      </div>
    </header>
  );
};
