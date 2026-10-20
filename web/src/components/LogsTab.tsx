import type { ColumnDef } from '@tanstack/react-table';
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
import type React from 'react';
import { useEffect, useMemo, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { DnsRetentionConfig, DomainQuerySummary, Language, QueryLogRecord } from '../types';
import { DataTable } from './DataTable';

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
  const [filterType, setFilterType] = useState<
    'all' | 'blocked' | 'allowed' | 'monitored' | 'errors'
  >('all');

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
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast?.(
        msg || (isVi ? 'Lỗi lưu cấu hình lưu trữ' : 'Failed to save retention'),
        'error'
      );
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
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast?.(msg || (isVi ? 'Lỗi dọn dẹp nhật ký' : 'Failed to prune logs'), 'error');
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
      const map = new Map<
        string,
        {
          total: number;
          allowed: number;
          blocked: number;
          monitored: number;
          errors: number;
          lastSeen: string;
        }
      >();
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
              className="inline-flex items-center gap-2 cursor-pointer group"
              onClick={() => handleOpenDomainDetail(domain)}
              title={isVi ? 'Nhấp để xem chi tiết các truy vấn' : 'Click to drill down queries'}
            >
              <Globe
                size={15}
                className="text-sky-600 shrink-0 group-hover:scale-110 transition-transform"
              />
              <span className="text-slate-900 font-semibold break-all group-hover:text-sky-600 transition-colors">
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
          <span className="bg-slate-100 text-slate-800 font-bold px-2.5 py-1 rounded-md text-xs">
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
            <div className="flex gap-1.5 flex-wrap items-center">
              {s.blocked_count > 0 && (
                <span className="bg-rose-50 text-rose-700 border border-rose-200/80 px-2 py-0.5 rounded text-xs font-semibold">
                  {s.blocked_count} BLOCK
                </span>
              )}
              {s.allowed_count > 0 && (
                <span className="bg-emerald-50 text-emerald-700 border border-emerald-200/80 px-2 py-0.5 rounded text-xs font-semibold">
                  {s.allowed_count} ALLOW
                </span>
              )}
              {s.monitored_count > 0 && (
                <span className="bg-amber-50 text-amber-700 border border-amber-200/80 px-2 py-0.5 rounded text-xs font-semibold">
                  {s.monitored_count} MONITOR
                </span>
              )}
              {s.error_count > 0 && (
                <span className="bg-slate-100 text-slate-600 border border-slate-200 px-2 py-0.5 rounded text-xs font-semibold">
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
            <span className="text-slate-600 text-xs font-mono">
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
            className="inline-flex items-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-200 px-2.5 py-1 rounded-md text-xs font-semibold transition-colors cursor-pointer"
          >
            <Eye size={13} className="text-slate-500" />
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
            <span className="text-slate-600 text-xs font-mono">
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
              className="text-slate-900 font-semibold break-all cursor-pointer hover:text-sky-600 transition-colors"
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
          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-1.5 py-0.5 rounded text-2xs font-mono font-medium">
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
          return (
            <span
              className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full font-bold text-2xs border ${
                isBlock
                  ? 'bg-rose-50 text-rose-700 border-rose-200/80'
                  : isMonitor
                    ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                    : isError
                      ? 'bg-slate-100 text-slate-600 border-slate-200'
                      : 'bg-emerald-50 text-emerald-700 border-emerald-200/80'
              }`}
            >
              {isBlock || isError ? <ShieldAlert size={12} /> : <ShieldCheck size={12} />}
              {isBlock ? t.tag_block : isMonitor ? 'MONITOR' : isError ? 'ERROR' : t.tag_allow}
            </span>
          );
        },
      },
      {
        accessorKey: 'reason',
        header: t.th_reason,
        cell: ({ getValue }) => (
          <span className="text-slate-500 text-xs">{getValue<string>()}</span>
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
    <div className="bg-white border border-slate-200/80 rounded-2xl p-6 flex flex-col gap-5 shadow-xs relative">
      {/* Header & Controls */}
      <div className="flex justify-between items-center flex-wrap gap-3.5">
        <div>
          <div className="flex items-center gap-2.5">
            <h2 className="text-xl font-bold text-slate-900">{t.dns_logs_title}</h2>
            <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded-md text-xs font-semibold">
              {viewMode === 'summary'
                ? `${filteredSummaries.length} domains`
                : `${filteredLogs.length} queries`}
            </span>
          </div>
          <p className="text-sm text-slate-500 mt-0.5">{t.dns_logs_desc}</p>
        </div>

        <div className="flex items-center gap-2.5 flex-wrap">
          {/* View Mode Toggle: Summary (Default) vs Stream */}
          <div className="inline-flex bg-slate-100 p-1 rounded-xl border border-slate-200/80">
            <button
              onClick={() => setViewMode('summary')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                viewMode === 'summary'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <Layers size={13} />
              <span>{t.view_mode_summary}</span>
            </button>
            <button
              onClick={() => setViewMode('stream')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                viewMode === 'stream'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <List size={13} />
              <span>{t.view_mode_stream}</span>
            </button>
          </div>

          {/* Filter Pills */}
          <div className="inline-flex bg-slate-100 p-1 rounded-xl border border-slate-200/80 gap-1">
            <button
              onClick={() => setFilterType('all')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                filterType === 'all'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.filter_all}
            </button>
            <button
              onClick={() => setFilterType('blocked')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                filterType === 'blocked'
                  ? 'bg-rose-50 text-rose-700 border border-rose-200/80 shadow-xs'
                  : 'text-slate-600 hover:text-rose-600'
              }`}
            >
              {t.filter_blocked}
            </button>
            <button
              onClick={() => setFilterType('allowed')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                filterType === 'allowed'
                  ? 'bg-emerald-50 text-emerald-700 border border-emerald-200/80 shadow-xs'
                  : 'text-slate-600 hover:text-emerald-600'
              }`}
            >
              {t.filter_allowed}
            </button>
            <button
              onClick={() => setFilterType('monitored')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                filterType === 'monitored'
                  ? 'bg-amber-50 text-amber-700 border border-amber-200/80 shadow-xs'
                  : 'text-slate-600 hover:text-amber-600'
              }`}
            >
              {t.filter_monitored}
            </button>
            <button
              onClick={() => setFilterType('errors')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                filterType === 'errors'
                  ? 'bg-slate-200 text-slate-800 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.filter_error}
            </button>
          </div>

          {/* Auto Refresh Toggle */}
          <button
            onClick={onToggleAutoRefresh}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold border cursor-pointer transition-all ${
              autoRefresh
                ? 'bg-sky-50 text-sky-700 border-sky-200'
                : 'bg-white text-slate-600 border-slate-200 hover:bg-slate-50'
            }`}
          >
            {autoRefresh
              ? isVi
                ? 'Tự động: BẬT'
                : 'Auto: ON'
              : isVi
                ? 'Tự động: TẮT'
                : 'Auto: OFF'}
          </button>

          {/* Refresh Button */}
          <button
            onClick={() => {
              onRefresh();
              fetchSummaries();
            }}
            className="flex items-center gap-1.5 bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 hover:text-slate-900 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all"
          >
            <RefreshCw size={13} className={isLoadingSummaries ? 'animate-spin' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Storage Retention & Optimization Toggle */}
          <button
            onClick={() => setShowRetentionSettings(!showRetentionSettings)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border cursor-pointer transition-all ${
              showRetentionSettings
                ? 'bg-sky-50 text-sky-700 border-sky-300'
                : 'bg-white text-slate-700 border-slate-200 hover:bg-slate-50'
            }`}
            title={
              isVi
                ? 'Cấu hình thời gian lưu trữ & dọn dẹp dung lượng'
                : 'Storage retention & cleanup'
            }
          >
            <Database size={13} />
            <span>{isVi ? 'Tối Ưu Lưu Trữ' : 'Storage Optimization'}</span>
          </button>
        </div>
      </div>

      {/* Storage Optimization & Retention Banner */}
      {showRetentionSettings && (
        <div className="bg-slate-50 border border-slate-200 rounded-xl p-4 flex items-center justify-between flex-wrap gap-4 shadow-xs">
          <div className="max-w-2xl">
            <div className="flex items-center gap-2 mb-1">
              <Database size={16} className="text-sky-600" />
              <span className="text-sm font-bold text-slate-900">
                {isVi
                  ? 'Cơ Chế Nén & Tối Ưu Dung Lượng Nhật Ký DNS'
                  : 'DNS Query Logs Compression & Retention'}
              </span>
            </div>
            <p className="text-xs text-slate-600 m-0 leading-relaxed">
              {isVi
                ? 'Để tiết kiệm ổ cứng, hệ thống tự động tổng hợp số liệu của từng tên miền vào bảng thống kê vĩnh viễn và dọn dẹp các dòng log chi tiết từng giây đã quá hạn. Dữ liệu tổng hợp (Tổng truy vấn, Cho phép, Chặn) luôn được bảo toàn 100%.'
                : 'To minimize disk space, raw second-by-second queries are rolled up into persistent domain aggregates and pruned when expired. Domain totals are always preserved 100%.'}
            </p>
            <div className="flex gap-4 mt-2 text-xs text-slate-600 flex-wrap">
              <span>
                {isVi ? 'Bản ghi chi tiết:' : 'Raw queries:'}{' '}
                <strong className="text-sky-600">
                  {retentionConfig?.raw_query_logs_count ?? 0}
                </strong>
              </span>
              <span>
                {isVi ? 'Tên miền tổng hợp vĩnh viễn:' : 'Aggregated domains:'}{' '}
                <strong className="text-emerald-600">
                  {retentionConfig?.aggregated_domains_count ?? 0}
                </strong>
              </span>
              {retentionConfig?.last_pruned_at && (
                <span className="text-slate-500">
                  {isVi ? 'Dọn dẹp gần nhất:' : 'Last pruned:'}{' '}
                  {new Date(retentionConfig.last_pruned_at).toLocaleTimeString()}
                </span>
              )}
            </div>
          </div>

          <div className="flex items-center gap-3 flex-wrap">
            <div className="flex items-center gap-2">
              <label className="text-xs text-slate-600 font-semibold">
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
                className="bg-white border border-slate-200 text-slate-800 px-2.5 py-1.5 rounded-lg text-xs cursor-pointer outline-none focus:border-sky-500"
              >
                <option value={1}>
                  {isVi ? '1 ngày (Tiết kiệm tối đa)' : '1 day (Max savings)'}
                </option>
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
              className="inline-flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white border-0 px-3.5 py-1.5 rounded-lg text-xs font-semibold cursor-pointer shadow-xs transition-all"
            >
              <Trash2 size={13} className={isPruningNow ? 'animate-spin' : ''} />
              {isPruningNow
                ? isVi
                  ? 'Đang dọn dẹp...'
                  : 'Pruning...'
                : isVi
                  ? 'Thu Gọn & Dọn Dẹp Ngay'
                  : 'Prune Now'}
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
          className="fixed inset-0 bg-slate-900/40 backdrop-blur-xs flex items-center justify-center z-50 p-4"
          onClick={handleCloseDomainDetail}
        >
          <div
            className="bg-white border border-slate-200 rounded-2xl w-full max-w-3xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Modal Header */}
            <div className="p-5 sm:p-6 border-b border-slate-100 flex justify-between items-center flex-wrap gap-3 bg-white">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-sky-50 text-sky-600 flex items-center justify-center shrink-0">
                  <Globe size={20} />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-slate-900 break-all">{selectedDomain}</h3>
                  <p className="text-xs text-slate-500">
                    {t.modal_domain_desc}:{' '}
                    <span className="text-sky-600 font-medium">{selectedDomain}</span>
                  </p>
                </div>
              </div>

              <div className="flex items-center gap-2.5">
                {/* Quick Block Button */}
                <button
                  onClick={() => handleQuickBlockDomain(selectedDomain)}
                  disabled={isBlocking}
                  className="inline-flex items-center gap-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 px-3.5 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all disabled:opacity-50"
                >
                  <Ban size={13} />
                  <span>{isBlocking ? 'Đang chặn...' : t.btn_quick_block}</span>
                </button>

                {/* Close Button */}
                <button
                  onClick={handleCloseDomainDetail}
                  className="w-8 h-8 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 flex items-center justify-center cursor-pointer transition-colors border-0"
                >
                  <X size={16} />
                </button>
              </div>
            </div>

            {/* Modal Subheader with Stats */}
            {activeSummary && (
              <div className="px-6 py-3 bg-slate-50 border-b border-slate-100 flex gap-4 flex-wrap text-xs text-slate-600 font-medium">
                <div className="flex items-center gap-1.5">
                  <span>{t.th_total_queries}:</span>
                  <strong className="text-sky-700 font-bold">{activeSummary.total_queries}</strong>
                </div>
                <div className="flex items-center gap-1.5">
                  <span>ALLOW:</span>
                  <strong className="text-emerald-700 font-bold">
                    {activeSummary.allowed_count}
                  </strong>
                </div>
                <div className="flex items-center gap-1.5">
                  <span>BLOCK:</span>
                  <strong className="text-rose-700 font-bold">{activeSummary.blocked_count}</strong>
                </div>
                {activeSummary.monitored_count > 0 && (
                  <div className="flex items-center gap-1.5">
                    <span>MONITOR:</span>
                    <strong className="text-amber-700 font-bold">
                      {activeSummary.monitored_count}
                    </strong>
                  </div>
                )}
                <div className="flex items-center gap-1.5">
                  <span>{t.th_last_seen}:</span>
                  <strong className="text-slate-800 font-mono">
                    {new Date(activeSummary.last_seen_at).toLocaleTimeString()}
                  </strong>
                </div>
              </div>
            )}

            {/* Modal Body: Query Log Table for Domain */}
            <div className="flex-1 overflow-y-auto p-5">
              {isLoadingDetails ? (
                <div className="text-center py-10 text-slate-500">
                  <RefreshCw size={24} className="animate-spin mx-auto mb-3 text-sky-600" />
                  <p className="text-sm">
                    {isVi
                      ? 'Đang truy xuất chi tiết truy vấn của tên miền...'
                      : 'Loading query resolution details...'}
                  </p>
                </div>
              ) : domainLogs.length === 0 ? (
                <div className="text-center py-10 text-slate-500 text-sm">
                  <p>{t.no_matching_logs}</p>
                </div>
              ) : (
                <table className="w-full border-collapse text-xs text-left">
                  <thead>
                    <tr className="border-b border-slate-200 bg-slate-50 text-slate-600 font-semibold">
                      <th className="py-2.5 px-3">{t.th_time}</th>
                      <th className="py-2.5 px-3">{t.th_type}</th>
                      <th className="py-2.5 px-3">{t.th_action}</th>
                      <th className="py-2.5 px-3">{t.th_reason}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {domainLogs.map((item, idx) => {
                      const isBlock = item.action === 'BLOCK';
                      const isMonitor = item.action === 'MONITOR';
                      const isError = item.action === 'ERROR';
                      return (
                        <tr
                          key={idx}
                          className="border-b border-slate-100 hover:bg-slate-50/50 transition-colors"
                        >
                          <td className="py-2.5 px-3 text-slate-700 font-mono">
                            {new Date(item.occurred_at).toLocaleTimeString()}{' '}
                            <span className="text-slate-400 text-2xs">
                              ({new Date(item.occurred_at).toLocaleDateString()})
                            </span>
                          </td>
                          <td className="py-2.5 px-3">
                            <span className="bg-slate-100 text-slate-700 border border-slate-200 px-1.5 py-0.5 rounded text-2xs font-mono font-medium">
                              {item.qtype}
                            </span>
                          </td>
                          <td className="py-2.5 px-3">
                            <span
                              className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full font-bold text-2xs border ${
                                isBlock || isError
                                  ? 'bg-rose-50 text-rose-700 border-rose-200/80'
                                  : isMonitor
                                    ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                                    : 'bg-emerald-50 text-emerald-700 border-emerald-200/80'
                              }`}
                            >
                              {isBlock || isError ? (
                                <ShieldAlert size={11} />
                              ) : (
                                <ShieldCheck size={11} />
                              )}
                              {item.action}
                            </span>
                          </td>
                          <td className="py-2.5 px-3 text-slate-600 text-xs">{item.reason}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              )}
            </div>

            {/* Modal Footer */}
            <div className="px-6 py-3.5 border-t border-slate-100 flex justify-end bg-slate-50">
              <button
                onClick={handleCloseDomainDetail}
                className="bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 px-4 py-1.5 rounded-lg text-xs font-semibold cursor-pointer shadow-xs transition-all"
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
