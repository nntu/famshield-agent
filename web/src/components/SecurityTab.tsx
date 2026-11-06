import type React from 'react';
import { useState, useEffect } from 'react';
import { Smartphone } from 'lucide-react';
import { api } from '../api';
import type {
  DnsConfigResponse,
  KeywordConfigResponse,
  Language,
  ServerConfig,
  StatusResponse,
  TotpStatusResponse,
} from '../types';

import { SecurityTabNav, type SecuritySubTab } from './security/SecurityTabNav';
import { CentralServerCard } from './security/CentralServerCard';
import { ProtectionModeCard } from './security/ProtectionModeCard';
import { BackupRestoreCard } from './security/BackupRestoreCard';
import { ChangePinCard } from './security/ChangePinCard';
import { KeywordMonitoringCard } from './security/KeywordMonitoringCard';
import { TwoFactorCard } from './security/TwoFactorCard';
import { DnsConfigCard } from './security/DnsConfigCard';
import { SoftwareUpdateCard } from './security/SoftwareUpdateCard';
import { DynamicSettingsCard } from './security/DynamicSettingsCard';
import { AuthorOpenSourceCard } from './security/AuthorOpenSourceCard';
import { DangerZoneCard } from './security/DangerZoneCard';

interface SecurityTabProps {
  status: StatusResponse | null;
  lang: Language;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
  onNavigateTab?: (tab: any) => void;
}

