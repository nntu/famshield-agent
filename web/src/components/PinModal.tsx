import React, { useState, useEffect } from 'react';
import { Lock, ShieldAlert, KeyRound, ShieldCheck, ArrowLeft } from 'lucide-react';
import { Language } from '../types';
import { translations } from '../i18n';
import { api } from '../api';

interface PinModalProps {
  isOpen: boolean;
  lang: Language;
  onSuccess: () => void;
}

export const PinModal: React.FC<PinModalProps> = ({ isOpen, lang, onSuccess }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  // Step: 'pin' | 'totp'
  const [step, setStep] = useState<'pin' | 'totp'>('pin');
  const [tempAuthToken, setTempAuthToken] = useState<string>('');

  // PIN State
  const [pin, setPin] = useState<string[]>(['', '', '', '', '', '']);

  // TOTP State
  const [otpCode, setOtpCode] = useState<string>('');

  const [error, setError] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [isShaking, setIsShaking] = useState<boolean>(false);

  useEffect(() => {
    if (!isOpen) {
      setStep('pin');
      setPin(['', '', '', '', '', '']);
      setOtpCode('');
      setError('');
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (step === 'pin') {
        if (e.key >= '0' && e.key <= '9') {
          handlePressDigit(e.key);
        } else if (e.key === 'Backspace') {
          handleBackspace();
        } else if (e.key === 'Escape' || e.key.toLowerCase() === 'c') {
          handleClear();
        }
      } else if (step === 'totp') {
        if (e.key === 'Enter') {
          if (otpCode.trim().length >= 6) {
            submitTotp(otpCode.trim());
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, pin, step, otpCode]);

  if (!isOpen) return null;

  const handlePressDigit = (digit: string) => {
    const nextPin = [...pin];
    const emptyIndex = nextPin.findIndex((d) => d === '');
    if (emptyIndex !== -1) {
      nextPin[emptyIndex] = digit;
      setPin(nextPin);
      setError('');
      if (emptyIndex === 5) {
        submitPin(nextPin.join(''));
      }
    }
  };

  const handleBackspace = () => {
    const nextPin = [...pin];
    for (let i = 5; i >= 0; i--) {
      if (nextPin[i] !== '') {
        nextPin[i] = '';
        setPin(nextPin);
        setError('');
        break;
      }
    }
  };

  const handleClear = () => {
    setPin(['', '', '', '', '', '']);
    setError('');
  };

  const submitPin = async (fullPin: string) => {
    setIsSubmitting(true);
    setError('');

    try {
      const res = await api.login(fullPin);
      if (res.requires_totp && res.temp_auth_token) {
        setTempAuthToken(res.temp_auth_token);
        setStep('totp');
        setOtpCode('');
      } else if (res.success) {
        onSuccess();
      } else {
        triggerShake(res.error || t.modal_pin_error);
      }
    } catch (e: any) {
      triggerShake(e.message || t.modal_pin_error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const submitTotp = async (code: string) => {
    if (!code.trim() || !tempAuthToken) return;
    setIsSubmitting(true);
    setError('');

    try {
      const res = await api.loginTotp(tempAuthToken, code.trim());
      if (res.success) {
        onSuccess();
      } else {
        triggerShake(res.error || (isVi ? 'Mã xác thực không hợp lệ' : 'Invalid verification code'));
      }
    } catch (e: any) {
      triggerShake(e.message || (isVi ? 'Lỗi xác thực 2FA' : '2FA authentication error'));
    } finally {
      setIsSubmitting(false);
    }
  };

  const triggerShake = (errMsg: string) => {
    setError(errMsg);
    setIsShaking(true);
    if (step === 'pin') {
      setPin(['', '', '', '', '', '']);
    }
    setTimeout(() => setIsShaking(false), 500);
  };

  return (
    <div style={{
      position: 'fixed',
      inset: 0,
      background: 'rgba(7, 11, 20, 0.85)',
      backdropFilter: 'blur(10px)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 100,
      padding: 20,
    }}>
      <div style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: '36px 32px',
        width: '100%',
        maxWidth: 420,
        textAlign: 'center',
        boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.7)',
        transform: isShaking ? 'translateX(-8px)' : 'none',
        transition: 'transform 0.1s ease',
      }}>
        {step === 'pin' ? (
          <>
            {/* PIN Icon */}
            <div style={{
              width: 64,
              height: 64,
              borderRadius: '50%',
              background: 'rgba(56, 189, 248, 0.12)',
              border: '1px solid rgba(56, 189, 248, 0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto 16px',
              color: '#38bdf8',
              boxShadow: '0 0 25px rgba(56, 189, 248, 0.25)',
            }}>
              <KeyRound size={30} />
            </div>

            <h2 style={{ fontSize: '1.35rem', fontWeight: 700, color: '#f8fafc', marginBottom: 6 }}>
              {t.modal_pin_title}
            </h2>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginBottom: 26 }}>
              {t.modal_pin_sub}
            </p>

            {/* 6 PIN Slots */}
            <div style={{ display: 'flex', justifyContent: 'center', gap: 10, marginBottom: 26 }}>
              {pin.map((digit, idx) => (
                <div
                  key={idx}
                  style={{
                    width: 46,
                    height: 54,
                    borderRadius: 12,
                    background: '#070b14',
                    border: `2px solid ${digit ? '#38bdf8' : '#1e293b'}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '1.6rem',
                    color: '#38bdf8',
                    boxShadow: digit ? '0 0 12px rgba(56, 189, 248, 0.35)' : 'none',
                    transition: 'all 0.15s ease',
                  }}
                >
                  {digit ? '•' : ''}
                </div>
              ))}
            </div>

            {/* Error alert */}
            <div style={{ minHeight: 24, marginBottom: 16 }}>
              {error && (
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 6,
                  color: '#f87171',
                  fontSize: '0.82rem',
                  fontWeight: 600,
                }}>
                  <ShieldAlert size={15} />
                  <span>{error}</span>
                </div>
              )}
              {isSubmitting && (
                <div style={{ color: '#38bdf8', fontSize: '0.85rem' }}>{t.modal_pin_verifying}</div>
              )}
            </div>

            {/* Keypad */}
            <div style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(3, 1fr)',
              gap: 10,
            }}>
              {['1', '2', '3', '4', '5', '6', '7', '8', '9'].map((n) => (
                <button
                  key={n}
                  onClick={() => handlePressDigit(n)}
                  disabled={isSubmitting}
                  style={{
                    background: '#070b14',
                    border: '1px solid #1e293b',
                    color: '#f8fafc',
                    fontSize: '1.35rem',
                    fontWeight: 600,
                    padding: '14px 0',
                    borderRadius: 12,
                    cursor: 'pointer',
                    transition: 'all 0.15s',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = '#1e293b')}
                  onMouseLeave={(e) => (e.currentTarget.style.background = '#070b14')}
                >
                  {n}
                </button>
              ))}
              <button
                onClick={handleClear}
                disabled={isSubmitting}
                style={{
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#94a3b8',
                  fontSize: '1rem',
                  fontWeight: 600,
                  padding: '14px 0',
                  borderRadius: 12,
                  cursor: 'pointer',
                }}
              >
                C
              </button>
              <button
                onClick={() => handlePressDigit('0')}
                disabled={isSubmitting}
                style={{
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#f8fafc',
                  fontSize: '1.35rem',
                  fontWeight: 600,
                  padding: '14px 0',
                  borderRadius: 12,
                  cursor: 'pointer',
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = '#1e293b')}
                onMouseLeave={(e) => (e.currentTarget.style.background = '#070b14')}
              >
                0
              </button>
              <button
                onClick={handleBackspace}
                disabled={isSubmitting}
                style={{
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#94a3b8',
                  fontSize: '1.2rem',
                  fontWeight: 600,
                  padding: '14px 0',
                  borderRadius: 12,
                  cursor: 'pointer',
                }}
              >
                ⌫
              </button>
            </div>
          </>
        ) : (
          /* Step 2: TOTP / Backup Code */
          <div>
            <div style={{
              width: 64,
              height: 64,
              borderRadius: '50%',
              background: 'rgba(52, 211, 153, 0.12)',
              border: '1px solid rgba(52, 211, 153, 0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto 16px',
              color: '#34d399',
              boxShadow: '0 0 25px rgba(52, 211, 153, 0.25)',
            }}>
              <ShieldCheck size={32} />
            </div>

            <h2 style={{ fontSize: '1.3rem', fontWeight: 700, color: '#f8fafc', marginBottom: 6 }}>
              {isVi ? 'Xác Thực 2 Lớp (2FA)' : 'Two-Factor Authentication'}
            </h2>
            <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginBottom: 24, lineHeight: 1.5 }}>
              {isVi
                ? 'Nhập mã 6 chữ số từ ứng dụng Authenticator hoặc một mã dự phòng khẩn cấp.'
                : 'Enter the 6-digit code from your Authenticator app or an emergency backup code.'}
            </p>

            <form
              onSubmit={(e) => {
                e.preventDefault();
                submitTotp(otpCode);
              }}
            >
              <div style={{ marginBottom: 20 }}>
                <input
                  type="text"
                  autoFocus
                  value={otpCode}
                  onChange={(e) => {
                    setOtpCode(e.target.value);
                    setError('');
                  }}
                  placeholder={isVi ? 'Nhập mã 6 số hoặc ABCD-1234' : 'Enter 6-digit or ABCD-1234'}
                  style={{
                    width: '100%',
                    background: '#070b14',
                    border: '2px solid rgba(56, 189, 248, 0.4)',
                    borderRadius: 12,
                    padding: '14px 16px',
                    fontSize: '1.3rem',
                    fontWeight: 700,
                    textAlign: 'center',
                    letterSpacing: '3px',
                    color: '#38bdf8',
                    outline: 'none',
                    boxShadow: '0 0 15px rgba(56, 189, 248, 0.2)',
                  }}
                />
              </div>

              {/* Error display */}
              <div style={{ minHeight: 24, marginBottom: 16 }}>
                {error && (
                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: 6,
                    color: '#f87171',
                    fontSize: '0.82rem',
                    fontWeight: 600,
                  }}>
                    <ShieldAlert size={15} />
                    <span>{error}</span>
                  </div>
                )}
                {isSubmitting && (
                  <div style={{ color: '#38bdf8', fontSize: '0.85rem' }}>
                    {isVi ? 'Đang xác thực...' : 'Verifying...'}
                  </div>
                )}
              </div>

              <div style={{ display: 'flex', gap: 10 }}>
                <button
                  type="button"
                  onClick={() => {
                    setStep('pin');
                    setPin(['', '', '', '', '', '']);
                    setError('');
                  }}
                  style={{
                    flex: 1,
                    display: 'inline-flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: 6,
                    background: 'rgba(255, 255, 255, 0.05)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    borderRadius: 10,
                    padding: '12px',
                    color: '#cbd5e1',
                    fontSize: '0.85rem',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  <ArrowLeft size={16} />
                  {isVi ? 'Quay lại PIN' : 'Back to PIN'}
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting || !otpCode.trim()}
                  style={{
                    flex: 2,
                    background: '#0284c7',
                    border: 'none',
                    borderRadius: 10,
                    padding: '12px',
                    color: '#ffffff',
                    fontSize: '0.9rem',
                    fontWeight: 700,
                    cursor: isSubmitting || !otpCode.trim() ? 'not-allowed' : 'pointer',
                    opacity: isSubmitting || !otpCode.trim() ? 0.6 : 1,
                  }}
                >
                  {isVi ? 'Xác Nhận Đăng Nhập' : 'Verify & Login'}
                </button>
              </div>
            </form>
          </div>
        )}
      </div>
    </div>
  );
};
