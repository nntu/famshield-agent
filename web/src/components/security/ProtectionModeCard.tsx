import type React from 'react';
import { useState, useEffect } from 'react';
import { Eye, ShieldCheck, Check, BellOff, Globe } from 'lucide-react';
import { api } from '../../api';
import { translations } from '../../i18n';
import type { Language, StatusResponse } from '../../types';

interface ProtectionModeCardProps {
  lang: Language;
  status: StatusResponse | null;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const ProtectionModeCard: React.FC<ProtectionModeCardProps> = ({
  lang,
  status,
  onRefresh,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

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

  return (
    <div
      id="section-protection-mode"
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
  );
};
