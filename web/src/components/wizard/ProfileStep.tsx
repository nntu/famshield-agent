import type React from 'react';

interface ProfileStepProps {
  presetProfile: 'balanced' | 'strict' | 'relaxed';
  onSelectProfile: (profile: 'balanced' | 'strict' | 'relaxed') => void;
  onBack: () => void;
  onNext: () => void;
  isVi: boolean;
}

export const ProfileStep: React.FC<ProfileStepProps> = ({
  presetProfile,
  onSelectProfile,
  onBack,
  onNext,
  isVi,
}) => {
  return (
    <div>
      <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8 }}>
        {isVi ? 'Chọn Cấu Hình Bảo Vệ Ban Đầu' : 'Select Initial Protection Preset'}
      </h2>
      <p style={{ color: '#475569', fontSize: '0.88rem', marginBottom: 20 }}>
        {isVi
          ? 'Chọn mức độ lọc phù hợp với độ tuổi của trẻ. Bạn có thể tùy chỉnh thêm bất kỳ lúc nào.'
          : 'Choose the baseline security level. You can add or modify custom domain & app rules anytime.'}
      </p>

      <div
        role="radiogroup"
        aria-label={isVi ? 'Cấu hình bảo vệ' : 'Protection preset'}
        style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 12, marginBottom: 24 }}
      >
        {/* Balanced Preset */}
        <div
          role="radio"
          tabIndex={0}
          aria-checked={presetProfile === 'balanced'}
          onClick={() => onSelectProfile('balanced')}
          onKeyDown={(event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelectProfile('balanced');
            }
          }}
          style={{
            background: presetProfile === 'balanced' ? '#f0f9ff' : '#f8fafc',
            border: `2px solid ${presetProfile === 'balanced' ? '#0284c7' : '#cbd5e1'}`,
            borderRadius: 16,
            padding: '16px 20px',
            cursor: 'pointer',
            transition: 'all 0.2s ease',
            position: 'relative',
          }}
        >
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 6,
            }}
          >
            <div
              style={{
                fontWeight: 700,
                fontSize: '1rem',
                color: '#0f172a',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
              }}
            >
              <span>⚖️</span>
              <span>{isVi ? 'Cân Bằng (Khuyên dùng)' : 'Balanced (Recommended)'}</span>
            </div>
            <span
              style={{
                background: '#0284c7',
                color: '#f0f9ff',
                fontSize: '0.72rem',
                fontWeight: 700,
                padding: '3px 8px',
                borderRadius: 8,
              }}
            >
              {isVi ? 'Phổ biến nhất' : 'Most Popular'}
            </span>
          </div>
          <div style={{ fontSize: '0.84rem', color: '#475569', lineHeight: 1.5 }}>
            {isVi
              ? 'Chặn web người lớn, cờ bạc, lừa đảo. Tự động kích hoạt SafeSearch Google/Bing/YouTube. Hạn chế Game/Roblox 60 phút/ngày.'
              : 'Blocks adult, gambling, and phishing sites. Forces SafeSearch on Google/Bing/YouTube. Caps gaming apps at 60 mins/day.'}
          </div>
        </div>

        {/* Strict Preset */}
        <div
          role="radio"
          tabIndex={0}
          aria-checked={presetProfile === 'strict'}
          onClick={() => onSelectProfile('strict')}
          onKeyDown={(event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelectProfile('strict');
            }
          }}
          style={{
            background: presetProfile === 'strict' ? '#eef2ff' : '#f8fafc',
            border: `2px solid ${presetProfile === 'strict' ? 'var(--accent)' : 'var(--border-strong)'}`,
            borderRadius: 16,
            padding: '16px 20px',
            cursor: 'pointer',
            transition: 'all 0.2s ease',
          }}
        >
          <div
            style={{
              fontWeight: 700,
              fontSize: '1rem',
              color: '#0f172a',
              marginBottom: 6,
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}
          >
            <span>🛡️</span>
            <span>
              {isVi ? 'Nghiêm Ngặt (Dành cho trẻ nhỏ)' : 'Strict (For Younger Kids)'}
            </span>
          </div>
          <div style={{ fontSize: '0.84rem', color: '#475569', lineHeight: 1.5 }}>
            {isVi
              ? 'Chặn toàn bộ web độc hại + Mạng xã hội (TikTok, Facebook, Instagram). Giới hạn thời gian màn hình 30 phút/ngày.'
              : 'Blocks malicious sites + Social Media (TikTok, Facebook, Instagram). Strict 30-minute daily app limit.'}
          </div>
        </div>

        {/* Relaxed / Basic Preset */}
        <div
          role="radio"
          tabIndex={0}
          aria-checked={presetProfile === 'relaxed'}
          onClick={() => onSelectProfile('relaxed')}
          onKeyDown={(event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelectProfile('relaxed');
            }
          }}
          style={{
            background: presetProfile === 'relaxed' ? '#ecfdf5' : '#f8fafc',
            border: `2px solid ${presetProfile === 'relaxed' ? '#059669' : '#cbd5e1'}`,
            borderRadius: 16,
            padding: '16px 20px',
            cursor: 'pointer',
            transition: 'all 0.2s ease',
          }}
        >
          <div
            style={{
              fontWeight: 700,
              fontSize: '1rem',
              color: '#0f172a',
              marginBottom: 6,
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}
          >
            <span>⚙️</span>
            <span>{isVi ? 'Cơ Bản (Tự Tùy Chỉnh)' : 'Basic (Custom Rules)'}</span>
          </div>
          <div style={{ fontSize: '0.84rem', color: '#475569', lineHeight: 1.5 }}>
            {isVi
              ? 'Chỉ chặn mã độc và trang lừa đảo nguy hiểm. Bạn sẽ tự bổ sung các tên miền và ứng dụng cần quản lý trong Bảng Quản Trị.'
              : 'Only filters critical malware and phishing threats. You will configure custom domain rules in the console.'}
          </div>
        </div>
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
