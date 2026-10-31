import type React from 'react';

interface WelcomeStepProps {
  onNext: () => void;
  isVi: boolean;
}

export const WelcomeStep: React.FC<WelcomeStepProps> = ({ onNext, isVi }) => {
  return (
    <div>
      <h1 style={{ fontSize: '1.75rem', fontWeight: 800, marginBottom: 8, color: '#0f172a' }}>
        {isVi ? 'Chào mừng bạn đến với FamShield' : 'Welcome to FamShield'}
      </h1>
      <p style={{ color: '#64748b', fontSize: '0.95rem', lineHeight: 1.6, marginBottom: 24 }}>
        {isVi
          ? 'Giải pháp bảo vệ toàn diện máy trạm Windows dành cho gia đình. Khởi tạo cấu hình an toàn chỉ trong 1 phút để bắt đầu bảo vệ con em của bạn.'
          : 'Comprehensive Windows endpoint protection for modern families. Configure your secure baseline in under 1 minute to start protecting your children.'}
      </p>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 16, marginBottom: 32 }}>
        <div
          style={{
            background: '#f8fafc',
            border: '1px solid #e2e8f0',
            borderRadius: 16,
            padding: '16px 20px',
            display: 'flex',
            gap: 16,
            alignItems: 'center',
          }}
        >
          <span
            style={{
              fontSize: '1.8rem',
              background: '#0284c7',
              color: '#fff',
              width: 44,
              height: 44,
              borderRadius: 12,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            🔒
          </span>
          <div>
            <div
              style={{
                fontWeight: 700,
                fontSize: '0.98rem',
                color: '#0f172a',
                marginBottom: 2,
              }}
            >
              {isVi
                ? '100% Ngoại Tuyến & Riêng Tư (Zero Cloud Dependency)'
                : '100% Offline & Private (Zero Cloud Dependency)'}
            </div>
            <div style={{ fontSize: '0.85rem', color: '#64748b' }}>
              {isVi
                ? 'Hoạt động độc lập hoàn toàn, không phụ thuộc máy chủ internet, dữ liệu duyệt web tuyệt đối không bị lộ.'
                : 'Runs fully self-contained on your PC with zero cloud dependency. Family web telemetry never leaves your device.'}
            </div>
          </div>
        </div>

        <div
          style={{
            background: '#f8fafc',
            border: '1px solid #e2e8f0',
            borderRadius: 16,
            padding: '16px 20px',
            display: 'flex',
            gap: 16,
            alignItems: 'center',
          }}
        >
          <span
            style={{
              fontSize: '1.8rem',
              background: 'var(--accent)',
              color: '#fff',
              width: 44,
              height: 44,
              borderRadius: 12,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            ⚡
          </span>
          <div>
            <div
              style={{
                fontWeight: 700,
                fontSize: '0.98rem',
                color: '#0f172a',
                marginBottom: 2,
              }}
            >
              {isVi
                ? 'Tường Lửa WFP & DNS Loopback Phản Ứng 0ms'
                : 'Dynamic WFP & Loopback DNS Filter (0ms Latency)'}
            </div>
            <div style={{ fontSize: '0.85rem', color: '#64748b' }}>
              {isVi
                ? 'Chặn đứng web đen, cờ bạc, lừa đảo tại cổng DNS 127.0.0.2:53 mà không làm suy giảm tốc độ mạng.'
                : 'Instantly intercepts adult, gambling, and phishing domains at loopback DNS without network slowdown.'}
            </div>
          </div>
        </div>

        <div
          style={{
            background: '#f8fafc',
            border: '1px solid #e2e8f0',
            borderRadius: 16,
            padding: '16px 20px',
            display: 'flex',
            gap: 16,
            alignItems: 'center',
          }}
        >
          <span
            style={{
              fontSize: '1.8rem',
              background: 'var(--accent)',
              color: '#fff',
              width: 44,
              height: 44,
              borderRadius: 12,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            👁️
          </span>
          <div>
            <div
              style={{
                fontWeight: 700,
                fontSize: '0.98rem',
                color: '#0f172a',
                marginBottom: 2,
              }}
            >
              {isVi
                ? 'Bằng Chứng Thị Giác Xóa Mờ (Privacy-by-Design)'
                : 'Privacy-Preserving Visual Evidence'}
            </div>
            <div style={{ fontSize: '0.85rem', color: '#64748b' }}>
              {isVi
                ? 'Ảnh chụp vi phạm được làm mờ Gaussian trong RAM trước khi lưu đĩa, bảo vệ sự riêng tư của con cái.'
                : 'Screenshots are Gaussian-blurred in RAM before saving. Raw text is never stored, auto-purged after 48h.'}
            </div>
          </div>
        </div>
      </div>

      <button
        onClick={onNext}
        style={{
          width: '100%',
          background: 'linear-gradient(135deg, #0284c7, #38bdf8)',
          color: '#030712',
          border: 'none',
          borderRadius: 14,
          padding: '14px 20px',
          fontWeight: 700,
          fontSize: '1rem',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 10,
          boxShadow: '0 8px 25px rgba(56, 189, 248, 0.4)',
        }}
      >
        <span>{isVi ? 'Bắt đầu Thiết lập (1 Phút)' : 'Start Guided Setup (1 Min)'}</span>
        <span>→</span>
      </button>
    </div>
  );
};
