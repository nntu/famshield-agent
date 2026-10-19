import React, { useState, useEffect } from 'react';
import { Lock, ShieldAlert, KeyRound } from 'lucide-react';
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
  const [pin, setPin] = useState<string[]>(['', '', '', '', '', '']);
  const [error, setError] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [isShaking, setIsShaking] = useState<boolean>(false);

  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key >= '0' && e.key <= '9') {
        handlePressDigit(e.key);
      } else if (e.key === 'Backspace') {
        handleBackspace();
      } else if (e.key === 'Escape' || e.key.toLowerCase() === 'c') {
        handleClear();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, pin]);

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
      if (res.success) {
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

  const triggerShake = (errMsg: string) => {
    setError(errMsg);
    setIsShaking(true);
    setPin(['', '', '', '', '', '']);
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
        maxWidth: 400,
        textAlign: 'center',
        boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.7)',
        transform: isShaking ? 'translateX(-8px)' : 'none',
        transition: 'transform 0.1s ease',
      }}>
        {/* Icon */}
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
      </div>
    </div>
  );
};
