import React, { useState } from 'react';
import { InitializeSetupResponse, Language } from '../types';
import { api } from '../api';

interface SetupWizardProps {
  lang: Language;
  onComplete: (token: string) => void;
}

export const SetupWizard: React.FC<SetupWizardProps> = ({ lang, onComplete }) => {
  const [step, setStep] = useState<number>(1);
  const [pin, setPin] = useState<string>('');
  const [confirmPin, setConfirmPin] = useState<string>('');
  const [pinError, setPinError] = useState<string>('');

  const [presetProfile, setPresetProfile] = useState<'balanced' | 'strict' | 'relaxed'>('balanced');
  const [applyDefaultRules, setApplyDefaultRules] = useState<boolean>(true);

  const [operatingMode, setOperatingMode] = useState<'StandaloneFree' | 'OnlineManaged'>('StandaloneFree');
  const [enrollmentCode, setEnrollmentCode] = useState<string>('');
  const [serverUrl, setServerUrl] = useState<string>('https://api.famshield.cloud');

  const [loading, setLoading] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string>('');
  const [setupResult, setSetupResult] = useState<InitializeSetupResponse | null>(null);
  const [copiedKey, setCopiedKey] = useState<boolean>(false);

  const isVi = lang === 'vi';

  const handleNextFromPin = () => {
    if (pin.length !== 6 || !/^\d{6}$/.test(pin)) {
      setPinError(isVi ? 'Vui lòng nhập đúng 6 chữ số' : 'Please enter exactly 6 digits');
      return;
    }
    if (pin !== confirmPin) {
      setPinError(isVi ? 'Mã xác nhận PIN không khớp' : 'PIN confirmation does not match');
      return;
    }
    setPinError('');
    setStep(3);
  };

  const handleSubmitSetup = async () => {
    setLoading(true);
    setErrorMessage('');
    try {
      const res = await api.initializeSetup({
        pin,
        profile: presetProfile,
        operating_mode: operatingMode,
        enrollment_code: operatingMode === 'OnlineManaged' ? enrollmentCode.trim() : undefined,
        server_url: operatingMode === 'OnlineManaged' ? serverUrl.trim() : undefined,
      });
      setSetupResult(res);
      setStep(5);
    } catch (err: any) {
      setErrorMessage(err.message || (isVi ? 'Khởi tạo thất bại, vui lòng thử lại' : 'Initialization failed'));
    } finally {
      setLoading(false);
    }
  };

  const copyRecoveryKey = () => {
    if (!setupResult?.recovery_key) return;
    navigator.clipboard.writeText(setupResult.recovery_key);
    setCopiedKey(true);
    setTimeout(() => setCopiedKey(false), 3000);
  };

  const handleKeypadPress = (num: string, isConfirm: boolean = false) => {
    if (isConfirm) {
      if (confirmPin.length < 6) setConfirmPin(confirmPin + num);
    } else {
      if (pin.length < 6) setPin(pin + num);
    }
  };

  const handleKeypadBackspace = (isConfirm: boolean = false) => {
    if (isConfirm) {
      setConfirmPin(confirmPin.slice(0, -1));
    } else {
      setPin(pin.slice(0, -1));
    }
  };

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      padding: '24px',
      background: 'radial-gradient(circle at 50% 10%, #1e1b4b 0%, #070b14 70%)',
    }}>
      <div style={{
        maxWidth: 720,
        width: '100%',
        background: 'rgba(15, 23, 42, 0.85)',
        backdropFilter: 'blur(20px)',
        border: '1px solid rgba(56, 189, 248, 0.25)',
        borderRadius: 24,
        padding: '36px',
        boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.7), 0 0 40px rgba(56, 189, 248, 0.1)',
        color: '#f8fafc',
      }}>
        {/* Progress Tracker */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 32 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <span style={{ fontSize: '1.8rem' }}>🛡️</span>
            <div>
              <div style={{ fontSize: '1.25rem', fontWeight: 800, background: 'linear-gradient(135deg, #38bdf8, #818cf8)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent' }}>
                FamShield
              </div>
              <div style={{ fontSize: '0.78rem', color: '#94a3b8' }}>
                {isVi ? 'Trợ lý Thiết lập Ban đầu' : 'Initial Setup Wizard'}
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            {[1, 2, 3, 4, 5].map((i) => (
              <div
                key={i}
                style={{
                  width: i === step ? 28 : 10,
                  height: 10,
                  borderRadius: 5,
                  background: i === step ? '#38bdf8' : i < step ? '#10b981' : '#334155',
                  transition: 'all 0.3s ease',
                }}
              />
            ))}
          </div>
        </div>

        {errorMessage && (
          <div style={{
            background: 'rgba(239, 68, 68, 0.15)',
            border: '1px solid #ef4444',
            color: '#fca5a5',
            padding: '12px 16px',
            borderRadius: 12,
            marginBottom: 20,
            fontSize: '0.88rem',
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}>
            <span>⚠️</span>
            <span>{errorMessage}</span>
          </div>
        )}

        {/* STEP 1: WELCOME */}
        {step === 1 && (
          <div>
            <h1 style={{ fontSize: '1.75rem', fontWeight: 800, marginBottom: 8, color: '#f8fafc' }}>
              {isVi ? 'Chào mừng bạn đến với FamShield' : 'Welcome to FamShield'}
            </h1>
            <p style={{ color: '#94a3b8', fontSize: '0.95rem', lineHeight: 1.6, marginBottom: 24 }}>
              {isVi
                ? 'Giải pháp bảo vệ toàn diện máy trạm Windows dành cho gia đình. Khởi tạo cấu hình an toàn chỉ trong 1 phút để bắt đầu bảo vệ con em của bạn.'
                : 'Comprehensive Windows endpoint protection for modern families. Configure your secure baseline in under 1 minute to start protecting your children.'}
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 16, marginBottom: 32 }}>
              <div style={{
                background: 'rgba(30, 41, 59, 0.5)',
                border: '1px solid #334155',
                borderRadius: 16,
                padding: '16px 20px',
                display: 'flex',
                gap: 16,
                alignItems: 'center',
              }}>
                <span style={{ fontSize: '1.8rem', background: '#0284c7', width: 44, height: 44, borderRadius: 12, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  🔒
                </span>
                <div>
                  <div style={{ fontWeight: 700, fontSize: '0.98rem', color: '#e2e8f0', marginBottom: 2 }}>
                    {isVi ? '100% Ngoại Tuyến & Riêng Tư (Zero Cloud Dependency)' : '100% Offline & Private (Zero Cloud Dependency)'}
                  </div>
                  <div style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
                    {isVi
                      ? 'Hoạt động độc lập hoàn toàn, không phụ thuộc máy chủ internet, dữ liệu duyệt web tuyệt đối không bị lộ.'
                      : 'Runs fully self-contained on your PC with zero cloud dependency. Family web telemetry never leaves your device.'}
                  </div>
                </div>
              </div>

              <div style={{
                background: 'rgba(30, 41, 59, 0.5)',
                border: '1px solid #334155',
                borderRadius: 16,
                padding: '16px 20px',
                display: 'flex',
                gap: 16,
                alignItems: 'center',
              }}>
                <span style={{ fontSize: '1.8rem', background: '#0d9488', width: 44, height: 44, borderRadius: 12, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  ⚡
                </span>
                <div>
                  <div style={{ fontWeight: 700, fontSize: '0.98rem', color: '#e2e8f0', marginBottom: 2 }}>
                    {isVi ? 'Tường Lửa WFP & DNS Loopback Phản Ứng 0ms' : 'Dynamic WFP & Loopback DNS Filter (0ms Latency)'}
                  </div>
                  <div style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
                    {isVi
                      ? 'Chặn đứng web đen, cờ bạc, lừa đảo tại cổng DNS 127.0.0.2:53 mà không làm suy giảm tốc độ mạng.'
                      : 'Instantly intercepts adult, gambling, and phishing domains at loopback DNS without network slowdown.'}
                  </div>
                </div>
              </div>

              <div style={{
                background: 'rgba(30, 41, 59, 0.5)',
                border: '1px solid #334155',
                borderRadius: 16,
                padding: '16px 20px',
                display: 'flex',
                gap: 16,
                alignItems: 'center',
              }}>
                <span style={{ fontSize: '1.8rem', background: '#6366f1', width: 44, height: 44, borderRadius: 12, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  👁️
                </span>
                <div>
                  <div style={{ fontWeight: 700, fontSize: '0.98rem', color: '#e2e8f0', marginBottom: 2 }}>
                    {isVi ? 'Bằng Chứng Thị Giác Xóa Mờ (Privacy-by-Design)' : 'Privacy-Preserving Visual Evidence'}
                  </div>
                  <div style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
                    {isVi
                      ? 'Ảnh chụp vi phạm được làm mờ Gaussian trong RAM trước khi lưu đĩa, bảo vệ sự riêng tư của con cái.'
                      : 'Screenshots are Gaussian-blurred in RAM before saving. Raw text is never stored, auto-purged after 48h.'}
                  </div>
                </div>
              </div>
            </div>

            <button
              onClick={() => setStep(2)}
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
                boxShadow: '0 8px 25px rgba(56, 189, 248, 0.3)',
              }}
            >
              <span>{isVi ? 'Bắt đầu Thiết lập' : 'Start Setup'}</span>
              <span>→</span>
            </button>
          </div>
        )}

        {/* STEP 2: MASTER PIN */}
        {step === 2 && (
          <div>
            <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8 }}>
              {isVi ? 'Đặt Mã Master PIN Phụ Huynh' : 'Set Guardian Master PIN'}
            </h2>
            <p style={{ color: '#94a3b8', fontSize: '0.88rem', marginBottom: 24, lineHeight: 1.5 }}>
              {isVi
                ? 'Mã PIN 6 chữ số này dùng để đăng nhập vào Bảng Quản Trị và mở khóa tạm thời khi có yêu cầu từ trẻ.'
                : 'This 6-digit Master PIN unlocks the admin console and allows temporary quota overrides.'}
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20, marginBottom: 20 }}>
              {/* Enter PIN */}
              <div>
                <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 8 }}>
                  {isVi ? 'Mã PIN mới (6 chữ số):' : 'New PIN (6 digits):'}
                </label>
                <input
                  type="password"
                  maxLength={6}
                  value={pin}
                  onChange={(e) => setPin(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  placeholder="••••••"
                  style={{
                    width: '100%',
                    background: '#1e293b',
                    border: '1px solid #475569',
                    borderRadius: 12,
                    padding: '12px 16px',
                    fontSize: '1.4rem',
                    textAlign: 'center',
                    letterSpacing: '0.5rem',
                    color: '#38bdf8',
                    outline: 'none',
                    fontWeight: 700,
                  }}
                />
              </div>

              {/* Confirm PIN */}
              <div>
                <label style={{ display: 'block', fontSize: '0.85rem', color: '#cbd5e1', fontWeight: 600, marginBottom: 8 }}>
                  {isVi ? 'Xác nhận lại mã PIN:' : 'Confirm PIN:'}
                </label>
                <input
                  type="password"
                  maxLength={6}
                  value={confirmPin}
                  onChange={(e) => setConfirmPin(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  placeholder="••••••"
                  style={{
                    width: '100%',
                    background: '#1e293b',
                    border: '1px solid #475569',
                    borderRadius: 12,
                    padding: '12px 16px',
                    fontSize: '1.4rem',
                    textAlign: 'center',
                    letterSpacing: '0.5rem',
                    color: '#38bdf8',
                    outline: 'none',
                    fontWeight: 700,
                  }}
                />
              </div>
            </div>

            {pinError && (
              <div style={{ color: '#f87171', fontSize: '0.85rem', marginBottom: 16, fontWeight: 500 }}>
                ⚠️ {pinError}
              </div>
            )}

            <div style={{
              background: 'rgba(30, 41, 59, 0.4)',
              border: '1px solid rgba(148, 163, 184, 0.15)',
              borderRadius: 12,
              padding: '12px 16px',
              fontSize: '0.82rem',
              color: '#94a3b8',
              marginBottom: 28,
            }}>
              💡 {isVi
                ? 'Gợi ý: Tránh dùng ngày sinh hoặc dãy số liên tiếp (123456). Hệ thống sẽ tự động cấp Mã Khôi Phục Khẩn Cấp sau khi hoàn tất.'
                : 'Tip: Avoid consecutive digits (123456). FamShield will generate an Emergency Recovery Key upon completion.'}
            </div>

            <div style={{ display: 'flex', gap: 12 }}>
              <button
                onClick={() => setStep(1)}
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
                onClick={handleNextFromPin}
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
        )}

        {/* STEP 3: PRESET PROTECTION PROFILE */}
        {step === 3 && (
          <div>
            <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8 }}>
              {isVi ? 'Chọn Cấu Hình Bảo Vệ Ban Đầu' : 'Select Initial Protection Preset'}
            </h2>
            <p style={{ color: '#94a3b8', fontSize: '0.88rem', marginBottom: 20 }}>
              {isVi
                ? 'Chọn mức độ lọc phù hợp với độ tuổi của trẻ. Bạn có thể tùy chỉnh thêm bất kỳ lúc nào.'
                : 'Choose the baseline security level. You can add or modify custom domain & app rules anytime.'}
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 12, marginBottom: 24 }}>
              {/* Balanced Preset */}
              <div
                onClick={() => setPresetProfile('balanced')}
                style={{
                  background: presetProfile === 'balanced' ? 'rgba(56, 189, 248, 0.1)' : 'rgba(30, 41, 59, 0.4)',
                  border: `2px solid ${presetProfile === 'balanced' ? '#38bdf8' : '#334155'}`,
                  borderRadius: 16,
                  padding: '16px 20px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                  position: 'relative',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                  <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span>⚖️</span>
                    <span>{isVi ? 'Cân Bằng (Khuyên dùng)' : 'Balanced (Recommended)'}</span>
                  </div>
                  <span style={{
                    background: '#0284c7',
                    color: '#f0f9ff',
                    fontSize: '0.72rem',
                    fontWeight: 700,
                    padding: '3px 8px',
                    borderRadius: 8,
                  }}>
                    {isVi ? 'Phổ biến nhất' : 'Most Popular'}
                  </span>
                </div>
                <div style={{ fontSize: '0.84rem', color: '#94a3b8', lineHeight: 1.5 }}>
                  {isVi
                    ? 'Chặn web người lớn, cờ bạc, lừa đảo. Tự động kích hoạt SafeSearch Google/Bing/YouTube. Hạn chế Game/Roblox 60 phút/ngày.'
                    : 'Blocks adult, gambling, and phishing sites. Forces SafeSearch on Google/Bing/YouTube. Caps gaming apps at 60 mins/day.'}
                </div>
              </div>

              {/* Strict Preset */}
              <div
                onClick={() => setPresetProfile('strict')}
                style={{
                  background: presetProfile === 'strict' ? 'rgba(129, 140, 248, 0.1)' : 'rgba(30, 41, 59, 0.4)',
                  border: `2px solid ${presetProfile === 'strict' ? '#818cf8' : '#334155'}`,
                  borderRadius: 16,
                  padding: '16px 20px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
              >
                <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span>🛡️</span>
                  <span>{isVi ? 'Nghiêm Ngặt (Dành cho trẻ nhỏ)' : 'Strict (For Younger Kids)'}</span>
                </div>
                <div style={{ fontSize: '0.84rem', color: '#94a3b8', lineHeight: 1.5 }}>
                  {isVi
                    ? 'Chặn toàn bộ web độc hại + Mạng xã hội (TikTok, Facebook, Instagram). Giới hạn thời gian màn hình 30 phút/ngày.'
                    : 'Blocks malicious sites + Social Media (TikTok, Facebook, Instagram). Strict 30-minute daily app limit.'}
                </div>
              </div>

              {/* Relaxed / Basic Preset */}
              <div
                onClick={() => setPresetProfile('relaxed')}
                style={{
                  background: presetProfile === 'relaxed' ? 'rgba(52, 211, 153, 0.1)' : 'rgba(30, 41, 59, 0.4)',
                  border: `2px solid ${presetProfile === 'relaxed' ? '#34d399' : '#334155'}`,
                  borderRadius: 16,
                  padding: '16px 20px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
              >
                <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span>⚙️</span>
                  <span>{isVi ? 'Cơ Bản (Tự Tùy Chỉnh)' : 'Basic (Custom Rules)'}</span>
                </div>
                <div style={{ fontSize: '0.84rem', color: '#94a3b8', lineHeight: 1.5 }}>
                  {isVi
                    ? 'Chỉ chặn mã độc và trang lừa đảo nguy hiểm. Bạn sẽ tự bổ sung các tên miền và ứng dụng cần quản lý trong Bảng Quản Trị.'
                    : 'Only filters critical malware and phishing threats. You will configure custom domain rules in the console.'}
                </div>
              </div>
            </div>

            <div style={{ display: 'flex', gap: 12 }}>
              <button
                onClick={() => setStep(2)}
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
                onClick={() => setStep(4)}
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
        )}

        {/* STEP 4: OPERATING MODE */}
        {step === 4 && (
          <div>
            <h2 style={{ fontSize: '1.5rem', fontWeight: 800, marginBottom: 8 }}>
              {isVi ? 'Chọn Chế Độ Vận Hành' : 'Select Operating Mode'}
            </h2>
            <p style={{ color: '#94a3b8', fontSize: '0.88rem', marginBottom: 20 }}>
              {isVi
                ? 'FamShield hỗ trợ cả hai mô hình: Ngoại tuyến hoàn toàn hoặc Quản lý từ xa.'
                : 'FamShield supports both 100% Offline Standalone and Cloud-Managed models.'}
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: 14, marginBottom: 24 }}>
              {/* StandaloneFree */}
              <div
                onClick={() => setOperatingMode('StandaloneFree')}
                style={{
                  background: operatingMode === 'StandaloneFree' ? 'rgba(56, 189, 248, 0.1)' : 'rgba(30, 41, 59, 0.4)',
                  border: `2px solid ${operatingMode === 'StandaloneFree' ? '#38bdf8' : '#334155'}`,
                  borderRadius: 16,
                  padding: '18px 20px',
                  cursor: 'pointer',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                  <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span>💻</span>
                    <span>{isVi ? 'Ngoại Tuyến Độc Lập (StandaloneFree - Khuyên Dùng)' : 'Offline Standalone (StandaloneFree - Recommended)'}</span>
                  </div>
                  <span style={{
                    background: '#10b981',
                    color: '#022c22',
                    fontSize: '0.72rem',
                    fontWeight: 800,
                    padding: '3px 8px',
                    borderRadius: 8,
                  }}>
                    {isVi ? 'MIỄN PHÍ TRỌN ĐỜI' : 'LIFETIME FREE'}
                  </span>
                </div>
                <div style={{ fontSize: '0.84rem', color: '#94a3b8', lineHeight: 1.5 }}>
                  {isVi
                    ? '100% dữ liệu nằm trên máy tính này. Không yêu cầu tài khoản, không cần internet. Bảo mật tối đa.'
                    : '100% local operation on this PC. No account or internet connection required. Maximum privacy.'}
                </div>
              </div>

              {/* OnlineManaged */}
              <div
                onClick={() => setOperatingMode('OnlineManaged')}
                style={{
                  background: operatingMode === 'OnlineManaged' ? 'rgba(129, 140, 248, 0.1)' : 'rgba(30, 41, 59, 0.4)',
                  border: `2px solid ${operatingMode === 'OnlineManaged' ? '#818cf8' : '#334155'}`,
                  borderRadius: 16,
                  padding: '18px 20px',
                  cursor: 'pointer',
                }}
              >
                <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', marginBottom: 6, display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span>☁️</span>
                  <span>{isVi ? 'Quản Lý Từ Xa Đám Mây (OnlineManaged)' : 'Cloud Remote Managed (OnlineManaged)'}</span>
                </div>
                <div style={{ fontSize: '0.84rem', color: '#94a3b8', lineHeight: 1.5, marginBottom: operatingMode === 'OnlineManaged' ? 12 : 0 }}>
                  {isVi
                    ? 'Cho phép phụ huynh giám sát và nhận thông báo qua ứng dụng di động hoặc Telegram Bot.'
                    : 'Enables mobile notifications and remote oversight via parent dashboard or Telegram Bot.'}
                </div>

                {operatingMode === 'OnlineManaged' && (
                  <div style={{ marginTop: 14, display: 'flex', flexDirection: 'column', gap: 10 }}>
                    <div>
                      <label style={{ display: 'block', fontSize: '0.82rem', color: '#cbd5e1', marginBottom: 4 }}>
                        {isVi ? 'Địa chỉ Máy Chủ Quản Trị (Server URL):' : 'Server Endpoint URL:'}
                      </label>
                      <input
                        type="text"
                        value={serverUrl}
                        onChange={(e) => setServerUrl(e.target.value)}
                        placeholder="https://api.famshield.cloud"
                        style={{
                          width: '100%',
                          background: '#0f172a',
                          border: '1px solid #475569',
                          borderRadius: 10,
                          padding: '8px 12px',
                          color: '#f8fafc',
                          fontSize: '0.85rem',
                          outline: 'none',
                        }}
                      />
                    </div>

                    <div>
                      <label style={{ display: 'block', fontSize: '0.82rem', color: '#cbd5e1', marginBottom: 4 }}>
                        {isVi ? 'Mã Ghép Nối (Enrollment Code - tùy chọn):' : 'Enrollment Code (Optional):'}
                      </label>
                      <input
                        type="text"
                        maxLength={16}
                        value={enrollmentCode}
                        onChange={(e) => setEnrollmentCode(e.target.value.toUpperCase())}
                        placeholder="VD: FS-9821"
                        style={{
                          width: '100%',
                          background: '#0f172a',
                          border: '1px solid #475569',
                          borderRadius: 10,
                          padding: '8px 12px',
                          color: '#f8fafc',
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
                onClick={() => setStep(3)}
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
                onClick={handleSubmitSetup}
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
                {loading ? (isVi ? 'Đang kích hoạt...' : 'Initializing...') : (isVi ? 'Hoàn tất & Kích Hoạt Bảo Vệ' : 'Complete & Activate')} ✨
              </button>
            </div>
          </div>
        )}

        {/* STEP 5: SUCCESS & RECOVERY KEY */}
        {step === 5 && setupResult && (
          <div>
            <div style={{ textAlign: 'center', marginBottom: 24 }}>
              <span style={{ fontSize: '3rem', display: 'inline-block', marginBottom: 8 }}>🎉</span>
              <h2 style={{ fontSize: '1.75rem', fontWeight: 800, color: '#f8fafc', marginBottom: 6 }}>
                {isVi ? 'Thiết Lập Thành Công!' : 'Setup Completed Successfully!'}
              </h2>
              <p style={{ color: '#94a3b8', fontSize: '0.9rem' }}>
                {isVi
                  ? `Đã khởi tạo hệ thống bảo vệ với ${setupResult.rules_created} quy tắc mẫu.`
                  : `Guardian initialized with ${setupResult.rules_created} baseline security rules.`}
              </p>
            </div>

            {/* Emergency Recovery Key Box */}
            <div style={{
              background: 'rgba(234, 179, 8, 0.1)',
              border: '1px solid rgba(234, 179, 8, 0.4)',
              borderRadius: 16,
              padding: '20px',
              marginBottom: 28,
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
                <span style={{ fontSize: '1.4rem' }}>🔑</span>
                <span style={{ fontWeight: 700, fontSize: '0.95rem', color: '#fef08a' }}>
                  {isVi ? 'MÃ KHÔI PHỤC KHẨN CẤP (EMERGENCY RECOVERY KEY)' : 'EMERGENCY RECOVERY KEY'}
                </span>
              </div>
              <p style={{ fontSize: '0.82rem', color: '#fef9c3', lineHeight: 1.5, marginBottom: 14 }}>
                {isVi
                  ? 'Hãy ghi lại hoặc chụp ảnh lưu giữ mã này. Trong trường hợp bạn quên mã PIN 6 số, đây là cách duy nhất để đặt lại quyền quản trị!'
                  : 'Please save this key securely. If you ever forget your Master PIN, this recovery key is required to reset access.'}
              </p>

              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                background: '#0f172a',
                border: '1px solid #334155',
                borderRadius: 10,
                padding: '10px 16px',
              }}>
                <span style={{
                  fontFamily: 'monospace',
                  fontSize: '1.15rem',
                  fontWeight: 700,
                  letterSpacing: '0.15rem',
                  color: '#38bdf8',
                }}>
                  {setupResult.recovery_key}
                </span>

                <button
                  onClick={copyRecoveryKey}
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
                  {copiedKey ? (isVi ? 'Đã sao chép! ✓' : 'Copied! ✓') : (isVi ? 'Sao chép' : 'Copy')}
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
        )}
      </div>
    </div>
  );
};
