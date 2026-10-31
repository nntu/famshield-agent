import type React from 'react';
import { useState } from 'react';
import { Database, Download, Upload, RefreshCw, Check } from 'lucide-react';
import { api } from '../../api';
import { translations } from '../../i18n';
import type { BackupPayload, Language } from '../../types';

interface BackupRestoreCardProps {
  lang: Language;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
  onConfigRestored?: () => void;
}

export const BackupRestoreCard: React.FC<BackupRestoreCardProps> = ({
  lang,
  onRefresh,
  onShowToast,
  onConfigRestored,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

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
      } catch {
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
      if (onConfigRestored) {
        onConfigRestored();
      }
      onRefresh();
    } catch (e: any) {
      onShowToast(e.message || t.backup_restore_invalid, 'error');
    } finally {
      setIsRestoringBackup(false);
    }
  };

  return (
    <>
      <div
        id="section-backup-restore"
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
            type="button"
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
                type="button"
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
                type="button"
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
    </>
  );
};
