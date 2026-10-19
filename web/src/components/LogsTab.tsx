import React, { useState } from 'react';
import { Search, RefreshCw, Filter } from 'lucide-react';
import { Language, QueryLogRecord } from '../types';
import { translations } from '../i18n';

interface LogsTabProps {
  logs: QueryLogRecord[];
  lang: Language;
  onRefresh: () => void;
  autoRefresh: boolean;
  onToggleAutoRefresh: () => void;
}

export const LogsTab: React.FC<LogsTabProps> = ({
  logs,
  lang,
  onRefresh,
  autoRefresh,
  onToggleAutoRefresh,
}) => {
  const t = translations[lang];
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'blocked' | 'allowed'>('all');

  const filteredLogs = logs.filter((log) => {
    if (filterType === 'blocked' && log.action !== 'BLOCK') return false;
    if (filterType === 'allowed' && log.action !== 'ALLOW') return false;
    if (searchTerm && !log.domain.toLowerCase().includes(searchTerm.toLowerCase())) return false;
    return true;
  });

  return (
    <div style={{
      background: '#0f172a',
      border: '1px solid #1e293b',
      borderRadius: 16,
      padding: 24,
      display: 'flex',
      flexDirection: 'column',
      gap: 20,
    }}>
      {/* Header & Controls */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: 14,
      }}>
        <div>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc' }}>
            {t.dns_logs_title}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
            {t.dns_logs_desc}
          </p>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
          {/* Search Box */}
          <div style={{ position: 'relative' }}>
            <Search size={16} color="#94a3b8" style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)' }} />
            <input
              type="text"
              placeholder={t.placeholder_search_logs}
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              style={{
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '8px 12px 8px 36px',
                borderRadius: 8,
                fontSize: '0.85rem',
                outline: 'none',
                width: 220,
              }}
            />
          </div>

          {/* Filter Pills */}
          <div style={{ display: 'inline-flex', background: '#070b14', borderRadius: 8, padding: 3, border: '1px solid #1e293b' }}>
            <button
              onClick={() => setFilterType('all')}
              style={{
                background: filterType === 'all' ? '#38bdf8' : 'transparent',
                color: filterType === 'all' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: filterType === 'all' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              {t.filter_all}
            </button>
            <button
              onClick={() => setFilterType('blocked')}
              style={{
                background: filterType === 'blocked' ? '#f87171' : 'transparent',
                color: filterType === 'blocked' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: filterType === 'blocked' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              {t.filter_blocked}
            </button>
            <button
              onClick={() => setFilterType('allowed')}
              style={{
                background: filterType === 'allowed' ? '#34d399' : 'transparent',
                color: filterType === 'allowed' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: filterType === 'allowed' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              {t.filter_allowed}
            </button>
          </div>

          {/* Refresh Button */}
          <button
            onClick={onRefresh}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              background: '#1e293b',
              color: '#f8fafc',
              border: '1px solid #334155',
              padding: '7px 12px',
              borderRadius: 8,
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            <RefreshCw size={14} />
            <span>{t.btn_refresh}</span>
          </button>
        </div>
      </div>

      {/* Logs Table */}
      {filteredLogs.length === 0 ? (
        <div style={{ textAlign: 'center', color: '#94a3b8', padding: '40px 0', fontSize: '0.9rem' }}>
          {t.no_matching_logs}
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
              {filteredLogs.map((log, idx) => {
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
  );
};
