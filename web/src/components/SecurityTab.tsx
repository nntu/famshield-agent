import type React from 'react';
import { useState, useEffect } from 'react';
import {
  Globe,
  Keyboard,
  Lock,
  Server,
  Smartphone,
  Trash2,
} from 'lucide-react';
import { api } from '../api';
import type {
  DnsConfigResponse,
  KeywordConfigResponse,
  Language,
  ServerConfig,
  StatusResponse,
  TotpStatusResponse,
} from '../types';

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
}

export const SecurityTab: React.FC<SecurityTabProps> = ({
  status,
  lang,
  onRefresh,
  onShowToast,
}) => {
  const isVi = lang === 'vi';

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
            document.getElementById('section-2fa-totp')?.scrollIntoView({ behavior: 'smooth' })
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
            onClick={() =>
              document.getElementById('section-2fa-totp')?.scrollIntoView({ behavior: 'smooth' })
            }
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
              cursor: 'pointer',
              boxShadow: '0 4px 16px rgba(245, 158, 11, 0.4)',
              whiteSpace: 'nowrap',
            }}
          >
            <Smartphone size={18} />
            {isVi ? '👉 Kích Hoạt 2FA Ngay' : '👉 Enable 2FA Now'}
          </button>
        </div>
      )}

      {/* SECTION 1: CENTRALIZED MANAGEMENT SERVER SETTINGS */}
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

      {/* SECTION 2: PROTECTION MODE (CHỦ ĐỘNG CHẶN VS CHỈ GIÁM SÁT) */}
      <ProtectionModeCard
        lang={lang}
        status={status}
        onRefresh={onRefresh}
        onShowToast={onShowToast}
      />

      {/* SECTION 3: BACKUP & RESTORE */}
      <BackupRestoreCard
        lang={lang}
        onRefresh={onRefresh}
        onShowToast={onShowToast}
        onConfigRestored={fetchServerConfig}
      />

      {/* SECTION 4: CHANGE MASTER PIN */}
      <ChangePinCard
        lang={lang}
        onShowToast={onShowToast}
      />

      {/* SECTION 5: KEYWORD MONITORING */}
      <KeywordMonitoringCard
        lang={lang}
        keywordConfig={keywordConfig}
        onUpdateKeywordConfig={setKeywordConfig}
        onShowToast={onShowToast}
      />

      {/* SECTION 6: 2FA TOTP */}
      <TwoFactorCard
        lang={lang}
        totpStatus={totpStatus}
        onRefreshTotp={fetchTotpStatus}
        onShowToast={onShowToast}
      />

      {/* SECTION 7: DNS CONFIG & CACHE */}
      <DnsConfigCard
        lang={lang}
        dnsConfig={dnsConfig}
        onRefreshDns={fetchDnsConfig}
        onShowToast={onShowToast}
      />

      {/* SECTION 8: SOFTWARE UPDATES */}
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

      {/* SECTION 9: DYNAMIC SYSTEM SETTINGS */}
      <DynamicSettingsCard
        lang={lang}
        onShowToast={onShowToast}
      />

      {/* SECTION 10: AUTHOR & OPEN SOURCE */}
      <AuthorOpenSourceCard
        lang={lang}
        onShowToast={onShowToast}
      />

      {/* SECTION 11: DANGER ZONE */}
      <DangerZoneCard
        lang={lang}
        totpEnabled={totpStatus?.enabled}
        onShowToast={onShowToast}
      />
    </div>
  );
};
