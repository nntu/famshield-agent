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
      <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#070b14', color: '#94a3b8' }}>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '2.5rem', marginBottom: 12 }}>🛡️</div>
          <div style={{ fontSize: '0.95rem', fontWeight: 500 }}>{lang === 'vi' ? 'Đang kiểm tra trạng thái FamShield...' : 'Connecting to FamShield...'}</div>
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
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column', background: '#070b14' }}>
      <Navbar
        currentTab={currentTab}
        onTabChange={setCurrentTab}
        lang={lang}
        onLangChange={handleLangChange}
        operatingMode={status?.operating_mode || 'StandaloneFree'}
        onLogout={handleLogout}
        isAuthenticated={isAuthenticated}
      />

      <main style={{ flex: 1, maxWidth: 1400, width: '100%', margin: '0 auto', padding: '28px 24px' }}>
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
            onRefresh={() => api.getLogs(150).then(setLogs)}
            autoRefresh={autoRefresh}
            onToggleAutoRefresh={() => setAutoRefresh(!autoRefresh)}
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
      <div style={{
        position: 'fixed',
        bottom: 24,
        right: 24,
        zIndex: 200,
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
      }}>
        {toasts.map((toast) => (
          <div
            key={toast.id}
            style={{
              background: '#0f172a',
              border: `1px solid ${toast.type === 'error' ? '#f87171' : toast.type === 'success' ? '#34d399' : '#38bdf8'}`,
              color: '#f8fafc',
              padding: '12px 18px',
              borderRadius: 10,
              fontSize: '0.88rem',
              fontWeight: 500,
              boxShadow: '0 10px 30px rgba(0,0,0,0.5)',
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
            <span>{toast.type === 'error' ? '⚠️' : toast.type === 'success' ? '✅' : 'ℹ️'}</span>
            <span>{toast.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
