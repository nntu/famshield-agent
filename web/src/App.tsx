import React, { useState } from 'react';
import { Device, ChildProfile, VisualEvidence } from './types';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'overview' | 'children' | 'devices' | 'evidence'>('overview');

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
    <div style={{ display: 'flex', minHeight: '100vh' }}>
      {/* Sidebar Navigation */}
      <aside style={{ width: '260px', background: '#111827', borderRight: '1px solid #1f2937', padding: '24px' }}>
        <h2 style={{ color: '#38bdf8', fontSize: '1.25rem', marginBottom: '2rem' }}>FamShield Cloud</h2>
        <nav style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {(['overview', 'children', 'devices', 'evidence'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                textAlign: 'left',
                padding: '10px 16px',
                borderRadius: '8px',
                border: 'none',
                cursor: 'pointer',
                background: activeTab === tab ? '#1e293b' : 'transparent',
                color: activeTab === tab ? '#38bdf8' : '#94a3b8',
                fontWeight: activeTab === tab ? 600 : 400,
                textTransform: 'capitalize',
              }}
            >
              {tab === 'overview' && 'Tổng quan'}
              {tab === 'children' && 'Hồ sơ trẻ & Giới hạn'}
              {tab === 'devices' && 'Thiết bị bảo vệ'}
              {tab === 'evidence' && 'Bằng chứng thông minh (48h)'}
            </button>
          ))}
        </nav>
      </aside>

      {/* Main Content Area */}
      <main style={{ flex: 1, padding: '32px', background: '#0b0f19' }}>
        {activeTab === 'overview' && (
          <div>
            <h1 style={{ fontSize: '1.75rem', marginBottom: '24px' }}>Tổng quan Hệ thống Quản trị Tập trung</h1>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '20px' }}>
              <div style={{ background: '#151d2f', padding: '20px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>Thiết bị trực tuyến</div>
                <div style={{ fontSize: '2rem', fontWeight: 700, color: '#4ade80', marginTop: '8px' }}>
                  {devices.filter((d) => d.status === 'online').length} / {devices.length}
                </div>
              </div>
              <div style={{ background: '#151d2f', padding: '20px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>Tên miền DNS đã chặn hôm nay</div>
                <div style={{ fontSize: '2rem', fontWeight: 700, color: '#38bdf8', marginTop: '8px' }}>42</div>
              </div>
              <div style={{ background: '#151d2f', padding: '20px', borderRadius: '12px', border: '1px solid #23314d' }}>
                <div style={{ color: '#94a3b8', fontSize: '0.875rem' }}>Bằng chứng xác minh cần duyệt</div>
                <div style={{ fontSize: '2rem', fontWeight: 700, color: '#f87171', marginTop: '8px' }}>1</div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'children' && (
          <div>
            <h1 style={{ fontSize: '1.75rem', marginBottom: '24px' }}>Quản lý Hồ sơ & Giới hạn Thời gian</h1>
            <div style={{ display: 'grid', gap: '16px' }}>
              {children.map((c) => (
                <div key={c.id} style={{ background: '#151d2f', padding: '24px', borderRadius: '12px', border: '1px solid #23314d' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <h3>{c.name}</h3>
                    <span style={{ color: '#4ade80' }}>Đang bảo vệ</span>
                  </div>
                  <div style={{ marginTop: '16px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.875rem', color: '#94a3b8' }}>
                      <span>Thời gian dùng hôm nay: {c.used_minutes_today} phút</span>
                      <span>Hạn ngạch: {c.daily_quota_minutes} phút</span>
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
            <h1 style={{ fontSize: '1.75rem', marginBottom: '24px' }}>Danh sách Thiết bị Endpoint Bảo vệ</h1>
            <div style={{ background: '#151d2f', borderRadius: '12px', border: '1px solid #23314d', overflow: 'hidden' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                <thead style={{ background: '#1e293b' }}>
                  <tr>
                    <th style={{ padding: '16px' }}>Tên thiết bị</th>
                    <th style={{ padding: '16px' }}>Hệ điều hành</th>
                    <th style={{ padding: '16px' }}>Phiên bản Policy</th>
                    <th style={{ padding: '16px' }}>Trạng thái</th>
                  </tr>
                </thead>
                <tbody>
                  {devices.map((d) => (
                    <tr key={d.id} style={{ borderTop: '1px solid #23314d' }}>
                      <td style={{ padding: '16px' }}>{d.device_name}</td>
                      <td style={{ padding: '16px', color: '#94a3b8' }}>{d.os_version}</td>
                      <td style={{ padding: '16px' }}>v{d.policy_version} (Đã đồng bộ)</td>
                      <td style={{ padding: '16px', color: '#4ade80' }}>● Trực tuyến</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'evidence' && (
          <div>
            <h1 style={{ fontSize: '1.75rem', marginBottom: '8px' }}>Thư viện Bằng chứng Xác minh Thông minh</h1>
            <p style={{ color: '#94a3b8', marginBottom: '24px' }}>
              Hình ảnh được tự động làm mờ tại thiết bị để bảo vệ riêng tư. Tuân thủ tuyệt đối quy định COPPA/NĐ 13/2023: Tự hủy sau 48 giờ.
            </p>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '20px' }}>
              {evidences.map((e) => (
                <div key={e.id} style={{ background: '#151d2f', borderRadius: '12px', border: '1px solid #23314d', overflow: 'hidden', padding: '16px' }}>
                  <div style={{ width: '100%', height: '160px', background: '#0b0f19', borderRadius: '8px', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#64748b', filter: 'blur(2px)' }}>
                    Ảnh 480p Đã Che Mờ
                  </div>
                  <div style={{ marginTop: '12px' }}>
                    <div style={{ fontWeight: 600 }}>{e.trigger}</div>
                    <div style={{ fontSize: '0.875rem', color: '#94a3b8', marginTop: '4px' }}>Thời điểm: {e.occurred_at}</div>
                    <div style={{ fontSize: '0.875rem', color: '#f87171', marginTop: '4px' }}>Tự hủy sau: {e.expires_in_hours} giờ</div>
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
