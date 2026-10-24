import type React from 'react';
import { useCallback, useEffect, useState } from 'react';
import { api, setSessionToken } from './api';
import { ActivityTab } from './components/ActivityTab';
import { DashboardTab } from './components/DashboardTab';
import { EvidenceTab } from './components/EvidenceTab';
import { LogsTab } from './components/LogsTab';
import { Navbar } from './components/Navbar';
import { PinModal } from './components/PinModal';
import { RulesTab } from './components/RulesTab';
import { SecurityTab } from './components/SecurityTab';
import { SetupWizard } from './components/SetupWizard';
import type {
  EvidenceItem,
  Language,
  LocalRuleRecord,
  QueryLogRecord,
  StatusResponse,
  TabType,
} from './types';

export const App: React.FC = () => {
  const [lang, setLang] = useState<Language>(() => {
    return (localStorage.getItem('famshield_lang') as Language) || 'vi';
  });

  const [currentTab, setCurrentTab] = useState<TabType>('dashboard');
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);
  const [isPinModalOpen, setIsPinModalOpen] = useState<boolean>(false);
  const [isConfigured, setIsConfigured] = useState<boolean | null>(null);

  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [rules, setRules] = useState<LocalRuleRecord[]>([]);
  const [logs, setLogs] = useState<QueryLogRecord[]>([]);
  const [evidenceList, setEvidenceList] = useState<EvidenceItem[]>([]);
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const [updateInfo, setUpdateInfo] = useState<{
    current_version: string;
    latest_version: string;
    update_available: boolean;
  } | null>(null);

  const [toasts, setToasts] = useState<
    { id: number; message: string; type: 'success' | 'error' | 'info' }[]
  >([]);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  };

  const refreshAllData = useCallback(async () => {
    const [statusRes, rulesRes, logsRes, evidenceRes] = await Promise.allSettled([
      api.getStatus(),
      api.getRules(),
      api.getLogs(150),
      api.getEvidence(50),
    ]);

    if (statusRes.status === 'fulfilled') {
      setStatus(statusRes.value);
    }
    if (rulesRes.status === 'fulfilled') {
      setRules(rulesRes.value);
    }
    if (logsRes.status === 'fulfilled') {
      setLogs(logsRes.value);
    }
    if (evidenceRes.status === 'fulfilled') {
      setEvidenceList(evidenceRes.value);
    }
  }, []);

  // Check URL token and setup status
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const urlToken = params.get('token') || params.get('auth_token');
    if (urlToken) {
      setSessionToken(urlToken);
      window.history.replaceState({}, document.title, window.location.pathname);
    }

    const checkInitAndAuth = async () => {
      try {
        const setupStatus = await api.getSetupStatus();
        if (!setupStatus.is_configured) {
          setIsConfigured(false);
          return;
        }
        setIsConfigured(true);
      } catch {
        // If error fetching setup status, assume configured and proceed to PIN auth
        setIsConfigured(true);
      }

      const auth = await api.verifyAuth();
      if (auth) {
        setIsAuthenticated(true);
        setIsPinModalOpen(false);
        refreshAllData();
      } else {
        setIsAuthenticated(false);
        setIsPinModalOpen(true);
      }
    };

    checkInitAndAuth();
    api.checkSoftwareUpdate().then(setUpdateInfo).catch(() => {});

    const handleUnauthorized = () => {
      setIsAuthenticated(false);
      setIsPinModalOpen(true);
    };

    window.addEventListener('famshield_unauthorized', handleUnauthorized);
    return () => window.removeEventListener('famshield_unauthorized', handleUnauthorized);
  }, [refreshAllData]);

  const handleLangChange = (newLang: Language) => {
    setLang(newLang);
    localStorage.setItem('famshield_lang', newLang);
  };

  const handleAuthSuccess = () => {
    setIsAuthenticated(true);
    setIsPinModalOpen(false);
    showToast(
      lang === 'vi' ? 'Xác thực Master PIN thành công!' : 'Master PIN verified successfully!',
      'success'
    );
    refreshAllData();
  };

  const handleSetupComplete = (token: string) => {
    setSessionToken(token);
    setIsConfigured(true);
    setIsAuthenticated(true);
    setIsPinModalOpen(false);
    showToast(
      lang === 'vi' ? 'Khởi tạo bảo vệ thành công!' : 'Guardian initialized successfully!',
      'success'
    );
    refreshAllData();
  };

  const handleLogout = async () => {
    await api.logout();
    setIsAuthenticated(false);
    setIsPinModalOpen(true);
    showToast(lang === 'vi' ? 'Đã đăng xuất' : 'Logged out', 'info');
  };

  // Auto-refresh interval (every 4 seconds for dashboard)
  useEffect(() => {
    if (!isAuthenticated || !autoRefresh) return;

    const timer = setInterval(() => {
      if (currentTab === 'dashboard') {
        api
          .getStatus()
          .then(setStatus)
          .catch(() => {});
        api
          .getLogs(10)
          .then((l) => setLogs(l))
          .catch(() => {});
      }
    }, 4000);

    return () => clearInterval(timer);
  }, [isAuthenticated, autoRefresh, currentTab]);

  if (isConfigured === null) {
    return (
      <div className="min-h-[100dvh] w-full flex items-center justify-center bg-slate-50 text-slate-600">
        <div className="text-center p-8 bg-white rounded-2xl shadow-sm border border-slate-200/80">
          <div aria-hidden="true" className="text-4xl mb-3">
            🛡️
          </div>
          <div className="text-sm font-medium text-slate-700">
            {lang === 'vi' ? 'Đang kiểm tra trạng thái FamShield...' : 'Connecting to FamShield...'}
          </div>
        </div>
      </div>
    );
  }

  if (isConfigured === false) {
    return <SetupWizard lang={lang} onComplete={handleSetupComplete} />;
  }

  const appVersion = status?.app_version || updateInfo?.current_version || '1.0.3';

  return (
    <div className="w-full min-h-[100dvh] flex flex-col bg-slate-50 text-slate-800 antialiased">
      <Navbar
        currentTab={currentTab}
        onTabChange={setCurrentTab}
        lang={lang}
        onLangChange={handleLangChange}
        operatingMode={status?.operating_mode || 'StandaloneFree'}
        appVersion={appVersion}
        updateAvailable={updateInfo?.update_available}
        latestVersion={updateInfo?.latest_version}
        onLogout={handleLogout}
        isAuthenticated={isAuthenticated}
      />

      <main className="flex-1 w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
        {currentTab === 'dashboard' && (
          <DashboardTab
            status={status}
            recentLogs={logs}
            lang={lang}
            onNavigateTab={setCurrentTab}
            onRefresh={refreshAllData}
          />
        )}

        {currentTab === 'activity' && (
          <ActivityTab lang={lang} onRefresh={refreshAllData} onShowToast={showToast} />
        )}

        {currentTab === 'rules' && (
          <RulesTab rules={rules} lang={lang} onRefresh={refreshAllData} onShowToast={showToast} />
        )}

        {currentTab === 'logs' && (
          <LogsTab
            logs={logs}
            lang={lang}
            onRefresh={refreshAllData}
            autoRefresh={autoRefresh}
            onToggleAutoRefresh={() => setAutoRefresh(!autoRefresh)}
            onShowToast={showToast}
          />
        )}

        {currentTab === 'evidence' && (
          <EvidenceTab
            evidenceList={evidenceList}
            lang={lang}
            onRefresh={() => api.getEvidence(50).then(setEvidenceList)}
          />
        )}

        {currentTab === 'security' && (
          <SecurityTab
            status={status}
            lang={lang}
            onRefresh={refreshAllData}
            onShowToast={showToast}
          />
        )}
      </main>

      {/* Global Application Footer */}
      <footer className="w-full border-t border-slate-200/80 bg-white/75 backdrop-blur-xs py-3.5 mt-auto">
        <div className="w-full max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-2.5 text-xs text-slate-500">
          <div className="flex items-center gap-2 flex-wrap justify-center sm:justify-start">
            <span className="font-semibold text-slate-700">FamShield Windows Agent</span>
            <span className="font-mono text-2xs bg-slate-100 text-slate-600 px-2 py-0.5 rounded-md border border-slate-200/60 font-semibold">
              v{appVersion}
            </span>
            <span className="text-slate-300">•</span>
            <span>
              {status?.operating_mode === 'OnlineManaged'
                ? lang === 'vi'
                  ? 'Chế độ Trực tuyến'
                  : 'Online Managed'
                : lang === 'vi'
                  ? 'Độc lập Ngoại tuyến'
                  : 'Standalone Offline'}
            </span>
          </div>

          <div className="flex items-center gap-3 text-2xs">
            <button
              type="button"
              onClick={() => setCurrentTab('security')}
              className="text-sky-600 hover:text-sky-800 hover:underline cursor-pointer font-medium"
            >
              {lang === 'vi' ? 'Kiểm tra cập nhật' : 'Check for updates'}
            </button>
            <span className="text-slate-300">•</span>
            <span>© 2026 FamShield Team</span>
          </div>
        </div>
      </footer>

      <PinModal
        isOpen={isPinModalOpen}
        lang={lang}
        onSuccess={handleAuthSuccess}
        appVersion={appVersion}
      />

      {/* Floating Toast Box */}
      <div
        aria-live="polite"
        className="fixed bottom-6 right-6 z-50 flex flex-col gap-2 pointer-events-none"
      >
        {toasts.map((toast) => (
          <div
            key={toast.id}
            role={toast.type === 'error' ? 'alert' : 'status'}
            className={`pointer-events-auto bg-white border rounded-xl px-4 py-3 text-sm font-medium shadow-md flex items-center gap-3 transition-all ${
              toast.type === 'error'
                ? 'border-rose-200 text-rose-800 bg-rose-50/90'
                : toast.type === 'success'
                  ? 'border-emerald-200 text-emerald-800 bg-emerald-50/90'
                  : 'border-sky-200 text-sky-800 bg-sky-50/90'
            }`}
          >
            <span aria-hidden="true" className="text-base">
              {toast.type === 'error' ? '⚠️' : toast.type === 'success' ? '✅' : 'ℹ️'}
            </span>
            <span>{toast.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
