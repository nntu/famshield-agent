import type React from 'react';
import { useState } from 'react';
import { api } from '../api';
import type { InitializeSetupResponse, Language } from '../types';
import { FinishStep } from './wizard/FinishStep';
import { ModeStep } from './wizard/ModeStep';
import { PinStep } from './wizard/PinStep';
import { ProfileStep } from './wizard/ProfileStep';
import { WelcomeStep } from './wizard/WelcomeStep';

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

  const [operatingMode, setOperatingMode] = useState<'StandaloneFree' | 'OnlineManaged'>(
    'StandaloneFree'
  );
  const [enrollmentCode, setEnrollmentCode] = useState<string>('');
  const [serverUrl, setServerUrl] = useState<string>('https://famshield.ttcantho.io.vn');

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
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(
        msg || (isVi ? 'Khởi tạo thất bại, vui lòng thử lại' : 'Initialization failed')
      );
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

  return (
    <div
      className="min-h-[100dvh] p-4 sm:p-6"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'radial-gradient(circle at 50% 10%, #e0f2fe 0%, #f8fafc 70%)',
      }}
    >
      <div
        className="p-5 sm:p-9"
        style={{
          maxWidth: 720,
          width: '100%',
          background: '#ffffff',
          border: '1px solid #e2e8f0',
          borderRadius: 24,
          boxShadow: '0 20px 50px -15px rgba(0, 0, 0, 0.08), 0 0 25px rgba(56, 189, 248, 0.08)',
          color: '#0f172a',
        }}
      >
        {/* Progress Tracker */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: 32,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <span style={{ fontSize: '1.8rem' }}>🛡️</span>
            <div>
              <div
                style={{
                  fontSize: '1.25rem',
                  fontWeight: 800,
                  background: 'linear-gradient(135deg, var(--accent), #38bdf8)',
                  WebkitBackgroundClip: 'text',
                  WebkitTextFillColor: 'transparent',
                }}
              >
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
          <div
            role="alert"
            style={{
              background: '#fef2f2',
              border: '1px solid #fecaca',
              color: '#b91c1c',
              padding: '12px 16px',
              borderRadius: 12,
              marginBottom: 20,
              fontSize: '0.88rem',
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
            <span>⚠️</span>
            <span>{errorMessage}</span>
          </div>
        )}

        {/* STEP 1: WELCOME */}
        {step === 1 && <WelcomeStep onNext={() => setStep(2)} isVi={isVi} />}

        {/* STEP 2: MASTER PIN */}
        {step === 2 && (
          <PinStep
            pin={pin}
            confirmPin={confirmPin}
            pinError={pinError}
            onPinChange={setPin}
            onConfirmPinChange={setConfirmPin}
            onBack={() => setStep(1)}
            onNext={handleNextFromPin}
            isVi={isVi}
          />
        )}

        {/* STEP 3: PRESET PROTECTION PROFILE */}
        {step === 3 && (
          <ProfileStep
            presetProfile={presetProfile}
            onSelectProfile={setPresetProfile}
            onBack={() => setStep(2)}
            onNext={() => setStep(4)}
            isVi={isVi}
          />
        )}

        {/* STEP 4: OPERATING MODE */}
        {step === 4 && (
          <ModeStep
            operatingMode={operatingMode}
            serverUrl={serverUrl}
            enrollmentCode={enrollmentCode}
            loading={loading}
            onSelectMode={setOperatingMode}
            onServerUrlChange={setServerUrl}
            onEnrollmentCodeChange={setEnrollmentCode}
            onBack={() => setStep(3)}
            onSubmit={handleSubmitSetup}
            isVi={isVi}
          />
        )}

        {/* STEP 5: SUCCESS & RECOVERY KEY */}
        {step === 5 && setupResult && (
          <FinishStep
            setupResult={setupResult}
            copiedKey={copiedKey}
            onCopyRecoveryKey={copyRecoveryKey}
            onComplete={onComplete}
            isVi={isVi}
          />
        )}
      </div>
    </div>
  );
};
