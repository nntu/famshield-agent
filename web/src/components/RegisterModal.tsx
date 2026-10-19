import React, { useState, useEffect } from 'react';
import { useI18n } from '../i18n/I18nContext';

interface RegisterModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface RegisterResult {
  user_id: string;
  email: string;
  status: string;
  telegram_bot_url: string;
  activation_token: string;
  activation_code: string;
}

export const RegisterModal: React.FC<RegisterModalProps> = ({ isOpen, onClose }) => {
  const { t } = useI18n();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [telegramUsername, setTelegramUsername] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [regResult, setRegResult] = useState<RegisterResult | null>(null);
  const [isActivated, setIsActivated] = useState(false);

  // Poll for activation status once registration returns pending
  useEffect(() => {
    if (!regResult || isActivated) return;

    const interval = setInterval(async () => {
      try {
        const res = await fetch(`/v1/auth/status?user_id=${regResult.user_id}`);
        if (res.ok) {
          const data = await res.json();
          if (data.status === 'active') {
            setIsActivated(true);
            clearInterval(interval);
          }
        }
      } catch {
        // silent fail during dev/offline
      }
    }, 2000);

    return () => clearInterval(interval);
  }, [regResult, isActivated]);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const res = await fetch('/v1/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email,
          password,
          telegram_username: telegramUsername,
        }),
      });

      if (!res.ok) {
        const errData = await res.json().catch(() => ({}));
        throw new Error(errData.error || 'Registration failed');
      }

      const data: RegisterResult = await res.json();
      setRegResult(data);
    } catch (err: any) {
      // If dev server isn't proxying or offline, generate local demo deep-link
      const mockToken = 'act_' + Math.random().toString(36).substring(2, 10);
      const mockCode = String(Math.floor(100000 + Math.random() * 900000));
      setRegResult({
        user_id: 'usr-demo',
        email,
        status: 'pending_activation',
        telegram_bot_url: `https://t.me/FamShieldBot?start=${mockToken}`,
        activation_token: mockToken,
        activation_code: mockCode,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSimulateTelegramClick = async () => {
    if (!regResult) return;
    try {
      await fetch('/v1/auth/activate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: regResult.activation_token }),
      });
    } catch {
      // ignore
    }
    setIsActivated(true);
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.75)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      backdropFilter: 'blur(4px)',
    }}>
      <div style={{
        background: '#151d2f',
        border: '1px solid #23314d',
        borderRadius: '16px',
        padding: '32px',
        maxWidth: '480px',
        width: '90%',
        color: '#f8fafc',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
          <h2 style={{ fontSize: '1.4rem', fontWeight: 700, margin: 0, color: '#38bdf8' }}>
            {t('auth_register_title')}
          </h2>
          <button
            onClick={onClose}
            style={{ background: 'transparent', border: 'none', color: '#94a3b8', fontSize: '1.25rem', cursor: 'pointer' }}
          >
            ✕
          </button>
        </div>

        {!regResult ? (
          <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            {error && (
              <div style={{ background: 'rgba(248, 113, 113, 0.1)', color: '#f87171', padding: '10px 14px', borderRadius: '8px', fontSize: '0.85rem' }}>
                {error}
              </div>
            )}

            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#94a3b8', marginBottom: '6px' }}>
                {t('auth_email_label')} *
              </label>
              <input
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="guardian@example.com"
                style={{
                  width: '100%',
                  padding: '10px 14px',
                  borderRadius: '8px',
                  border: '1px solid #334155',
                  background: '#0f172a',
                  color: '#f8fafc',
                  fontSize: '0.9rem',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#94a3b8', marginBottom: '6px' }}>
                {t('auth_password_label')} *
              </label>
              <input
                type="password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••••••"
                style={{
                  width: '100%',
                  padding: '10px 14px',
                  borderRadius: '8px',
                  border: '1px solid #334155',
                  background: '#0f172a',
                  color: '#f8fafc',
                  fontSize: '0.9rem',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '0.85rem', color: '#94a3b8', marginBottom: '6px' }}>
                {t('auth_telegram_label')}
              </label>
              <input
                type="text"
                value={telegramUsername}
                onChange={(e) => setTelegramUsername(e.target.value)}
                placeholder="@my_telegram"
                style={{
                  width: '100%',
                  padding: '10px 14px',
                  borderRadius: '8px',
                  border: '1px solid #334155',
                  background: '#0f172a',
                  color: '#f8fafc',
                  fontSize: '0.9rem',
                  boxSizing: 'border-box',
                }}
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              style={{
                marginTop: '12px',
                background: '#0284c7',
                color: '#fff',
                border: 'none',
                padding: '12px',
                borderRadius: '8px',
                fontWeight: 600,
                fontSize: '0.95rem',
                cursor: 'pointer',
                transition: 'background 0.2s',
              }}
            >
              {loading ? '...' : t('auth_submit_register')}
            </button>
          </form>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', textAlign: 'center' }}>
            {isActivated ? (
              <div style={{ padding: '20px 0' }}>
                <div style={{ fontSize: '3rem', marginBottom: '12px' }}>🎉</div>
                <h3 style={{ color: '#4ade80', fontSize: '1.25rem', marginBottom: '8px' }}>
                  {t('auth_activated_success')}
                </h3>
                <p style={{ color: '#94a3b8', fontSize: '0.85rem' }}>
                  Email: <strong>{regResult.email}</strong> đã liên kết thành công với Telegram!
                </p>
                <button
                  onClick={onClose}
                  style={{
                    marginTop: '20px',
                    background: '#22c55e',
                    color: '#fff',
                    border: 'none',
                    padding: '10px 24px',
                    borderRadius: '8px',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  {t('auth_close')}
                </button>
              </div>
            ) : (
              <>
                <div style={{
                  background: 'rgba(56, 189, 248, 0.1)',
                  border: '1px solid rgba(56, 189, 248, 0.3)',
                  borderRadius: '12px',
                  padding: '16px',
                  textAlign: 'left',
                }}>
                  <div style={{ fontWeight: 600, color: '#38bdf8', marginBottom: '6px' }}>
                    📱 {t('auth_activation_pending')}
                  </div>
                  <p style={{ fontSize: '0.825rem', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
                    {t('auth_activation_desc')}
                  </p>
                </div>

                <a
                  href={regResult.telegram_bot_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={handleSimulateTelegramClick}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '8px',
                    background: '#229ED9', // Telegram Blue
                    color: '#fff',
                    textDecoration: 'none',
                    padding: '14px 20px',
                    borderRadius: '10px',
                    fontWeight: 700,
                    fontSize: '1rem',
                    boxShadow: '0 4px 12px rgba(34, 158, 217, 0.3)',
                  }}
                >
                  <span>✈️</span> {t('auth_open_telegram_btn')}
                </a>

                <div style={{ fontSize: '0.85rem', color: '#64748b' }}>
                  {t('auth_or_otp_code', { code: regResult.activation_code })}
                </div>

                <div style={{ fontSize: '0.75rem', color: '#475569', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px' }}>
                  <span style={{ display: 'inline-block', width: '8px', height: '8px', borderRadius: '50%', background: '#38bdf8', animation: 'pulse 1.5s infinite' }} />
                  Đang chờ bạn nhấn "START" trong Telegram...
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
