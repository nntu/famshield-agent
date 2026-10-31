import type React from 'react';

interface PinStepProps {
  pin: string;
  confirmPin: string;
  pinError: string;
  onPinChange: (val: string) => void;
  onConfirmPinChange: (val: string) => void;
  onBack: () => void;
  onNext: () => void;
  isVi: boolean;
}

export const PinStep: React.FC<PinStepProps> = ({
  pin,
  confirmPin,
  pinError,
  onPinChange,
  onConfirmPinChange,
  onBack,
  onNext,
  isVi,
}) => {
  return (
    <div>
      <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8, color: '#0f172a' }}>
        {isVi ? 'Đặt Mã Master PIN Phụ Huynh' : 'Set Guardian Master PIN'}
      </h2>
      <p style={{ color: '#64748b', fontSize: '0.88rem', marginBottom: 24, lineHeight: 1.5 }}>
        {isVi
          ? 'Mã PIN 6 chữ số này dùng để đăng nhập vào Bảng Quản Trị và mở khóa tạm thời khi có yêu cầu từ trẻ.'
          : 'This 6-digit Master PIN unlocks the admin console and allows temporary quota overrides.'}
      </p>

      <div className="grid grid-cols-1 sm:grid-cols-2" style={{ gap: 20, marginBottom: 20 }}>
        {/* Enter PIN */}
        <div>
          <label
            htmlFor="setup-pin"
            style={{
              display: 'block',
              fontSize: '0.85rem',
              color: '#475569',
              fontWeight: 600,
              marginBottom: 8,
            }}
          >
            {isVi ? 'Mã PIN mới (6 chữ số):' : 'New PIN (6 digits):'}
          </label>
          <input
            id="setup-pin"
            inputMode="numeric"
            autoComplete="new-password"
            type="password"
            maxLength={6}
            value={pin}
            onChange={(e) => onPinChange(e.target.value.replace(/\D/g, '').slice(0, 6))}
            placeholder="••••••"
            style={{
              width: '100%',
              background: '#f8fafc',
              border: '1px solid #cbd5e1',
              borderRadius: 12,
              padding: '12px 16px',
              fontSize: '1.4rem',
              textAlign: 'center',
              letterSpacing: '0.5rem',
              color: '#0284c7',
              outline: 'none',
              fontWeight: 700,
            }}
          />
        </div>

        {/* Confirm PIN */}
        <div>
          <label
            htmlFor="setup-confirm-pin"
            style={{
              display: 'block',
              fontSize: '0.85rem',
              color: '#475569',
              fontWeight: 600,
              marginBottom: 8,
            }}
          >
            {isVi ? 'Xác nhận lại mã PIN:' : 'Confirm PIN:'}
          </label>
          <input
            id="setup-confirm-pin"
            inputMode="numeric"
            autoComplete="new-password"
            type="password"
            maxLength={6}
            value={confirmPin}
            onChange={(e) => onConfirmPinChange(e.target.value.replace(/\D/g, '').slice(0, 6))}
            placeholder="••••••"
            style={{
              width: '100%',
              background: '#f8fafc',
              border: '1px solid #cbd5e1',
              borderRadius: 12,
              padding: '12px 16px',
              fontSize: '1.4rem',
              textAlign: 'center',
              letterSpacing: '0.5rem',
              color: '#0284c7',
              outline: 'none',
              fontWeight: 700,
            }}
          />
        </div>
      </div>

      {pinError && (
        <div
          role="alert"
          style={{ color: '#b91c1c', fontSize: '0.85rem', marginBottom: 16, fontWeight: 600 }}
        >
          ⚠️ {pinError}
        </div>
      )}

      <div
        style={{
          background: 'rgba(30, 41, 59, 0.4)',
          border: '1px solid rgba(148, 163, 184, 0.15)',
          borderRadius: 12,
          padding: '12px 16px',
          fontSize: '0.82rem',
          color: '#94a3b8',
          marginBottom: 28,
        }}
      >
        💡{' '}
        {isVi
          ? 'Gợi ý: Tránh dùng ngày sinh hoặc dãy số liên tiếp (123456). Hệ thống sẽ tự động cấp Mã Khôi Phục Khẩn Cấp sau khi hoàn tất.'
          : 'Tip: Avoid consecutive digits (123456). FamShield will generate an Emergency Recovery Key upon completion.'}
      </div>

      <div style={{ display: 'flex', gap: 12 }}>
        <button
          onClick={onBack}
          style={{
            flex: 1,
            background: 'transparent',
            border: '1px solid #475569',
            color: '#94a3b8',
            borderRadius: 12,
            padding: '12px',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          {isVi ? 'Quay lại' : 'Back'}
        </button>
        <button
          onClick={onNext}
          style={{
            flex: 2,
            background: 'linear-gradient(135deg, #0284c7, #38bdf8)',
            color: '#030712',
            border: 'none',
            borderRadius: 12,
            padding: '12px',
            fontWeight: 700,
            cursor: 'pointer',
            boxShadow: '0 4px 15px rgba(56, 189, 248, 0.3)',
          }}
        >
          {isVi ? 'Tiếp tục' : 'Next'} →
        </button>
      </div>
    </div>
  );
};
