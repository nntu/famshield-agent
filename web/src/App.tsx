import React, { useState, useEffect } from 'react';
import { Navbar } from './components/Navbar';
import { PinModal } from './components/PinModal';
import { DashboardTab } from './components/DashboardTab';
import { ActivityTab } from './components/ActivityTab';
import { RulesTab } from './components/RulesTab';
import { LogsTab } from './components/LogsTab';
import { EvidenceTab } from './components/EvidenceTab';
import { SecurityTab } from './components/SecurityTab';
import { SetupWizard } from './components/SetupWizard';
import { EvidenceItem, Language, LocalRuleRecord, QueryLogRecord, StatusResponse, TabType } from './types';
import { api, setSessionToken } from './api';

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

  const [toasts, setToasts] = useState<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([]);

  const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  };

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

    const handleUnauthorized = () => {
      setIsAuthenticated(false);
      setIsPinModalOpen(true);
    };

    window.addEventListener('famshield_unauthorized', handleUnauthorized);
    return () => window.removeEventListener('famshield_unauthorized', handleUnauthorized);
  }, []);

  const handleLangChange = (newLang: Language) => {
    setLang(newLang);
    localStorage.setItem('famshield_lang', newLang);
  };

  const handleAuthSuccess = () => {
    setIsAuthenticated(true);
    setIsPinModalOpen(false);
    showToast(lang === 'vi' ? 'Xác thực Master PIN thành công!' : 'Master PIN verified successfully!', 'success');
    refreshAllData();
  };

  const handleSetupComplete = (token: string) => {
    setSessionToken(token);
    setIsConfigured(true);
    setIsAuthenticated(true);
    setIsPinModalOpen(false);
    showToast(lang === 'vi' ? 'Khởi tạo bảo vệ thành công!' : 'Guardian initialized successfully!', 'success');
    refreshAllData();
  };

  const handleLogout = async () => {
    await api.logout();
    setIsAuthenticated(false);
    setIsPinModalOpen(true);
    showToast(lang === 'vi' ? 'Đã đăng xuất' : 'Logged out', 'info');
  };

  const refreshAllData = async () => {
    try {
      const s = await api.getStatus();
      setStatus(s);
    } catch {}

    try {
      const r = await api.getRules();
      setRules(r);
    } catch {}

    try {
      const l = await api.getLogs(150);
      setLogs(l);
    } catch {}

    try {
      const e = await api.getEvidence(50);
      setEvidenceList(e);
    } catch {}
  };

  // Auto-refresh interval (every 4 seconds for dashboard & logs)
  useEffect(() => {
    if (!isAuthenticated || !autoRefresh) return;

    const timer = setInterval(() => {
      if (currentTab === 'dashboard') {
        api.getStatus().then(setStatus).catch(() => {});
        api.getLogs(10).then((l) => setLogs(l)).catch(() => {});
      } else if (currentTab === 'logs') {
        api.getLogs(150).then(setLogs).catch(() => {});
      }
    }, 4000);

    return () => clearInterval(timer);
  }, [isAuthenticated, autoRefresh, currentTab]);

  if (isConfigured === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50 text-slate-600">
        <div className="text-center p-8 bg-white rounded-2xl shadow-sm border border-slate-200">
          <div className="text-4xl mb-3">🛡️</div>
          <div className="text-sm font-medium text-slate-700">{lang === 'vi' ? 'Đang kiểm tra trạng thái FamShield...' : 'Connecting to FamShield...'}</div>
        </div>
      </div>
    );
  }

  if (isConfigured === false) {
    return (
      <SetupWizard
        lang={lang}
        onComplete={handleSetupComplete}
      />
    );
  }

  return (
    <div className="min-h-screen flex flex-col bg-slate-50 text-slate-800 antialiased">
      <Navbar
        currentTab={currentTab}
        onTabChange={setCurrentTab}
        lang={lang}
        onLangChange={handleLangChange}
        operatingMode={status?.operating_mode || 'StandaloneFree'}
        onLogout={handleLogout}
        isAuthenticated={isAuthenticated}
      />

      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 py-6 sm:py-8">
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
          <ActivityTab
            lang={lang}
            onRefresh={refreshAllData}
            onShowToast={showToast}
          />
        )}

        {currentTab === 'rules' && (
          <RulesTab
            rules={rules}
            lang={lang}
            onRefresh={refreshAllData}
            onShowToast={showToast}
          />
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

      <PinModal
        isOpen={isPinModalOpen}
        lang={lang}
        onSuccess={handleAuthSuccess}
      />

      {/* Floating Toast Box */}
      <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2">
        {toasts.map((toast) => (
          <div
            key={toast.id}
            className={`bg-white border rounded-xl px-4 py-3 text-sm font-medium shadow-lg flex items-center gap-3 transition-all ${
              toast.type === 'error'
                ? 'border-red-300 text-red-700 bg-red-50/80'
                : toast.type === 'success'
                ? 'border-emerald-300 text-emerald-700 bg-emerald-50/80'
                : 'border-sky-300 text-sky-700 bg-sky-50/80'
            }`}
          >
            <span className="text-base">{toast.type === 'error' ? '⚠️' : toast.type === 'success' ? '✅' : 'ℹ️'}</span>
            <span>{toast.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
