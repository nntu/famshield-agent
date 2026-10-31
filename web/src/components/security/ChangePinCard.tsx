import type React from 'react';
import { useState } from 'react';
import { Lock, Key } from 'lucide-react';
import { api } from '../../api';
import { translations } from '../../i18n';
import type { Language } from '../../types';

interface ChangePinCardProps {
  lang: Language;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const ChangePinCard: React.FC<ChangePinCardProps> = ({
  lang,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [currPin, setCurrPin] = useState('');
  const [newPin, setNewPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [isChangingPin, setIsChangingPin] = useState(false);
  const [generatedRecoveryKey, setGeneratedRecoveryKey] = useState<string | null>(null);

  const handleChangePin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPin.length < 6 || !/^\d+$/.test(newPin)) {
      onShowToast(
        isVi ? 'Mã PIN mới phải gồm ít nhất 6 chữ số' : 'New PIN must be at least 6 digits',
        'error'
      );
      return;
    }
    if (newPin !== confirmPin) {
      onShowToast(isVi ? 'Mã PIN xác nhận không khớp' : 'PIN confirmation does not match', 'error');
      return;
    }

    setIsChangingPin(true);
    try {
      const res = await api.changePin({
        current_pin: currPin,
        new_pin: newPin,
      });

      if (res.success) {
        onShowToast(t.pin_success, 'success');
        setCurrPin('');
        setNewPin('');
        setConfirmPin('');
        if (res.recovery_key) {
          setGeneratedRecoveryKey(res.recovery_key);
        }
      }
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi đổi PIN' : 'Failed to change PIN'), 'error');
    } finally {
      setIsChangingPin(false);
    }
  };

  return (
    <div
      id="section-master-pin"
      style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: 28,
        boxShadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
      }}
    >
      <div style={{ marginBottom: 20 }}>
        <h2
          style={{
            fontSize: '1.25rem',
            fontWeight: 700,
            color: '#f8fafc',
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}
        >
          <Lock size={22} color="#38bdf8" />
          {t.title_change_pin}
        </h2>
        <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>{t.desc_change_pin}</p>
      </div>

      <form
        onSubmit={handleChangePin}
        style={{ display: 'flex', flexDirection: 'column', gap: 14, maxWidth: 500 }}
      >
        <div>
          <label
            style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
          >
            {t.lbl_curr_pin}
          </label>
          <input
            type="password"
            maxLength={8}
            value={currPin}
            onChange={(e) => setCurrPin(e.target.value)}
            style={{
              width: '100%',
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#fff',
              padding: '10px 14px',
              borderRadius: 8,
              fontSize: '0.9rem',
              outline: 'none',
            }}
          />
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 14 }}>
          <div>
            <label
              style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
            >
              {t.lbl_new_pin}
            </label>
            <input
              type="password"
              maxLength={8}
              value={newPin}
              onChange={(e) => setNewPin(e.target.value)}
              style={{
                width: '100%',
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '10px 14px',
                borderRadius: 8,
                fontSize: '0.9rem',
                outline: 'none',
              }}
            />
          </div>

          <div>
            <label
              style={{ display: 'block', fontSize: '0.82rem', color: '#94a3b8', marginBottom: 6 }}
            >
              {t.lbl_confirm_pin}
            </label>
            <input
              type="password"
              maxLength={8}
              value={confirmPin}
              onChange={(e) => setConfirmPin(e.target.value)}
              style={{
                width: '100%',
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '10px 14px',
                borderRadius: 8,
                fontSize: '0.9rem',
                outline: 'none',
              }}
            />
          </div>
        </div>

        <button
          type="submit"
          disabled={isChangingPin}
          style={{
            marginTop: 6,
            background: '#38bdf8',
            color: '#070b14',
            border: 'none',
            padding: '11px',
            borderRadius: 8,
            fontSize: '0.9rem',
            fontWeight: 700,
            cursor: isChangingPin ? 'not-allowed' : 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 8,
          }}
        >
          <Key size={16} />
          <span>{t.btn_save_pin}</span>
        </button>
      </form>

      {/* Generated Recovery Key Display */}
      {generatedRecoveryKey && (
        <div
          style={{
            marginTop: 20,
            background: '#070b14',
            border: '1px dashed #34d399',
            borderRadius: 12,
            padding: 16,
            textAlign: 'center',
            maxWidth: 500,
          }}
        >
          <div
            style={{ fontSize: '0.82rem', color: '#34d399', fontWeight: 600, marginBottom: 6 }}
          >
            {t.pin_recovery_notice}
          </div>
          <div
            style={{
              fontSize: '1.2rem',
              fontWeight: 800,
              fontFamily: 'monospace',
              letterSpacing: '0.15em',
              color: '#38bdf8',
              userSelect: 'all',
            }}
          >
            {generatedRecoveryKey}
          </div>
        </div>
      )}
    </div>
  );
};
