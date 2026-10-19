import React from 'react';
import { Activity, ShieldCheck, AlertOctagon, Layers, ArrowRight, RefreshCw, Smartphone } from 'lucide-react';
import { Language, QueryLogRecord, StatusResponse, TabType } from '../types';
import { translations } from '../i18n';

interface DashboardTabProps {
  status: StatusResponse | null;
  recentLogs: QueryLogRecord[];
  lang: Language;
  onNavigateTab: (tab: TabType) => void;
  onRefresh: () => void;
}

export const DashboardTab: React.FC<DashboardTabProps> = ({
  status,
  recentLogs,
  lang,
  onNavigateTab,
  onRefresh,
}) => {
  const t = translations[lang];

  const quotaTotal = status?.daily_quota_minutes || 60;
  const quotaUsed = status?.used_today_minutes || 0;
  const quotaPct = Math.min(100, Math.round((quotaUsed / quotaTotal) * 100));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
      {/* Monitor Mode Alert Banner */}
      {status?.monitor_mode && (
        <div style={{
          background: 'linear-gradient(90deg, rgba(245, 158, 11, 0.15) 0%, rgba(217, 119, 6, 0.1) 100%)',
          border: '1px solid #f59e0b',
          borderRadius: 14,
          padding: '16px 20px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 16,
          boxShadow: '0 4px 20px rgba(245, 158, 11, 0.1)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
            <span style={{ fontSize: '1.6rem' }}>👁️</span>
            <div>
              <div style={{ fontWeight: 700, color: '#fbbf24', fontSize: '1rem', marginBottom: 2 }}>
                {t.mode_monitor}
              </div>
              <div style={{ color: '#cbd5e1', fontSize: '0.85rem' }}>
                {t.banner_monitor_mode}
              </div>
            </div>
          </div>
          <button
            onClick={() => onNavigateTab('security')}
            style={{
              background: '#f59e0b',
              color: '#0f172a',
              border: 'none',
              padding: '8px 16px',
              borderRadius: 8,
              fontWeight: 700,
              fontSize: '0.85rem',
              cursor: 'pointer',
              whiteSpace: 'nowrap',
            }}
          >
            {t.nav_security} &rarr;
          </button>
        </div>
      )}

      {/* 4 Stat Cards */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
        gap: 20,
      }}>
        {/* Card 1: Active App */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
          position: 'relative',
          overflow: 'hidden',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.9rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Smartphone size={16} color="#38bdf8" />
              {t.card_app_title}
            </span>
            <span style={{
              background: 'rgba(52, 211, 153, 0.15)',
              color: '#34d399',
              padding: '3px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {t.tag_active}
            </span>
          </div>

          <div style={{ fontSize: '1.8rem', fontWeight: 800, color: '#f8fafc', marginBottom: 6 }}>
            {status?.active_foreground_app || 'Roblox.exe'}
          </div>

          <div style={{ fontSize: '0.82rem', color: '#94a3b8', marginBottom: 12 }}>
            {t.dash_quota_text.replace('{used}', quotaUsed.toString()).replace('{total}', quotaTotal.toString())}
          </div>

          {/* Quota Progress Bar */}
          <div style={{ height: 8, background: '#1e293b', borderRadius: 4, overflow: 'hidden' }}>
            <div style={{
              height: '100%',
              width: `${quotaPct}%`,
              background: quotaPct > 80 ? 'linear-gradient(90deg, #f59e0b, #ef4444)' : 'linear-gradient(90deg, #38bdf8, #34d399)',
              borderRadius: 4,
              transition: 'width 0.5s ease',
            }} />
          </div>

          <button
            onClick={() => onNavigateTab('activity')}
            style={{
              marginTop: 12,
              background: 'transparent',
              color: '#38bdf8',
              border: 'none',
              padding: 0,
              fontSize: '0.8rem',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 4,
            }}
          >
            <span>{lang === 'vi' ? 'Xem chi tiết hoạt động & nhật ký ứng dụng' : 'View full activity & app timeline'} &rarr;</span>
          </button>
        </div>

        {/* Card 2: DNS & WFP Filter */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.9rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <ShieldCheck size={16} color={status?.monitor_mode ? "#fbbf24" : "#34d399"} />
              {t.card_dns_title}
            </span>
            <span style={{
              background: 'rgba(56, 189, 248, 0.15)',
              color: '#38bdf8',
              padding: '3px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {status?.dns_proxy_addr || '127.0.0.2:53'}
            </span>
          </div>

          <div style={{ fontSize: '1.8rem', fontWeight: 800, color: status?.monitor_mode ? '#fbbf24' : '#34d399', marginBottom: 6 }}>
            {status?.monitor_mode ? t.mode_monitor : t.status_protecting}
          </div>

          <div style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
            {t.dash_dns_sub}
          </div>
        </div>

        {/* Card 3: Blocked Today */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.9rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <AlertOctagon size={16} color="#f87171" />
              {t.card_blocked_today}
            </span>
            <span style={{
              background: 'rgba(248, 113, 113, 0.15)',
              color: '#f87171',
              padding: '3px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {t.tag_block}
            </span>
          </div>

          <div style={{ fontSize: '1.8rem', fontWeight: 800, color: '#f87171', marginBottom: 6 }}>
            {status?.blocked_queries_today ?? 0}
          </div>

          <div style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
            {t.dash_blocked_sub.replace('{total}', (status?.total_queries_today ?? 0).toString())}
          </div>
        </div>

        {/* Card 4: Rules in effect */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.9rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Layers size={16} color="#a855f7" />
              {t.card_rules_title}
            </span>
            <span style={{
              background: '#1e293b',
              color: '#cbd5e1',
              padding: '3px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 600,
            }}>
              {status?.rules_count ?? 0} quy tắc
            </span>
          </div>

          <div style={{ fontSize: '1.8rem', fontWeight: 800, color: '#f8fafc', marginBottom: 6 }}>
            {status?.rules_count ?? 0}
          </div>

          <div style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
            {t.dash_rules_sub}
          </div>
        </div>
      </div>

      {/* Mini Recent Query Feed */}
      <div style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 16,
        padding: 24,
      }}>
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 18,
          flexWrap: 'wrap',
          gap: 12,
        }}>
          <div>
            <h2 style={{ fontSize: '1.2rem', fontWeight: 700, color: '#f8fafc' }}>
              {t.recent_logs_title}
            </h2>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
              Theo dõi trực tiếp các yêu cầu phân giải DNS loopback mới nhất
            </p>
          </div>

          <div style={{ display: 'flex', gap: 10 }}>
            <button
              onClick={onRefresh}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                background: '#1e293b',
                color: '#f8fafc',
                border: '1px solid #334155',
                padding: '8px 14px',
                borderRadius: 8,
                fontSize: '0.82rem',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              <RefreshCw size={14} />
              {t.btn_refresh}
            </button>
            <button
              onClick={() => onNavigateTab('logs')}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                background: '#38bdf8',
                color: '#070b14',
                border: 'none',
                padding: '8px 14px',
                borderRadius: 8,
                fontSize: '0.82rem',
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              <span>{t.btn_view_all_logs}</span>
              <ArrowRight size={14} />
            </button>
          </div>
        </div>

        {recentLogs.length === 0 ? (
          <div style={{ textAlign: 'center', color: '#94a3b8', padding: '36px 0', fontSize: '0.9rem' }}>
            {t.no_logs}
          </div>
        ) : (
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.88rem' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #1e293b', background: 'rgba(15, 23, 42, 0.5)' }}>
                  <th style={{ textAlign: 'left', padding: '12px 14px', color: '#94a3b8', fontWeight: 600 }}>{t.th_time}</th>
                  <th style={{ textAlign: 'left', padding: '12px 14px', color: '#94a3b8', fontWeight: 600 }}>{t.th_domain}</th>
                  <th style={{ textAlign: 'left', padding: '12px 14px', color: '#94a3b8', fontWeight: 600 }}>{t.th_type}</th>
                  <th style={{ textAlign: 'left', padding: '12px 14px', color: '#94a3b8', fontWeight: 600 }}>{t.th_action}</th>
                  <th style={{ textAlign: 'left', padding: '12px 14px', color: '#94a3b8', fontWeight: 600 }}>{t.th_reason}</th>
                </tr>
              </thead>
              <tbody>
                {recentLogs.slice(0, 5).map((log, idx) => {
                  const isBlock = log.action === 'BLOCK';
                  const timeStr = new Date(log.occurred_at).toLocaleTimeString();
                  return (
                    <tr key={idx} style={{ borderBottom: '1px solid #1e293b' }}>
                      <td style={{ padding: '12px 14px', color: '#94a3b8' }}>{timeStr}</td>
                      <td style={{ padding: '12px 14px', color: '#f8fafc', fontWeight: 600 }}>{log.domain}</td>
                      <td style={{ padding: '12px 14px' }}>
                        <span style={{ background: '#1e293b', color: '#cbd5e1', padding: '2px 8px', borderRadius: 4, fontSize: '0.75rem' }}>
                          {log.qtype}
                        </span>
                      </td>
                      <td style={{ padding: '12px 14px' }}>
                        <span style={{
                          background: isBlock ? 'rgba(248, 113, 113, 0.15)' : 'rgba(52, 211, 153, 0.15)',
                          color: isBlock ? '#f87171' : '#34d399',
                          padding: '3px 10px',
                          borderRadius: 6,
                          fontWeight: 700,
                          fontSize: '0.78rem',
                        }}>
                          {isBlock ? t.tag_block : t.tag_allow}
                        </span>
                      </td>
                      <td style={{ padding: '12px 14px', color: '#94a3b8', fontSize: '0.8rem' }}>{log.reason}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