export const SecurityTab: React.FC<SecurityTabProps> = ({
  status,
  lang,
  onRefresh,
  onShowToast,
  onNavigateTab,
}) => {
  const isVi = lang === 'vi';

  // Submenu Tab State with persistence
  const [activeSubTab, setActiveSubTabState] = useState<SecuritySubTab>(() => {
    return (localStorage.getItem('famshield_security_subtab') as SecuritySubTab) || 'auth';
  });

  const setActiveSubTab = (tab: SecuritySubTab) => {
    setActiveSubTabState(tab);
    localStorage.setItem('famshield_security_subtab', tab);
  };

  // Centralized Server Config State
  const [serverConfig, setServerConfig] = useState<ServerConfig | null>(null);
  const [serverUrl, setServerUrl] = useState('https://famshield.ttcantho.io.vn');
  const [versionCheckUrl, setVersionCheckUrl] = useState('');
  const [tenantId, setTenantId] = useState('tenant-default');
  const [deviceName, setDeviceName] = useState('DESKTOP-FAMSHIELD');
  const [enrollCode, setEnrollCode] = useState('');
  const [syncInterval, setSyncInterval] = useState<number>(60);
  const [tlsVerify, setTlsVerify] = useState<boolean>(true);

  // Keyword Monitoring State
  const [keywordConfig, setKeywordConfig] = useState<KeywordConfigResponse | null>(null);

  // 2FA TOTP State
  const [totpStatus, setTotpStatus] = useState<TotpStatusResponse | null>(null);

  // DNS Configuration & Cache State
  const [dnsConfig, setDnsConfig] = useState<DnsConfigResponse | null>(null);

  const fetchServerConfig = async () => {
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
      // ignore
    }
  };

  const fetchKeywordConfig = async () => {
    try {
      const cfg = await api.getKeywordConfig();
      setKeywordConfig(cfg);
    } catch {
      // ignore
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

  const fetchDnsConfig = async () => {
    try {
      const cfg = await api.getDnsConfig();
      setDnsConfig(cfg);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    fetchServerConfig();
    fetchKeywordConfig();
    fetchTotpStatus();
    fetchDnsConfig();
  }, []);

  return (
    <div className="flex flex-col gap-6">
      {/* 0. SUBMENU NAVIGATION BAR */}
      <SecurityTabNav
        activeSubTab={activeSubTab}
        setActiveSubTab={setActiveSubTab}
        totpEnabled={Boolean(totpStatus?.enabled)}
        operatingMode={status?.operating_mode}
        isVi={isVi}
      />

      {/* ============================================================== */}
      {/* SUB-PAGE 1: TÀI KHOẢN & BẢO MẬT (AUTH, PIN, 2FA, PROTECTION)   */}
      {/* ============================================================== */}
      {activeSubTab === 'auth' && (
        <div className="flex flex-col gap-6">
          {/* Prominent 2FA Activation Banner if not enabled */}
          {!totpStatus?.enabled && (
            <div className="bg-gradient-to-r from-amber-500/15 via-slate-900/90 to-slate-900/95 border border-amber-500/40 rounded-2xl p-5 sm:p-6 flex items-center justify-between flex-wrap gap-4 shadow-md">
              <div className="flex items-center gap-3.5">
                <div className="bg-amber-500/20 p-3 rounded-xl text-amber-500 shrink-0">
                  <Smartphone size={26} />
                </div>
                <div>
                  <div className="font-extrabold text-sm sm:text-base text-amber-100 flex items-center gap-2 flex-wrap">
                    <span>
                      {isVi
                        ? 'Bảo Vệ Tài Khoản Phụ Huynh: Chưa Kích Hoạt Xác Thực 2 Lớp (2FA / OTP)'
                        : 'Parent Account Security: 2FA / OTP is Disabled'}
                    </span>
                    <span className="bg-amber-500 text-slate-950 px-2 py-0.5 rounded text-[11px] font-black">
                      {isVi ? 'KHUYÊN DÙNG' : 'RECOMMENDED'}
                    </span>
                  </div>
                  <p className="text-xs text-slate-300 mt-1 max-w-2xl leading-relaxed">
                    {isVi
                      ? 'Bảo vệ bảng điều khiển chống nhìn lén Master PIN bằng mã OTP 6 chữ số sinh từ Google Authenticator, Microsoft Authenticator hoặc Apple Keychain (chuẩn RFC 6238, 100% ngoại tuyến).'
                      : 'Protect the parental dashboard against shoulder-surfing using 6-digit TOTP codes from Google/Microsoft Authenticator or Apple Keychain (RFC 6238, 100% offline).'}
                  </p>
                </div>
              </div>
              <button
                type="button"
                onClick={() =>
                  document.getElementById('section-2fa-totp')?.scrollIntoView({ behavior: 'smooth' })
                }
                className="inline-flex items-center gap-2 bg-amber-500 hover:bg-amber-400 text-slate-950 font-extrabold px-5 py-2.5 rounded-xl text-xs sm:text-sm cursor-pointer shadow-md transition whitespace-nowrap border-0"
              >
                <Smartphone size={16} />
                <span>{isVi ? 'Thiết Lập 2FA Ngay' : 'Setup 2FA Now'}</span>
              </button>
            </div>
          )}

          {/* 2FA TOTP Card */}
          <div id="section-2fa-totp">
            <TwoFactorCard
              lang={lang}
              totpStatus={totpStatus}
              onRefreshTotp={fetchTotpStatus}
              onShowToast={onShowToast}
            />
          </div>

          {/* Change Master PIN */}
          <div id="section-master-pin">
            <ChangePinCard
              lang={lang}
              onShowToast={onShowToast}
            />
          </div>

          {/* Protection Mode: Active Blocking vs Monitoring Only */}
          <ProtectionModeCard
            lang={lang}
            status={status}
            onRefresh={onRefresh}
            onShowToast={onShowToast}
          />
        </div>
      )}

      {/* ============================================================== */}
      {/* SUB-PAGE 2: ĐÁM MÂY & GHÉP NỐI (CLOUD SERVER, ENROLLMENT, UPDATE) */}
      {/* ============================================================== */}
      {activeSubTab === 'cloud' && (
        <div className="flex flex-col gap-6">
          {/* Centralized Cloud Server Config */}
          <div id="section-cloud-server">
            <CentralServerCard
              lang={lang}
              serverConfig={serverConfig}
              setServerConfig={setServerConfig}
              serverUrl={serverUrl}
              setServerUrl={setServerUrl}
              versionCheckUrl={versionCheckUrl}
              setVersionCheckUrl={setVersionCheckUrl}
              tenantId={tenantId}
              setTenantId={setTenantId}
              deviceName={deviceName}
              setDeviceName={setDeviceName}
              enrollCode={enrollCode}
              setEnrollCode={setEnrollCode}
              syncInterval={syncInterval}
              setSyncInterval={setSyncInterval}
              tlsVerify={tlsVerify}
              setTlsVerify={setTlsVerify}
              onRefreshServerConfig={fetchServerConfig}
              onRefreshTab={onRefresh}
              onShowToast={onShowToast}
            />
          </div>

          {/* Software Updates & Version Check */}
          <SoftwareUpdateCard
            lang={lang}
            serverConfig={serverConfig}
            serverUrl={serverUrl}
            setServerUrl={setServerUrl}
            versionCheckUrl={versionCheckUrl}
            setVersionCheckUrl={setVersionCheckUrl}
            onRefreshServerConfig={fetchServerConfig}
            onShowToast={onShowToast}
          />
        </div>
      )}

      {/* ============================================================== */}
      {/* SUB-PAGE 3: MẠNG & HỆ THỐNG (DNS CONFIG, SYSTEM SETTINGS, KEYWORDS) */}
      {/* ============================================================== */}
      {activeSubTab === 'network' && (
        <div className="flex flex-col gap-6">
          {/* DNS Configuration & Cache Management */}
          <div id="section-dns-config">
            <DnsConfigCard
              lang={lang}
              dnsConfig={dnsConfig}
              onRefreshDns={fetchDnsConfig}
              onShowToast={onShowToast}
            />
          </div>

          {/* Dynamic System Settings */}
          <DynamicSettingsCard
            lang={lang}
            onShowToast={onShowToast}
          />

          {/* Sensitive Keyword Monitoring */}
          <div id="section-keywords-monitor">
            <KeywordMonitoringCard
              lang={lang}
              keywordConfig={keywordConfig}
              onUpdateKeywordConfig={setKeywordConfig}
              onShowToast={onShowToast}
            />
          </div>
        </div>
      )}

      {/* ============================================================== */}
      {/* SUB-PAGE 4: DỮ LIỆU & BẢO TRÌ (BACKUP, AUTHOR, DANGER ZONE)     */}
      {/* ============================================================== */}
      {activeSubTab === 'maintenance' && (
        <div className="flex flex-col gap-6">
          {/* Backup & Restore Configuration */}
          <BackupRestoreCard
            lang={lang}
            onRefresh={onRefresh}
            onShowToast={onShowToast}
            onConfigRestored={fetchServerConfig}
          />

          {/* Author & Open Source Licenses */}
          <AuthorOpenSourceCard
            lang={lang}
            onShowToast={onShowToast}
            onNavigateTab={onNavigateTab}
          />

          {/* Danger Zone: Factory Reset & Uninstall */}
          <div id="section-danger-zone">
            <DangerZoneCard
              lang={lang}
              totpEnabled={totpStatus?.enabled}
              onShowToast={onShowToast}
            />
          </div>
        </div>
      )}
    </div>
  );
};
