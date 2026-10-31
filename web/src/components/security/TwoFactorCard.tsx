import type React from 'react';
import { useState } from 'react';
import {
  Smartphone,
  CheckCircle2,
  QrCode,
  Copy,
  AlertTriangle,
} from 'lucide-react';
import { api } from '../../api';
import type { Language, TotpSetupResponse, TotpStatusResponse } from '../../types';

interface TwoFactorCardProps {
  lang: Language;
  totpStatus: TotpStatusResponse | null;
  onRefreshTotp: () => Promise<void>;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const TwoFactorCard: React.FC<TwoFactorCardProps> = ({
  lang,
  totpStatus,
  onRefreshTotp,
  onShowToast,
}) => {
  const isVi = lang === 'vi';

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
      await onRefreshTotp();
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
      await onRefreshTotp();
    } catch (e: any) {
      setTotpDisableError(e.message || (isVi ? 'Không thể tắt 2FA' : 'Failed to disable 2FA'));
    } finally {
      setIsDisablingTotp(false);
    }
  };

  return (
    <>
      <div
        id="section-2fa-totp"
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
    </>
  );
};
