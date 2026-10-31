import type React from 'react';

interface ModeStepProps {
  operatingMode: 'StandaloneFree' | 'OnlineManaged';
  serverUrl: string;
  enrollmentCode: string;
  loading: boolean;
  onSelectMode: (mode: 'StandaloneFree' | 'OnlineManaged') => void;
  onServerUrlChange: (url: string) => void;
  onEnrollmentCodeChange: (code: string) => void;
  onBack: () => void;
  onSubmit: () => void;
  isVi: boolean;
}

export const ModeStep: React.FC<ModeStepProps> = ({
  operatingMode,
  serverUrl,
  enrollmentCode,
  loading,
  onSelectMode,
  onServerUrlChange,
  onEnrollmentCodeChange,
  onBack,
  onSubmit,
  isVi,
}) => {
  return (
    <div>
      <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8 }}>
        {isVi ? 'Chọn Chế Độ Vận Hành' : 'Select Operating Mode'}
      </h2>
      <p style={{ color: '#475569', fontSize: '0.88rem', marginBottom: 20 }}>
        {isVi
          ? 'FamShield hỗ trợ cả hai mô hình: Ngoại tuyến hoàn toàn hoặc Quản lý từ xa.'
          : 'FamShield supports both 100% Offline Standalone and Cloud-Managed models.'}
      </p>

      <div
        role="radiogroup"
        aria-label={isVi ? 'Chế độ vận hành' : 'Operating mode'}
        style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14, marginBottom: 24 }}
      >
        {/* StandaloneFree */}
        <div
          role="radio"
          tabIndex={0}
          aria-checked={operatingMode === 'StandaloneFree'}
          onClick={() => onSelectMode('StandaloneFree')}
          onKeyDown={(event) => {
            if (event.target !== event.currentTarget) return;
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelectMode('StandaloneFree');
            }
          }}
          style={{
            background: operatingMode === 'StandaloneFree' ? '#f0f9ff' : '#f8fafc',
            border: `2px solid ${operatingMode === 'StandaloneFree' ? '#0284c7' : '#cbd5e1'}`,
            borderRadius: 16,
            padding: '18px 20px',
            cursor: 'pointer',
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
              <span>💻</span>
              <span>
                {isVi
                  ? 'Ngoại Tuyến Độc Lập (StandaloneFree - Khuyên Dùng)'
                  : 'Offline Standalone (StandaloneFree - Recommended)'}
              </span>
            </div>
            <span
              style={{
                background: '#10b981',
                color: '#022c22',
                fontSize: '0.72rem',
                fontWeight: 800,
                padding: '3px 8px',
                borderRadius: 8,
              }}
            >
              {isVi ? 'MIỄN PHÍ TRỌN ĐỜI' : 'LIFETIME FREE'}
            </span>
          </div>
          <div style={{ fontSize: '0.84rem', color: '#475569', lineHeight: 1.5 }}>
            {isVi
              ? '100% dữ liệu nằm trên máy tính này. Không yêu cầu tài khoản, không cần internet. Bảo mật tối đa.'
              : '100% local operation on this PC. No account or internet connection required. Maximum privacy.'}
          </div>
        </div>

        {/* OnlineManaged */}
        <div
          role="radio"
          tabIndex={0}
          aria-checked={operatingMode === 'OnlineManaged'}
          onClick={() => onSelectMode('OnlineManaged')}
          onKeyDown={(event) => {
            if (event.target !== event.currentTarget) return;
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelectMode('OnlineManaged');
            }
          }}
          style={{
            background: operatingMode === 'OnlineManaged' ? '#eef2ff' : '#f8fafc',
            border: `2px solid ${operatingMode === 'OnlineManaged' ? 'var(--accent)' : 'var(--border-strong)'}`,
            borderRadius: 16,
            padding: '18px 20px',
            cursor: 'pointer',
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
            <span>☁️</span>
            <span>
              {isVi
                ? 'Quản Lý Từ Xa Đám Mây (OnlineManaged)'
                : 'Cloud Remote Managed (OnlineManaged)'}
            </span>
          </div>
          <div
            style={{
              fontSize: '0.84rem',
              color: '#475569',
              lineHeight: 1.5,
              marginBottom: operatingMode === 'OnlineManaged' ? 12 : 0,
            }}
          >
            {isVi
              ? 'Cho phép phụ huynh giám sát và nhận thông báo qua ứng dụng di động hoặc Telegram Bot.'
              : 'Enables mobile notifications and remote oversight via parent dashboard or Telegram Bot.'}
          </div>

          {operatingMode === 'OnlineManaged' && (
            <div style={{ marginTop: 14, display: 'flex', flexDirection: 'column', gap: 10 }}>
              <div>
                <label
                  htmlFor="setup-server-url"
                  style={{
                    display: 'block',
                    fontSize: '0.82rem',
                    color: '#334155',
                    marginBottom: 4,
                  }}
                >
                  {isVi ? 'Địa chỉ Máy Chủ Quản Trị (Server URL):' : 'Server Endpoint URL:'}
                </label>
                <input
                  id="setup-server-url"
                  type="text"
                  value={serverUrl}
                  onChange={(e) => onServerUrlChange(e.target.value)}
                  placeholder="https://famshield.ttcantho.io.vn"
                  style={{
                    width: '100%',
                    background: '#ffffff',
                    border: '1px solid #cbd5e1',
                    borderRadius: 10,
                    padding: '8px 12px',
                    color: '#0f172a',
                    fontSize: '0.85rem',
                    outline: 'none',
                  }}
                />
              </div>

              <div>
                <label
                  htmlFor="setup-enrollment-code"
                  style={{
                    display: 'block',
                    fontSize: '0.82rem',
                    color: '#334155',
                    marginBottom: 4,
                  }}
                >
                  {isVi
                    ? 'Mã Ghép Nối (Enrollment Code - tùy chọn):'
                    : 'Enrollment Code (Optional):'}
                </label>
                <input
                  id="setup-enrollment-code"
                  type="text"
                  maxLength={16}
                  value={enrollmentCode}
                  onChange={(e) => onEnrollmentCodeChange(e.target.value.toUpperCase())}
                  placeholder={isVi ? 'VD: FS-9821' : 'Example: FS-9821'}
                  style={{
                    width: '100%',
                    background: '#ffffff',
                    border: '1px solid #cbd5e1',
                    borderRadius: 10,
                    padding: '8px 12px',
                    color: '#0f172a',
                    fontSize: '0.85rem',
                    outline: 'none',
                  }}
                />
              </div>
            </div>
          )}
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
          disabled={loading}
          onClick={onSubmit}
          style={{
            flex: 2,
            background: loading ? '#334155' : 'linear-gradient(135deg, #10b981, #34d399)',
            color: loading ? '#94a3b8' : '#022c22',
            border: 'none',
            borderRadius: 12,
            padding: '12px',
            fontWeight: 800,
            cursor: loading ? 'not-allowed' : 'pointer',
            boxShadow: '0 4px 15px rgba(16, 185, 129, 0.3)',
          }}
        >
          {loading
            ? isVi
              ? 'Đang kích hoạt...'
              : 'Initializing...'
            : isVi
              ? 'Hoàn tất & Kích Hoạt Bảo Vệ'
              : 'Complete & Activate'}{' '}
          ✨
        </button>
      </div>
    </div>
  );
};
