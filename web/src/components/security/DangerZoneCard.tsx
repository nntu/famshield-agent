import type React from 'react';
import { useState } from 'react';
import { AlertTriangle, Trash2, RefreshCw } from 'lucide-react';
import { api } from '../../api';
import type { Language } from '../../types';

interface DangerZoneCardProps {
  lang: Language;
  totpEnabled?: boolean;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const DangerZoneCard: React.FC<DangerZoneCardProps> = ({
  lang,
  totpEnabled,
  onShowToast,
}) => {
  const isVi = lang === 'vi';
  const [showUninstallModal, setShowUninstallModal] = useState(false);
  const [uninstallPin, setUninstallPin] = useState('');
  const [uninstallOtp, setUninstallOtp] = useState('');
  const [isUninstalling, setIsUninstalling] = useState(false);
  const [uninstallError, setUninstallError] = useState<string | null>(null);

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

  return (
    <>
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
              setUninstallOtp('');
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

            {totpEnabled && (
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
    </>
  );
};
