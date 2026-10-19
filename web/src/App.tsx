import React, { useState } from 'react';
import { Device, ChildProfile, VisualEvidence } from './types';
import { useI18n } from './i18n/I18nContext';
import { LanguageSwitcher } from './components/LanguageSwitcher';
import { RegisterModal } from './components/RegisterModal';

export const App: React.FC = () => {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<'overview' | 'children' | 'devices' | 'evidence'>('overview');
  const [isRegisterOpen, setIsRegisterOpen] = useState(false);


  const [devices] = useState<Device[]>([
    {
      id: 'dev-001',
      device_name: 'PC Phòng Trẻ (Windows 11)',
      os_version: 'Windows 11 Pro 23H2',
      status: 'online',
      policy_version: 1,
      last_seen: 'Vừa xong',
    },
    {
      id: 'dev-002',
      device_name: 'Laptop Học Tập',
      os_version: 'Windows 10 Home',
      status: 'online',
      policy_version: 1,
      last_seen: '2 phút trước',
    },
  ]);

  const [children] = useState<ChildProfile[]>([
    {
      id: 'child-1',
      name: 'Minh (10 tuổi)',
      daily_quota_minutes: 60,
      used_minutes_today: 35,
      assigned_devices: ['dev-001'],
    },
  ]);

  const [evidences] = useState<VisualEvidence[]>([
    {
      id: 'evi-991',
      event_id: 'evt-block-game',
      device_id: 'dev-001',
      trigger: 'Chặn game đêm ngoài giờ',
      occurred_at: '17:15 - Hôm nay',
      expires_in_hours: 47,
    },
  ]);

  return (
    <div style={{ display: 'flex', minHeight: '100vh', fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif' }}>
      {/* Sidebar Navigation */}
      <aside style={{ width: '270px', background: '#111827', borderRight: '1px solid #1f2937', padding: '24px', display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
        <div>
          <div style={{ marginBottom: '2rem' }}>
            <h2 style={{ color: '#38bdf8', fontSize: '1.35rem', fontWeight: 700, margin: 0 }}>{t('brand')}</h2>
            <div style={{ fontSize: '0.75rem', color: '#64748b', marginTop: '4px' }}>{t('brand_tagline')}</div>
          </div>
          <nav style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {(['overview', 'children', 'devices', 'evidence'] as const).map((tab) => {
              const tabLabels = {
                overview: t('nav_overview'),
                children: t('nav_children'),
                devices: t('nav_devices'),
                evidence: t('nav_evidence'),
              };
              return (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  style={{
                    textAlign: 'left',
                    padding: '12px 16px',
                    borderRadius: '8px',
                    border: 'none',
                    cursor: 'pointer',
                    background: activeTab === tab ? '#1e293b' : 'transparent',
                    color: activeTab === tab ? '#38bdf8' : '#94a3b8',
                    fontWeight: activeTab === tab ? 600 : 400,
                    fontSize: '0.9rem',
                    transition: 'all 0.15s ease',
                  }}
                >
                  {tabLabels[tab]}
                </button>
              );
            })}
          </nav>
        </div>

        {/* Language selector in sidebar footer */}
        <div style={{ borderTop: '1px solid #1f2937', paddingTop: '16px' }}>
          <LanguageSwitcher />
        </div>
      </aside>

      {/* Main Content Area */}
      <main style={{ flex: 1, padding: '36px', background: '#0b0f19', color: '#f8fafc' }}>
        {/* Top bar with quick status */}
        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '32px', borderBottom: '1px solid #1e293b', paddingBottom: '16px' }}>
          <div style={{ fontSize: '0.875rem', color: '#94a3b8' }}>
            {t('footer_status')}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <button
              onClick={() => setIsRegisterOpen(true)}
              style={{
                background: '#0284c7',
                color: '#fff',
                border: 'none',
                padding: '6px 14px',
                borderRadius: '8px',
                cursor: 'pointer',
                fontSize: '0.85rem',
                fontWeight: 600,
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
              }}
            >
              <span>✈️</span> {t('auth_register_btn')}
            </button>
            <span style={{ fontSize: '0.8rem', background: 'rgba(74, 222, 128, 0.15)', color: '#4ade80', padding: '4px 12px', borderRadius: '9999px', fontWeight: 600 }}>
              {t('devices_status_online')}
            </span>
          </div>
        </header>

        <RegisterModal isOpen={isRegisterOpen} onClose={() => setIsRegisterOpen(false)} />


        {activeTab === 'overview' && (
          <div>
            <h1 style={{ fontSize: '1.75rem', fontWeight: 700, marginBottom: '8px' }}>{t('overview_title')}</h1>
            <p style={{ color: '#94a3b8', marginBottom: '28px', fontSize: '0.95rem' }}>{t('overview_subtitle')}</p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '20px' }}>
              <div style={{ background: '#151d2f', padding: '24px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>{t('stat_online_devices')}</div>
                <div style={{ fontSize: '2.25rem', fontWeight: 700, color: '#4ade80', marginTop: '8px' }}>
                  {devices.filter((d) => d.status === 'online').length} / {devices.length}
                </div>
              </div>
              <div style={{ background: '#151d2f', padding: '24px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>{t('stat_blocked_dns')}</div>
                <div style={{ fontSize: '2.25rem', fontWeight: 700, color: '#38bdf8', marginTop: '8px' }}>42</div>
              </div>
              <div style={{ background: '#151d2f', padding: '24px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>{t('stat_pending_evidence')}</div>
                <div style={{ fontSize: '2.25rem', fontWeight: 700, color: '#f87171', marginTop: '8px' }}>1</div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'children' && (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h1 style={{ fontSize: '1.75rem', fontWeight: 700, margin: 0 }}>{t('children_title')}</h1>
              <button style={{ background: '#0284c7', color: '#fff', border: 'none', padding: '10px 18px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600 }}>
                + {t('children_add_profile')}
              </button>
            </div>
            <div style={{ display: 'grid', gap: '16px' }}>
              {children.map((c) => (
                <div key={c.id} style={{ background: '#151d2f', padding: '24px', borderRadius: '12px', border: '1px solid #23314d' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <h3 style={{ fontSize: '1.2rem', margin: 0 }}>{c.name}</h3>
                    <span style={{ color: '#4ade80', fontSize: '0.875rem', fontWeight: 600, background: 'rgba(74, 222, 128, 0.1)', padding: '4px 10px', borderRadius: '6px' }}>
                      {t('children_status_protected')}
                    </span>
                  </div>
                  <div style={{ marginTop: '16px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.875rem', color: '#94a3b8' }}>
                      <span>{t('children_used_today', { used: c.used_minutes_today })}</span>
                      <span>{t('children_quota', { quota: c.daily_quota_minutes })}</span>
                    </div>
                    <div style={{ height: '8px', background: '#334155', borderRadius: '4px', overflow: 'hidden', marginTop: '8px' }}>
                      <div style={{ height: '100%', width: `${(c.used_minutes_today / c.daily_quota_minutes) * 100}%`, background: '#38bdf8' }} />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'devices' && (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h1 style={{ fontSize: '1.75rem', fontWeight: 700, margin: 0 }}>{t('devices_title')}</h1>
              <button style={{ background: '#0284c7', color: '#fff', border: 'none', padding: '10px 18px', borderRadius: '8px', cursor: 'pointer', fontWeight: 600 }}>
                + {t('devices_enroll_button')}
              </button>
            </div>
            <div style={{ background: '#151d2f', borderRadius: '12px', border: '1px solid #23314d', overflow: 'hidden' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                <thead style={{ background: '#1e293b' }}>
                  <tr>
                    <th style={{ padding: '16px' }}>{t('devices_th_name')}</th>
                    <th style={{ padding: '16px' }}>{t('devices_th_os')}</th>
                    <th style={{ padding: '16px' }}>{t('devices_th_policy')}</th>
                    <th style={{ padding: '16px' }}>{t('devices_th_status')}</th>
                  </tr>
                </thead>
                <tbody>
                  {devices.map((d) => (
                    <tr key={d.id} style={{ borderTop: '1px solid #23314d' }}>
                      <td style={{ padding: '16px', fontWeight: 500 }}>{d.device_name}</td>
                      <td style={{ padding: '16px', color: '#94a3b8' }}>{d.os_version}</td>
                      <td style={{ padding: '16px' }}>{t('devices_synced', { version: d.policy_version })}</td>
                      <td style={{ padding: '16px', color: d.status === 'online' ? '#4ade80' : '#f87171' }}>
                        {d.status === 'online' ? t('devices_status_online') : t('devices_status_offline')}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'evidence' && (
          <div>
            <h1 style={{ fontSize: '1.75rem', fontWeight: 700, marginBottom: '8px' }}>{t('evidence_title')}</h1>
            <p style={{ color: '#94a3b8', marginBottom: '24px', fontSize: '0.9rem', lineHeight: '1.5' }}>
              {t('evidence_desc')}
            </p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '20px' }}>
              {evidences.map((e) => (
                <div key={e.id} style={{ background: '#151d2f', borderRadius: '12px', border: '1px solid #23314d', overflow: 'hidden', padding: '16px' }}>
                  <div style={{ width: '100%', height: '160px', background: '#0b0f19', borderRadius: '8px', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#64748b', filter: 'blur(2px)', fontSize: '0.85rem' }}>
                    {t('evidence_thumb_placeholder')}
                  </div>
                  <div style={{ marginTop: '14px' }}>
                    <div style={{ fontWeight: 600 }}>{e.trigger}</div>
                    <div style={{ fontSize: '0.875rem', color: '#94a3b8', marginTop: '6px' }}>
                      {t('evidence_timestamp', { time: e.occurred_at })}
                    </div>
                    <div style={{ fontSize: '0.875rem', color: '#f87171', marginTop: '4px' }}>
                      {t('evidence_expires', { hours: e.expires_in_hours })}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>
    </div>
  );
};
