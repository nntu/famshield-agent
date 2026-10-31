import type React from 'react';
import { useState, useEffect } from 'react';
import { Sliders, RefreshCw, Check, Cloud, Globe, Lock } from 'lucide-react';
import { api } from '../../api';
import type { Language, SystemSettings } from '../../types';

interface DynamicSettingsCardProps {
  lang: Language;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const DynamicSettingsCard: React.FC<DynamicSettingsCardProps> = ({
  lang,
  onShowToast,
}) => {
  const isVi = lang === 'vi';

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

  useEffect(() => {
    fetchSystemSettings();
  }, []);

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

  return (
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
  );
};
