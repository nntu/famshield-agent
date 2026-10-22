import { ArrowLeft, KeyRound, ShieldAlert, ShieldCheck } from 'lucide-react';
import type React from 'react';
import { useEffect, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { Language } from '../types';

interface PinModalProps {
  isOpen: boolean;
  lang: Language;
  onSuccess: () => void;
  appVersion?: string;
}

export const PinModal: React.FC<PinModalProps> = ({ isOpen, lang, onSuccess, appVersion }) => {
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
  const dialogRef = useRef<HTMLDivElement>(null);

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

  useEffect(() => {
    if (!isOpen) return;

    const previouslyFocused = document.activeElement as HTMLElement | null;
    const dialog = dialogRef.current;
    const focusableSelector =
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
    const focusFirst = () => dialog?.querySelector<HTMLElement>(focusableSelector)?.focus();
    const frame = window.requestAnimationFrame(focusFirst);

    const trapFocus = (event: KeyboardEvent) => {
      if (event.key !== 'Tab' || !dialog) return;
      const focusable = Array.from(dialog.querySelectorAll<HTMLElement>(focusableSelector));
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    document.addEventListener('keydown', trapFocus);
    return () => {
      window.cancelAnimationFrame(frame);
      document.removeEventListener('keydown', trapFocus);
      previouslyFocused?.focus();
    };
  }, [isOpen, step]);

  if (!isOpen) return null;

  const handlePressDigit = (digit: string) => {
    const nextPin = [...pin];
    const emptyIndex = nextPin.indexOf('');
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
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      triggerShake(msg || t.modal_pin_error);
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
        triggerShake(
          res.error || (isVi ? 'Mã xác thực không hợp lệ' : 'Invalid verification code')
        );
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      triggerShake(msg || (isVi ? 'Lỗi xác thực 2FA' : '2FA authentication error'));
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
    <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-200">
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="pin-modal-title"
        aria-describedby="pin-modal-description"
        className={`bg-white border border-slate-200 rounded-3xl p-8 sm:p-9 w-full max-w-md text-center shadow-2xl transition-transform duration-100 ${
          isShaking ? '-translate-x-2' : ''
        }`}
      >
        {step === 'pin' ? (
          <>
            {/* PIN Icon */}
            <div className="w-16 h-16 rounded-full bg-sky-50 border border-sky-100 flex items-center justify-center mx-auto mb-4 text-sky-600 shadow-inner">
              <KeyRound size={28} />
            </div>

            <h2 id="pin-modal-title" className="text-xl font-bold text-slate-900 mb-1">
              {t.modal_pin_title}
            </h2>
            <p id="pin-modal-description" className="text-sm text-slate-500 mb-6 font-medium">
              {t.modal_pin_sub}
            </p>

            {/* 6 PIN Slots */}
            <div
              className="flex justify-center gap-2.5 mb-6"
              role="status"
              aria-label={
                isVi
                  ? `Đã nhập ${pin.filter(Boolean).length} trên 6 chữ số`
                  : `${pin.filter(Boolean).length} of 6 digits entered`
              }
            >
              {pin.map((digit, idx) => (
                <div
                  key={`pin-slot-${idx}`}
                  className={`w-11 h-13 rounded-xl flex items-center justify-center text-2xl font-bold transition-all ${
                    digit
                      ? 'bg-sky-50 border-2 border-sky-500 text-sky-600 shadow-xs'
                      : 'bg-slate-50 border-2 border-slate-200 text-slate-400'
                  }`}
                >
                  {digit ? '•' : ''}
                </div>
              ))}
            </div>

            {/* Error alert */}
            <div className="min-h-[24px] mb-4">
              {error && (
                <div
                  role="alert"
                  className="flex items-center justify-center gap-1.5 text-rose-700 text-xs font-semibold"
                >
                  <ShieldAlert size={15} />
                  <span>{error}</span>
                </div>
              )}
              {isSubmitting && (
                <div className="text-sky-600 text-xs font-semibold">{t.modal_pin_verifying}</div>
              )}
            </div>

            {/* Keypad */}
            <div className="grid grid-cols-3 gap-2.5 max-w-[300px] mx-auto">
              {['1', '2', '3', '4', '5', '6', '7', '8', '9'].map((n) => (
                <button
                  key={n}
                  type="button"
                  aria-label={isVi ? `Số ${n}` : `Digit ${n}`}
                  onClick={() => handlePressDigit(n)}
                  disabled={isSubmitting}
                  className="bg-slate-100 hover:bg-slate-200 text-slate-800 border border-slate-200/80 text-xl font-semibold py-3 rounded-xl cursor-pointer active:scale-95 transition-all shadow-2xs"
                >
                  {n}
                </button>
              ))}
              <button
                type="button"
                aria-label={isVi ? 'Xóa toàn bộ mã PIN' : 'Clear PIN'}
                onClick={handleClear}
                disabled={isSubmitting}
                className="bg-slate-100 hover:bg-slate-200 text-slate-500 border border-slate-200/80 text-base font-semibold py-3 rounded-xl cursor-pointer active:scale-95 transition-all"
              >
                C
              </button>
              <button
                type="button"
                aria-label={isVi ? 'Số 0' : 'Digit 0'}
                onClick={() => handlePressDigit('0')}
                disabled={isSubmitting}
                className="bg-slate-100 hover:bg-slate-200 text-slate-800 border border-slate-200/80 text-xl font-semibold py-3 rounded-xl cursor-pointer active:scale-95 transition-all shadow-2xs"
              >
                0
              </button>
              <button
                type="button"
                aria-label={isVi ? 'Xóa chữ số cuối' : 'Delete last digit'}
                onClick={handleBackspace}
                disabled={isSubmitting}
                className="bg-slate-100 hover:bg-slate-200 text-slate-500 border border-slate-200/80 text-lg font-semibold py-3 rounded-xl cursor-pointer active:scale-95 transition-all"
              >
                ⌫
              </button>
            </div>
          </>
        ) : (
          /* Step 2: TOTP / Backup Code */
          <div>
            <div className="w-16 h-16 rounded-full bg-emerald-50 border border-emerald-100 flex items-center justify-center mx-auto mb-4 text-emerald-600 shadow-inner">
              <ShieldCheck size={32} />
            </div>

            <h2 id="pin-modal-title" className="text-xl font-bold text-slate-900 mb-1">
              {isVi ? 'Xác Thực 2 Lớp (2FA)' : 'Two-Factor Authentication'}
            </h2>
            <p
              id="pin-modal-description"
              className="text-sm text-slate-500 mb-6 font-medium leading-relaxed"
            >
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
              <div className="mb-5">
                <label htmlFor="totp-code" className="sr-only">
                  {isVi ? 'Mã xác thực hoặc mã dự phòng' : 'Verification or backup code'}
                </label>
                <input
                  id="totp-code"
                  type="text"
                  autoFocus
                  value={otpCode}
                  onChange={(e) => {
                    setOtpCode(e.target.value);
                    setError('');
                  }}
                  placeholder={isVi ? 'Nhập mã 6 số hoặc ABCD-1234' : 'Enter 6-digit or ABCD-1234'}
                  className="w-full bg-slate-50 border-2 border-sky-400 rounded-xl px-4 py-3.5 text-xl font-bold text-center tracking-widest text-sky-700 outline-none focus:ring-2 focus:ring-sky-200 shadow-inner"
                />
              </div>

              {/* Error display */}
              <div className="min-h-[24px] mb-4">
                {error && (
                  <div
                    role="alert"
                    className="flex items-center justify-center gap-1.5 text-rose-700 text-xs font-semibold"
                  >
                    <ShieldAlert size={15} />
                    <span>{error}</span>
                  </div>
                )}
                {isSubmitting && (
                  <div className="text-sky-600 text-xs font-semibold">
                    {isVi ? 'Đang xác thực...' : 'Verifying...'}
                  </div>
                )}
              </div>

              <div className="flex gap-2.5">
                <button
                  type="button"
                  onClick={() => {
                    setStep('pin');
                    setPin(['', '', '', '', '', '']);
                    setError('');
                  }}
                  className="flex-1 inline-flex items-center justify-center gap-1.5 bg-slate-100 hover:bg-slate-200 border border-slate-200 rounded-xl p-3 text-slate-700 text-sm font-semibold cursor-pointer transition-colors"
                >
                  <ArrowLeft size={16} />
                  {isVi ? 'Quay lại PIN' : 'Back to PIN'}
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting || !otpCode.trim()}
                  className="flex-2 bg-sky-600 hover:bg-sky-700 disabled:bg-sky-300 rounded-xl p-3 text-white text-sm font-semibold cursor-pointer disabled:cursor-not-allowed transition-colors shadow-xs"
                >
                  {isVi ? 'Xác Nhận Đăng Nhập' : 'Verify & Login'}
                </button>
              </div>
            </form>
          </div>
        )}

        {/* Footer version indicator */}
        <div className="mt-5 pt-3 border-t border-slate-100 flex items-center justify-between text-2xs text-slate-400 font-medium">
          <span>FamShield Windows Agent</span>
          <span className="font-mono text-slate-500 bg-slate-100 px-2 py-0.5 rounded border border-slate-200/60 font-semibold">
            v{appVersion || '1.0.2'}
          </span>
        </div>
      </div>
    </div>
  );
};
