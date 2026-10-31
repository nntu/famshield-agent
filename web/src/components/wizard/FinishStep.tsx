import type React from 'react';
import type { InitializeSetupResponse } from '../../types';

interface FinishStepProps {
  setupResult: InitializeSetupResponse;
  copiedKey: boolean;
  onCopyRecoveryKey: () => void;
  onComplete: (token: string) => void;
  isVi: boolean;
}

export const FinishStep: React.FC<FinishStepProps> = ({
  setupResult,
  copiedKey,
  onCopyRecoveryKey,
  onComplete,
  isVi,
}) => {
  return (
    <div>
      <div style={{ textAlign: 'center', marginBottom: 24 }}>
        <span style={{ fontSize: '3rem', display: 'inline-block', marginBottom: 8 }}>🎉</span>
        <h2
          style={{ fontSize: '1.75rem', fontWeight: 800, color: '#0f172a', marginBottom: 6 }}
        >
          {isVi ? 'Thiết Lập Thành Công!' : 'Setup Completed Successfully!'}
        </h2>
        <p style={{ color: '#475569', fontSize: '0.9rem' }}>
          {isVi
            ? `Đã khởi tạo hệ thống bảo vệ với ${setupResult.rules_created} quy tắc mẫu.`
            : `Guardian initialized with ${setupResult.rules_created} baseline security rules.`}
        </p>
      </div>

      {/* Emergency Recovery Key Box */}
      <div
        style={{
          background: '#fffbeb',
          border: '1px solid #fcd34d',
          borderRadius: 16,
          padding: '20px',
          marginBottom: 28,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
          <span style={{ fontSize: '1.4rem' }}>🔑</span>
          <span style={{ fontWeight: 700, fontSize: '0.95rem', color: '#854d0e' }}>
            {isVi
              ? 'MÃ KHÔI PHỤC KHẨN CẤP (EMERGENCY RECOVERY KEY)'
              : 'EMERGENCY RECOVERY KEY'}
          </span>
        </div>
        <p
          style={{ fontSize: '0.82rem', color: '#713f12', lineHeight: 1.5, marginBottom: 14 }}
        >
          {isVi
            ? 'Hãy ghi lại hoặc chụp ảnh lưu giữ mã này. Trong trường hợp bạn quên mã PIN 6 số, đây là cách duy nhất để đặt lại quyền quản trị!'
            : 'Please save this key securely. If you ever forget your Master PIN, this recovery key is required to reset access.'}
        </p>

        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            background: '#0f172a',
            border: '1px solid #334155',
            borderRadius: 10,
            padding: '10px 16px',
          }}
        >
          <span
            style={{
              fontFamily: 'monospace',
              fontSize: '1.15rem',
              fontWeight: 700,
              letterSpacing: '0.15rem',
              color: '#38bdf8',
            }}
          >
            {setupResult.recovery_key}
          </span>

          <button
            onClick={onCopyRecoveryKey}
            style={{
              background: copiedKey ? '#10b981' : '#1e293b',
              color: copiedKey ? '#022c22' : '#f8fafc',
              border: '1px solid #475569',
              borderRadius: 8,
              padding: '6px 14px',
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: 'pointer',
              transition: 'all 0.2s ease',
            }}
          >
            {copiedKey ? (isVi ? 'Đã sao chép! ✓' : 'Copied! ✓') : isVi ? 'Sao chép' : 'Copy'}
          </button>
        </div>
      </div>

      <button
        onClick={() => onComplete(setupResult.token)}
        style={{
          width: '100%',
          background: 'linear-gradient(135deg, #0284c7, #38bdf8)',
          color: '#030712',
          border: 'none',
          borderRadius: 14,
          padding: '14px 20px',
          fontWeight: 800,
          fontSize: '1rem',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 10,
          boxShadow: '0 8px 25px rgba(56, 189, 248, 0.4)',
        }}
      >
        <span>{isVi ? 'Truy cập Bảng Quản Trị Ngay' : 'Open Guardian Dashboard'}</span>
        <span>→</span>
      </button>
    </div>
  );
};
