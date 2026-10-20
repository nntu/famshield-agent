import React, { useMemo, useState } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import { RefreshCw, ShieldAlert, ShieldCheck } from 'lucide-react';
import { Language, QueryLogRecord } from '../types';
import { translations } from '../i18n';
import { DataTable } from './DataTable';

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
  const isVi = lang === 'vi';
  const [filterType, setFilterType] = useState<'all' | 'blocked' | 'allowed' | 'monitored' | 'errors'>('all');

  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      if (filterType === 'blocked' && log.action !== 'BLOCK') return false;
      if (filterType === 'allowed' && log.action !== 'ALLOW') return false;
        if (filterType === 'monitored' && log.action !== 'MONITOR') return false;
        if (filterType === 'errors' && log.action !== 'ERROR') return false;
      return true;
    });
  }, [logs, filterType]);

  const columns = useMemo<ColumnDef<QueryLogRecord>[]>(
    () => [
      {
        accessorKey: 'occurred_at',
        header: t.th_time,
        cell: ({ getValue }) => {
          const val = getValue<string>();
          return (
            <span style={{ color: '#94a3b8', fontSize: '0.84rem' }}>
              {new Date(val).toLocaleTimeString()}
            </span>
          );
        },
      },
      {
        accessorKey: 'domain',
        header: t.th_domain,
        cell: ({ getValue }) => (
          <span style={{ color: '#f8fafc', fontWeight: 600, wordBreak: 'break-all' }}>
            {getValue<string>()}
          </span>
        ),
      },
      {
        accessorKey: 'qtype',
        header: t.th_type,
        cell: ({ getValue }) => (
          <span style={{
            background: '#1e293b',
            color: '#cbd5e1',
            padding: '2px 8px',
            borderRadius: 4,
            fontSize: '0.75rem',
            fontWeight: 700,
          }}>
            {getValue<string>()}
          </span>
        ),
      },
      {
        accessorKey: 'action',
        header: t.th_action,
        cell: ({ getValue }) => {
            const action = getValue<'ALLOW' | 'BLOCK' | 'MONITOR' | 'ERROR'>();
          const isBlock = action === 'BLOCK';
            const isMonitor = action === 'MONITOR';
            const isError = action === 'ERROR';
            const color = isBlock || isError ? '#f87171' : isMonitor ? '#fbbf24' : '#34d399';
          return (
            <span
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 5,
                  background: `${color}26`,
                  color,
                padding: '3px 10px',
                borderRadius: 6,
                fontWeight: 700,
                fontSize: '0.78rem',
              }}
            >
                {isBlock || isError ? <ShieldAlert size={13} /> : <ShieldCheck size={13} />}
                {isBlock ? t.tag_block : isMonitor ? 'MONITOR' : isError ? 'ERROR' : t.tag_allow}
            </span>
          );
        },
      },
      {
        accessorKey: 'reason',
        header: t.th_reason,
        cell: ({ getValue }) => (
          <span style={{ color: '#94a3b8', fontSize: '0.82rem' }}>
            {getValue<string>()}
          </span>
        ),
      },
    ],
    [t]
  );

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
              <button
                onClick={() => setFilterType('monitored')}
                style={{
                  background: filterType === 'monitored' ? '#fbbf24' : 'transparent',
                  color: filterType === 'monitored' ? '#070b14' : '#94a3b8',
                  border: 'none', padding: '4px 10px', borderRadius: 6,
                  fontSize: '0.8rem', fontWeight: filterType === 'monitored' ? 700 : 500,
                  cursor: 'pointer',
                }}
              >
                Monitor
              </button>
              <button
                onClick={() => setFilterType('errors')}
                style={{
                  background: filterType === 'errors' ? '#f87171' : 'transparent',
                  color: filterType === 'errors' ? '#070b14' : '#94a3b8',
                  border: 'none', padding: '4px 10px', borderRadius: 6,
                  fontSize: '0.8rem', fontWeight: filterType === 'errors' ? 700 : 500,
                  cursor: 'pointer',
                }}
              >
                Error
              </button>
          </div>

          {/* Auto Refresh Toggle */}
          <button
            onClick={onToggleAutoRefresh}
            style={{
              background: autoRefresh ? 'rgba(56, 189, 248, 0.15)' : '#1e293b',
              color: autoRefresh ? '#38bdf8' : '#94a3b8',
              border: '1px solid #334155',
              padding: '6px 12px',
              borderRadius: 8,
              fontSize: '0.8rem',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            {autoRefresh ? (isVi ? 'Tự động làm mới: BẬT' : 'Auto: ON') : (isVi ? 'Tự động: TẮT' : 'Auto: OFF')}
          </button>

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

      {/* TanStack React Table with Sorting, Filter, and Pagination */}
      <DataTable
        data={filteredLogs}
        columns={columns}
        pageSize={10}
        searchPlaceholder={t.placeholder_search_logs}
        emptyText={t.no_matching_logs}
        lang={lang}
      />
    </div>
  );
};
