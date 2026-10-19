import React from 'react';
import { useI18n } from '../i18n/I18nContext';

interface DownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const DownloadModal: React.FC<DownloadModalProps> = ({ isOpen, onClose }) => {
  const { t } = useI18n();

  if (!isOpen) return null;

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.75)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      backdropFilter: 'blur(4px)',
    }}>
      <div style={{
        background: '#151d2f',
        border: '1px solid #23314d',
        borderRadius: '16px',
        padding: '32px',
        maxWidth: '520px',
        width: '90%',
        color: '#f8fafc',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <span style={{ fontSize: '1.75rem' }}>🪟</span>
            <h2 style={{ fontSize: '1.35rem', fontWeight: 700, margin: 0, color: '#38bdf8' }}>
              {t('download_agent_title')}
            </h2>
          </div>
          <button
            onClick={onClose}
            style={{ background: 'transparent', border: 'none', color: '#94a3b8', fontSize: '1.25rem', cursor: 'pointer' }}
          >
            ✕
          </button>
        </div>

        <p style={{ color: '#94a3b8', fontSize: '0.9rem', marginBottom: '24px', lineHeight: '1.5' }}>
          {t('download_agent_desc')}
        </p>

        {/* Download Button */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '24px' }}>
          <a
            href="/v1/download/agent"
            download="FamShieldSetup.exe"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '10px',
              background: '#0284c7',
              color: '#fff',
              textDecoration: 'none',
              padding: '14px 24px',
              borderRadius: '10px',
              fontWeight: 700,
              fontSize: '1.05rem',
              boxShadow: '0 4px 14px rgba(2, 132, 199, 0.4)',
              transition: 'background 0.2s',
            }}
          >
            <span>⬇️</span> {t('download_exe_btn')}
          </a>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75rem', color: '#64748b' }}>
            <span>Hệ điều hành: Windows 10/11 x64</span>
            <span>Dung lượng: ~4.5 MB</span>
            <span>Phiên bản: v0.1.0</span>
          </div>
        </div>

        {/* Installation Instructions */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: '10px',
          padding: '16px',
          fontSize: '0.85rem',
          lineHeight: '1.6',
          color: '#cbd5e1',
        }}>
          <div style={{ fontWeight: 600, color: '#38bdf8', marginBottom: '6px' }}>
            📋 {t('download_instructions')}
          </div>
          <div>{t('download_step1')}</div>
          <div>{t('download_step2')}</div>
          <div>{t('download_step3')}</div>
        </div>

        <button
          onClick={onClose}
          style={{
            width: '100%',
            marginTop: '20px',
            background: 'transparent',
            border: '1px solid #334155',
            color: '#94a3b8',
            padding: '10px',
            borderRadius: '8px',
            cursor: 'pointer',
            fontWeight: 500,
          }}
        >
          {t('auth_close')}
        </button>
      </div>
    </div>
  );
};
