import type React from 'react';
import { useState } from 'react';
import { Server, Activity, RefreshCw, Check, AlertTriangle, Cloud, Unlink } from 'lucide-react';
import { api } from '../../api';
import { translations } from '../../i18n';
import type { Language, ServerConfig } from '../../types';

interface CentralServerCardProps {
  lang: Language;
  serverConfig: ServerConfig | null;
  setServerConfig: (cfg: ServerConfig) => void;
  serverUrl: string;
  setServerUrl: (url: string) => void;
  versionCheckUrl: string;
  setVersionCheckUrl: (url: string) => void;
  tenantId: string;
  setTenantId: (id: string) => void;
  deviceName: string;
  setDeviceName: (name: string) => void;
  enrollCode: string;
  setEnrollCode: (code: string) => void;
  syncInterval: number;
  setSyncInterval: (interval: number) => void;
  tlsVerify: boolean;
  setTlsVerify: (verify: boolean) => void;
  onRefreshServerConfig: () => Promise<void>;
  onRefreshTab: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const CentralServerCard: React.FC<CentralServerCardProps> = ({
  lang,
  serverConfig,
  setServerConfig,
  serverUrl,
  setServerUrl,
  versionCheckUrl,
  setVersionCheckUrl,
  tenantId,
  setTenantId,
  deviceName,
  setDeviceName,
  enrollCode,
  setEnrollCode,
  syncInterval,
  setSyncInterval,
  tlsVerify,
  setTlsVerify,
  onRefreshServerConfig,
  onRefreshTab,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [isSavingServer, setIsSavingServer] = useState(false);
  const [isTestingConn, setIsTestingConn] = useState(false);
  const [isSyncingNow, setIsSyncingNow] = useState(false);
  const [isDisenrolling, setIsDisenrolling] = useState(false);
  const [testResult, setTestResult] = useState<{
    success: boolean;
    latency_ms: number;
    message: string;
  } | null>(null);

  const isEnrolled = serverConfig?.operating_mode === 'OnlineManaged';

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
        await onRefreshServerConfig();
        onRefreshTab();
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
      onRefreshTab();
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
      await onRefreshServerConfig();
      onRefreshTab();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi hủy kết nối' : 'Disenroll failed'), 'error');
    } finally {
      setIsDisenrolling(false);
    }
  };

  return (
    <div
      id="section-central-server"
      style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: 28,
        boxShadow: '0 10px 30px rgba(0,0,0,0.3)',
      }}
    >
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
  );
};
