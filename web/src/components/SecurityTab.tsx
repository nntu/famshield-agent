import React, { useState, useEffect } from 'react';
import {
  Lock,
  Cloud,
  Key,
  CheckCircle2,
  ShieldCheck,
  Unlink,
  Server,
  RefreshCw,
  Radio,
  Wifi,
  WifiOff,
  Globe,
  Activity,
  Check,
  AlertTriangle,
  Database,
  Download,
  Upload,
  Eye,
  ShieldAlert,
  Keyboard,
  Trash2,
  Plus,
} from 'lucide-react';
import {
  BackupPayload,
  KeywordConfigResponse,
  KeywordRule,
  Language,
  ServerConfig,
  StatusResponse,
} from '../types';
import { translations } from '../i18n';
import { api } from '../api';

interface SecurityTabProps {
  status: StatusResponse | null;
  lang: Language;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const SecurityTab: React.FC<SecurityTabProps> = ({
  status,
  lang,
  onRefresh,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  // PIN Change State
  const [currPin, setCurrPin] = useState('');
  const [newPin, setNewPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [isChangingPin, setIsChangingPin] = useState(false);
  const [generatedRecoveryKey, setGeneratedRecoveryKey] = useState<string | null>(null);

  // Centralized Server Config State
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);
  const [serverUrl, setServerUrl] = useState('https://api.famshield.cloud');
  const [tenantId, setTenantId] = useState('tenant-default');
  const [deviceName, setDeviceName] = useState('DESKTOP-FAMSHIELD');
  const [enrollCode, setEnrollCode] = useState('');
  const [syncInterval, setSyncInterval] = useState<number>(60);
  const [tlsVerify, setTlsVerify] = useState<boolean>(true);

  // Operation States
  const [isLoadingConfig, setIsLoadingConfig] = useState(false);
  const [isSavingServer, setIsSavingServer] = useState(false);
  const [isTestingConn, setIsTestingConn] = useState(false);
  const [isSyncingNow, setIsSyncingNow] = useState(false);
  const [isDisenrolling, setIsDisenrolling] = useState(false);
  const [testResult, setTestResult] = useState<{
    success: boolean;
    latency_ms: number;
    message: string;
  } | null>(null);

  // Keyword Monitoring State
  const [keywordConfig, setKeywordConfig] = useState<KeywordConfigResponse | null>(null);
  const [kwSearch, setKwSearch] = useState('');
  const [newKwPattern, setNewKwPattern] = useState('');
  const [newKwCategory, setNewKwCategory] = useState(isVi ? 'Chửi bậy / Thô tục' : 'Profanity / Vulgarity');
  const [newKwSeverity, setNewKwSeverity] = useState('medium');
  const [isAddingKw, setIsAddingKw] = useState(false);
  const [isTogglingKwMonitoring, setIsTogglingKwMonitoring] = useState(false);

  // Fetch current configs on mount
  useEffect(() => {
    fetchServerConfig();
    fetchKeywordConfig();
  }, []);

  const fetchKeywordConfig = async () => {
    try {
      const cfg = await api.getKeywordConfig();
      setKeywordConfig(cfg);
    } catch {
      // ignore
    }
  };

  const handleToggleMonitoring = async () => {
    if (!keywordConfig) return;
    try {
      setIsTogglingKwMonitoring(true);
      const next = !keywordConfig.keyboard_monitoring_enabled;
      await api.toggleKeyboardMonitoring(next);
      setKeywordConfig({ ...keywordConfig, keyboard_monitoring_enabled: next });
      onShowToast(t.msg_kw_monitoring_toggled, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi cập nhật giám sát bàn phím' : 'Failed to update keyboard monitoring'), 'error');
    } finally {
      setIsTogglingKwMonitoring(false);
    }
  };

  const handleAddKeyword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKwPattern.trim()) return;
    try {
      setIsAddingKw(true);
      const added = await api.addCustomKeyword(newKwPattern.trim(), newKwCategory, newKwSeverity);
      setNewKwPattern('');
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          total_rules_count: keywordConfig.total_rules_count + 1,
          rules: [added, ...keywordConfig.rules],
        });
      }
      onShowToast(t.msg_kw_added, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi thêm từ khóa' : 'Failed to add keyword'), 'error');
    } finally {
      setIsAddingKw(false);
    }
  };

  const handleToggleKeyword = async (rule: KeywordRule) => {
    try {
      await api.toggleKeyword(rule.id, !rule.enabled);
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          rules: keywordConfig.rules.map((r) =>
            r.id === rule.id ? { ...r, enabled: !r.enabled } : r
          ),
        });
      }
      onShowToast(t.msg_kw_toggled, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi cập nhật từ khóa' : 'Failed to toggle keyword'), 'error');
    }
  };

  const handleDeleteKeyword = async (ruleId: string) => {
    try {
      await api.deleteCustomKeyword(ruleId);
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          total_rules_count: Math.max(0, keywordConfig.total_rules_count - 1),
          rules: keywordConfig.rules.filter((r) => r.id !== ruleId),
        });
      }
      onShowToast(t.msg_kw_deleted, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi xóa từ khóa' : 'Failed to delete keyword'), 'error');
    }
  };

  const fetchServerConfig = async () => {
    setIsLoadingConfig(true);
    try {
      const cfg = await api.getServerConfig();
      setServerConfig(cfg);
      setServerUrl(cfg.server_url || 'https://api.famshield.cloud');
      setTenantId(cfg.tenant_id || 'tenant-default');
      setDeviceName(cfg.device_name || 'DESKTOP-FAMSHIELD');
      setEnrollCode(cfg.enrollment_code || '');
      setSyncInterval(cfg.sync_interval_secs || 60);
      setTlsVerify(cfg.tls_verify ?? true);
    } catch {
      // Fallback
    } finally {
      setIsLoadingConfig(false);
    }
  };

  const handleChangePin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPin.length < 6 || !/^\d+$/.test(newPin)) {
      onShowToast(isVi ? 'Mã PIN mới phải gồm ít nhất 6 chữ số' : 'New PIN must be at least 6 digits', 'error');
      return;
    }
    if (newPin !== confirmPin) {
      onShowToast(isVi ? 'Mã PIN xác nhận không khớp' : 'PIN confirmation does not match', 'error');
      return;
    }

    setIsChangingPin(true);
    try {
      const res = await api.changePin({
        current_pin: currPin,
        new_pin: newPin,
      });

      if (res.success) {
        onShowToast(t.pin_success, 'success');
        setCurrPin('');
        setNewPin('');
        setConfirmPin('');
        if (res.recovery_key) {
          setGeneratedRecoveryKey(res.recovery_key);
        }
      }
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi đổi PIN' : 'Failed to change PIN'), 'error');
    } finally {
      setIsChangingPin(false);
    }
  };

  const handleTestConnection = async () => {
    if (!serverUrl) {
      onShowToast(isVi ? 'Vui lòng nhập URL máy chủ' : 'Please enter server URL', 'error');
      return;
    }
    setIsTestingConn(true);
    setTestResult(null);
    try {
      const res = await api.testServerConnection(serverUrl);
      setTestResult(res);
      if (res.success) {
        onShowToast(
          isVi
            ? `Kết nối thành công tới máy chủ (${res.latency_ms}ms)`
            : `Server connection successful (${res.latency_ms}ms)`,
          'success'
        );
      } else {
        onShowToast(res.message, 'error');
      }
    } catch (e: any) {
      setTestResult({
        success: false,
        latency_ms: 0,
        message: e.message || (isVi ? 'Lỗi kết nối' : 'Connection error'),
      });
      onShowToast(e.message || (isVi ? 'Lỗi kiểm tra kết nối' : 'Connection test failed'), 'error');
    } finally {
      setIsTestingConn(false);
    }
  };

  const handleSyncNow = async () => {
    setIsSyncingNow(true);
    try {
      const res = await api.syncServer();
      if (res.success) {
        onShowToast(
          isVi ? 'Đã đồng bộ chính sách thành công!' : 'Policies synchronized successfully!',
          'success'
        );
        fetchServerConfig();
        onRefresh();
      }
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi đồng bộ máy chủ' : 'Sync failed'), 'error');
    } finally {
      setIsSyncingNow(false);
    }
  };

  const handleSaveServerConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!serverUrl) {
      onShowToast(isVi ? 'URL máy chủ không được để trống' : 'Server URL cannot be empty', 'error');
      return;
    }

    setIsSavingServer(true);
    try {
      const updated = await api.updateServerConfig({
        server_url: serverUrl.trim(),
        tenant_id: tenantId.trim(),
        device_name: deviceName.trim(),
        enrollment_code: enrollCode.trim() || undefined,
        sync_interval_secs: syncInterval,
        tls_verify: tlsVerify,
        operating_mode: enrollCode.trim() ? 'OnlineManaged' : undefined,
      });

      setServerConfig(updated);
      onShowToast(
        isVi
          ? 'Đã lưu cấu hình máy chủ quản lý tập trung!'
          : 'Central server configuration saved!',
        'success'
      );
      onRefresh();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi lưu cấu hình' : 'Failed to save configuration'), 'error');
    } finally {
      setIsSavingServer(false);
    }
  };

  const handleDisenroll = async () => {
    if (
      !confirm(
        isVi
          ? 'Bạn có chắc chắn muốn ngắt kết nối khỏi máy chủ quản lý tập trung và quay về chế độ Ngoại tuyến StandaloneFree?'
          : 'Are you sure you want to disconnect from central management and return to StandaloneFree mode?'
      )
    ) {
      return;
    }

    setIsDisenrolling(true);
    try {
      await api.updateServerConfig({ operating_mode: 'StandaloneFree' });
      onShowToast(t.cloud_unlinked_success, 'success');
      fetchServerConfig();
      onRefresh();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi hủy kết nối' : 'Disenroll failed'), 'error');
    } finally {
      setIsDisenrolling(false);
    }
  };

  // Protection Mode State
  const [monitorMode, setMonitorMode] = useState<boolean>(status?.monitor_mode || false);
  const [isUpdatingMode, setIsUpdatingMode] = useState(false);

  useEffect(() => {
    if (status) {
      setMonitorMode(status.monitor_mode);
    }
  }, [status?.monitor_mode]);

  const handleToggleProtectionMode = async (newMode: boolean) => {
    setIsUpdatingMode(true);
    try {
      await api.setProtectionMode(newMode);
      setMonitorMode(newMode);
      onShowToast(t.protection_mode_updated, 'success');
      onRefresh();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi cập nhật chế độ bảo vệ' : 'Failed to update protection mode'), 'error');
    } finally {
      setIsUpdatingMode(false);
    }
  };

  // Backup & Restore State
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isRestoringBackup, setIsRestoringBackup] = useState(false);
  const [pendingRestoreBackup, setPendingRestoreBackup] = useState<BackupPayload | null>(null);

  const handleExportBackup = async () => {
    setIsCreatingBackup(true);
    try {
      const backup = await api.exportBackup();
      const jsonStr = JSON.stringify(backup, null, 2);
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      const filename = `famshield-backup-${new Date().toISOString().slice(0, 10)}.famshield-bak`;
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      onShowToast(t.backup_success, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi tạo bản sao lưu' : 'Failed to create backup'), 'error');
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleSelectRestoreFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const content = event.target?.result as string;
        const payload: BackupPayload = JSON.parse(content);
        if (!payload.format_version || !payload.rules || !payload.checksum) {
          throw new Error('Cấu trúc file sao lưu không hợp lệ');
        }
        setPendingRestoreBackup(payload);
      } catch (err: any) {
        onShowToast(t.backup_restore_invalid, 'error');
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  const handleConfirmRestore = async () => {
    if (!pendingRestoreBackup) return;
    setIsRestoringBackup(true);
    try {
      const res = await api.restoreBackup(pendingRestoreBackup);
      onShowToast(res.message || t.backup_restore_success, 'success');
      setPendingRestoreBackup(null);
      fetchServerConfig();
      onRefresh();
    } catch (e: any) {
      onShowToast(e.message || t.backup_restore_invalid, 'error');
    } finally {
      setIsRestoringBackup(false);
    }
  };

  const isEnrolled = serverConfig?.operating_mode === 'OnlineManaged';

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
      {/* SECTION 1: CENTRALIZED MANAGEMENT SERVER SETTINGS */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
          boxShadow: '0 10px 30px rgba(0,0,0,0.3)',
        }}
      >
        {/* Header */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 24 }}>
          <div>
            <h2 style={{ fontSize: '1.35rem', fontWeight: 800, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 12 }}>
              <Server size={24} color="#818cf8" />
              {t.title_central_server}
            </h2>
            <p style={{ fontSize: '0.88rem', color: '#94a3b8', marginTop: 6, lineHeight: 1.5 }}>
              {t.desc_central_server}
            </p>
          </div>

          {/* Mode Pill */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '6px 14px',
              borderRadius: 20,
              background: isEnrolled ? 'rgba(16, 185, 129, 0.15)' : 'rgba(148, 163, 184, 0.12)',
              border: `1px solid ${isEnrolled ? '#10b981' : '#475569'}`,
            }}
          >
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: '50%',
                background: isEnrolled ? '#10b981' : '#94a3b8',
                boxShadow: isEnrolled ? '0 0 8px #10b981' : 'none',
              }}
            />
            <span style={{ fontSize: '0.82rem', fontWeight: 700, color: isEnrolled ? '#34d399' : '#cbd5e1' }}>
              {isEnrolled ? (isVi ? 'Đang Quản Lý Tập Trung' : 'Centralized Online') : (isVi ? 'Ngoại Tuyến Độc Lập (Standalone)' : 'Standalone Offline')}
            </span>
          </div>
        </div>

        {/* Live Status & Telemetry Ribbon */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: 16,
            background: '#070b14',
            border: '1px solid #1e293b',
            borderRadius: 14,
            padding: '16px 20px',
            marginBottom: 26,
          }}
        >
          <div>
            <div style={{ fontSize: '0.75rem', color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              {isVi ? 'Mã Thiết Bị (Device ID)' : 'Device ID'}
            </div>
            <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#38bdf8', marginTop: 4, fontFamily: 'monospace' }}>
              {serverConfig?.device_id || 'standalone-local-001'}
            </div>
          </div>

          <div>
            <div style={{ fontSize: '0.75rem', color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              {isVi ? 'Mã Tổ Chức (Tenant ID)' : 'Tenant ID'}
            </div>
            <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#f8fafc', marginTop: 4 }}>
              {serverConfig?.tenant_id || 'tenant-default'}
            </div>
          </div>

          <div>
            <div style={{ fontSize: '0.75rem', color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              {t.metric_last_sync}
            </div>
            <div style={{ fontSize: '0.92rem', fontWeight: 600, color: serverConfig?.last_sync_time ? '#34d399' : '#64748b', marginTop: 4 }}>
              {serverConfig?.last_sync_time
                ? new Date(serverConfig.last_sync_time).toLocaleTimeString(isVi ? 'vi-VN' : 'en-US')
                : (isVi ? 'Chưa đồng bộ' : 'Never')}
            </div>
          </div>

          <div>
            <div style={{ fontSize: '0.75rem', color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              {t.metric_policy_ver}
            </div>
            <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#a855f7', marginTop: 4 }}>
              v{serverConfig?.policy_version || 1}
            </div>
          </div>
        </div>

        {/* Quick Operations: Test Connection & Sync Now */}
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, alignItems: 'center', marginBottom: 24 }}>
          <button
            type="button"
            onClick={handleTestConnection}
            disabled={isTestingConn}
            style={{
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              borderRadius: 10,
              padding: '10px 18px',
              fontSize: '0.85rem',
              fontWeight: 600,
              cursor: isTestingConn ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              transition: 'all 0.2s',
            }}
          >
            <Activity size={16} />
            <span>{isTestingConn ? t.testing_connection : t.btn_test_conn}</span>
          </button>

          {isEnrolled && (
            <button
              type="button"
              onClick={handleSyncNow}
              disabled={isSyncingNow}
              style={{
                background: '#1e293b',
                color: '#10b981',
                border: '1px solid #334155',
                borderRadius: 10,
                padding: '10px 18px',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: isSyncingNow ? 'not-allowed' : 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s',
              }}
            >
              <RefreshCw size={16} className={isSyncingNow ? 'animate-spin' : ''} />
              <span>{isSyncingNow ? t.syncing_policies : t.btn_sync_now}</span>
            </button>
          )}

          {/* Test connection result pill */}
          {testResult && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '8px 14px',
                borderRadius: 10,
                background: testResult.success ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)',
                border: `1px solid ${testResult.success ? '#10b981' : '#ef4444'}`,
                color: testResult.success ? '#34d399' : '#f87171',
                fontSize: '0.82rem',
                fontWeight: 600,
              }}
            >
              {testResult.success ? <Check size={15} /> : <AlertTriangle size={15} />}
              <span>{testResult.message}</span>
            </div>
          )}
        </div>

        {/* Server Connection Form */}
        <form onSubmit={handleSaveServerConfig} style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          {/* Server URL with quick presets */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <label style={{ fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600 }}>
                {t.lbl_server_url}
              </label>
              {/* Presets */}
              <div style={{ display: 'flex', gap: 8 }}>
                <button
                  type="button"
                  onClick={() => setServerUrl('https://api.famshield.cloud')}
                  style={{
                    background: '#1e293b',
                    border: '1px solid #334155',
                    color: '#38bdf8',
                    padding: '3px 8px',
                    borderRadius: 6,
                    fontSize: '0.72rem',
                    cursor: 'pointer',
                  }}
                >
                  Cloud Default
                </button>
                <button
                  type="button"
                  onClick={() => setServerUrl('https://192.168.1.100:8443')}
                  style={{
                    background: '#1e293b',
                    border: '1px solid #334155',
                    color: '#94a3b8',
                    padding: '3px 8px',
                    borderRadius: 6,
                    fontSize: '0.72rem',
                    cursor: 'pointer',
                  }}
                >
                  Local LAN
                </button>
              </div>
            </div>
            <input
              type="text"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              placeholder="https://api.famshield.cloud"
              style={{
                width: '100%',
                background: '#070b14',
                border: '1px solid #334155',
                color: '#fff',
                padding: '11px 14px',
                borderRadius: 10,
                fontSize: '0.9rem',
                outline: 'none',
              }}
            />
          </div>

          {/* Grid: Tenant ID & Device Name */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: 16 }}>
            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 6 }}>
                {t.lbl_tenant_id}
              </label>
              <input
                type="text"
                value={tenantId}
                onChange={(e) => setTenantId(e.target.value)}
                placeholder="tenant-default"
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  color: '#fff',
                  padding: '11px 14px',
                  borderRadius: 10,
                  fontSize: '0.9rem',
                  outline: 'none',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 6 }}>
                {t.lbl_device_name}
              </label>
              <input
                type="text"
                value={deviceName}
                onChange={(e) => setDeviceName(e.target.value)}
                placeholder="PC-PhongKhach"
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  color: '#fff',
                  padding: '11px 14px',
                  borderRadius: 10,
                  fontSize: '0.9rem',
                  outline: 'none',
                }}
              />
            </div>
          </div>

          {/* Grid: Enrollment Code & Sync Interval */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: 16 }}>
            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 6 }}>
                {t.lbl_enroll_code}
              </label>
              <input
                type="text"
                maxLength={16}
                value={enrollCode}
                onChange={(e) => setEnrollCode(e.target.value)}
                placeholder={isVi ? 'Nhập mã ghép nối 6-12 ký tự' : 'Enter 6-12 digit code'}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  color: '#fff',
                  padding: '11px 14px',
                  borderRadius: 10,
                  fontSize: '0.9rem',
                  outline: 'none',
                  letterSpacing: '0.05em',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 6 }}>
                {t.lbl_sync_freq}
              </label>
              <select
                value={syncInterval}
                onChange={(e) => setSyncInterval(Number(e.target.value))}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  color: '#fff',
                  padding: '11px 14px',
                  borderRadius: 10,
                  fontSize: '0.9rem',
                  outline: 'none',
                }}
              >
                <option value={15}>{isVi ? '15 giây (Nhanh)' : '15 seconds (Fast)'}</option>
                <option value={30}>{isVi ? '30 giây' : '30 seconds'}</option>
                <option value={60}>{isVi ? '60 giây (Mặc định)' : '60 seconds (Default)'}</option>
                <option value={300}>{isVi ? '5 phút' : '5 minutes'}</option>
                <option value={900}>{isVi ? '15 phút' : '15 minutes'}</option>
              </select>
            </div>
          </div>

          {/* TLS Checkbox */}
          <div
            style={{
              background: 'rgba(30, 41, 59, 0.4)',
              border: '1px solid #1e293b',
              borderRadius: 12,
              padding: '14px 18px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <div style={{ fontSize: '0.88rem', fontWeight: 600, color: '#f8fafc' }}>
                {t.lbl_tls_check}
              </div>
              <div style={{ fontSize: '0.78rem', color: '#94a3b8', marginTop: 2 }}>
                {t.desc_tls_check}
              </div>
            </div>
            <input
              type="checkbox"
              checked={tlsVerify}
              onChange={(e) => setTlsVerify(e.target.checked)}
              style={{ width: 18, height: 18, accentColor: '#818cf8', cursor: 'pointer' }}
            />
          </div>

          {/* Action Buttons */}
          <div style={{ display: 'flex', gap: 12, marginTop: 8 }}>
            <button
              type="submit"
              disabled={isSavingServer}
              style={{
                flex: 2,
                background: 'linear-gradient(135deg, #6366f1, #818cf8)',
                color: '#fff',
                border: 'none',
                padding: '12px 20px',
                borderRadius: 10,
                fontSize: '0.92rem',
                fontWeight: 700,
                cursor: isSavingServer ? 'not-allowed' : 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: 8,
                boxShadow: '0 4px 15px rgba(99, 102, 241, 0.3)',
              }}
            >
              <Cloud size={18} />
              <span>{isSavingServer ? (isVi ? 'Đang lưu...' : 'Saving...') : t.btn_save_server}</span>
            </button>

            {isEnrolled && (
              <button
                type="button"
                onClick={handleDisenroll}
                disabled={isDisenrolling}
                style={{
                  flex: 1,
                  background: 'transparent',
                  color: '#f87171',
                  border: '1px solid #ef4444',
                  padding: '12px 16px',
                  borderRadius: 10,
                  fontSize: '0.88rem',
                  fontWeight: 600,
                  cursor: isDisenrolling ? 'not-allowed' : 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 6,
                }}
              >
                <Unlink size={16} />
                <span>{t.btn_unlink_cloud}</span>
              </button>
            )}
          </div>
        </form>
      </div>

      {/* SECTION 2: PROTECTION MODE (CHỦ ĐỘNG CHẶN VS CHỈ GIÁM SÁT) */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
          boxShadow: '0 10px 30px rgba(0,0,0,0.3)',
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
            <Eye size={22} color="#38bdf8" />
            {t.title_protection_mode}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_protection_mode}
          </p>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: 16 }}>
          {/* Option A: Active Blocking */}
          <div
            onClick={() => !isUpdatingMode && handleToggleProtectionMode(false)}
            style={{
              background: !monitorMode ? 'rgba(52, 211, 153, 0.08)' : '#070b14',
              border: `2px solid ${!monitorMode ? '#34d399' : '#1e293b'}`,
              borderRadius: 14,
              padding: 20,
              cursor: isUpdatingMode ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s ease',
              position: 'relative',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
              <span style={{ fontSize: '1rem', fontWeight: 700, color: !monitorMode ? '#34d399' : '#e2e8f0', display: 'flex', alignItems: 'center', gap: 8 }}>
                <ShieldCheck size={18} color={!monitorMode ? '#34d399' : '#94a3b8'} />
                {t.mode_blocking}
              </span>
              {!monitorMode && (
                <span style={{ background: '#34d399', color: '#070b14', borderRadius: '50%', padding: 2, display: 'flex' }}>
                  <Check size={14} />
                </span>
              )}
            </div>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
              {t.mode_blocking_desc}
            </p>
          </div>

          {/* Option B: Monitor / Audit Mode */}
          <div
            onClick={() => !isUpdatingMode && handleToggleProtectionMode(true)}
            style={{
              background: monitorMode ? 'rgba(245, 158, 11, 0.08)' : '#070b14',
              border: `2px solid ${monitorMode ? '#f59e0b' : '#1e293b'}`,
              borderRadius: 14,
              padding: 20,
              cursor: isUpdatingMode ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s ease',
              position: 'relative',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
              <span style={{ fontSize: '1rem', fontWeight: 700, color: monitorMode ? '#fbbf24' : '#e2e8f0', display: 'flex', alignItems: 'center', gap: 8 }}>
                <Eye size={18} color={monitorMode ? '#fbbf24' : '#94a3b8'} />
                {t.mode_monitor}
              </span>
              {monitorMode && (
                <span style={{ background: '#f59e0b', color: '#070b14', borderRadius: '50%', padding: 2, display: 'flex' }}>
                  <Check size={14} />
                </span>
              )}
            </div>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
              {t.mode_monitor_desc}
            </p>
          </div>
        </div>
      </div>

      {/* SECTION 3: BACKUP & RESTORE */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
          boxShadow: '0 10px 30px rgba(0,0,0,0.3)',
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
            <Database size={22} color="#38bdf8" />
            {t.title_backup_restore}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_backup_restore}
          </p>
        </div>

        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 16 }}>
          {/* Create Backup Button */}
          <button
            onClick={handleExportBackup}
            disabled={isCreatingBackup}
            style={{
              background: '#0284c7',
              color: '#fff',
              border: 'none',
              padding: '12px 22px',
              borderRadius: 10,
              fontSize: '0.9rem',
              fontWeight: 700,
              cursor: isCreatingBackup ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              boxShadow: '0 4px 15px rgba(2, 132, 199, 0.25)',
            }}
          >
            {isCreatingBackup ? <RefreshCw size={16} className="animate-spin" /> : <Download size={16} />}
            <span>{t.btn_create_backup}</span>
          </button>

          {/* Restore Backup File Input & Button */}
          <label
            style={{
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              padding: '12px 22px',
              borderRadius: 10,
              fontSize: '0.9rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}
          >
            <Upload size={16} />
            <span>{t.btn_restore_backup}</span>
            <input
              type="file"
              accept=".famshield-bak,.json"
              onChange={handleSelectRestoreFile}
              style={{ display: 'none' }}
            />
          </label>
        </div>
      </div>

      {/* SECTION 4: CHANGE MASTER PIN CARD */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
            <Lock size={22} color="#38bdf8" />
            {t.title_change_pin}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_change_pin}
          </p>
        </div>

        <form onSubmit={handleChangePin} style={{ display: 'flex', flexDirection: 'column', gap: 14, maxWidth: 500 }}>
          <div>
            <label style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}>
              {t.lbl_curr_pin}
            </label>
            <input
              type="password"
              maxLength={8}
              value={currPin}
              onChange={(e) => setCurrPin(e.target.value)}
              style={{
                width: '100%',
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '10px 14px',
                borderRadius: 8,
                fontSize: '0.9rem',
                outline: 'none',
              }}
            />
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 14 }}>
            <div>
              <label style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}>
                {t.lbl_new_pin}
              </label>
              <input
                type="password"
                maxLength={8}
                value={newPin}
                onChange={(e) => setNewPin(e.target.value)}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#fff',
                  padding: '10px 14px',
                  borderRadius: 8,
                  fontSize: '0.9rem',
                  outline: 'none',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}>
                {t.lbl_confirm_pin}
              </label>
              <input
                type="password"
                maxLength={8}
                value={confirmPin}
                onChange={(e) => setConfirmPin(e.target.value)}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#fff',
                  padding: '10px 14px',
                  borderRadius: 8,
                  fontSize: '0.9rem',
                  outline: 'none',
                }}
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={isChangingPin}
            style={{
              marginTop: 6,
              background: '#38bdf8',
              color: '#070b14',
              border: 'none',
              padding: '11px',
              borderRadius: 8,
              fontSize: '0.9rem',
              fontWeight: 700,
              cursor: isChangingPin ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 8,
            }}
          >
            <Key size={16} />
            <span>{t.btn_save_pin}</span>
          </button>
        </form>

        {/* Generated Recovery Key Display */}
        {generatedRecoveryKey && (
          <div
            style={{
              marginTop: 20,
              background: '#070b14',
              border: '1px dashed #34d399',
              borderRadius: 12,
              padding: 16,
              textAlign: 'center',
              maxWidth: 500,
            }}
          >
            <div style={{ fontSize: '0.82rem', color: '#34d399', fontWeight: 600, marginBottom: 6 }}>
              {t.pin_recovery_notice}
            </div>
            <div
              style={{
                fontSize: '1.2rem',
                fontWeight: 800,
                fontFamily: 'monospace',
                letterSpacing: '0.15em',
                color: '#38bdf8',
                userSelect: 'all',
              }}
            >
              {generatedRecoveryKey}
            </div>
          </div>
        )}
      </div>

      {/* 5. Keyword Monitoring & Keystroke Filter Card */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 24,
          boxShadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            flexWrap: 'wrap',
            gap: 16,
            marginBottom: 20,
            borderBottom: '1px solid #1e293b',
            paddingBottom: 16,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: '1.25rem',
                fontWeight: 800,
                color: '#f8fafc',
                margin: 0,
                display: 'flex',
                alignItems: 'center',
                gap: 10,
              }}
            >
              <Keyboard size={24} color="#f59e0b" />
              {t.title_keyword_monitor}
            </h3>
            <p
              style={{
                fontSize: '0.85rem',
                color: '#94a3b8',
                marginTop: 6,
                marginBottom: 0,
                maxWidth: 800,
                lineHeight: 1.5,
              }}
            >
              {t.desc_keyword_monitor}
            </p>
          </div>

          {/* Toggle monitoring switch button */}
          <button
            onClick={handleToggleMonitoring}
            disabled={isTogglingKwMonitoring}
            style={{
              background: keywordConfig?.keyboard_monitoring_enabled
                ? 'rgba(52, 211, 153, 0.15)'
                : 'rgba(239, 68, 68, 0.15)',
              color: keywordConfig?.keyboard_monitoring_enabled ? '#34d399' : '#f87171',
              border: `1px solid ${
                keywordConfig?.keyboard_monitoring_enabled ? 'rgba(52, 211, 153, 0.4)' : 'rgba(239, 68, 68, 0.4)'
              }`,
              padding: '9px 18px',
              borderRadius: 10,
              fontSize: '0.88rem',
              fontWeight: 700,
              cursor: isTogglingKwMonitoring ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              transition: 'all 0.2s ease',
            }}
          >
            {isTogglingKwMonitoring ? (
              <RefreshCw size={14} className="animate-spin" />
            ) : keywordConfig?.keyboard_monitoring_enabled ? (
              <Check size={14} />
            ) : (
              <AlertTriangle size={14} />
            )}
            <span>
              {keywordConfig?.keyboard_monitoring_enabled
                ? isVi
                  ? '🟢 Đang Giám Sát Bàn Phím (Bật)'
                  : '🟢 Keystroke Monitor: Active'
                : isVi
                ? '⚪ Đã Tạm Dừng Giám Sát (Tắt)'
                : '⚪ Keystroke Monitor: Disabled'}
            </span>
          </button>
        </div>

        {/* 3 Metrics Badges */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: 14,
            marginBottom: 20,
          }}
        >
          <div style={{ background: '#070b14', border: '1px solid #1e293b', borderRadius: 12, padding: '14px 16px' }}>
            <div style={{ fontSize: '0.78rem', color: '#64748b', fontWeight: 600, textTransform: 'uppercase' }}>
              {isVi ? 'Tổng số từ khóa đang theo dõi' : 'Total Monitored Keywords'}
            </div>
            <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#38bdf8', marginTop: 4 }}>
              {keywordConfig?.total_rules_count || 0}
            </div>
          </div>

          <div style={{ background: '#070b14', border: '1px solid #1e293b', borderRadius: 12, padding: '14px 16px' }}>
            <div style={{ fontSize: '0.78rem', color: '#64748b', fontWeight: 600, textTransform: 'uppercase' }}>
              {isVi ? 'Vi phạm phát hiện hôm nay' : 'Violations Detected Today'}
            </div>
            <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#fbbf24', marginTop: 4 }}>
              {keywordConfig?.violations_today_count || 0}
            </div>
          </div>

          <div style={{ background: '#070b14', border: '1px solid #1e293b', borderRadius: 12, padding: '14px 16px' }}>
            <div style={{ fontSize: '0.78rem', color: '#64748b', fontWeight: 600, textTransform: 'uppercase' }}>
              {isVi ? 'Cơ chế bảo vệ riêng tư' : 'Privacy Mechanism'}
            </div>
            <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#34d399', marginTop: 8 }}>
              RAM Buffer 128 chars • 0 Disk Logs
            </div>
          </div>
        </div>

        {/* Add Custom Keyword Form */}
        <form
          onSubmit={handleAddKeyword}
          style={{
            background: '#070b14',
            border: '1px solid #1e293b',
            borderRadius: 14,
            padding: 16,
            marginBottom: 20,
            display: 'flex',
            flexWrap: 'wrap',
            gap: 12,
            alignItems: 'flex-end',
          }}
        >
          <div style={{ flex: '1 1 240px' }}>
            <label style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}>
              {t.lbl_kw_pattern}
            </label>
            <input
              type="text"
              placeholder={isVi ? 'Ví dụ: đánh nhau, chửi thề...' : 'e.g., hate you, fighting...'}
              value={newKwPattern}
              onChange={(e) => setNewKwPattern(e.target.value)}
              style={{
                width: '100%',
                background: '#0f172a',
                border: '1px solid #334155',
                color: '#fff',
                padding: '9px 12px',
                borderRadius: 8,
                fontSize: '0.88rem',
                outline: 'none',
              }}
            />
          </div>

          <div style={{ flex: '0 1 200px' }}>
            <label style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}>
              {t.lbl_kw_category}
            </label>
            <select
              value={newKwCategory}
              onChange={(e) => setNewKwCategory(e.target.value)}
              style={{
                width: '100%',
                background: '#0f172a',
                border: '1px solid #334155',
                color: '#fff',
                padding: '9px 12px',
                borderRadius: 8,
                fontSize: '0.88rem',
                outline: 'none',
              }}
            >
              <option value="Chửi bậy / Thô tục">{t.cat_profanity}</option>
              <option value="Bắt nạt / Đe dọa">{t.cat_bullying}</option>
              <option value="Tự hại / Nguy hiểm">{t.cat_self_harm}</option>
              <option value="Người lớn / Khiêu dâm">{t.cat_adult}</option>
              <option value="Tùy chỉnh phụ huynh">{t.cat_custom}</option>
            </select>
          </div>

          <div style={{ flex: '0 1 150px' }}>
            <label style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}>
              {t.lbl_kw_severity}
            </label>
            <select
              value={newKwSeverity}
              onChange={(e) => setNewKwSeverity(e.target.value)}
              style={{
                width: '100%',
                background: '#0f172a',
                border: '1px solid #334155',
                color: '#fff',
                padding: '9px 12px',
                borderRadius: 8,
                fontSize: '0.88rem',
                outline: 'none',
              }}
            >
              <option value="low">{t.sev_low}</option>
              <option value="medium">{t.sev_medium}</option>
              <option value="high">{t.sev_high}</option>
              <option value="critical">{t.sev_critical}</option>
            </select>
          </div>

          <button
            type="submit"
            disabled={isAddingKw || !newKwPattern.trim()}
            style={{
              background: '#f59e0b',
              color: '#070b14',
              border: 'none',
              padding: '9px 18px',
              borderRadius: 8,
              fontSize: '0.85rem',
              fontWeight: 700,
              cursor: isAddingKw || !newKwPattern.trim() ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              height: 38,
            }}
          >
            {isAddingKw ? <RefreshCw size={14} className="animate-spin" /> : <Plus size={15} />}
            <span>{t.btn_add_keyword}</span>
          </button>
        </form>

        {/* Search & Keyword List */}
        <div>
          <div style={{ marginBottom: 14, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <input
              type="text"
              placeholder={isVi ? 'Tìm kiếm từ khóa trong danh sách...' : 'Search monitored keywords...'}
              value={kwSearch}
              onChange={(e) => setKwSearch(e.target.value)}
              style={{
                width: '100%',
                maxWidth: 320,
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '8px 12px',
                borderRadius: 8,
                fontSize: '0.82rem',
                outline: 'none',
              }}
            />
          </div>

          <div style={{ overflowX: 'auto', maxHeight: 320, overflowY: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '0.85rem' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #1e293b', color: '#64748b', fontSize: '0.78rem', textTransform: 'uppercase' }}>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_pattern}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_category}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_severity}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_type}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_status}</th>
                  <th style={{ padding: '10px 14px', textAlign: 'right' }}>{isVi ? 'Hành động' : 'Action'}</th>
                </tr>
              </thead>
              <tbody>
                {(keywordConfig?.rules || [])
                  .filter((r) =>
                    r.pattern.toLowerCase().includes(kwSearch.toLowerCase()) ||
                    r.category_name.toLowerCase().includes(kwSearch.toLowerCase())
                  )
                  .map((rule) => (
                    <tr key={rule.id} style={{ borderBottom: '1px solid rgba(30, 41, 59, 0.6)' }}>
                      <td style={{ padding: '10px 14px', fontWeight: 700, color: '#f8fafc' }}>
                        "{rule.pattern}"
                      </td>
                      <td style={{ padding: '10px 14px', color: '#cbd5e1' }}>
                        <span style={{ background: '#1e293b', padding: '2px 8px', borderRadius: 6, fontSize: '0.75rem' }}>
                          {rule.category_name}
                        </span>
                      </td>
                      <td style={{ padding: '10px 14px' }}>
                        <span
                          style={{
                            padding: '2px 7px',
                            borderRadius: 6,
                            fontSize: '0.72rem',
                            fontWeight: 700,
                            textTransform: 'uppercase',
                            background:
                              rule.severity === 'critical'
                                ? 'rgba(239, 68, 68, 0.2)'
                                : rule.severity === 'high'
                                ? 'rgba(249, 115, 22, 0.2)'
                                : 'rgba(245, 158, 11, 0.2)',
                            color:
                              rule.severity === 'critical'
                                ? '#ef4444'
                                : rule.severity === 'high'
                                ? '#f97316'
                                : '#fbbf24',
                          }}
                        >
                          {rule.severity}
                        </span>
                      </td>
                      <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '0.8rem' }}>
                        {rule.is_custom ? (
                          <span style={{ color: '#c084fc', fontWeight: 600 }}>{isVi ? 'Phụ huynh' : 'Custom'}</span>
                        ) : (
                          <span>{isVi ? 'Hệ thống' : 'Built-in'}</span>
                        )}
                      </td>
                      <td style={{ padding: '10px 14px' }}>
                        <button
                          type="button"
                          onClick={() => handleToggleKeyword(rule)}
                          style={{
                            background: rule.enabled ? 'rgba(52, 211, 153, 0.15)' : 'rgba(100, 116, 139, 0.15)',
                            color: rule.enabled ? '#34d399' : '#64748b',
                            border: `1px solid ${rule.enabled ? 'rgba(52, 211, 153, 0.3)' : 'rgba(100, 116, 139, 0.3)'}`,
                            padding: '3px 9px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 700,
                            cursor: 'pointer',
                          }}
                        >
                          {rule.enabled ? (isVi ? 'BẬT' : 'ON') : (isVi ? 'TẮT' : 'OFF')}
                        </button>
                      </td>
                      <td style={{ padding: '10px 14px', textAlign: 'right' }}>
                        {rule.is_custom && (
                          <button
                            type="button"
                            onClick={() => handleDeleteKeyword(rule.id)}
                            style={{
                              background: 'transparent',
                              border: 'none',
                              color: '#f87171',
                              cursor: 'pointer',
                              padding: 4,
                            }}
                            title={isVi ? 'Xóa từ khóa' : 'Delete keyword'}
                          >
                            <Trash2 size={15} />
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Restore Confirmation Modal */}
      {pendingRestoreBackup && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.75)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 9999,
            backdropFilter: 'blur(6px)',
          }}
        >
          <div
            style={{
              background: '#0f172a',
              border: '1px solid #334155',
              borderRadius: 16,
              padding: 28,
              maxWidth: 500,
              width: '90%',
              boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
            }}
          >
            <h3
              style={{
                fontSize: '1.2rem',
                fontWeight: 800,
                color: '#f8fafc',
                marginBottom: 12,
                display: 'flex',
                alignItems: 'center',
                gap: 10,
              }}
            >
              <Database size={22} color="#38bdf8" />
              {t.backup_restore_modal_title}
            </h3>
            <p style={{ fontSize: '0.88rem', color: '#94a3b8', lineHeight: 1.6, marginBottom: 18 }}>
              {t.backup_restore_modal_desc
                .replace('{rules}', (pendingRestoreBackup.rules?.length || 0).toString())
                .replace('{time}', new Date(pendingRestoreBackup.created_at).toLocaleString())}
            </p>
            <div
              style={{
                background: '#070b14',
                borderRadius: 8,
                padding: 14,
                marginBottom: 20,
                fontSize: '0.82rem',
                color: '#cbd5e1',
                fontFamily: 'monospace',
                display: 'flex',
                flexDirection: 'column',
                gap: 6,
              }}
            >
              <div>
                <span style={{ color: '#64748b' }}>Device ID: </span>
                <span style={{ color: '#38bdf8' }}>{pendingRestoreBackup.device_id}</span>
              </div>
              <div>
                <span style={{ color: '#64748b' }}>Operating Mode: </span>
                <span>{pendingRestoreBackup.settings?.operating_mode}</span>
              </div>
              <div>
                <span style={{ color: '#64748b' }}>Protection Mode: </span>
                <span style={{ color: pendingRestoreBackup.settings?.monitor_mode ? '#fbbf24' : '#34d399' }}>
                  {pendingRestoreBackup.settings?.monitor_mode ? 'Monitor (Only Observe)' : 'Active Blocking'}
                </span>
              </div>
              <div>
                <span style={{ color: '#64748b' }}>SHA-256: </span>
                <span style={{ color: '#94a3b8' }}>{pendingRestoreBackup.checksum.slice(0, 24)}...</span>
              </div>
            </div>
            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                onClick={() => setPendingRestoreBackup(null)}
                style={{
                  background: '#1e293b',
                  color: '#94a3b8',
                  border: 'none',
                  padding: '10px 18px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {t.import_cancel_btn}
              </button>
              <button
                onClick={handleConfirmRestore}
                disabled={isRestoringBackup}
                style={{
                  background: '#38bdf8',
                  color: '#070b14',
                  border: 'none',
                  padding: '10px 20px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 700,
                  cursor: isRestoringBackup ? 'not-allowed' : 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                {isRestoringBackup ? <RefreshCw size={14} className="animate-spin" /> : <Check size={14} />}
                {t.backup_restore_confirm_btn}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
