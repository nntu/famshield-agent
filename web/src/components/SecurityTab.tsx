import {
  Activity,
  AlertTriangle,
  BellOff,
  Check,
  CheckCircle2,
  Cloud,
  Copy,
  Database,
  Download,
  Eye,
  Globe,
  Key,
  Keyboard,
  Lock,
  Plus,
  QrCode,
  Radio,
  RefreshCw,
  Server,
  ShieldAlert,
  Sliders,
  ShieldCheck,
  Smartphone,
  Trash2,
  Unlink,
  Upload,
  Wifi,
  WifiOff,
} from 'lucide-react';
import type React from 'react';
import { useEffect, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type {
  BackupPayload,
  DnsConfigResponse,
  KeywordConfigResponse,
  KeywordRule,
  Language,
  ServerConfig,
  SystemSettings,
  StatusResponse,
  TotpSetupResponse,
  TotpStatusResponse,
} from '../types';

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
  const [serverUrl, setServerUrl] = useState('https://famshield.ttcantho.io.vn');
  const [versionCheckUrl, setVersionCheckUrl] = useState('');
  const [showUrlSettings, setShowUrlSettings] = useState(false);
  const [isUpdatingUrls, setIsUpdatingUrls] = useState(false);
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
  const [newKwCategory, setNewKwCategory] = useState(
    isVi ? 'Chửi bậy / Thô tục' : 'Profanity / Vulgarity'
  );
  const [newKwSeverity, setNewKwSeverity] = useState('medium');
  const [isAddingKw, setIsAddingKw] = useState(false);
  const [isTogglingKwMonitoring, setIsTogglingKwMonitoring] = useState(false);

  // 2FA TOTP State
  const [totpStatus, setTotpStatus] = useState<TotpStatusResponse | null>(null);
  const [showTotpSetupModal, setShowTotpSetupModal] = useState(false);
  const [totpSetupData, setTotpSetupData] = useState<TotpSetupResponse | null>(null);
  const [totpConfirmCode, setTotpConfirmCode] = useState('');
  const [isSettingUpTotp, setIsSettingUpTotp] = useState(false);
  const [isConfirmingTotp, setIsConfirmingTotp] = useState(false);
  const [totpError, setTotpError] = useState<string | null>(null);

  const [showTotpDisableModal, setShowTotpDisableModal] = useState(false);
  const [totpDisablePin, setTotpDisablePin] = useState('');
  const [totpDisableOtp, setTotpDisableOtp] = useState('');
  const [isDisablingTotp, setIsDisablingTotp] = useState(false);
  const [totpDisableError, setTotpDisableError] = useState<string | null>(null);

  // DNS Configuration & Cache State
  const [dnsConfig, setDnsConfig] = useState<DnsConfigResponse | null>(null);
  const [selectedUpstream, setSelectedUpstream] = useState('1.1.1.1');
  const [customUpstreamInput, setCustomUpstreamInput] = useState('');
  const [isSavingDns, setIsSavingDns] = useState(false);
  const [isFlushingCache, setIsFlushingCache] = useState(false);

  // Uninstall State
  const [showUninstallModal, setShowUninstallModal] = useState(false);
  const [uninstallPin, setUninstallPin] = useState('');
  const [uninstallOtp, setUninstallOtp] = useState('');
  const [isUninstalling, setIsUninstalling] = useState(false);
  const [uninstallError, setUninstallError] = useState<string | null>(null);

  // Software Update State
  const [updateInfo, setUpdateInfo] = useState<{
    current_version: string;
    latest_version: string;
    update_available: boolean;
    release_date?: string;
    changelog?: string[];
    download_url?: string;
    check_url?: string;
    sha256?: string;
    channel?: string;
    message?: string;
  } | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [updateCheckError, setUpdateCheckError] = useState<string | null>(null);

  const handleCheckUpdate = async () => {
    setIsCheckingUpdate(true);
    setUpdateCheckError(null);
    try {
      const res = await api.checkSoftwareUpdate();
      setUpdateInfo(res);
      if (res.update_available) {
        onShowToast(
          isVi
            ? `Đã có bản cập nhật mới v${res.latest_version}!`
            : `New update v${res.latest_version} is available!`,
          'success'
        );
      } else {
        onShowToast(
          isVi
            ? `Bạn đang dùng phiên bản mới nhất v${res.current_version}.`
            : `You are on the latest version v${res.current_version}.`,
          'info'
        );
      }
    } catch (err: any) {
      setUpdateCheckError(
        err?.message ||
          (isVi ? 'Không thể kết nối máy chủ kiểm tra cập nhật' : 'Failed to connect to update server')
      );
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  const handleConfirmUninstall = async () => {
    if (!uninstallPin.trim()) {
      setUninstallError(isVi ? 'Vui lòng nhập Master PIN' : 'Please enter Master PIN');
      return;
    }
    setIsUninstalling(true);
    setUninstallError(null);
    try {
      const res = await api.uninstallSystem(uninstallPin, uninstallOtp.trim() || undefined);
      if (res.requires_otp) {
        setUninstallError(res.message);
        return;
      }
      onShowToast(res.message, 'success');
      setShowUninstallModal(false);
      setTimeout(() => {
        window.location.reload();
      }, 3000);
    } catch (err: any) {
      setUninstallError(
        err.message || (isVi ? 'Mật khẩu PIN hoặc OTP không chính xác' : 'Incorrect PIN or OTP')
      );
    } finally {
      setIsUninstalling(false);
    }
  };

  const fetchTotpStatus = async () => {
    try {
      const s = await api.getTotpStatus();
      setTotpStatus(s);
    } catch {
      // ignore
    }
  };

  const handleStartTotpSetup = async () => {
    setIsSettingUpTotp(true);
    setTotpError(null);
    setTotpConfirmCode('');
    try {
      const data = await api.setupTotp();
      setTotpSetupData(data);
      setShowTotpSetupModal(true);
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi khởi tạo 2FA' : 'Failed to initialize 2FA'), 'error');
    } finally {
      setIsSettingUpTotp(false);
    }
  };

  const handleConfirmTotp = async () => {
    if (!totpConfirmCode.trim()) {
      setTotpError(isVi ? 'Vui lòng nhập mã 6 số từ ứng dụng' : 'Please enter 6-digit code');
      return;
    }
    setIsConfirmingTotp(true);
    setTotpError(null);
    try {
      const res = await api.confirmTotp(totpConfirmCode.trim());
      onShowToast(res.message, 'success');
      setShowTotpSetupModal(false);
      await fetchTotpStatus();
    } catch (e: any) {
      setTotpError(e.message || (isVi ? 'Mã xác nhận không đúng' : 'Invalid code'));
    } finally {
      setIsConfirmingTotp(false);
    }
  };

  const handleDisableTotp = async () => {
    if (!totpDisablePin.trim()) {
      setTotpDisableError(isVi ? 'Vui lòng nhập Master PIN' : 'Please enter Master PIN');
      return;
    }
    setIsDisablingTotp(true);
    setTotpDisableError(null);
    try {
      const res = await api.disableTotp(totpDisablePin.trim(), totpDisableOtp.trim() || undefined);
      onShowToast(res.message, 'success');
      setShowTotpDisableModal(false);
      setTotpDisablePin('');
      setTotpDisableOtp('');
      await fetchTotpStatus();
    } catch (e: any) {
      setTotpDisableError(e.message || (isVi ? 'Không thể tắt 2FA' : 'Failed to disable 2FA'));
    } finally {
      setIsDisablingTotp(false);
    }
  };

  const fetchDnsConfig = async () => {
    try {
      const cfg = await api.getDnsConfig();
      setDnsConfig(cfg);
      if (cfg.current_upstream) {
        setSelectedUpstream(cfg.current_upstream);
      }
    } catch {
      // ignore
    }
  };

  const handleSaveDnsConfig = async () => {
    const target = selectedUpstream === 'custom' ? customUpstreamInput.trim() : selectedUpstream;
    if (!target) {
      onShowToast(
        isVi ? 'Vui lòng chọn hoặc nhập máy chủ DNS' : 'Please select or enter DNS server',
        'error'
      );
      return;
    }
    setIsSavingDns(true);
    try {
      const res = await api.setDnsConfig(target);
      onShowToast(
        res.message || (isVi ? 'Đã lưu cấu hình DNS' : 'DNS configured successfully'),
        'success'
      );
      await fetchDnsConfig();
    } catch (e: any) {
      onShowToast(
        e.message || (isVi ? 'Lỗi lưu cấu hình DNS' : 'Failed to save DNS configuration'),
        'error'
      );
    } finally {
      setIsSavingDns(false);
    }
  };

  const handleFlushCache = async () => {
    setIsFlushingCache(true);
    try {
      const res = await api.flushDnsCache();
      onShowToast(res.message || (isVi ? 'Đã xóa bộ nhớ đệm DNS' : 'DNS cache flushed'), 'success');
      await fetchDnsConfig();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi xóa cache DNS' : 'Failed to flush DNS cache'), 'error');
    } finally {
      setIsFlushingCache(false);
    }
  };


  const [systemSettings, setSystemSettings] = useState<SystemSettings | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isSavingSettings, setIsSavingSettings] = useState(false);

  const fetchSystemSettings = async () => {
    setIsLoadingSettings(true);
    try {
      const s = await api.getSystemSettings();
      setSystemSettings(s);
    } catch {
      // ignore
    } finally {
      setIsLoadingSettings(false);
    }
  };

  const handleSaveSystemSettings = async () => {
    if (!systemSettings) return;
    setIsSavingSettings(true);
    try {
      const updated = await api.updateSystemSettings(systemSettings);
      setSystemSettings(updated);
      onShowToast(
        isVi
          ? 'Đã cập nhật các tham số hệ thống thành công!'
          : 'System parameters updated successfully!',
        'success'
      );
    } catch (err: any) {
      onShowToast(
        err?.message || (isVi ? 'Không thể lưu tham số hệ thống' : 'Failed to save system parameters'),
        'error'
      );
    } finally {
      setIsSavingSettings(false);
    }
  };

  // Fetch current configs on mount
  useEffect(() => {
    fetchServerConfig();
    fetchSystemSettings();
    fetchKeywordConfig();
    fetchDnsConfig();
    fetchTotpStatus();
    api.checkSoftwareUpdate().then(setUpdateInfo).catch(() => {});
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
      onShowToast(
        e.message ||
          (isVi ? 'Lỗi cập nhật giám sát bàn phím' : 'Failed to update keyboard monitoring'),
        'error'
      );
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
      onShowToast(
        e.message || (isVi ? 'Lỗi cập nhật từ khóa' : 'Failed to toggle keyword'),
        'error'
      );
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
      setServerUrl(cfg.server_url || 'https://famshield.ttcantho.io.vn');
      setVersionCheckUrl(cfg.version_check_url || '');
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
      onShowToast(
        isVi ? 'Mã PIN mới phải gồm ít nhất 6 chữ số' : 'New PIN must be at least 6 digits',
        'error'
      );
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

  const handleUpdateCustomUrls = async () => {
    if (!serverUrl) {
      onShowToast(isVi ? 'URL máy chủ không được để trống' : 'Server URL cannot be empty', 'error');
      return;
    }
    setIsUpdatingUrls(true);
    try {
      const res = await api.updateServerUrl(serverUrl.trim(), versionCheckUrl.trim() || undefined);
      onShowToast(
        res.message || (isVi ? 'Đã cập nhật URL máy chủ thành công!' : 'Server URL updated successfully!'),
        'success'
      );
      setShowUrlSettings(false);
      await fetchServerConfig();
      await handleCheckUpdate();
    } catch (err: any) {
      onShowToast(err?.message || (isVi ? 'Lỗi cập nhật URL máy chủ' : 'Failed to update URL'), 'error');
    } finally {
      setIsUpdatingUrls(false);
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
        version_check_url: versionCheckUrl.trim() || undefined,
        tenant_id: tenantId.trim(),
        device_name: deviceName.trim(),
        enrollment_code: enrollCode.trim() || undefined,
        sync_interval_secs: syncInterval,
        tls_verify: tlsVerify,
        operating_mode: enrollCode.trim() ? 'OnlineManaged' : undefined,
      });

      setServerConfig(updated);
      onShowToast(
        isVi ? 'Đã lưu cấu hình máy chủ quản lý tập trung!' : 'Central server configuration saved!',
        'success'
      );
      onRefresh();
    } catch (e: any) {
      onShowToast(
        e.message || (isVi ? 'Lỗi lưu cấu hình' : 'Failed to save configuration'),
        'error'
      );
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
      onShowToast(
        e.message || (isVi ? 'Lỗi cập nhật chế độ bảo vệ' : 'Failed to update protection mode'),
        'error'
      );
    } finally {
      setIsUpdatingMode(false);
    }
  };

  // Silent Monitoring Mode State
  const [silentMode, setSilentMode] = useState<boolean>(status?.silent_mode || false);
  const [isUpdatingSilentMode, setIsUpdatingSilentMode] = useState(false);

  useEffect(() => {
    if (status && status.silent_mode !== undefined) {
      setSilentMode(status.silent_mode);
    }
  }, [status?.silent_mode]);

  const handleToggleSilentMode = async (newSilent: boolean) => {
    setIsUpdatingSilentMode(true);
    try {
      await api.setSilentMode(newSilent);
      setSilentMode(newSilent);
      onShowToast(t.silent_mode_updated, 'success');
      onRefresh();
    } catch (e: any) {
      onShowToast(
        e.message || (isVi ? 'Lỗi cập nhật chế độ giám sát yên lặng' : 'Failed to update silent mode'),
        'error'
      );
    } finally {
      setIsUpdatingSilentMode(false);
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
      {/* 0. QUICK NAVIGATION ANCHOR BAR */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          flexWrap: 'wrap',
          background: 'rgba(15, 23, 42, 0.85)',
          border: '1px solid #1e293b',
          borderRadius: 14,
          padding: '12px 18px',
          backdropFilter: 'blur(8px)',
          position: 'sticky',
          top: 70,
          zIndex: 40,
          boxShadow: '0 4px 16px rgba(0,0,0,0.2)',
        }}
      >
        <span
          style={{
            fontSize: '0.8rem',
            fontWeight: 700,
            color: '#94a3b8',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}
        >
          {isVi ? 'Chuyển nhanh:' : 'Jump to:'}
        </span>
        <button
          type="button"
          onClick={() =>
            document.getElementById('section-totp-2fa')?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: totpStatus?.enabled
              ? 'rgba(52, 211, 153, 0.15)'
              : 'rgba(234, 179, 8, 0.15)',
            border: `1px solid ${totpStatus?.enabled ? 'rgba(52, 211, 153, 0.4)' : 'rgba(234, 179, 8, 0.4)'}`,
            color: totpStatus?.enabled ? '#34d399' : '#fbbf24',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 700,
            cursor: 'pointer',
          }}
        >
          <Smartphone size={15} />
          {isVi ? '🔐 Xác Thực 2 Lớp (2FA / OTP)' : '🔐 Two-Factor (2FA / OTP)'}
          {!totpStatus?.enabled && (
            <span
              style={{
                background: '#f59e0b',
                color: '#0f172a',
                padding: '1px 6px',
                borderRadius: 10,
                fontSize: '0.7rem',
                fontWeight: 800,
              }}
            >
              {isVi ? 'Chưa Bật' : 'Off'}
            </span>
          )}
        </button>
        <button
          type="button"
          onClick={() =>
            document.getElementById('section-master-pin')?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--accent-tint)',
            border: '1px solid var(--accent-border)',
            color: 'var(--accent)',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          <Lock size={15} />
          {isVi ? '🔑 Đổi Master PIN' : '🔑 Change PIN'}
        </button>
        <button
          type="button"
          onClick={() =>
            document.getElementById('section-cloud-server')?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--accent-tint)',
            border: '1px solid var(--accent-border)',
            color: 'var(--accent)',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          <Server size={15} />
          {isVi ? '🌐 Máy Chủ Cloud' : '🌐 Cloud Server'}
        </button>
        <button
          type="button"
          onClick={() =>
            document.getElementById('section-dns-config')?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--accent-tint)',
            border: '1px solid var(--accent-border)',
            color: 'var(--accent)',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          <Globe size={15} />
          {isVi ? '⚡ DNS & Bộ Nhớ Đệm' : '⚡ DNS & Cache'}
        </button>
        <button
          type="button"
          onClick={() =>
            document
              .getElementById('section-keywords-monitor')
              ?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--accent-tint)',
            border: '1px solid var(--accent-border)',
            color: 'var(--accent)',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          <Keyboard size={15} />
          {isVi ? '⌨️ Giám Sát Từ Khóa' : '⌨️ Keywords'}
        </button>
        <button
          type="button"
          onClick={() =>
            document.getElementById('section-danger-zone')?.scrollIntoView({ behavior: 'smooth' })
          }
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            background: 'rgba(239, 68, 68, 0.1)',
            border: '1px solid rgba(239, 68, 68, 0.3)',
            color: '#f87171',
            padding: '6px 14px',
            borderRadius: 8,
            fontSize: '0.82rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          <Trash2 size={15} />
          {isVi ? '🗑️ Gỡ Cài Đặt' : '🗑️ Uninstall'}
        </button>
      </div>

      {/* TOP PROMINENT 2FA ACTIVATION BANNER */}
      {!totpStatus?.enabled && (
        <div
          style={{
            background:
              'linear-gradient(135deg, rgba(245, 158, 11, 0.18) 0%, rgba(15, 23, 42, 0.95) 100%)',
            border: '1px solid rgba(245, 158, 11, 0.5)',
            borderRadius: 16,
            padding: '20px 24px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
            boxShadow: '0 6px 24px rgba(245, 158, 11, 0.15)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
            <div
              style={{
                background: 'rgba(245, 158, 11, 0.25)',
                padding: 12,
                borderRadius: 12,
                color: '#f59e0b',
              }}
            >
              <Smartphone size={28} />
            </div>
            <div>
              <div
                style={{
                  fontWeight: 800,
                  fontSize: '1.05rem',
                  color: '#fef3c7',
                  marginBottom: 4,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                {isVi
                  ? 'Bảo Vệ Tài Khoản Phụ Huynh: Chưa Kích Hoạt Xác Thực 2 Lớp (2FA / OTP)'
                  : 'Parent Account Security: 2FA / OTP is Disabled'}
                <span
                  style={{
                    background: '#f59e0b',
                    color: '#0f172a',
                    padding: '2px 8px',
                    borderRadius: 6,
                    fontSize: '0.72rem',
                    fontWeight: 800,
                  }}
                >
                  {isVi ? 'KHUYÊN DÙNG' : 'RECOMMENDED'}
                </span>
              </div>
              <div style={{ fontSize: '0.85rem', color: '#cbd5e1', maxWidth: 680 }}>
                {isVi
                  ? 'Bảo vệ bảng điều khiển chống nhìn lén Master PIN bằng mã OTP 6 chữ số sinh từ Google Authenticator, Microsoft Authenticator hoặc Apple Keychain (chuẩn RFC 6238, 100% ngoại tuyến).'
                  : 'Protect the parental dashboard against shoulder-surfing using 6-digit TOTP codes from Google/Microsoft Authenticator or Apple Keychain (RFC 6238, 100% offline).'}
              </div>
            </div>
          </div>
          <button
            type="button"
            onClick={handleStartTotpSetup}
            disabled={isSettingUpTotp}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              background: '#f59e0b',
              color: '#0f172a',
              border: 'none',
              padding: '11px 24px',
              borderRadius: 10,
              fontSize: '0.92rem',
              fontWeight: 800,
              cursor: isSettingUpTotp ? 'not-allowed' : 'pointer',
              boxShadow: '0 4px 16px rgba(245, 158, 11, 0.4)',
              whiteSpace: 'nowrap',
            }}
          >
            <Smartphone size={18} />
            {isSettingUpTotp
              ? isVi
                ? 'Đang tạo mã...'
                : 'Setting up...'
              : isVi
                ? '👉 Kích Hoạt 2FA Ngay'
                : '👉 Enable 2FA Now'}
          </button>
        </div>
      )}

      {/* SECTION 1: CENTRALIZED MANAGEMENT SERVER SETTINGS */}
      <div
        id="section-cloud-server"
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
          boxShadow: '0 10px 30px rgba(0,0,0,0.3)',
        }}
      >
        {/* Header */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            marginBottom: 24,
          }}
        >
          <div>
            <h2
              style={{
                fontSize: '1.35rem',
                fontWeight: 800,
                color: '#f8fafc',
                display: 'flex',
                alignItems: 'center',
                gap: 12,
              }}
            >
              <Server size={24} color="var(--accent)" />
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
            <span
              style={{
                fontSize: '0.82rem',
                fontWeight: 700,
                color: isEnrolled ? '#34d399' : '#cbd5e1',
              }}
            >
              {isEnrolled
                ? isVi
                  ? 'Đang Quản Lý Tập Trung'
                  : 'Centralized Online'
                : isVi
                  ? 'Ngoại Tuyến Độc Lập (Standalone)'
                  : 'Standalone Offline'}
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
            <div
              style={{
                fontSize: '0.75rem',
                color: '#94a3b8',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
              }}
            >
              {isVi ? 'Mã Thiết Bị (Device ID)' : 'Device ID'}
            </div>
            <div
              style={{
                fontSize: '0.95rem',
                fontWeight: 700,
                color: '#38bdf8',
                marginTop: 4,
                fontFamily: 'monospace',
              }}
            >
              {serverConfig?.device_id || 'standalone-local-001'}
            </div>
          </div>

          <div>
            <div
              style={{
                fontSize: '0.75rem',
                color: '#94a3b8',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
              }}
            >
              {isVi ? 'Mã Tổ Chức (Tenant ID)' : 'Tenant ID'}
            </div>
            <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#f8fafc', marginTop: 4 }}>
              {serverConfig?.tenant_id || 'tenant-default'}
            </div>
          </div>

          <div>
            <div
              style={{
                fontSize: '0.75rem',
                color: '#94a3b8',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
              }}
            >
              {t.metric_last_sync}
            </div>
            <div
              style={{
                fontSize: '0.92rem',
                fontWeight: 600,
                color: serverConfig?.last_sync_time ? '#34d399' : '#64748b',
                marginTop: 4,
              }}
            >
              {serverConfig?.last_sync_time
                ? new Date(serverConfig.last_sync_time).toLocaleTimeString(isVi ? 'vi-VN' : 'en-US')
                : isVi
                  ? 'Chưa đồng bộ'
                  : 'Never'}
            </div>
          </div>

          <div>
            <div
              style={{
                fontSize: '0.75rem',
                color: '#94a3b8',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
              }}
            >
              {t.metric_policy_ver}
            </div>
            <div
              style={{ fontSize: '0.95rem', fontWeight: 700, color: 'var(--accent)', marginTop: 4 }}
            >
              v{serverConfig?.policy_version || 1}
            </div>
          </div>
        </div>

        {/* Quick Operations: Test Connection & Sync Now */}
        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: 12,
            alignItems: 'center',
            marginBottom: 24,
          }}
        >
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
                background: testResult.success
                  ? 'rgba(16, 185, 129, 0.15)'
                  : 'rgba(239, 68, 68, 0.15)',
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
        <form
          onSubmit={handleSaveServerConfig}
          style={{ display: 'flex', flexDirection: 'column', gap: 18 }}
        >
          {/* Server URL with quick presets */}
          <div>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: 6,
              }}
            >
              <label style={{ fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600 }}>
                {t.lbl_server_url}
              </label>
              {/* Presets */}
              <div style={{ display: 'flex', gap: 8 }}>
                <button
                  type="button"
                  onClick={() => setServerUrl('https://famshield.ttcantho.io.vn')}
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
              placeholder="https://famshield.ttcantho.io.vn"
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

          {/* Version Check URL (Optional / Custom) */}
          <div style={{ marginBottom: 16 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <label
                style={{
                  fontSize: '0.85rem',
                  color: '#cbd5e1',
                  fontWeight: 600,
                }}
              >
                {isVi ? 'URL Kiểm Tra Bản Cập Nhật (Tùy Chọn):' : 'Version Check URL (Optional):'}
              </label>
              <span style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                {isVi ? 'Mặc định: Tự động theo URL Máy Chủ' : 'Default: Auto-derived from Server URL'}
              </span>
            </div>
            <input
              type="text"
              value={versionCheckUrl}
              onChange={(e) => setVersionCheckUrl(e.target.value)}
              placeholder={isVi ? 'Để trống để tự động sinh từ URL Máy Chủ' : 'Leave empty to auto-derive from Server URL'}
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
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))',
              gap: 16,
            }}
          >
            <div>
              <label
                style={{
                  display: 'block',
                  fontSize: '0.85rem',
                  color: '#cbd5e1',
                  fontWeight: 600,
                  marginBottom: 6,
                }}
              >
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
              <span style={{ fontSize: '0.75rem', color: '#94a3b8', marginTop: 5, display: 'block' }}>
                Định dạng chuẩn: tenant-usr-&lt;id&gt;. Tự động cập nhật khi ghép nối với máy chủ.
              </span>
            </div>

            <div>
              <label
                style={{
                  display: 'block',
                  fontSize: '0.85rem',
                  color: '#cbd5e1',
                  fontWeight: 600,
                  marginBottom: 6,
                }}
              >
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
              <span style={{ fontSize: '0.75rem', color: '#94a3b8', marginTop: 5, display: 'block' }}>
                Tên hiển thị nhận diện máy tính. Tự động gắn hậu tố Machine GUID chống trùng lặp tên.
              </span>
            </div>
          </div>

          {/* Grid: Enrollment Code & Sync Interval */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))',
              gap: 16,
            }}
          >
            <div>
              <label
                style={{
                  display: 'block',
                  fontSize: '0.85rem',
                  color: '#cbd5e1',
                  fontWeight: 600,
                  marginBottom: 6,
                }}
              >
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
              <label
                style={{
                  display: 'block',
                  fontSize: '0.85rem',
                  color: '#cbd5e1',
                  fontWeight: 600,
                  marginBottom: 6,
                }}
              >
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
              style={{ width: 18, height: 18, accentColor: 'var(--accent)', cursor: 'pointer' }}
            />
          </div>

          {/* Action Buttons */}
          <div style={{ display: 'flex', gap: 12, marginTop: 8 }}>
            <button
              type="submit"
              disabled={isSavingServer}
              style={{
                flex: 2,
                background: 'linear-gradient(135deg, var(--accent), #38bdf8)',
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
                boxShadow: '0 4px 15px var(--accent-shadow)',
              }}
            >
              <Cloud size={18} />
              <span>
                {isSavingServer ? (isVi ? 'Đang lưu...' : 'Saving...') : t.btn_save_server}
              </span>
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
          <h2
            style={{
              fontSize: '1.25rem',
              fontWeight: 700,
              color: '#f8fafc',
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
            <Eye size={22} color="#38bdf8" />
            {t.title_protection_mode}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_protection_mode}
          </p>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
            gap: 16,
          }}
        >
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
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: 8,
              }}
            >
              <span
                style={{
                  fontSize: '1rem',
                  fontWeight: 700,
                  color: !monitorMode ? '#34d399' : '#e2e8f0',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <ShieldCheck size={18} color={!monitorMode ? '#34d399' : '#94a3b8'} />
                {t.mode_blocking}
              </span>
              {!monitorMode && (
                <span
                  style={{
                    background: '#34d399',
                    color: '#070b14',
                    borderRadius: '50%',
                    padding: 2,
                    display: 'flex',
                  }}
                >
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
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: 8,
              }}
            >
              <span
                style={{
                  fontSize: '1rem',
                  fontWeight: 700,
                  color: monitorMode ? '#fbbf24' : '#e2e8f0',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <Eye size={18} color={monitorMode ? '#fbbf24' : '#94a3b8'} />
                {t.mode_monitor}
              </span>
              {monitorMode && (
                <span
                  style={{
                    background: '#f59e0b',
                    color: '#070b14',
                    borderRadius: '50%',
                    padding: 2,
                    display: 'flex',
                  }}
                >
                  <Check size={14} />
                </span>
              )}
            </div>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
              {t.mode_monitor_desc}
            </p>
          </div>
        </div>

        {/* Silent Monitoring Mode & Stealth DNS Blocking Sub-Panel */}
        <div
          style={{
            marginTop: 20,
            background: 'rgba(15, 23, 42, 0.6)',
            border: '1px solid #1e293b',
            borderRadius: 14,
            padding: 20,
            display: 'flex',
            flexDirection: 'column',
            gap: 16,
          }}
        >
          {/* Row 1: Silent Mode Toggle */}
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              flexWrap: 'wrap',
              gap: 12,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, flex: 1, minWidth: 280 }}>
              <div
                style={{
                  width: 40,
                  height: 40,
                  borderRadius: 10,
                  background: silentMode ? 'rgba(168, 85, 247, 0.15)' : 'rgba(148, 163, 184, 0.1)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: silentMode ? '#c084fc' : '#94a3b8',
                }}
              >
                <BellOff size={20} />
              </div>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontSize: '0.95rem', fontWeight: 700, color: '#f8fafc' }}>
                    {t.title_silent_mode}
                  </span>
                  <span
                    style={{
                      fontSize: '0.72rem',
                      fontWeight: 600,
                      padding: '2px 8px',
                      borderRadius: 12,
                      background: silentMode ? 'rgba(168, 85, 247, 0.2)' : 'rgba(100, 116, 139, 0.2)',
                      color: silentMode ? '#d8b4fe' : '#94a3b8',
                      border: `1px solid ${silentMode ? 'rgba(168, 85, 247, 0.4)' : '#334155'}`,
                    }}
                  >
                    {silentMode ? (isVi ? 'Đang Bật' : 'Enabled') : (isVi ? 'Đang Tắt' : 'Disabled')}
                  </span>
                </div>
                <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: '4px 0 0', lineHeight: 1.4 }}>
                  {t.desc_silent_mode}
                </p>
              </div>
            </div>

            <button
              type="button"
              disabled={isUpdatingSilentMode}
              onClick={() => handleToggleSilentMode(!silentMode)}
              style={{
                background: silentMode ? '#9333ea' : '#1e293b',
                color: '#ffffff',
                border: 'none',
                borderRadius: 10,
                padding: '10px 18px',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: isUpdatingSilentMode ? 'not-allowed' : 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s ease',
              }}
            >
              <BellOff size={16} />
              {silentMode
                ? (isVi ? 'Tắt Giám Sát Yên Lặng' : 'Disable Silent Mode')
                : (isVi ? 'Bật Giám Sát Yên Lặng' : 'Enable Silent Mode')}
            </button>
          </div>

          {/* Row 2: Stealth DNS Blocking Mechanism Notice */}
          <div
            style={{
              background: 'rgba(56, 189, 248, 0.05)',
              border: '1px solid rgba(56, 189, 248, 0.2)',
              borderRadius: 10,
              padding: '12px 16px',
              display: 'flex',
              alignItems: 'flex-start',
              gap: 12,
            }}
          >
            <Globe size={18} color="#38bdf8" style={{ marginTop: 2, flexShrink: 0 }} />
            <div>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#38bdf8', marginBottom: 2 }}>
                {t.dns_nxdomain_title}
              </div>
              <div style={{ fontSize: '0.8rem', color: '#cbd5e1', lineHeight: 1.45 }}>
                {t.dns_nxdomain_desc}
              </div>
            </div>
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
          <h2
            style={{
              fontSize: '1.25rem',
              fontWeight: 700,
              color: '#f8fafc',
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
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
            {isCreatingBackup ? (
              <RefreshCw size={16} className="animate-spin" />
            ) : (
              <Download size={16} />
            )}
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
        id="section-master-pin"
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 20,
          padding: 28,
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2
            style={{
              fontSize: '1.25rem',
              fontWeight: 700,
              color: '#f8fafc',
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
            <Lock size={22} color="#38bdf8" />
            {t.title_change_pin}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>{t.desc_change_pin}</p>
        </div>

        <form
          onSubmit={handleChangePin}
          style={{ display: 'flex', flexDirection: 'column', gap: 14, maxWidth: 500 }}
        >
          <div>
            <label
              style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
            >
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
              <label
                style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
              >
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
              <label
                style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
              >
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
            <div
              style={{ fontSize: '0.82rem', color: '#34d399', fontWeight: 600, marginBottom: 6 }}
            >
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
        id="section-keywords-monitor"
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
                keywordConfig?.keyboard_monitoring_enabled
                  ? 'rgba(52, 211, 153, 0.4)'
                  : 'rgba(239, 68, 68, 0.4)'
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
          <div
            style={{
              background: '#070b14',
              border: '1px solid #1e293b',
              borderRadius: 12,
              padding: '14px 16px',
            }}
          >
            <div
              style={{
                fontSize: '0.78rem',
                color: '#64748b',
                fontWeight: 600,
                textTransform: 'uppercase',
              }}
            >
              {isVi ? 'Tổng số từ khóa đang theo dõi' : 'Total Monitored Keywords'}
            </div>
            <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#38bdf8', marginTop: 4 }}>
              {keywordConfig?.total_rules_count || 0}
            </div>
          </div>

          <div
            style={{
              background: '#070b14',
              border: '1px solid #1e293b',
              borderRadius: 12,
              padding: '14px 16px',
            }}
          >
            <div
              style={{
                fontSize: '0.78rem',
                color: '#64748b',
                fontWeight: 600,
                textTransform: 'uppercase',
              }}
            >
              {isVi ? 'Vi phạm phát hiện hôm nay' : 'Violations Detected Today'}
            </div>
            <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#fbbf24', marginTop: 4 }}>
              {keywordConfig?.violations_today_count || 0}
            </div>
          </div>

          <div
            style={{
              background: '#070b14',
              border: '1px solid #1e293b',
              borderRadius: 12,
              padding: '14px 16px',
            }}
          >
            <div
              style={{
                fontSize: '0.78rem',
                color: '#64748b',
                fontWeight: 600,
                textTransform: 'uppercase',
              }}
            >
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
            <label
              style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
            >
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
            <label
              style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
            >
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
            <label
              style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
            >
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
          <div
            style={{
              marginBottom: 14,
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <input
              type="text"
              placeholder={
                isVi ? 'Tìm kiếm từ khóa trong danh sách...' : 'Search monitored keywords...'
              }
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
            <table
              style={{
                width: '100%',
                borderCollapse: 'collapse',
                textAlign: 'left',
                fontSize: '0.85rem',
              }}
            >
              <thead>
                <tr
                  style={{
                    borderBottom: '1px solid #1e293b',
                    color: '#64748b',
                    fontSize: '0.78rem',
                    textTransform: 'uppercase',
                  }}
                >
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_pattern}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_category}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_severity}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_type}</th>
                  <th style={{ padding: '10px 14px' }}>{t.th_kw_status}</th>
                  <th style={{ padding: '10px 14px', textAlign: 'right' }}>
                    {isVi ? 'Hành động' : 'Action'}
                  </th>
                </tr>
              </thead>
              <tbody>
                {(keywordConfig?.rules || [])
                  .filter(
                    (r) =>
                      r.pattern.toLowerCase().includes(kwSearch.toLowerCase()) ||
                      r.category_name.toLowerCase().includes(kwSearch.toLowerCase())
                  )
                  .map((rule) => (
                    <tr key={rule.id} style={{ borderBottom: '1px solid rgba(30, 41, 59, 0.6)' }}>
                      <td style={{ padding: '10px 14px', fontWeight: 700, color: '#f8fafc' }}>
                        "{rule.pattern}"
                      </td>
                      <td style={{ padding: '10px 14px', color: '#cbd5e1' }}>
                        <span
                          style={{
                            background: '#1e293b',
                            padding: '2px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                          }}
                        >
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
                          <span style={{ color: 'var(--accent)', fontWeight: 600 }}>
                            {isVi ? 'Phụ huynh' : 'Custom'}
                          </span>
                        ) : (
                          <span>{isVi ? 'Hệ thống' : 'Built-in'}</span>
                        )}
                      </td>
                      <td style={{ padding: '10px 14px' }}>
                        <button
                          type="button"
                          onClick={() => handleToggleKeyword(rule)}
                          style={{
                            background: rule.enabled
                              ? 'rgba(52, 211, 153, 0.15)'
                              : 'rgba(100, 116, 139, 0.15)',
                            color: rule.enabled ? '#34d399' : '#64748b',
                            border: `1px solid ${rule.enabled ? 'rgba(52, 211, 153, 0.3)' : 'rgba(100, 116, 139, 0.3)'}`,
                            padding: '3px 9px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 700,
                            cursor: 'pointer',
                          }}
                        >
                          {rule.enabled ? (isVi ? 'BẬT' : 'ON') : isVi ? 'TẮT' : 'OFF'}
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

      {/* 2FA / TOTP Authenticator for Parents */}
      <div
        id="section-totp-2fa"
        style={{
          background: 'rgba(30, 41, 59, 0.4)',
          border: '1px solid rgba(255, 255, 255, 0.08)',
          borderRadius: 14,
          padding: 24,
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
            marginBottom: 20,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: '#f8fafc',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                marginBottom: 4,
              }}
            >
              <Smartphone size={18} style={{ color: '#34d399' }} />
              {isVi
                ? 'Xác Thực 2 Lớp (2FA / TOTP) Cho Phụ Huynh'
                : 'Parent Two-Factor Authentication (2FA)'}
            </h3>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', margin: 0, maxWidth: 640 }}>
              {isVi
                ? 'Bảo vệ bảng điều khiển chống nhìn lén Master PIN bằng mã OTP 6 chữ số sinh từ Google Authenticator, Microsoft Authenticator hoặc Apple Keychain (chuẩn RFC 6238, 100% ngoại tuyến).'
                : 'Protect the parental dashboard against shoulder-surfing using 6-digit TOTP codes from Google/Microsoft Authenticator or Apple Keychain (RFC 6238, 100% offline).'}
            </p>
          </div>
          <div>
            {totpStatus?.enabled ? (
              <button
                type="button"
                onClick={() => {
                  setTotpDisablePin('');
                  setTotpDisableOtp('');
                  setTotpDisableError(null);
                  setShowTotpDisableModal(true);
                }}
                style={{
                  background: 'rgba(239, 68, 68, 0.1)',
                  color: '#f87171',
                  border: '1px solid rgba(239, 68, 68, 0.3)',
                  padding: '8px 16px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {isVi ? 'Tắt Xác Thực 2 Lớp' : 'Disable 2FA'}
              </button>
            ) : (
              <button
                type="button"
                onClick={handleStartTotpSetup}
                disabled={isSettingUpTotp}
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 8,
                  background: '#059669',
                  color: '#ffffff',
                  border: 'none',
                  padding: '8px 18px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  cursor: isSettingUpTotp ? 'not-allowed' : 'pointer',
                }}
              >
                <Smartphone size={16} />
                {isSettingUpTotp
                  ? isVi
                    ? 'Đang tạo mã...'
                    : 'Setting up...'
                  : isVi
                    ? 'Kích Hoạt 2FA Authenticator'
                    : 'Enable 2FA Authenticator'}
              </button>
            )}
          </div>
        </div>

        {/* 2FA Status summary */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: 12,
          }}
        >
          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Trạng Thái Bảo Vệ' : 'Protection Status'}
            </span>
            <span
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: totpStatus?.enabled ? '#34d399' : '#94a3b8',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              {totpStatus?.enabled ? (
                <>
                  <CheckCircle2 size={16} color="#34d399" />
                  {isVi ? 'Đang Bật (Đã Bảo Vệ 2 Lớp)' : 'Active & Protected'}
                </>
              ) : isVi ? (
                'Chưa Bật (Chỉ dùng Master PIN)'
              ) : (
                'Disabled (PIN Only)'
              )}
            </span>
          </div>

          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Mã Dự Phòng Khẩn Cấp Còn Lại' : 'Available Backup Codes'}
            </span>
            <span
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: totpStatus?.enabled ? '#38bdf8' : '#64748b',
              }}
            >
              {totpStatus?.enabled
                ? `${totpStatus.unused_backup_codes} / 8 mã`
                : isVi
                  ? 'Chưa cấp phát'
                  : 'Not configured'}
            </span>
          </div>

          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Lần Đăng Nhập OTP Gần Nhất' : 'Last OTP Login'}
            </span>
            <span style={{ fontSize: '0.85rem', fontWeight: 600, color: '#e2e8f0' }}>
              {totpStatus?.last_verified_at
                ? new Date(totpStatus.last_verified_at).toLocaleString()
                : isVi
                  ? 'Chưa có'
                  : 'Never'}
            </span>
          </div>
        </div>
      </div>

      {/* DNS Configuration & In-Memory Cache Management */}
      <div
        id="section-dns-config"
        style={{
          background: 'rgba(30, 41, 59, 0.4)',
          border: '1px solid rgba(255, 255, 255, 0.08)',
          borderRadius: 14,
          padding: 24,
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
            marginBottom: 20,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: '#f8fafc',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                marginBottom: 4,
              }}
            >
              <Globe size={18} style={{ color: '#38bdf8' }} />
              {isVi
                ? 'Cấu Hình DNS Chuyển Tiếp & Bộ Nhớ Đệm (DNS Upstream & Cache)'
                : 'DNS Upstream & Cache Management'}
            </h3>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', margin: 0, maxWidth: 640 }}>
              {isVi
                ? 'FamShield chuyển tiếp truy vấn phân giải tên miền an toàn qua danh sách upstream dự phòng (failover) và cache phản hồi cục bộ trong RAM (≤ 2 MB) để tăng tốc độ lướt web.'
                : 'FamShield forwards DNS queries through fallback upstream servers and caches responses locally in RAM (≤ 2 MB) to accelerate browsing speed.'}
            </p>
          </div>
          <button
            type="button"
            onClick={handleFlushCache}
            disabled={isFlushingCache}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              background: 'rgba(56, 189, 248, 0.1)',
              color: '#38bdf8',
              border: '1px solid rgba(56, 189, 248, 0.3)',
              padding: '8px 16px',
              borderRadius: 8,
              fontSize: '0.85rem',
              fontWeight: 600,
              cursor: isFlushingCache ? 'not-allowed' : 'pointer',
            }}
          >
            <RefreshCw size={15} className={isFlushingCache ? 'animate-spin' : ''} />
            {isVi ? 'Xóa Bộ Nhớ Đệm DNS' : 'Flush DNS Cache'}
          </button>
        </div>

        {/* Cache Statistics Row */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
            gap: 12,
            marginBottom: 20,
          }}
        >
          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Bản Ghi Đang Cache' : 'Active Cached Records'}
            </span>
            <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#38bdf8' }}>
              {dnsConfig?.cache_entries ?? 0}{' '}
              <span style={{ fontSize: '0.8rem', color: '#64748b' }}>/ 2.000 (RAM ≤ 2MB)</span>
            </span>
          </div>
          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Tỷ Lệ Trúng Cache (Hits)' : 'Cache Hits'}
            </span>
            <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#34d399' }}>
              {dnsConfig?.cache_hits ?? 0}
            </span>
          </div>
          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'Yêu Cầu Ra Upstream (Misses)' : 'Upstream Queries (Misses)'}
            </span>
            <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#fbbf24' }}>
              {dnsConfig?.cache_misses ?? 0}
            </span>
          </div>
          <div
            style={{
              background: 'rgba(15, 23, 42, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.05)',
              borderRadius: 10,
              padding: 14,
            }}
          >
            <span
              style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
            >
              {isVi ? 'DNS Gốc Hệ Thống Nhận Diện' : 'Detected System DNS'}
            </span>
            <span style={{ fontSize: '0.95rem', fontWeight: 600, color: '#a78bfa' }}>
              {dnsConfig?.detected_system_dns && dnsConfig.detected_system_dns.length > 0
                ? dnsConfig.detected_system_dns.join(', ')
                : isVi
                  ? 'Không phát hiện'
                  : 'None detected'}
            </span>
          </div>
        </div>

        {/* Upstream Selector */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <label style={{ fontSize: '0.85rem', fontWeight: 600, color: '#cbd5e1' }}>
            {isVi
              ? 'Chọn Máy Chủ DNS Chuyển Tiếp Ưu Tiên (Primary Upstream):'
              : 'Select Primary Upstream DNS Server:'}
          </label>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
              gap: 10,
            }}
          >
            {/* Detected System DNS Option (if any) */}
            {dnsConfig?.detected_system_dns?.map((sysDns) => (
              <div
                key={sysDns}
                onClick={() => setSelectedUpstream(sysDns)}
                style={{
                  padding: '12px 14px',
                  borderRadius: 10,
                  cursor: 'pointer',
                  border: `1px solid ${selectedUpstream === sysDns ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                  background:
                    selectedUpstream === sysDns
                      ? 'rgba(56, 189, 248, 0.12)'
                      : 'rgba(15, 23, 42, 0.4)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div>
                  <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                    {isVi ? 'DNS Gốc Của Máy / Router' : 'System / Router DNS'}
                  </div>
                  <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>{sysDns}</div>
                </div>
                {selectedUpstream === sysDns && <Check size={16} color="#38bdf8" />}
              </div>
            ))}

            {/* Cloudflare 1.1.1.1 */}
            <div
              onClick={() => setSelectedUpstream('1.1.1.1')}
              style={{
                padding: '12px 14px',
                borderRadius: 10,
                cursor: 'pointer',
                border: `1px solid ${selectedUpstream === '1.1.1.1' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                background:
                  selectedUpstream === '1.1.1.1'
                    ? 'rgba(56, 189, 248, 0.12)'
                    : 'rgba(15, 23, 42, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                  Cloudflare DNS
                </div>
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                  1.1.1.1 (Tốc độ cao & Riêng tư)
                </div>
              </div>
              {selectedUpstream === '1.1.1.1' && <Check size={16} color="#38bdf8" />}
            </div>

            {/* Google DNS 8.8.8.8 */}
            <div
              onClick={() => setSelectedUpstream('8.8.8.8')}
              style={{
                padding: '12px 14px',
                borderRadius: 10,
                cursor: 'pointer',
                border: `1px solid ${selectedUpstream === '8.8.8.8' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                background:
                  selectedUpstream === '8.8.8.8'
                    ? 'rgba(56, 189, 248, 0.12)'
                    : 'rgba(15, 23, 42, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                  Google Public DNS
                </div>
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                  8.8.8.8 (Độ ổn định toàn cầu)
                </div>
              </div>
              {selectedUpstream === '8.8.8.8' && <Check size={16} color="#38bdf8" />}
            </div>

            {/* Quad9 9.9.9.9 */}
            <div
              onClick={() => setSelectedUpstream('9.9.9.9')}
              style={{
                padding: '12px 14px',
                borderRadius: 10,
                cursor: 'pointer',
                border: `1px solid ${selectedUpstream === '9.9.9.9' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                background:
                  selectedUpstream === '9.9.9.9'
                    ? 'rgba(56, 189, 248, 0.12)'
                    : 'rgba(15, 23, 42, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                  Quad9 DNS
                </div>
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                  9.9.9.9 (Lọc mã độc đám mây)
                </div>
              </div>
              {selectedUpstream === '9.9.9.9' && <Check size={16} color="#38bdf8" />}
            </div>

            {/* Custom Option */}
            <div
              onClick={() => setSelectedUpstream('custom')}
              style={{
                padding: '12px 14px',
                borderRadius: 10,
                cursor: 'pointer',
                border: `1px solid ${selectedUpstream === 'custom' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                background:
                  selectedUpstream === 'custom'
                    ? 'rgba(56, 189, 248, 0.12)'
                    : 'rgba(15, 23, 42, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                  {isVi ? 'Tùy Chỉnh Khác...' : 'Custom Upstream...'}
                </div>
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>IP:Port hoặc DNS nội bộ</div>
              </div>
              {selectedUpstream === 'custom' && <Check size={16} color="#38bdf8" />}
            </div>
          </div>

          {selectedUpstream === 'custom' && (
            <div style={{ marginTop: 8 }}>
              <input
                type="text"
                value={customUpstreamInput}
                onChange={(e) => setCustomUpstreamInput(e.target.value)}
                placeholder={
                  isVi
                    ? 'Nhập địa chỉ DNS upstream (ví dụ: 192.168.1.1 hoặc 10.0.0.1:53)'
                    : 'Enter DNS upstream address (e.g., 192.168.1.1 or 10.0.0.1:53)'
                }
                style={{
                  width: '100%',
                  background: 'rgba(15, 23, 42, 0.6)',
                  border: '1px solid rgba(255, 255, 255, 0.15)',
                  borderRadius: 8,
                  padding: '10px 14px',
                  color: '#ffffff',
                  fontSize: '0.85rem',
                }}
              />
            </div>
          )}

          <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 12 }}>
            <button
              type="button"
              onClick={handleSaveDnsConfig}
              disabled={isSavingDns}
              style={{
                background: '#0284c7',
                color: '#ffffff',
                border: 'none',
                padding: '9px 20px',
                borderRadius: 8,
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: isSavingDns ? 'not-allowed' : 'pointer',
                display: 'inline-flex',
                alignItems: 'center',
                gap: 8,
              }}
            >
              {isSavingDns
                ? isVi
                  ? 'Đang lưu...'
                  : 'Saving...'
                : isVi
                  ? 'Áp Dụng Cấu Hình DNS'
                  : 'Apply DNS Settings'}
            </button>
          </div>
        </div>

        {/* NUMA Dynamic Learning DNS Balancer Diagnostics */}
        {dnsConfig?.upstream_diagnostics && dnsConfig.upstream_diagnostics.length > 0 && (
          <div
            style={{
              marginTop: 18,
              padding: 16,
              borderRadius: 12,
              background: 'rgba(15, 23, 42, 0.5)',
              border: '1px solid rgba(56, 189, 248, 0.2)',
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexWrap: 'wrap',
                gap: 8,
                marginBottom: 12,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Activity size={16} color="#38bdf8" />
                <span style={{ fontSize: '0.85rem', fontWeight: 700, color: '#f8fafc' }}>
                  {isVi
                    ? 'Bộ Cân Bằng Tải Tự Học NUMA (Dynamic DNS Balancer)'
                    : 'NUMA Dynamic Learning DNS Balancer'}
                </span>
              </div>
              {dnsConfig.optimal_upstream && (
                <span
                  style={{
                    fontSize: '0.75rem',
                    fontWeight: 600,
                    background: 'rgba(56, 189, 248, 0.15)',
                    color: '#38bdf8',
                    border: '1px solid rgba(56, 189, 248, 0.3)',
                    padding: '3px 8px',
                    borderRadius: 6,
                  }}
                >
                  🚀 {isVi ? 'Đang chọn tối ưu:' : 'Active Optimal:'} {dnsConfig.optimal_upstream}
                </span>
              )}
            </div>

            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
                gap: 10,
              }}
            >
              {dnsConfig.upstream_diagnostics.map((diag) => (
                <div
                  key={diag.addr}
                  style={{
                    padding: 12,
                    borderRadius: 8,
                    background: diag.is_optimal
                      ? 'rgba(56, 189, 248, 0.08)'
                      : 'rgba(255, 255, 255, 0.03)',
                    border: `1px solid ${
                      diag.is_optimal ? 'rgba(56, 189, 248, 0.4)' : 'rgba(255, 255, 255, 0.06)'
                    }`,
                  }}
                >
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      marginBottom: 6,
                    }}
                  >
                    <span style={{ fontSize: '0.8rem', fontWeight: 700, color: '#f8fafc' }}>
                      {diag.target}
                    </span>
                    {diag.is_optimal && (
                      <span
                        style={{
                          fontSize: '0.65rem',
                          fontWeight: 700,
                          color: '#38bdf8',
                          background: 'rgba(56, 189, 248, 0.2)',
                          padding: '1px 6px',
                          borderRadius: 4,
                        }}
                      >
                        OPTIMAL
                      </span>
                    )}
                  </div>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      fontSize: '0.75rem',
                      color: '#94a3b8',
                      marginBottom: 3,
                    }}
                  >
                    <span>{isVi ? 'Độ trễ (RTT):' : 'Latency (RTT):'}</span>
                    <strong
                      style={{
                        color:
                          diag.latency_ms < 25
                            ? '#4ade80'
                            : diag.latency_ms < 80
                              ? '#facc15'
                              : '#f87171',
                      }}
                    >
                      {diag.latency_ms} ms
                    </strong>
                  </div>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      fontSize: '0.7rem',
                      color: '#64748b',
                    }}
                  >
                    <span>{isVi ? 'Thành công:' : 'Success:'}</span>
                    <span>
                      {diag.success_count} / {diag.total_queries}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Software Update & Version Management */}
      <div
        id="section-software-updates"
        style={{
          background: updateInfo?.update_available ? 'rgba(56, 189, 248, 0.06)' : 'var(--bg-card)',
          border: updateInfo?.update_available ? '1px solid rgba(56, 189, 248, 0.4)' : '1px solid var(--border-card)',
          borderRadius: 14,
          padding: 24,
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
            marginBottom: updateInfo?.update_available ? 18 : 0,
          }}
        >
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 6 }}>
              <h3
                style={{
                  fontSize: '1.05rem',
                  fontWeight: 700,
                  color: 'var(--text-main)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  margin: 0,
                }}
              >
                <RefreshCw size={18} className={isCheckingUpdate ? 'animate-spin text-sky-400' : 'text-sky-400'} />
                {isVi ? 'Phiên Bản & Cập Nhật Phần Mềm' : 'Software Version & Updates'}
              </h3>
              <span
                style={{
                  padding: '2px 8px',
                  borderRadius: 12,
                  fontSize: '0.75rem',
                  fontWeight: 700,
                  background: updateInfo?.update_available ? '#f59e0b20' : '#10b98120',
                  color: updateInfo?.update_available ? '#fbbf24' : '#34d399',
                  border: updateInfo?.update_available ? '1px solid #f59e0b40' : '1px solid #10b98140',
                }}
              >
                v{updateInfo?.current_version || '1.0.3'}
              </span>
            </div>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', margin: 0, maxWidth: 650 }}>
              {updateInfo?.update_available
                ? isVi
                  ? `Đã phát hiện phiên bản mới v${updateInfo.latest_version} trên Cloudflare Edge / GitHub Releases.`
                  : `New release v${updateInfo.latest_version} detected on Cloudflare Edge / GitHub Releases.`
                : isVi
                  ? 'Hệ thống tự động đồng bộ phiên bản từ máy chủ đám mây hoặc qua Cloudflare Worker không tốn băng thông server.'
                  : 'The system synchronizes updates from cloud server or via zero-cost Cloudflare Worker.'}
            </p>
          </div>

          <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
            <button
              type="button"
              onClick={handleCheckUpdate}
              disabled={isCheckingUpdate}
              style={{
                background: '#1e293b',
                color: '#38bdf8',
                border: '1px solid #334155',
                borderRadius: 8,
                padding: '9px 18px',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: isCheckingUpdate ? 'not-allowed' : 'pointer',
                display: 'inline-flex',
                alignItems: 'center',
                gap: 8,
              }}
            >
              <RefreshCw size={15} className={isCheckingUpdate ? 'animate-spin' : ''} />
              {isCheckingUpdate
                ? isVi ? 'Đang kiểm tra...' : 'Checking...'
                : isVi ? 'Kiểm Tra Bản Mới' : 'Check for Updates'}
            </button>

            <button
              type="button"
              onClick={() => setShowUrlSettings(!showUrlSettings)}
              style={{
                background: showUrlSettings ? '#0284c7' : '#1e293b',
                color: showUrlSettings ? '#ffffff' : '#94a3b8',
                border: '1px solid #334155',
                borderRadius: 8,
                padding: '9px 14px',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: 'pointer',
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                transition: 'all 0.2s',
              }}
              title={isVi ? 'Đổi URL máy chủ hoặc tùy chỉnh URL kiểm tra bản mới' : 'Change server URL or custom update endpoints'}
            >
              <Globe size={15} />
              <span>{isVi ? 'Đổi URL Server' : 'Change Server URL'}</span>
            </button>

            {updateInfo?.update_available && (
              <a
                href={updateInfo.download_url || 'https://famshield.ttcantho.io.vn/download/latest'}
                target="_blank"
                rel="noreferrer"
                style={{
                  background: 'var(--accent)',
                  color: '#fff',
                  border: 'none',
                  borderRadius: 8,
                  padding: '9px 18px',
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 8,
                  textDecoration: 'none',
                }}
              >
                <Download size={15} />
                {isVi ? `Tải Bản Cài Đặt (.exe) v${updateInfo.latest_version}` : `Download Installer (.exe) v${updateInfo.latest_version}`}
              </a>
            )}
          </div>
        </div>

        {/* Collapsible Dynamic Server & Update URL Configuration Drawer */}
        {showUrlSettings && (
          <div
            style={{
              marginTop: 16,
              marginBottom: 16,
              padding: '16px 20px',
              borderRadius: 12,
              background: 'rgba(15, 23, 42, 0.95)',
              border: '1px solid rgba(56, 189, 248, 0.35)',
              boxShadow: '0 10px 25px -5px rgba(0, 0, 0, 0.5)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10, flexWrap: 'wrap', gap: 8 }}>
              <span style={{ fontSize: '0.9rem', fontWeight: 700, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: 8 }}>
                <Server size={16} />
                {isVi ? 'Cấu Hình Động URL Máy Chủ & Bản Cập Nhật:' : 'Dynamic Server & Update URL Configuration:'}
              </span>
              <span
                style={{
                  fontSize: '0.72rem',
                  padding: '2px 8px',
                  borderRadius: 4,
                  background: serverConfig?.operating_mode === 'OnlineManaged' ? 'rgba(74, 222, 128, 0.15)' : 'rgba(56, 189, 248, 0.15)',
                  color: serverConfig?.operating_mode === 'OnlineManaged' ? '#4ade80' : '#38bdf8',
                  border: `1px solid ${serverConfig?.operating_mode === 'OnlineManaged' ? 'rgba(74, 222, 128, 0.3)' : 'rgba(56, 189, 248, 0.3)'}`,
                  fontWeight: 600,
                }}
              >
                {serverConfig?.operating_mode === 'OnlineManaged'
                  ? (isVi ? 'Chế độ: Bản Server (Online Managed)' : 'Mode: Server Managed')
                  : (isVi ? 'Chế độ: Bản Standalone (Độc lập)' : 'Mode: Standalone Free')}
              </span>
            </div>

            <p style={{ fontSize: '0.78rem', color: '#94a3b8', margin: '0 0 12px 0', lineHeight: 1.5 }}>
              {isVi
                ? (serverConfig?.operating_mode === 'OnlineManaged'
                    ? 'Bản Server: URL kiểm tra phiên bản tự động điều chỉnh theo địa chỉ máy chủ (cú pháp <server_url>/version.json). Bạn có thể đổi URL server hoặc nhập URL tùy chỉnh.'
                    : 'Bản Standalone: Mặc định kiểm tra bản mới qua https://famshield.ttcantho.io.vn/version.json. Bạn có thể nhập URL tùy chỉnh nếu tự lưu trữ file version.json.')
                : (serverConfig?.operating_mode === 'OnlineManaged'
                    ? 'Server Managed: Version check URL automatically tracks the server address (<server_url>/version.json).'
                    : 'Standalone Free: Defaults to https://famshield.ttcantho.io.vn/version.json. Custom endpoints supported.')}
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 12, marginBottom: 14 }}>
              <div>
                <label style={{ display: 'block', fontSize: '0.75rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 4 }}>
                  {isVi ? 'URL Máy Chủ (Server URL):' : 'Server URL:'}
                </label>
                <input
                  type="text"
                  value={serverUrl}
                  onChange={(e) => {
                    const newServer = e.target.value;
                    setServerUrl(newServer);
                    if (serverConfig?.operating_mode === 'OnlineManaged' && (!versionCheckUrl || versionCheckUrl.endsWith('/version.json'))) {
                      const trimmed = newServer.trim().replace(/\/+$/, '');
                      if (trimmed) {
                        setVersionCheckUrl(`${trimmed}/version.json`);
                      }
                    }
                  }}
                  placeholder="https://famshield.ttcantho.io.vn"
                  style={{
                    width: '100%',
                    background: '#070b14',
                    border: '1px solid #334155',
                    color: '#fff',
                    padding: '8px 12px',
                    borderRadius: 8,
                    fontSize: '0.85rem',
                    outline: 'none',
                    boxSizing: 'border-box',
                  }}
                />
              </div>

              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                  <label style={{ fontSize: '0.75rem', color: '#cbd5e1', fontWeight: 600 }}>
                    {isVi ? 'URL Kiểm Tra Bản Mới (Version Check URL):' : 'Version Check URL:'}
                  </label>
                  <span style={{ fontSize: '0.7rem', color: '#38bdf8', fontStyle: 'italic' }}>
                    {versionCheckUrl ? (isVi ? 'Tùy chỉnh' : 'Custom') : (isVi ? 'Tự động theo chế độ' : 'Auto by mode')}
                  </span>
                </div>
                <input
                  type="text"
                  value={versionCheckUrl}
                  onChange={(e) => setVersionCheckUrl(e.target.value)}
                  placeholder={
                    serverConfig?.operating_mode === 'OnlineManaged'
                      ? (serverUrl ? `${serverUrl.trim().replace(/\/+$/, '')}/version.json` : 'https://famshield.ttcantho.io.vn/version.json')
                      : 'https://famshield.ttcantho.io.vn/version.json'
                  }
                  style={{
                    width: '100%',
                    background: '#070b14',
                    border: '1px solid #334155',
                    color: '#fff',
                    padding: '8px 12px',
                    borderRadius: 8,
                    fontSize: '0.85rem',
                    outline: 'none',
                    boxSizing: 'border-box',
                  }}
                />
              </div>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 10 }}>
              <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
                <button
                  type="button"
                  onClick={() => {
                    const base = (serverUrl || 'https://famshield.ttcantho.io.vn').trim().replace(/\/+$/, '');
                    if (serverConfig?.operating_mode === 'OnlineManaged') {
                      setVersionCheckUrl(`${base}/version.json`);
                    } else {
                      setVersionCheckUrl('https://famshield.ttcantho.io.vn/version.json');
                    }
                  }}
                  style={{
                    background: '#1e293b',
                    color: '#38bdf8',
                    border: '1px solid #334155',
                    padding: '4px 10px',
                    borderRadius: 6,
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                  title={isVi ? 'Cú pháp chuẩn <url>/version.json' : 'Standard <url>/version.json syntax'}
                >
                  {serverConfig?.operating_mode === 'OnlineManaged'
                    ? (isVi ? 'Tự Động Theo Server (/version.json)' : 'Auto by Server (/version.json)')
                    : (isVi ? 'Mặc Định Standalone (version.json)' : 'Default Standalone (version.json)')}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setVersionCheckUrl('');
                  }}
                  style={{
                    background: '#1e293b',
                    color: '#a78bfa',
                    border: '1px solid #334155',
                    padding: '4px 10px',
                    borderRadius: 6,
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                  title={isVi ? 'Xóa tùy chỉnh để hệ thống tự động nhận diện theo chế độ' : 'Clear custom URL'}
                >
                  {isVi ? 'Tự Động (Auto)' : 'Auto'}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setServerUrl('https://famshield.ttcantho.io.vn');
                    setVersionCheckUrl('https://famshield.ttcantho.io.vn/version.json');
                  }}
                  style={{
                    background: '#1e293b',
                    color: '#cbd5e1',
                    border: '1px solid #334155',
                    padding: '4px 10px',
                    borderRadius: 6,
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                >
                  Cloud Edge
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setServerUrl('http://127.0.0.1:8080');
                    setVersionCheckUrl('http://127.0.0.1:8080/version.json');
                  }}
                  style={{
                    background: '#1e293b',
                    color: '#cbd5e1',
                    border: '1px solid #334155',
                    padding: '4px 10px',
                    borderRadius: 6,
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                >
                  Local Server (:8080)
                </button>
              </div>

              <div style={{ display: 'flex', gap: 8 }}>
                <button
                  type="button"
                  onClick={() => setShowUrlSettings(false)}
                  style={{
                    background: 'transparent',
                    color: '#94a3b8',
                    border: 'none',
                    padding: '6px 12px',
                    fontSize: '0.8rem',
                    cursor: 'pointer',
                  }}
                >
                  {isVi ? 'Đóng' : 'Close'}
                </button>
                <button
                  type="button"
                  onClick={handleUpdateCustomUrls}
                  disabled={isUpdatingUrls}
                  style={{
                    background: 'var(--accent)',
                    color: '#fff',
                    border: 'none',
                    padding: '6px 16px',
                    borderRadius: 8,
                    fontSize: '0.8rem',
                    fontWeight: 700,
                    cursor: isUpdatingUrls ? 'not-allowed' : 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                  }}
                >
                  {isUpdatingUrls ? <RefreshCw size={12} className="animate-spin" /> : <Check size={12} />}
                  <span>{isVi ? 'Lưu & Kiểm Tra Ngay' : 'Save & Check Now'}</span>
                </button>
              </div>
            </div>
          </div>
        )}

        {updateInfo?.update_available && (
          <div
            style={{
              marginTop: 16,
              padding: '16px 20px',
              borderRadius: 10,
              background: 'rgba(15, 23, 42, 0.7)',
              border: '1px solid rgba(56, 189, 248, 0.25)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 10, marginBottom: 10 }}>
              <div style={{ fontSize: '0.85rem', fontWeight: 700, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: 8 }}>
                <Download size={16} />
                {updateInfo.release_date && (
                <div style={{ fontSize: '0.75rem', color: '#94a3b8', marginBottom: 8 }}>
                  <span>{isVi ? '📅 Ngày phát hành: ' : '📅 Released: '}</span>
                  <strong style={{ color: '#cbd5e1' }}>{updateInfo.release_date}</strong>
                  {updateInfo.check_url && (
                    <span style={{ marginLeft: 12 }}>
                      {isVi ? '📡 Nguồn: ' : '📡 Source: '}
                      <code style={{ background: '#070b14', padding: '2px 6px', borderRadius: 4, color: '#38bdf8' }}>{updateInfo.check_url}</code>
                    </span>
                  )}
                </div>
              )}
              
              {isVi ? 'Hướng Dẫn Nâng Cấp Bằng Tay (Manual Install):' : 'Manual Install Instructions:'}
              </div>
              <a
                href={updateInfo.download_url || 'https://famshield.ttcantho.io.vn/download/latest'}
                target="_blank"
                rel="noreferrer"
                style={{
                  fontSize: '0.8rem',
                  color: '#38bdf8',
                  textDecoration: 'underline',
                  fontWeight: 600,
                }}
              >
                {isVi ? 'Tải trực tiếp qua Cloudflare Edge' : 'Direct Download via Cloudflare Edge'}
              </a>
            </div>

            <ol style={{ margin: 0, paddingLeft: 20, fontSize: '0.8rem', color: '#cbd5e1', lineHeight: '1.6' }}>
              <li>
                {isVi
                  ? 'Bấm nút "Tải Bản Cài Đặt (.exe)" ở trên để tải file FamShieldSetup về máy.'
                  : 'Click the "Download Installer (.exe)" button above to download FamShieldSetup.'}
              </li>
              <li>
                {isVi
                  ? 'Nhấp đúp vào file cài đặt vừa tải để mở trình cài đặt Inno Setup.'
                  : 'Double-click the downloaded setup file to launch the Inno Setup wizard.'}
              </li>
              <li>
                {isVi
                  ? 'Trình cài đặt sẽ tự động dừng service cũ, cập nhật toàn bộ file (Service, Tray, Web UI) và khởi động lại dịch vụ an toàn.'
                  : 'The installer will automatically stop legacy processes, update all components, and restart services cleanly.'}
              </li>
            </ol>

            {updateInfo.changelog && updateInfo.changelog.length > 0 && (
              <div style={{ marginTop: 14, paddingTop: 12, borderTop: '1px solid rgba(255, 255, 255, 0.08)' }}>
                <div style={{ fontSize: '0.8rem', fontWeight: 700, color: '#94a3b8', marginBottom: 6 }}>
                  {isVi ? `✨ Điểm mới trong phiên bản v${updateInfo.latest_version}:` : `✨ What's new in v${updateInfo.latest_version}:`}
                </div>
                <ul style={{ margin: 0, paddingLeft: 18, fontSize: '0.8rem', color: '#cbd5e1' }}>
                  {updateInfo.changelog.map((item, idx) => (
                    <li key={idx} style={{ marginBottom: 4 }}>{item}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {updateCheckError && (
          <div style={{ marginTop: 12, fontSize: '0.8rem', color: '#f87171' }}>
            ⚠️ {updateCheckError}
          </div>
        )}
      </div>


      {/* Dynamic System Settings Card */}
      <div
        id="section-system-settings"
        style={{
          background: 'var(--card-bg)',
          border: '1px solid var(--border-color)',
          borderRadius: 14,
          padding: 24,
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 12,
            marginBottom: 20,
            borderBottom: '1px solid rgba(255,255,255,0.06)',
            paddingBottom: 14,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: 'var(--text-main)',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                marginBottom: 4,
              }}
            >
              <Sliders size={18} style={{ color: '#38bdf8' }} />
              {isVi ? 'Tham Số Hệ Thống Động (Dynamic Settings)' : 'Dynamic System Settings'}
            </h3>
            <p style={{ fontSize: '0.82rem', color: 'var(--text-muted)', margin: 0 }}>
              {isVi
                ? 'Điều chỉnh tham số vận hành (DNS Upstream, TTL, Timeout, Chu kỳ Sync/Heartbeat, Giới hạn PIN) tại runtime mà không cần cập nhật mã nguồn.'
                : 'Configure runtime parameters (Upstream DNS, TTL, Timeouts, Sync/Heartbeat intervals, PIN limits) dynamically without code recompilation.'}
            </p>
          </div>
          <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
            <button
              type="button"
              onClick={fetchSystemSettings}
              disabled={isLoadingSettings}
              style={{
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-main)',
                padding: '7px 14px',
                borderRadius: 8,
                fontSize: '0.8rem',
                fontWeight: 600,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              <RefreshCw size={14} className={isLoadingSettings ? 'animate-spin' : ''} />
              <span>{isVi ? 'Làm mới' : 'Reload'}</span>
            </button>
            <button
              type="button"
              onClick={handleSaveSystemSettings}
              disabled={isSavingSettings || !systemSettings}
              style={{
                background: 'var(--accent)',
                color: '#fff',
                border: 'none',
                padding: '7px 18px',
                borderRadius: 8,
                fontSize: '0.85rem',
                fontWeight: 700,
                cursor: isSavingSettings ? 'not-allowed' : 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              {isSavingSettings ? <RefreshCw size={14} className="animate-spin" /> : <Check size={14} />}
              <span>{isVi ? 'Lưu Thay Đổi' : 'Save Changes'}</span>
            </button>
          </div>
        </div>

        {systemSettings && (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 18 }}>
            {/* Sync & Heartbeat Intervals */}
            <div style={{ background: 'rgba(0,0,0,0.15)', padding: 16, borderRadius: 10, border: '1px solid rgba(255,255,255,0.05)' }}>
              <div style={{ fontSize: '0.85rem', fontWeight: 700, color: '#38bdf8', marginBottom: 12, display: 'flex', alignItems: 'center', gap: 6 }}>
                <Cloud size={15} />
                <span>{isVi ? 'Đồng Bộ & Nhịp Tim' : 'Sync & Heartbeat'}</span>
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                    {isVi ? 'Chu kỳ đồng bộ Telemetry (giây):' : 'Telemetry sync interval (secs):'}
                  </label>
                  <input
                    type="number"
                    min={5}
                    max={3600}
                    value={systemSettings.sync_interval_secs}
                    onChange={(e) => setSystemSettings({ ...systemSettings, sync_interval_secs: parseInt(e.target.value) || 15 })}
                    style={{
                      width: '100%',
                      background: 'rgba(255,255,255,0.05)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-main)',
                      padding: '7px 10px',
                      borderRadius: 6,
                      fontSize: '0.85rem',
                    }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                    {isVi ? 'Chu kỳ Heartbeat (giây):' : 'Heartbeat interval (secs):'}
                  </label>
                  <input
                    type="number"
                    min={10}
                    max={7200}
                    value={systemSettings.heartbeat_interval_secs}
                    onChange={(e) => setSystemSettings({ ...systemSettings, heartbeat_interval_secs: parseInt(e.target.value) || 60 })}
                    style={{
                      width: '100%',
                      background: 'rgba(255,255,255,0.05)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-main)',
                      padding: '7px 10px',
                      borderRadius: 6,
                      fontSize: '0.85rem',
                    }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                    {isVi ? 'NATS WebSocket URL:' : 'NATS WebSocket URL:'}
                  </label>
                  <input
                    type="text"
                    value={systemSettings.nats_url}
                    onChange={(e) => setSystemSettings({ ...systemSettings, nats_url: e.target.value })}
                    placeholder="ws://127.0.0.1:9222"
                    style={{
                      width: '100%',
                      background: 'rgba(255,255,255,0.05)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-main)',
                      padding: '7px 10px',
                      borderRadius: 6,
                      fontSize: '0.85rem',
                    }}
                  />
                </div>
              </div>
            </div>

            {/* DNS Interception & Resolvers */}
            <div style={{ background: 'rgba(0,0,0,0.15)', padding: 16, borderRadius: 10, border: '1px solid rgba(255,255,255,0.05)' }}>
              <div style={{ fontSize: '0.85rem', fontWeight: 700, color: '#4ade80', marginBottom: 12, display: 'flex', alignItems: 'center', gap: 6 }}>
                <Globe size={15} />
                <span>{isVi ? 'DNS & Thời Gian Chờ (Timeout)' : 'DNS & Timeout Settings'}</span>
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                    {isVi ? 'DNS Thượng Nguồn (Upstream):' : 'Upstream DNS Resolver:'}
                  </label>
                  <input
                    type="text"
                    value={systemSettings.dns_upstream}
                    onChange={(e) => setSystemSettings({ ...systemSettings, dns_upstream: e.target.value })}
                    placeholder="1.1.1.1:53"
                    style={{
                      width: '100%',
                      background: 'rgba(255,255,255,0.05)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-main)',
                      padding: '7px 10px',
                      borderRadius: 6,
                      fontSize: '0.85rem',
                    }}
                  />
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'DNS TTL (giây):' : 'DNS TTL (secs):'}
                    </label>
                    <input
                      type="number"
                      min={10}
                      max={86400}
                      value={systemSettings.dns_cache_ttl_secs}
                      onChange={(e) => setSystemSettings({ ...systemSettings, dns_cache_ttl_secs: parseInt(e.target.value) || 300 })}
                      style={{
                        width: '100%',
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'Timeout (ms):' : 'Timeout (ms):'}
                    </label>
                    <input
                      type="number"
                      min={500}
                      max={10000}
                      value={systemSettings.dns_timeout_ms}
                      onChange={(e) => setSystemSettings({ ...systemSettings, dns_timeout_ms: parseInt(e.target.value) || 2000 })}
                      style={{
                        width: '100%',
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    />
                  </div>
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                    {isVi ? 'Hành vi khi chặn DNS:' : 'DNS Block Action:'}
                  </label>
                  <select
                    value={systemSettings.dns_block_action}
                    onChange={(e) => setSystemSettings({ ...systemSettings, dns_block_action: e.target.value })}
                    style={{
                      width: '100%',
                      background: '#0f172a',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-main)',
                      padding: '7px 10px',
                      borderRadius: 6,
                      fontSize: '0.85rem',
                    }}
                  >
                    <option value="refused">REFUSED (Tiêu chuẩn)</option>
                    <option value="nxdomain">NXDOMAIN (Không tồn tại)</option>
                  </select>
                </div>
              </div>
            </div>

            {/* PIN Security & Retention */}
            <div style={{ background: 'rgba(0,0,0,0.15)', padding: 16, borderRadius: 10, border: '1px solid rgba(255,255,255,0.05)' }}>
              <div style={{ fontSize: '0.85rem', fontWeight: 700, color: '#f59e0b', marginBottom: 12, display: 'flex', alignItems: 'center', gap: 6 }}>
                <Lock size={15} />
                <span>{isVi ? 'Bảo Vệ Mã PIN & Lưu Trữ Log' : 'PIN Security & Log Retention'}</span>
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'Thử sai tối đa:' : 'Max PIN retries:'}
                    </label>
                    <input
                      type="number"
                      min={3}
                      max={20}
                      value={systemSettings.max_pin_attempts}
                      onChange={(e) => setSystemSettings({ ...systemSettings, max_pin_attempts: parseInt(e.target.value) || 5 })}
                      style={{
                        width: '100%',
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'Khóa tạm (phút):' : 'Lockout (mins):'}
                    </label>
                    <input
                      type="number"
                      min={1}
                      max={120}
                      value={systemSettings.pin_lockout_duration_mins}
                      onChange={(e) => setSystemSettings({ ...systemSettings, pin_lockout_duration_mins: parseInt(e.target.value) || 5 })}
                      style={{
                        width: '100%',
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    />
                  </div>
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'Lưu log DNS (ngày):' : 'Log retention (days):'}
                    </label>
                    <input
                      type="number"
                      min={1}
                      max={365}
                      value={systemSettings.dns_log_retention_days}
                      onChange={(e) => setSystemSettings({ ...systemSettings, dns_log_retention_days: parseInt(e.target.value) || 7 })}
                      style={{
                        width: '100%',
                        background: 'rgba(255,255,255,0.05)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: 4 }}>
                      {isVi ? 'Tự dọn dẹp log cũ:' : 'Auto-prune logs:'}
                    </label>
                    <select
                      value={systemSettings.dns_auto_prune_enabled ? 'true' : 'false'}
                      onChange={(e) => setSystemSettings({ ...systemSettings, dns_auto_prune_enabled: e.target.value === 'true' })}
                      style={{
                        width: '100%',
                        background: '#0f172a',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-main)',
                        padding: '7px 10px',
                        borderRadius: 6,
                        fontSize: '0.85rem',
                      }}
                    >
                      <option value="true">{isVi ? 'Bật (Khuyến nghị)' : 'Enabled'}</option>
                      <option value="false">{isVi ? 'Tắt' : 'Disabled'}</option>
                    </select>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Danger Zone: Gỡ Cài Đặt FamShield */}
      <div
        id="section-danger-zone"
        style={{
          background: 'rgba(239, 68, 68, 0.05)',
          border: '1px solid rgba(239, 68, 68, 0.25)',
          borderRadius: 14,
          padding: 24,
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
          }}
        >
          <div>
            <h3
              style={{
                fontSize: '1.05rem',
                fontWeight: 700,
                color: '#f87171',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                marginBottom: 6,
              }}
            >
              <AlertTriangle size={18} />
              {isVi ? 'Vùng Nguy Hiểm: Gỡ Cài Đặt Ứng Dụng' : 'Danger Zone: Uninstall FamShield'}
            </h3>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', margin: 0, maxWidth: 600 }}>
              {isVi
                ? 'Gỡ bỏ hoàn toàn FamShield, dừng toàn bộ dịch vụ và tự động phục hồi cấu hình DNS tất cả card mạng về ban đầu. Thao tác này yêu cầu xác thực Master PIN.'
                : 'Completely uninstall FamShield, stop all services, and automatically revert network DNS adapter settings to default. Requires Master PIN authentication.'}
            </p>
          </div>
          <button
            type="button"
            onClick={() => {
              setUninstallPin('');
              setUninstallError(null);
              setShowUninstallModal(true);
            }}
            style={{
              background: '#ef4444',
              color: '#ffffff',
              border: 'none',
              padding: '10px 20px',
              borderRadius: 8,
              fontSize: '0.88rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              boxShadow: '0 4px 12px rgba(239, 68, 68, 0.25)',
            }}
          >
            <Trash2 size={16} />
            {isVi ? 'Gỡ Cài Đặt FamShield' : 'Uninstall FamShield'}
          </button>
        </div>
      </div>

      {/* Uninstall Confirmation Modal */}
      {showUninstallModal && (
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
              border: '1px solid #ef4444',
              borderRadius: 16,
              padding: 28,
              maxWidth: 480,
              width: '90%',
              boxShadow: '0 25px 50px -12px rgba(239, 68, 68, 0.25)',
            }}
          >
            <h3
              style={{
                fontSize: '1.2rem',
                fontWeight: 800,
                color: '#f87171',
                marginBottom: 12,
                display: 'flex',
                alignItems: 'center',
                gap: 10,
              }}
            >
              <AlertTriangle size={22} color="#ef4444" />
              {isVi ? 'Xác Nhận Gỡ Cài Đặt FamShield' : 'Confirm FamShield Uninstallation'}
            </h3>
            <p style={{ fontSize: '0.88rem', color: '#cbd5e1', lineHeight: 1.6, marginBottom: 18 }}>
              {isVi
                ? 'Thao tác này sẽ dừng toàn bộ phân hệ bảo vệ, hoàn trả cấu hình DNS các card mạng về DHCP và xóa sạch các dịch vụ chạy ngầm. Vui lòng nhập Master PIN để xác nhận:'
                : 'This will terminate all monitoring subsystems, revert network DNS adapters to DHCP, and cleanly remove background services. Please enter Master PIN to confirm:'}
            </p>

            <div style={{ marginBottom: 18 }}>
              <input
                type="password"
                placeholder={isVi ? 'Nhập Master PIN Phụ Huynh' : 'Enter Master PIN'}
                value={uninstallPin}
                onChange={(e) => setUninstallPin(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleConfirmUninstall();
                }}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  borderRadius: 8,
                  padding: '10px 14px',
                  color: '#f8fafc',
                  fontSize: '0.95rem',
                  outline: 'none',
                  boxSizing: 'border-box',
                }}
              />
              {uninstallError && (
                <div style={{ color: '#ef4444', fontSize: '0.8rem', marginTop: 6 }}>
                  {uninstallError}
                </div>
              )}
            </div>

            {totpStatus?.enabled && (
              <div style={{ marginBottom: 18 }}>
                <label
                  style={{
                    display: 'block',
                    fontSize: '0.8rem',
                    color: '#38bdf8',
                    marginBottom: 6,
                    fontWeight: 600,
                  }}
                >
                  {isVi
                    ? 'Mã OTP 2FA hoặc Mã Dự Phòng Khẩn Cấp:'
                    : '2FA OTP or Emergency Backup Code:'}
                </label>
                <input
                  type="text"
                  placeholder={isVi ? 'Nhập mã 6 số hoặc ABCD-1234' : 'Enter 6-digit or ABCD-1234'}
                  value={uninstallOtp}
                  onChange={(e) => setUninstallOtp(e.target.value)}
                  style={{
                    width: '100%',
                    background: '#070b14',
                    border: '1px solid #334155',
                    borderRadius: 8,
                    padding: '10px 14px',
                    color: '#38bdf8',
                    fontSize: '1rem',
                    fontWeight: 600,
                    outline: 'none',
                    boxSizing: 'border-box',
                  }}
                />
              </div>
            )}

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                type="button"
                onClick={() => setShowUninstallModal(false)}
                disabled={isUninstalling}
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
                {isVi ? 'Hủy Bỏ' : 'Cancel'}
              </button>
              <button
                type="button"
                onClick={handleConfirmUninstall}
                disabled={isUninstalling}
                style={{
                  background: '#ef4444',
                  color: '#ffffff',
                  border: 'none',
                  padding: '10px 20px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 700,
                  cursor: isUninstalling ? 'not-allowed' : 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                {isUninstalling ? (
                  <RefreshCw size={14} className="animate-spin" />
                ) : (
                  <Trash2 size={14} />
                )}
                {isVi ? 'Xác Nhận Gỡ Bỏ' : 'Confirm Uninstall'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 2FA Setup Modal */}
      {showTotpSetupModal && totpSetupData && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 9999,
            backdropFilter: 'blur(8px)',
            padding: 20,
          }}
        >
          <div
            style={{
              background: '#0f172a',
              border: '1px solid rgba(56, 189, 248, 0.4)',
              borderRadius: 20,
              padding: 28,
              maxWidth: 520,
              width: '100%',
              maxHeight: '90vh',
              overflowY: 'auto',
              boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.8)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
              <div
                style={{
                  padding: 10,
                  borderRadius: 12,
                  background: 'rgba(56, 189, 248, 0.1)',
                  color: '#38bdf8',
                }}
              >
                <QrCode size={24} />
              </div>
              <div>
                <h3 style={{ fontSize: '1.2rem', fontWeight: 800, color: '#f8fafc', margin: 0 }}>
                  {isVi ? 'Thiết Lập 2FA Authenticator' : 'Setup 2FA Authenticator'}
                </h3>
                <p style={{ fontSize: '0.8rem', color: '#94a3b8', margin: 0 }}>
                  {isVi
                    ? 'Quét mã QR bằng ứng dụng trên điện thoại'
                    : 'Scan QR code using Authenticator app'}
                </p>
              </div>
            </div>

            {/* QR Code */}
            <div
              style={{
                textAlign: 'center',
                background: '#ffffff',
                padding: 16,
                borderRadius: 14,
                maxWidth: 220,
                margin: '0 auto 16px',
              }}
            >
              <img
                src={totpSetupData.qr_base64}
                alt="2FA QR Code"
                style={{ width: '100%', height: 'auto', display: 'block' }}
              />
            </div>

            <div
              style={{
                background: '#070b14',
                border: '1px solid #1e293b',
                borderRadius: 10,
                padding: 12,
                marginBottom: 20,
              }}
            >
              <div style={{ fontSize: '0.75rem', color: '#94a3b8', marginBottom: 4 }}>
                {isVi ? 'Hoặc nhập khóa bí mật thủ công:' : 'Or enter secret key manually:'}
              </div>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  gap: 8,
                }}
              >
                <code
                  style={{
                    fontSize: '0.95rem',
                    fontWeight: 700,
                    color: '#38bdf8',
                    wordBreak: 'break-all',
                  }}
                >
                  {totpSetupData.secret}
                </code>
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(totpSetupData.secret);
                    onShowToast(isVi ? 'Đã sao chép khóa bí mật' : 'Copied secret key', 'info');
                  }}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    color: '#94a3b8',
                    cursor: 'pointer',
                    padding: 4,
                  }}
                  title={isVi ? 'Sao chép' : 'Copy'}
                >
                  <Copy size={16} />
                </button>
              </div>
            </div>

            {/* Backup Codes */}
            <div
              style={{
                background: 'rgba(245, 158, 11, 0.08)',
                border: '1px solid rgba(245, 158, 11, 0.25)',
                borderRadius: 10,
                padding: 14,
                marginBottom: 20,
              }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  marginBottom: 8,
                }}
              >
                <span style={{ fontSize: '0.85rem', fontWeight: 700, color: '#fbbf24' }}>
                  {isVi
                    ? '⚠️ 8 Mã Dự Phòng Khẩn Cấp (Lưu giữ cẩn thận):'
                    : '⚠️ 8 Emergency Backup Codes (Keep safe):'}
                </span>
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(totpSetupData.backup_codes.join('\n'));
                    onShowToast(
                      isVi ? 'Đã sao chép danh sách 8 mã dự phòng' : 'Copied backup codes',
                      'info'
                    );
                  }}
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 4,
                    background: 'transparent',
                    border: 'none',
                    color: '#fbbf24',
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                    fontWeight: 600,
                  }}
                >
                  <Copy size={13} /> {isVi ? 'Sao chép tất cả' : 'Copy all'}
                </button>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 6 }}>
                {totpSetupData.backup_codes.map((code) => (
                  <div
                    key={code}
                    style={{
                      background: 'rgba(15, 23, 42, 0.6)',
                      padding: '6px 10px',
                      borderRadius: 6,
                      fontSize: '0.82rem',
                      fontFamily: 'monospace',
                      color: '#f8fafc',
                      textAlign: 'center',
                    }}
                  >
                    {code}
                  </div>
                ))}
              </div>
            </div>

            {/* Confirm Code Input */}
            <div style={{ marginBottom: 20 }}>
              <label
                style={{
                  display: 'block',
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  color: '#f8fafc',
                  marginBottom: 6,
                }}
              >
                {isVi
                  ? 'Nhập mã 6 số trên ứng dụng để xác nhận kích hoạt:'
                  : 'Enter the 6-digit code from your app to activate:'}
              </label>
              <input
                type="text"
                autoFocus
                placeholder="582910"
                value={totpConfirmCode}
                onChange={(e) => {
                  setTotpConfirmCode(e.target.value);
                  setTotpError(null);
                }}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '2px solid rgba(56, 189, 248, 0.4)',
                  borderRadius: 10,
                  padding: '12px 16px',
                  fontSize: '1.25rem',
                  fontWeight: 700,
                  letterSpacing: '4px',
                  textAlign: 'center',
                  color: '#38bdf8',
                  outline: 'none',
                  boxSizing: 'border-box',
                }}
              />
              {totpError && (
                <div
                  style={{
                    color: '#ef4444',
                    fontSize: '0.8rem',
                    marginTop: 6,
                    textAlign: 'center',
                  }}
                >
                  {totpError}
                </div>
              )}
            </div>

            {/* Actions */}
            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                type="button"
                onClick={() => setShowTotpSetupModal(false)}
                disabled={isConfirmingTotp}
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
                {isVi ? 'Hủy' : 'Cancel'}
              </button>
              <button
                type="button"
                onClick={handleConfirmTotp}
                disabled={isConfirmingTotp || !totpConfirmCode.trim()}
                style={{
                  background: '#0284c7',
                  color: '#ffffff',
                  border: 'none',
                  padding: '10px 22px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 700,
                  cursor: isConfirmingTotp || !totpConfirmCode.trim() ? 'not-allowed' : 'pointer',
                }}
              >
                {isConfirmingTotp
                  ? isVi
                    ? 'Đang kích hoạt...'
                    : 'Activating...'
                  : isVi
                    ? 'Kích Hoạt 2FA'
                    : 'Activate 2FA'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 2FA Disable Modal */}
      {showTotpDisableModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 9999,
            backdropFilter: 'blur(8px)',
            padding: 20,
          }}
        >
          <div
            style={{
              background: '#0f172a',
              border: '1px solid rgba(239, 68, 68, 0.4)',
              borderRadius: 16,
              padding: 28,
              maxWidth: 440,
              width: '100%',
              boxShadow: '0 25px 50px -12px rgba(239, 68, 68, 0.25)',
            }}
          >
            <h3
              style={{
                fontSize: '1.2rem',
                fontWeight: 800,
                color: '#f87171',
                marginBottom: 12,
                display: 'flex',
                alignItems: 'center',
                gap: 8,
              }}
            >
              <AlertTriangle size={22} color="#f87171" />
              {isVi ? 'Xác Nhận Tắt 2FA' : 'Confirm Disabling 2FA'}
            </h3>
            <p style={{ fontSize: '0.85rem', color: '#cbd5e1', lineHeight: 1.5, marginBottom: 18 }}>
              {isVi
                ? 'Tắt tính năng xác thực 2 lớp sẽ làm giảm mức độ bảo vệ của bảng điều khiển. Nhập Master PIN để xác nhận:'
                : 'Disabling 2FA will reduce the security level of the parent dashboard. Enter Master PIN to confirm:'}
            </p>

            <div style={{ marginBottom: 14 }}>
              <input
                type="password"
                placeholder={isVi ? 'Nhập Master PIN' : 'Enter Master PIN'}
                value={totpDisablePin}
                onChange={(e) => setTotpDisablePin(e.target.value)}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  borderRadius: 8,
                  padding: '10px 14px',
                  color: '#f8fafc',
                  fontSize: '0.95rem',
                  outline: 'none',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            <div style={{ marginBottom: 18 }}>
              <input
                type="text"
                placeholder={
                  isVi ? 'Mã OTP hoặc Mã dự phòng (tùy chọn)' : 'OTP or Backup code (optional)'
                }
                value={totpDisableOtp}
                onChange={(e) => setTotpDisableOtp(e.target.value)}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #334155',
                  borderRadius: 8,
                  padding: '10px 14px',
                  color: '#38bdf8',
                  fontSize: '0.95rem',
                  outline: 'none',
                  boxSizing: 'border-box',
                }}
              />
              {totpDisableError && (
                <div style={{ color: '#ef4444', fontSize: '0.8rem', marginTop: 6 }}>
                  {totpDisableError}
                </div>
              )}
            </div>

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                type="button"
                onClick={() => setShowTotpDisableModal(false)}
                disabled={isDisablingTotp}
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
                {isVi ? 'Hủy' : 'Cancel'}
              </button>
              <button
                type="button"
                onClick={handleDisableTotp}
                disabled={isDisablingTotp || !totpDisablePin.trim()}
                style={{
                  background: '#ef4444',
                  color: '#ffffff',
                  border: 'none',
                  padding: '10px 20px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 700,
                  cursor: isDisablingTotp || !totpDisablePin.trim() ? 'not-allowed' : 'pointer',
                }}
              >
                {isDisablingTotp
                  ? isVi
                    ? 'Đang tắt...'
                    : 'Disabling...'
                  : isVi
                    ? 'Xác Nhận Tắt'
                    : 'Confirm Disable'}
              </button>
            </div>
          </div>
        </div>
      )}

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
                <span
                  style={{
                    color: pendingRestoreBackup.settings?.monitor_mode ? '#fbbf24' : '#34d399',
                  }}
                >
                  {pendingRestoreBackup.settings?.monitor_mode
                    ? 'Monitor (Only Observe)'
                    : 'Active Blocking'}
                </span>
              </div>
              <div>
                <span style={{ color: '#64748b' }}>SHA-256: </span>
                <span style={{ color: '#94a3b8' }}>
                  {pendingRestoreBackup.checksum.slice(0, 24)}...
                </span>
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
                {isRestoringBackup ? (
                  <RefreshCw size={14} className="animate-spin" />
                ) : (
                  <Check size={14} />
                )}
                {t.backup_restore_confirm_btn}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
