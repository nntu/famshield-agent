import type React from 'react';
import { useState } from 'react';
import { RefreshCw, Globe, Download, Server, Check } from 'lucide-react';
import { api } from '../../api';
import type { Language, ServerConfig } from '../../types';

interface SoftwareUpdateCardProps {
  lang: Language;
  serverConfig: ServerConfig | null;
  serverUrl: string;
  setServerUrl: (url: string) => void;
  versionCheckUrl: string;
  setVersionCheckUrl: (url: string) => void;
  onRefreshServerConfig: () => Promise<void>;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const SoftwareUpdateCard: React.FC<SoftwareUpdateCardProps> = ({
  lang,
  serverConfig,
  serverUrl,
  setServerUrl,
  versionCheckUrl,
  setVersionCheckUrl,
  onRefreshServerConfig,
  onShowToast,
}) => {
  const isVi = lang === 'vi';

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
  const [showUrlSettings, setShowUrlSettings] = useState(false);
  const [isUpdatingUrls, setIsUpdatingUrls] = useState(false);

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
      await onRefreshServerConfig();
      await handleCheckUpdate();
    } catch (err: any) {
      onShowToast(err?.message || (isVi ? 'Lỗi cập nhật URL máy chủ' : 'Failed to update URL'), 'error');
    } finally {
      setIsUpdatingUrls(false);
    }
  };

  return (
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
              v{updateInfo?.current_version || '1.0.4'}
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
  );
};
