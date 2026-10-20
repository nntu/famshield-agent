import React, { useEffect, useMemo, useState } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import {
  Ban,
  CheckCircle,
  Database,
  Eye,
  Globe,
  Layers,
  List,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  Trash2,
  X,
} from 'lucide-react';
import { DnsRetentionConfig, DomainQuerySummary, Language, QueryLogRecord } from '../types';
import { translations } from '../i18n';
import { DataTable } from './DataTable';
import { api } from '../api';

interface LogsTabProps {
  logs: QueryLogRecord[];
  lang: Language;
  onRefresh: () => void;
  autoRefresh: boolean;
  onToggleAutoRefresh: () => void;
  onShowToast?: (message: string, type?: 'success' | 'error' | 'info') => void;
}

export const LogsTab: React.FC<LogsTabProps> = ({
  logs,
  lang,
  onRefresh,
  autoRefresh,
  onToggleAutoRefresh,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  // View mode: 'summary' (Bản tổng hợp theo domain - Mặc định) or 'stream' (Toàn bộ chi tiết)
  const [viewMode, setViewMode] = useState<'summary' | 'stream'>('summary');
  const [filterType, setFilterType] = useState<'all' | 'blocked' | 'allowed' | 'monitored' | 'errors'>('all');

  // Aggregated domain summary state
  const [summaries, setSummaries] = useState<DomainQuerySummary[]>([]);
  const [isLoadingSummaries, setIsLoadingSummaries] = useState<boolean>(false);

  // Drilldown detail modal state
  const [selectedDomain, setSelectedDomain] = useState<string | null>(null);
  const [domainLogs, setDomainLogs] = useState<QueryLogRecord[]>([]);
  const [isLoadingDetails, setIsLoadingDetails] = useState<boolean>(false);
  const [isBlocking, setIsBlocking] = useState<boolean>(false);

  // Retention & Pruning State
  const [retentionConfig, setRetentionConfig] = useState<DnsRetentionConfig | null>(null);
  const [showRetentionSettings, setShowRetentionSettings] = useState<boolean>(false);
  const [selectedRetentionDays, setSelectedRetentionDays] = useState<number>(7);
  const [autoPruneEnabled, setAutoPruneEnabled] = useState<boolean>(true);
  const [isSavingRetention, setIsSavingRetention] = useState<boolean>(false);
  const [isPruningNow, setIsPruningNow] = useState<boolean>(false);

  const fetchRetentionConfig = async () => {
    try {
      const cfg = await api.getDnsRetention();
      setRetentionConfig(cfg);
      setSelectedRetentionDays(cfg.retention_days);
      setAutoPruneEnabled(cfg.auto_prune);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    fetchRetentionConfig();
  }, []);

  const handleSaveRetention = async (days: number, auto: boolean) => {
    setIsSavingRetention(true);
    try {
      const res = await api.setDnsRetention(days, auto);
      onShowToast?.(res.message, 'success');
      await fetchRetentionConfig();
    } catch (e: any) {
      onShowToast?.(e.message || (isVi ? 'Lỗi lưu cấu hình lưu trữ' : 'Failed to save retention'), 'error');
    } finally {
      setIsSavingRetention(false);
    }
  };

  const handlePruneNow = async () => {
    setIsPruningNow(true);
    try {
      const res = await api.pruneDnsLogsNow();
      onShowToast?.(res.message, 'success');
      await fetchRetentionConfig();
      fetchSummaries();
      onRefresh();
    } catch (e: any) {
      onShowToast?.(e.message || (isVi ? 'Lỗi dọn dẹp nhật ký' : 'Failed to prune logs'), 'error');
    } finally {
      setIsPruningNow(false);
    }
  };

  // Fetch domain summaries
  const fetchSummaries = async () => {
    setIsLoadingSummaries(true);
    try {
      const data = await api.getDnsSummary(500);
      setSummaries(data);
    } catch {
      // Fallback: aggregate in memory from logs prop if summary endpoint fails
      const map = new Map<string, {
        total: number;
        allowed: number;
        blocked: number;
        monitored: number;
        errors: number;
        lastSeen: string;
      }>();
      for (const l of logs) {
        const item = map.get(l.domain) || {
          total: 0,
          allowed: 0,
          blocked: 0,
          monitored: 0,
          errors: 0,
          lastSeen: l.occurred_at,
        };
        item.total++;
        if (l.action === 'BLOCK') item.blocked++;
        else if (l.action === 'ALLOW') item.allowed++;
        else if (l.action === 'MONITOR') item.monitored++;
        else if (l.action === 'ERROR') item.errors++;
        if (new Date(l.occurred_at) > new Date(item.lastSeen)) {
          item.lastSeen = l.occurred_at;
        }
        map.set(l.domain, item);
      }
      const aggregated: DomainQuerySummary[] = Array.from(map.entries()).map(([domain, val]) => ({
        domain,
        total_queries: val.total,
        allowed_count: val.allowed,
        blocked_count: val.blocked,
        monitored_count: val.monitored,
        error_count: val.errors,
        last_seen_at: val.lastSeen,
      }));
      aggregated.sort((a, b) => b.total_queries - a.total_queries);
      setSummaries(aggregated);
    } finally {
      setIsLoadingSummaries(false);
    }
  };

  useEffect(() => {
    fetchSummaries();
  }, [logs]);

  // Open detail drilldown for a specific domain
  const handleOpenDomainDetail = async (domain: string) => {
    setSelectedDomain(domain);
    setIsLoadingDetails(true);
    try {
      const details = await api.getLogs(150, domain);
      setDomainLogs(details);
    } catch {
      // Fallback: filter from loaded logs
      setDomainLogs(logs.filter((l) => l.domain === domain));
    } finally {
      setIsLoadingDetails(false);
    }
  };

  const handleCloseDomainDetail = () => {
    setSelectedDomain(null);
    setDomainLogs([]);
  };

  // Quick block domain action from detail modal
  const handleQuickBlockDomain = async (domain: string) => {
    setIsBlocking(true);
    try {
      await api.createRule({
        rule_id: domain,
        rule_type: 'domain',
        payload_json: JSON.stringify({ domain, action: 'block' }),
        enabled: true,
      });
      if (onShowToast) {
        onShowToast(
          isVi ? `Đã thêm quy tắc chặn cho tên miền ${domain}` : `Added block rule for ${domain}`,
          'success'
        );
      }
      onRefresh();
      fetchSummaries();
    } catch {
      if (onShowToast) {
        onShowToast(
          isVi ? `Không thể chặn domain ${domain}` : `Failed to block ${domain}`,
          'error'
        );
      }
    } finally {
      setIsBlocking(false);
    }
  };

  // Filtered summaries
  const filteredSummaries = useMemo(() => {
    return summaries.filter((s) => {
      if (filterType === 'blocked' && s.blocked_count === 0) return false;
      if (filterType === 'allowed' && s.allowed_count === 0) return false;
      if (filterType === 'monitored' && s.monitored_count === 0) return false;
      if (filterType === 'errors' && s.error_count === 0) return false;
      return true;
    });
  }, [summaries, filterType]);

  // Filtered raw logs for stream view
  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      if (filterType === 'blocked' && log.action !== 'BLOCK') return false;
      if (filterType === 'allowed' && log.action !== 'ALLOW') return false;
      if (filterType === 'monitored' && log.action !== 'MONITOR') return false;
      if (filterType === 'errors' && log.action !== 'ERROR') return false;
      return true;
    });
  }, [logs, filterType]);

  // Columns for Domain Summary View
  const summaryColumns = useMemo<ColumnDef<DomainQuerySummary>[]>(
    () => [
      {
        accessorKey: 'domain',
        header: t.th_domain,
        cell: ({ getValue }) => {
          const domain = getValue<string>();
          return (
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 8,
                cursor: 'pointer',
              }}
              onClick={() => handleOpenDomainDetail(domain)}
              title={isVi ? 'Nhấp để xem chi tiết các truy vấn' : 'Click to drill down queries'}
            >
              <Globe size={15} style={{ color: '#38bdf8', flexShrink: 0 }} />
              <span style={{ color: '#f8fafc', fontWeight: 600, wordBreak: 'break-all' }}>
                {domain}
              </span>
            </div>
          );
        },
      },
      {
        accessorKey: 'total_queries',
        header: t.th_total_queries,
        cell: ({ getValue }) => (
          <span
            style={{
              background: '#1e293b',
              color: '#38bdf8',
              padding: '3px 10px',
              borderRadius: 6,
              fontSize: '0.82rem',
              fontWeight: 700,
            }}
          >
            {getValue<number>().toLocaleString()}
          </span>
        ),
      },
      {
        id: 'breakdown',
        header: t.th_status_breakdown,
        cell: ({ row }) => {
          const s = row.original;
          return (
            <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
              {s.blocked_count > 0 && (
                <span
                  style={{
                    background: '#f8717126',
                    color: '#f87171',
                    padding: '2px 7px',
                    borderRadius: 4,
                    fontSize: '0.74rem',
                    fontWeight: 700,
                  }}
                >
                  {s.blocked_count} BLOCK
                </span>
              )}
              {s.allowed_count > 0 && (
                <span
                  style={{
                    background: '#34d39926',
                    color: '#34d399',
                    padding: '2px 7px',
                    borderRadius: 4,
                    fontSize: '0.74rem',
                    fontWeight: 700,
                  }}
                >
                  {s.allowed_count} ALLOW
                </span>
              )}
              {s.monitored_count > 0 && (
                <span
                  style={{
                    background: '#fbbf2426',
                    color: '#fbbf24',
                    padding: '2px 7px',
                    borderRadius: 4,
                    fontSize: '0.74rem',
                    fontWeight: 700,
                  }}
                >
                  {s.monitored_count} MONITOR
                </span>
              )}
              {s.error_count > 0 && (
                <span
                  style={{
                    background: '#94a3b826',
                    color: '#94a3b8',
                    padding: '2px 7px',
                    borderRadius: 4,
                    fontSize: '0.74rem',
                    fontWeight: 700,
                  }}
                >
                  {s.error_count} ERROR
                </span>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: 'last_seen_at',
        header: t.th_last_seen,
        cell: ({ getValue }) => {
          const val = getValue<string>();
          return (
            <span style={{ color: '#94a3b8', fontSize: '0.84rem' }}>
              {new Date(val).toLocaleTimeString()} ({new Date(val).toLocaleDateString()})
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: t.btn_domain_details,
        cell: ({ row }) => (
          <button
            onClick={() => handleOpenDomainDetail(row.original.domain)}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 5,
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              padding: '4px 10px',
              borderRadius: 6,
              fontSize: '0.78rem',
              fontWeight: 600,
              cursor: 'pointer',
              transition: 'all 0.15s ease',
            }}
          >
            <Eye size={13} />
            <span>{t.btn_domain_details}</span>
          </button>
        ),
      },
    ],
    [t, isVi]
  );

  // Columns for Raw Logs Stream View
  const rawLogColumns = useMemo<ColumnDef<QueryLogRecord>[]>(
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
        cell: ({ getValue }) => {
          const domain = getValue<string>();
          return (
            <span
              style={{
                color: '#f8fafc',
                fontWeight: 600,
                wordBreak: 'break-all',
                cursor: 'pointer',
              }}
              onClick={() => handleOpenDomainDetail(domain)}
              title={isVi ? 'Nhấp để lọc riêng tên miền này' : 'Click to drill down'}
            >
              {domain}
            </span>
          );
        },
      },
      {
        accessorKey: 'qtype',
        header: t.th_type,
        cell: ({ getValue }) => (
          <span
            style={{
              background: '#1e293b',
              color: '#cbd5e1',
              padding: '2px 8px',
              borderRadius: 4,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}
          >
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
    [t, isVi]
  );

  // Selected domain summary stats
  const activeSummary = useMemo(() => {
    if (!selectedDomain) return null;
    return summaries.find((s) => s.domain === selectedDomain);
  }, [selectedDomain, summaries]);

  return (
    <div
      style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 16,
        padding: 24,
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        position: 'relative',
      }}
    >
      {/* Header & Controls */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          flexWrap: 'wrap',
          gap: 14,
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc' }}>
              {t.dns_logs_title}
            </h2>
            <span
              style={{
                background: '#1e293b',
                color: '#38bdf8',
                padding: '2px 8px',
                borderRadius: 6,
                fontSize: '0.75rem',
                fontWeight: 600,
              }}
            >
              {viewMode === 'summary' ? `${filteredSummaries.length} domains` : `${filteredLogs.length} queries`}
            </span>
          </div>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
            {t.dns_logs_desc}
          </p>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
          {/* View Mode Toggle: Summary (Default) vs Stream */}
          <div
            style={{
              display: 'inline-flex',
              background: '#070b14',
              borderRadius: 8,
              padding: 3,
              border: '1px solid #1e293b',
            }}
          >
            <button
              onClick={() => setViewMode('summary')}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 5,
                background: viewMode === 'summary' ? '#38bdf8' : 'transparent',
                color: viewMode === 'summary' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '5px 12px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: viewMode === 'summary' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              <Layers size={14} />
              <span>{t.view_mode_summary}</span>
            </button>
            <button
              onClick={() => setViewMode('stream')}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 5,
                background: viewMode === 'stream' ? '#38bdf8' : 'transparent',
                color: viewMode === 'stream' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '5px 12px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: viewMode === 'stream' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              <List size={14} />
              <span>{t.view_mode_stream}</span>
            </button>
          </div>

          {/* Filter Pills */}
          <div
            style={{
              display: 'inline-flex',
              background: '#070b14',
              borderRadius: 8,
              padding: 3,
              border: '1px solid #1e293b',
            }}
          >
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
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: filterType === 'monitored' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              {t.filter_monitored}
            </button>
            <button
              onClick={() => setFilterType('errors')}
              style={{
                background: filterType === 'errors' ? '#f87171' : 'transparent',
                color: filterType === 'errors' ? '#070b14' : '#94a3b8',
                border: 'none',
                padding: '4px 10px',
                borderRadius: 6,
                fontSize: '0.8rem',
                fontWeight: filterType === 'errors' ? 700 : 500,
                cursor: 'pointer',
              }}
            >
              {t.filter_error}
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
            {autoRefresh ? (isVi ? 'Tự động: BẬT' : 'Auto: ON') : (isVi ? 'Tự động: TẮT' : 'Auto: OFF')}
          </button>

          {/* Refresh Button */}
          <button
            onClick={() => {
              onRefresh();
              fetchSummaries();
            }}
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
            <RefreshCw size={14} className={isLoadingSummaries ? 'animate-spin' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Storage Retention & Optimization Toggle */}
          <button
            onClick={() => setShowRetentionSettings(!showRetentionSettings)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              background: showRetentionSettings ? 'rgba(56, 189, 248, 0.2)' : '#1e293b',
              color: showRetentionSettings ? '#38bdf8' : '#94a3b8',
              border: `1px solid ${showRetentionSettings ? '#38bdf8' : '#334155'}`,
              padding: '7px 12px',
              borderRadius: 8,
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: 'pointer',
            }}
            title={isVi ? 'Cấu hình thời gian lưu trữ & dọn dẹp dung lượng' : 'Storage retention & cleanup'}
          >
            <Database size={14} />
            <span>{isVi ? 'Tối Ưu Lưu Trữ' : 'Storage Optimization'}</span>
          </button>
        </div>
      </div>

      {/* Storage Optimization & Retention Banner */}
      {showRetentionSettings && (
        <div
          style={{
            background: 'rgba(15, 23, 42, 0.7)',
            border: '1px solid rgba(56, 189, 248, 0.3)',
            borderRadius: 12,
            padding: '16px 20px',
            marginTop: 14,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
          }}
        >
          <div style={{ maxWidth: 600 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <Database size={16} color="#38bdf8" />
              <span style={{ fontSize: '0.95rem', fontWeight: 700, color: '#f8fafc' }}>
                {isVi ? 'Cơ Chế Nén & Tối Ưu Dung Lượng Nhật Ký DNS' : 'DNS Query Logs Compression & Retention'}
              </span>
            </div>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8', margin: 0, lineHeight: 1.5 }}>
              {isVi
                ? 'Để tiết kiệm ổ cứng, hệ thống tự động tổng hợp số liệu của từng tên miền vào bảng thống kê vĩnh viễn và dọn dẹp các dòng log chi tiết từng giây đã quá hạn. Dữ liệu tổng hợp (Tổng truy vấn, Cho phép, Chặn) luôn được bảo toàn 100%.'
                : 'To minimize disk space, raw second-by-second queries are rolled up into persistent domain aggregates and pruned when expired. Domain totals are always preserved 100%.'}
            </p>
            <div style={{ display: 'flex', gap: 16, marginTop: 8, fontSize: '0.8rem', color: '#cbd5e1' }}>
              <span>
                {isVi ? 'Bản ghi chi tiết:' : 'Raw queries:'}{' '}
                <strong style={{ color: '#38bdf8' }}>{retentionConfig?.raw_query_logs_count ?? 0}</strong>
              </span>
              <span>
                {isVi ? 'Tên miền tổng hợp vĩnh viễn:' : 'Aggregated domains:'}{' '}
                <strong style={{ color: '#34d399' }}>{retentionConfig?.aggregated_domains_count ?? 0}</strong>
              </span>
              {retentionConfig?.last_pruned_at && (
                <span style={{ color: '#94a3b8' }}>
                  {isVi ? 'Dọn dẹp gần nhất:' : 'Last pruned:'}{' '}
                  {new Date(retentionConfig.last_pruned_at).toLocaleTimeString()}
                </span>
              )}
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <label style={{ fontSize: '0.82rem', color: '#94a3b8', fontWeight: 600 }}>
                {isVi ? 'Lưu chi tiết:' : 'Retain raw:'}
              </label>
              <select
                value={selectedRetentionDays}
                onChange={(e) => {
                  const days = parseInt(e.target.value, 10);
                  setSelectedRetentionDays(days);
                  handleSaveRetention(days, autoPruneEnabled);
                }}
                disabled={isSavingRetention}
                style={{
                  background: '#070b14',
                  border: '1px solid #334155',
                  color: '#f8fafc',
                  padding: '6px 10px',
                  borderRadius: 6,
                  fontSize: '0.82rem',
                  cursor: 'pointer',
                  outline: 'none',
                }}
              >
                <option value={1}>{isVi ? '1 ngày (Tiết kiệm tối đa)' : '1 day (Max savings)'}</option>
                <option value={3}>{isVi ? '3 ngày' : '3 days'}</option>
                <option value={7}>{isVi ? '7 ngày (Khuyên dùng)' : '7 days (Recommended)'}</option>
                <option value={14}>{isVi ? '14 ngày' : '14 days'}</option>
                <option value={30}>{isVi ? '30 ngày' : '30 days'}</option>
                <option value={90}>{isVi ? '90 ngày' : '90 days'}</option>
              </select>
            </div>

            <button
              type="button"
              onClick={handlePruneNow}
              disabled={isPruningNow}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                background: '#0284c7',
                color: '#ffffff',
                border: 'none',
                padding: '7px 14px',
                borderRadius: 6,
                fontSize: '0.82rem',
                fontWeight: 600,
                cursor: isPruningNow ? 'not-allowed' : 'pointer',
              }}
            >
              <Trash2 size={13} className={isPruningNow ? 'animate-spin' : ''} />
              {isPruningNow
                ? (isVi ? 'Đang dọn dẹp...' : 'Pruning...')
                : (isVi ? 'Thu Gọn & Dọn Dẹp Ngay' : 'Prune Now')}
            </button>
          </div>
        </div>
      )}

      {/* Main Table: Summary or Stream */}
      {viewMode === 'summary' ? (
        <DataTable
          data={filteredSummaries}
          columns={summaryColumns}
          pageSize={10}
          searchPlaceholder={t.placeholder_search_logs}
          emptyText={t.no_matching_logs}
          lang={lang}
        />
      ) : (
        <DataTable
          data={filteredLogs}
          columns={rawLogColumns}
          pageSize={10}
          searchPlaceholder={t.placeholder_search_logs}
          emptyText={t.no_matching_logs}
          lang={lang}
        />
      )}

      {/* DRILLDOWN DETAIL MODAL FOR A SPECIFIC DOMAIN */}
      {selectedDomain && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0, 0, 0, 0.75)',
            backdropFilter: 'blur(4px)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 100,
            padding: 16,
          }}
          onClick={handleCloseDomainDetail}
        >
          <div
            style={{
              background: '#0f172a',
              border: '1px solid #334155',
              borderRadius: 16,
              width: '100%',
              maxWidth: 850,
              maxHeight: '90vh',
              display: 'flex',
              flexDirection: 'column',
              boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.7)',
              overflow: 'hidden',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            {/* Modal Header */}
            <div
              style={{
                padding: '20px 24px',
                borderBottom: '1px solid #1e293b',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                flexWrap: 'wrap',
                gap: 12,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <div
                  style={{
                    width: 40,
                    height: 40,
                    borderRadius: 10,
                    background: 'rgba(56, 189, 248, 0.15)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    color: '#38bdf8',
                  }}
                >
                  <Globe size={22} />
                </div>
                <div>
                  <h3 style={{ fontSize: '1.2rem', fontWeight: 700, color: '#f8fafc' }}>
                    {selectedDomain}
                  </h3>
                  <p style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
                    {t.modal_domain_desc}: <span style={{ color: '#38bdf8' }}>{selectedDomain}</span>
                  </p>
                </div>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                {/* Quick Block Button */}
                <button
                  onClick={() => handleQuickBlockDomain(selectedDomain)}
                  disabled={isBlocking}
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 6,
                    background: '#f8717126',
                    color: '#f87171',
                    border: '1px solid #f8717140',
                    padding: '7px 14px',
                    borderRadius: 8,
                    fontSize: '0.82rem',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  <Ban size={14} />
                  <span>{isBlocking ? 'Đang chặn...' : t.btn_quick_block}</span>
                </button>

                {/* Close Button */}
                <button
                  onClick={handleCloseDomainDetail}
                  style={{
                    background: '#1e293b',
                    color: '#94a3b8',
                    border: 'none',
                    borderRadius: 8,
                    width: 32,
                    height: 32,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    cursor: 'pointer',
                  }}
                >
                  <X size={18} />
                </button>
              </div>
            </div>

            {/* Modal Subheader with Stats */}
            {activeSummary && (
              <div
                style={{
                  padding: '12px 24px',
                  background: '#070b14',
                  borderBottom: '1px solid #1e293b',
                  display: 'flex',
                  gap: 12,
                  flexWrap: 'wrap',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem', color: '#94a3b8' }}>
                  <span>{t.th_total_queries}:</span>
                  <strong style={{ color: '#38bdf8' }}>{activeSummary.total_queries}</strong>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem', color: '#94a3b8' }}>
                  <span>ALLOW:</span>
                  <strong style={{ color: '#34d399' }}>{activeSummary.allowed_count}</strong>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem', color: '#94a3b8' }}>
                  <span>BLOCK:</span>
                  <strong style={{ color: '#f87171' }}>{activeSummary.blocked_count}</strong>
                </div>
                {activeSummary.monitored_count > 0 && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem', color: '#94a3b8' }}>
                    <span>MONITOR:</span>
                    <strong style={{ color: '#fbbf24' }}>{activeSummary.monitored_count}</strong>
                  </div>
                )}
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem', color: '#94a3b8' }}>
                  <span>{t.th_last_seen}:</span>
                  <strong style={{ color: '#cbd5e1' }}>
                    {new Date(activeSummary.last_seen_at).toLocaleTimeString()}
                  </strong>
                </div>
              </div>
            )}

            {/* Modal Body: Query Log Table for Domain */}
            <div style={{ flex: 1, overflowY: 'auto', padding: 20 }}>
              {isLoadingDetails ? (
                <div style={{ textAlign: 'center', padding: 40, color: '#94a3b8' }}>
                  <RefreshCw size={24} className="animate-spin" style={{ margin: '0 auto 12px' }} />
                  <p>{isVi ? 'Đang truy xuất chi tiết truy vấn của tên miền...' : 'Loading query resolution details...'}</p>
                </div>
              ) : domainLogs.length === 0 ? (
                <div style={{ textAlign: 'center', padding: 40, color: '#94a3b8' }}>
                  <p>{t.no_matching_logs}</p>
                </div>
              ) : (
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.85rem' }}>
                  <thead>
                    <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                      <th style={{ padding: '10px 12px' }}>{t.th_time}</th>
                      <th style={{ padding: '10px 12px' }}>{t.th_type}</th>
                      <th style={{ padding: '10px 12px' }}>{t.th_action}</th>
                      <th style={{ padding: '10px 12px' }}>{t.th_reason}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {domainLogs.map((item, idx) => {
                      const isBlock = item.action === 'BLOCK';
                      const isMonitor = item.action === 'MONITOR';
                      const isError = item.action === 'ERROR';
                      const color = isBlock || isError ? '#f87171' : isMonitor ? '#fbbf24' : '#34d399';
                      return (
                        <tr
                          key={idx}
                          style={{
                            borderBottom: '1px solid #1e293b',
                            transition: 'background 0.15s ease',
                          }}
                        >
                          <td style={{ padding: '10px 12px', color: '#cbd5e1' }}>
                            {new Date(item.occurred_at).toLocaleTimeString()}{' '}
                            <span style={{ color: '#64748b', fontSize: '0.75rem' }}>
                              ({new Date(item.occurred_at).toLocaleDateString()})
                            </span>
                          </td>
                          <td style={{ padding: '10px 12px' }}>
                            <span
                              style={{
                                background: '#1e293b',
                                color: '#cbd5e1',
                                padding: '2px 8px',
                                borderRadius: 4,
                                fontSize: '0.75rem',
                                fontWeight: 700,
                              }}
                            >
                              {item.qtype}
                            </span>
                          </td>
                          <td style={{ padding: '10px 12px' }}>
                            <span
                              style={{
                                display: 'inline-flex',
                                alignItems: 'center',
                                gap: 4,
                                background: `${color}26`,
                                color,
                                padding: '2px 8px',
                                borderRadius: 6,
                                fontWeight: 700,
                                fontSize: '0.76rem',
                              }}
                            >
                              {isBlock || isError ? <ShieldAlert size={12} /> : <ShieldCheck size={12} />}
                              {item.action}
                            </span>
                          </td>
                          <td style={{ padding: '10px 12px', color: '#94a3b8', fontSize: '0.8rem' }}>
                            {item.reason}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              )}
            </div>

            {/* Modal Footer */}
            <div
              style={{
                padding: '14px 24px',
                borderTop: '1px solid #1e293b',
                display: 'flex',
                justifyContent: 'flex-end',
                background: '#070b14',
              }}
            >
              <button
                onClick={handleCloseDomainDetail}
                style={{
                  background: '#1e293b',
                  color: '#f8fafc',
                  border: '1px solid #334155',
                  padding: '7px 18px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {isVi ? 'Đóng' : 'Close'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
