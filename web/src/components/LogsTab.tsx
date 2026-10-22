import type { ColumnDef } from '@tanstack/react-table';
import {
  Ban,
  BarChart3,
  Calendar,
  ChevronDown,
  ChevronUp,
  Clock,
  Database,
  Eye,
  Flame,
  Globe,
  Layers,
  List,
  RefreshCw,
  RotateCcw,
  ShieldAlert,
  ShieldCheck,
  Trash2,
  X,
} from 'lucide-react';
import type React from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type {
  DnsRetentionConfig,
  DomainQuerySummary,
  HourlyFrequency,
  Language,
  QueryLogRecord,
} from '../types';
import { DataTable } from './DataTable';
import { ResetReportModal } from './ResetReportModal';
import { DnsHourlyChart } from './charts/DnsHourlyChart';

interface LogsTabProps {
  logs: QueryLogRecord[];
  lang: Language;
  onRefresh: () => void;
  autoRefresh: boolean;
  onToggleAutoRefresh: () => void;
  onShowToast?: (message: string, type?: 'success' | 'error' | 'info') => void;
}

type TimePreset = 'today' | '24h' | '7d' | 'all' | 'custom';

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

  // Time Range Filter State
  const [timePreset, setTimePreset] = useState<TimePreset>('today');
  const [customStartDate, setCustomStartDate] = useState<string>('');
  const [showResetModal, setShowResetModal] = useState<boolean>(false);
  const [customEndDate, setCustomEndDate] = useState<string>('');

  // Aggregated domain summary state
  const [summaries, setSummaries] = useState<DomainQuerySummary[]>([]);
  const [filteredStreamLogs, setFilteredStreamLogs] = useState<QueryLogRecord[]>(logs);

  // Loading & refresh states: differentiate initial full fetch vs silent background sync
  const [isInitialLoading, setIsInitialLoading] = useState<boolean>(true);
  const [isBackgroundRefreshing, setIsBackgroundRefreshing] = useState<boolean>(false);
  const [isLoadingSummaries, setIsLoadingSummaries] = useState<boolean>(false);
  const [isLoadingHourly, setIsLoadingHourly] = useState<boolean>(false);
  const hasLoadedOnceRef = useRef<boolean>(false);
  const logsRef = useRef<QueryLogRecord[]>(logs);
  logsRef.current = logs;

  // Hourly Frequency Chart State
  const [hourlyData, setHourlyData] = useState<HourlyFrequency[]>([]);
  const [showHourlyChart, setShowHourlyChart] = useState<boolean>(true);

  // Drilldown detail modal state
  const [selectedDomain, setSelectedDomain] = useState<string | null>(null);
  const [domainLogs, setDomainLogs] = useState<QueryLogRecord[]>([]);
  const [domainHourlyData, setDomainHourlyData] = useState<HourlyFrequency[]>([]);
  const [isLoadingDetails, setIsLoadingDetails] = useState<boolean>(false);
  const [isBlocking, setIsBlocking] = useState<boolean>(false);

  // Retention & Pruning State
  const [retentionConfig, setRetentionConfig] = useState<DnsRetentionConfig | null>(null);
  const [showRetentionSettings, setShowRetentionSettings] = useState<boolean>(false);
  const [selectedRetentionDays, setSelectedRetentionDays] = useState<number>(7);
  const [autoPruneEnabled, setAutoPruneEnabled] = useState<boolean>(true);
  const [isSavingRetention, setIsSavingRetention] = useState<boolean>(false);
  const [isPruningNow, setIsPruningNow] = useState<boolean>(false);

  // Compute ISO time range strings from preset or custom inputs
  const computeTimeRange = useCallback((): {
    startTime?: string;
    endTime?: string;
  } => {
    const now = new Date();
    if (timePreset === 'today') {
      const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate(), 0, 0, 0, 0);
      return { startTime: todayStart.toISOString(), endTime: now.toISOString() };
    }
    if (timePreset === '24h') {
      const past24h = new Date(now.getTime() - 24 * 3600 * 1000);
      return { startTime: past24h.toISOString(), endTime: now.toISOString() };
    }
    if (timePreset === '7d') {
      const past7d = new Date(now.getTime() - 7 * 24 * 3600 * 1000);
      return { startTime: past7d.toISOString(), endTime: now.toISOString() };
    }
    if (timePreset === 'custom') {
      const s = customStartDate ? new Date(customStartDate + 'T00:00:00').toISOString() : undefined;
      const e = customEndDate ? new Date(customEndDate + 'T23:59:59').toISOString() : undefined;
      return { startTime: s, endTime: e };
    }
    return {};
  }, [timePreset, customStartDate, customEndDate]);

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
      await fetchTelemetryData(false);
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast?.(msg || (isVi ? 'Lỗi dọn dẹp nhật ký' : 'Failed to prune logs'), 'error');
    } finally {
      setIsPruningNow(false);
    }
  };

  // Unified telemetry fetcher: handles initial load, manual refresh, and silent background polling
  const fetchTelemetryData = useCallback(
    async (silent = false) => {
      if (!hasLoadedOnceRef.current && !silent) {
        setIsInitialLoading(true);
      } else if (!silent) {
        setIsLoadingSummaries(true);
      } else {
        setIsBackgroundRefreshing(true);
      }

      const { startTime, endTime } = computeTimeRange();

      try {
        const promises: [
          Promise<DomainQuerySummary[]>,
          Promise<HourlyFrequency[]>,
          Promise<QueryLogRecord[] | null>,
        ] = [
          api.getDnsSummary(500, startTime, endTime),
          api.getDnsHourly(undefined, startTime, endTime),
          viewMode === 'stream'
            ? api.getLogs(300, undefined, startTime, endTime)
            : Promise.resolve(null),
        ];

        const [summaryData, hourlyRes, streamData] = await Promise.all(promises);

        setSummaries(summaryData);
        setHourlyData(hourlyRes);
        if (streamData) {
          setFilteredStreamLogs(streamData);
        }
        hasLoadedOnceRef.current = true;
      } catch {
        // Fallback: aggregate in memory from logsRef if summary endpoint fails
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
        for (const l of logsRef.current) {
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
        setFilteredStreamLogs(logsRef.current);
      } finally {
        setIsInitialLoading(false);
        setIsLoadingSummaries(false);
        setIsLoadingHourly(false);
        setIsBackgroundRefreshing(false);
      }
    },
    [computeTimeRange, viewMode]
  );

  // Fetch when time filter or view mode changes
  useEffect(() => {
    fetchTelemetryData(false);
  }, [fetchTelemetryData]);

  // Periodic silent background auto-refresh (zero layout shift, zero chart unmounting)
  useEffect(() => {
    if (!autoRefresh) return;
    const interval = setInterval(() => {
      fetchTelemetryData(true);
    }, 5000);
    return () => clearInterval(interval);
  }, [autoRefresh, fetchTelemetryData]);

  // Open detail drilldown for a specific domain
  const handleOpenDomainDetail = async (domain: string) => {
    setSelectedDomain(domain);
    setIsLoadingDetails(true);
    const { startTime, endTime } = computeTimeRange();
    try {
      const [details, hourly] = await Promise.all([
        api.getLogs(200, domain, startTime, endTime),
        api.getDnsHourly(domain, startTime, endTime),
      ]);
      setDomainLogs(details);
      setDomainHourlyData(hourly);
    } catch {
      // Fallback: filter from loaded logs
      const dLogs = logs.filter((l) => l.domain === domain);
      setDomainLogs(dLogs);
      const empty = Array.from({ length: 24 }, (_, i) => ({
        hour: i,
        total_count: 0,
        allowed_count: 0,
        blocked_count: 0,
      }));
      setDomainHourlyData(empty);
    } finally {
      setIsLoadingDetails(false);
    }
  };

  const handleCloseDomainDetail = () => {
    setSelectedDomain(null);
    setDomainLogs([]);
    setDomainHourlyData([]);
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
      fetchTelemetryData(false);
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
  const activeStreamLogs = useMemo(() => {
    const src = filteredStreamLogs.length > 0 ? filteredStreamLogs : logs;
    return src.filter((log) => {
      if (filterType === 'blocked' && log.action !== 'BLOCK') return false;
      if (filterType === 'allowed' && log.action !== 'ALLOW') return false;
      if (filterType === 'monitored' && log.action !== 'MONITOR') return false;
      if (filterType === 'errors' && log.action !== 'ERROR') return false;
      return true;
    });
  }, [filteredStreamLogs, logs, filterType]);

  // Calculated hourly peak and max
  const { maxHourlyCount, peakHour } = useMemo(() => {
    let max = 1;
    let peak = 0;
    for (const item of hourlyData) {
      if (item.total_count > max) {
        max = item.total_count;
        peak = item.hour;
      }
    }
    return { maxHourlyCount: max, peakHour: peak };
  }, [hourlyData]);

  // Total queries in current hourly range
  const totalHourlyQueries = useMemo(() => {
    return hourlyData.reduce((acc, h) => acc + h.total_count, 0);
  }, [hourlyData]);

  // Group domain logs by time periods for daily timeline drilldown
  const domainTimeClusters = useMemo(() => {
    if (!domainLogs.length) return null;
    const morning: string[] = []; // 06:00 - 12:00
    const afternoon: string[] = []; // 12:00 - 18:00
    const evening: string[] = []; // 18:00 - 22:00
    const night: string[] = []; // 22:00 - 06:00

    for (const log of domainLogs) {
      const dt = new Date(log.occurred_at);
      const h = dt.getHours();
      const timeStr = dt.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      if (h >= 6 && h < 12) {
        if (!morning.includes(timeStr)) morning.push(timeStr);
      } else if (h >= 12 && h < 18) {
        if (!afternoon.includes(timeStr)) afternoon.push(timeStr);
      } else if (h >= 18 && h < 22) {
        if (!evening.includes(timeStr)) evening.push(timeStr);
      } else {
        if (!night.includes(timeStr)) night.push(timeStr);
      }
    }

    return { morning, afternoon, evening, night };
  }, [domainLogs]);

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
              title={
                isVi
                  ? 'Nhấp để xem chi tiết timeline và tần suất truy cập'
                  : 'Click to view timeline and access frequency'
              }
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
            <Clock size={13} className="text-slate-500" />
            <span>{isVi ? 'Timeline & Chi tiết' : 'Timeline & Details'}</span>
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
              {new Date(val).toLocaleTimeString()} ({new Date(val).toLocaleDateString()})
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
              title={isVi ? 'Nhấp để xem timeline tên miền này' : 'Click to drill down'}
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
                : `${activeStreamLogs.length} queries`}
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
            type="button"
            onClick={() => {
              fetchTelemetryData(false);
              onRefresh();
            }}
            disabled={isLoadingSummaries}
            className="flex items-center gap-1.5 bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 hover:text-slate-900 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all disabled:opacity-60"
          >
            <RefreshCw
              size={13}
              className={isLoadingSummaries || isBackgroundRefreshing ? 'animate-spin' : ''}
            />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Toggle Hourly Frequency Chart */}
          <button
            onClick={() => setShowHourlyChart(!showHourlyChart)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border cursor-pointer transition-all ${
              showHourlyChart
                ? 'bg-sky-50 text-sky-700 border-sky-200'
                : 'bg-white text-slate-700 border-slate-200 hover:bg-slate-50'
            }`}
            title={isVi ? 'Bật/tắt biểu đồ tần suất 24h' : 'Toggle 24h frequency chart'}
          >
            <BarChart3 size={13} />
            <span>{isVi ? 'Biểu đồ 24h' : '24h Chart'}</span>
            {showHourlyChart ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
          </button>

          {/* Storage Retention & Optimization Toggle */}
          <button
            onClick={() => setShowRetentionSettings(!showRetentionSettings)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border cursor-pointer transition-all ${
              showRetentionSettings
                ? 'bg-slate-900 text-white border-slate-900 shadow-xs'
                : 'bg-white text-slate-700 border-slate-200 hover:bg-slate-50 hover:text-slate-900'
            }`}
          >
            <Database size={13} />
            <span>{isVi ? 'Lưu trữ & Dọn dẹp' : 'Storage & Pruning'}</span>
          </button>

          {/* Reset DNS Reports */}
          <button
            type="button"
            onClick={() => setShowResetModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-rose-50 text-rose-700 border border-rose-200/80 hover:bg-rose-100 cursor-pointer transition-all shadow-2xs"
            title={isVi ? 'Đặt lại nhật ký DNS về 0' : 'Reset DNS logs to 0'}
          >
            <RotateCcw size={13} />
            <span>{t.btn_reset_reports}</span>
          </button>
        </div>
      </div>

      {/* TIME RANGE FILTER BAR */}
      <div className="bg-slate-50 border border-slate-200/80 rounded-xl p-3 flex items-center justify-between flex-wrap gap-3">
        <div className="flex items-center gap-2 flex-wrap">
          <div className="flex items-center gap-1.5 text-xs text-slate-600 font-semibold mr-1">
            <Calendar size={14} className="text-sky-600" />
            <span>{isVi ? 'Khoảng thời gian:' : 'Time Range:'}</span>
          </div>

          <div className="inline-flex bg-white p-0.5 rounded-lg border border-slate-200 shadow-2xs gap-0.5">
            <button
              onClick={() => setTimePreset('today')}
              className={`px-3 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                timePreset === 'today'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.time_preset_today}
            </button>
            <button
              onClick={() => setTimePreset('24h')}
              className={`px-3 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                timePreset === '24h'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.time_preset_24h}
            </button>
            <button
              onClick={() => setTimePreset('7d')}
              className={`px-3 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                timePreset === '7d'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.time_preset_7d}
            </button>
            <button
              onClick={() => setTimePreset('all')}
              className={`px-3 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                timePreset === 'all'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.time_preset_all}
            </button>
            <button
              onClick={() => setTimePreset('custom')}
              className={`px-3 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                timePreset === 'custom'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {t.time_preset_custom}
            </button>
          </div>

          {/* Custom Date Inputs if Custom selected */}
          {timePreset === 'custom' && (
            <div className="flex items-center gap-2 ml-2 animate-in fade-in duration-200">
              <label className="text-2xs text-slate-500 font-medium">{t.lbl_start_time}</label>
              <input
                type="date"
                value={customStartDate}
                onChange={(e) => setCustomStartDate(e.target.value)}
                className="bg-white border border-slate-200 text-slate-800 px-2 py-0.5 rounded text-xs outline-none focus:border-sky-500"
              />
              <label className="text-2xs text-slate-500 font-medium">{t.lbl_end_time}</label>
              <input
                type="date"
                value={customEndDate}
                onChange={(e) => setCustomEndDate(e.target.value)}
                className="bg-white border border-slate-200 text-slate-800 px-2 py-0.5 rounded text-xs outline-none focus:border-sky-500"
              />
            </div>
          )}
        </div>

        {/* Peak hour notice badge */}
        {totalHourlyQueries > 0 && (
          <div className="inline-flex items-center gap-1.5 text-xs text-amber-700 bg-amber-50 border border-amber-200/80 px-2.5 py-1 rounded-lg">
            <Flame size={13} className="text-amber-500" />
            <span>
              {t.peak_hour}{' '}
              <strong>
                {peakHour.toString().padStart(2, '0')}:00 -{' '}
                {(peakHour + 1).toString().padStart(2, '0')}:00
              </strong>
            </span>
          </div>
        )}
      </div>

      {/* 24-HOUR HOURLY ACCESS FREQUENCY CHART */}
      {showHourlyChart && (
        <div className="bg-slate-50/70 border border-slate-200/80 rounded-2xl p-4 sm:p-5 flex flex-col gap-3.5 transition-all">
          <div className="flex justify-between items-center flex-wrap gap-2">
            <div>
              <h3 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                <BarChart3 size={16} className="text-sky-600" />
                <span>{t.hourly_chart_title}</span>
                {isBackgroundRefreshing && (
                  <span className="inline-flex items-center gap-1 text-2xs text-sky-600 font-normal">
                    <RefreshCw size={10} className="animate-spin" />
                    <span className="hidden sm:inline">
                      {isVi ? 'Đang đồng bộ...' : 'Syncing...'}
                    </span>
                  </span>
                )}
              </h3>
              <p className="text-2xs sm:text-xs text-slate-500">{t.hourly_chart_desc}</p>
            </div>

            <div className="flex items-center gap-3 text-2xs font-semibold">
              <span className="flex items-center gap-1.5 text-sky-700">
                <span className="w-2.5 h-2.5 rounded-sm bg-sky-500 inline-block" />
                {t.filter_allowed}
              </span>
              <span className="flex items-center gap-1.5 text-rose-700">
                <span className="w-2.5 h-2.5 rounded-sm bg-rose-500 inline-block" />
                {t.filter_blocked}
              </span>
              <span className="text-slate-500">
                {t.th_total_queries}:{' '}
                <strong className="text-slate-800">{totalHourlyQueries.toLocaleString()}</strong>
              </span>
            </div>
          </div>

          {/* Chart.js 24-Hour Access Frequency Chart */}
          <div className="relative pt-2 pb-1 min-h-[170px]">
            {isInitialLoading && hourlyData.length === 0 ? (
              <div className="py-12 text-center text-slate-400 text-xs flex items-center justify-center gap-2">
                <RefreshCw size={14} className="animate-spin text-sky-600" />
                <span>{isVi ? 'Đang nạp phân bố giờ...' : 'Loading hourly chart...'}</span>
              </div>
            ) : totalHourlyQueries === 0 && !isInitialLoading ? (
              <div className="py-12 text-center text-slate-400 text-xs">
                {t.no_activity_time} ({t.time_preset_all})
              </div>
            ) : (
              <DnsHourlyChart
                hourlyData={hourlyData}
                lang={lang}
                peakHour={peakHour}
                height={170}
              />
            )}
          </div>
        </div>
      )}

      {/* Storage Retention & Pruning Drawer */}
      {showRetentionSettings && retentionConfig && (
        <div className="bg-slate-50 border border-slate-200 rounded-2xl p-4 sm:p-5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 animate-in fade-in duration-200">
          <div>
            <h3 className="text-sm font-bold text-slate-900 flex items-center gap-2">
              <Database size={15} className="text-sky-600" />
              <span>{isVi ? 'Tối Ưu Hóa Bộ Nhớ SQLite' : 'SQLite Storage Retention'}</span>
            </h3>
            <p className="text-xs text-slate-500 mt-0.5">
              {isVi
                ? 'Tự động dọn dẹp các truy vấn thô cũ hơn số ngày quy định, giữ lại bảng tổng hợp tên miền nhẹ dưới 2MB.'
                : 'Raw query logs older than retention period are pruned into domain aggregates to keep DB lightweight.'}
            </p>
            <div className="flex gap-4 mt-2 text-2xs font-semibold text-slate-600 flex-wrap">
              <span>
                {isVi ? 'Truy vấn chi tiết hiện có:' : 'Raw query logs:'}{' '}
                <strong className="text-slate-800">
                  {retentionConfig.raw_query_logs_count.toLocaleString()}
                </strong>
              </span>
              <span>
                {isVi ? 'Tên miền đã tổng hợp:' : 'Aggregated domains:'}{' '}
                <strong className="text-slate-800">
                  {retentionConfig.aggregated_domains_count.toLocaleString()}
                </strong>
              </span>
              {retentionConfig.last_pruned_at && (
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
          data={activeStreamLogs}
          columns={rawLogColumns}
          pageSize={10}
          searchPlaceholder={t.placeholder_search_logs}
          emptyText={t.no_matching_logs}
          lang={lang}
        />
      )}

      {/* DRILLDOWN DETAIL MODAL WITH TIMELINE & ACCESS PATTERNS FOR A SPECIFIC DOMAIN */}
      {selectedDomain && (
        <div
          className="fixed inset-0 bg-slate-900/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-150"
          onClick={handleCloseDomainDetail}
        >
          <div
            className="bg-white border border-slate-200 rounded-2xl w-full max-w-3xl max-h-[92vh] flex flex-col shadow-2xl overflow-hidden"
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
                    <span className="text-sky-600 font-semibold">{selectedDomain}</span>
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

            {/* Modal Subheader with Summary Stats */}
            {activeSummary && (
              <div className="px-6 py-2.5 bg-slate-50 border-b border-slate-100 flex gap-4 flex-wrap text-xs text-slate-600 font-medium">
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
                    {new Date(activeSummary.last_seen_at).toLocaleTimeString()}{' '}
                    <span className="text-slate-400 font-normal">
                      ({new Date(activeSummary.last_seen_at).toLocaleDateString()})
                    </span>
                  </strong>
                </div>
              </div>
            )}

            {/* Modal Body: Timeline & Query Log Table for Domain */}
            <div className="flex-1 overflow-y-auto p-5 sm:p-6 flex flex-col gap-5">
              {/* 1. VISUAL 24-HOUR ACCESS TIMELINE RIBBON */}
              <div className="bg-sky-50/60 border border-sky-100 rounded-xl p-4 flex flex-col gap-2.5">
                <div className="flex justify-between items-center text-xs">
                  <span className="font-bold text-slate-900 flex items-center gap-1.5">
                    <Clock size={14} className="text-sky-600" />
                    <span>{t.domain_timeline_title}</span>
                  </span>
                  <span className="text-2xs text-slate-500 font-medium font-mono">
                    00:00 ─── 24:00
                  </span>
                </div>

                {/* 24-segment timeline ribbon */}
                <div className="grid grid-cols-24 gap-0.5 h-6 bg-slate-200/60 p-0.5 rounded-md overflow-hidden">
                  {domainHourlyData.map((h) => {
                    const hasTraffic = h.total_count > 0;
                    const hasBlock = h.blocked_count > 0;
                    return (
                      <div
                        key={h.hour}
                        title={`${h.hour.toString().padStart(2, '0')}:00 - ${h.total_count} ${
                          isVi ? 'truy vấn' : 'queries'
                        }`}
                        className={`h-full rounded-xs transition-all ${
                          !hasTraffic
                            ? 'bg-slate-100'
                            : hasBlock
                              ? 'bg-rose-500 hover:brightness-110'
                              : 'bg-sky-500 hover:brightness-110'
                        }`}
                      />
                    );
                  })}
                </div>

                <div className="flex justify-between text-3xs font-mono text-slate-400 px-0.5">
                  <span>00h</span>
                  <span>06h</span>
                  <span>12h</span>
                  <span>18h</span>
                  <span>24h</span>
                </div>
              </div>

              {/* 2. TIME-OF-DAY ACCESS CLUSTERS (Các khung giờ trang web này được truy cập) */}
              {domainTimeClusters && (
                <div className="bg-slate-50 border border-slate-200/80 rounded-xl p-3.5 flex flex-col gap-2">
                  <span className="text-xs font-bold text-slate-800">
                    {isVi
                      ? '📍 Các thời điểm truy cập trong ngày'
                      : '📍 Access intervals throughout the day'}
                  </span>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-2xs">
                    {/* Morning */}
                    <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                      <span className="font-semibold text-amber-700 flex items-center gap-1">
                        <span>🌅</span> {t.time_morning}
                      </span>
                      {domainTimeClusters.morning.length > 0 ? (
                        <div className="flex gap-1 flex-wrap mt-0.5">
                          {domainTimeClusters.morning.map((ts, idx) => (
                            <span
                              key={idx}
                              className="bg-amber-50 text-amber-900 border border-amber-200 px-1.5 py-0.5 rounded font-mono"
                            >
                              {ts}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400 italic text-3xs">
                          {isVi ? 'Không có truy cập' : 'No activity'}
                        </span>
                      )}
                    </div>

                    {/* Afternoon */}
                    <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                      <span className="font-semibold text-sky-700 flex items-center gap-1">
                        <span>☀️</span> {t.time_afternoon}
                      </span>
                      {domainTimeClusters.afternoon.length > 0 ? (
                        <div className="flex gap-1 flex-wrap mt-0.5">
                          {domainTimeClusters.afternoon.map((ts, idx) => (
                            <span
                              key={idx}
                              className="bg-sky-50 text-sky-900 border border-sky-200 px-1.5 py-0.5 rounded font-mono"
                            >
                              {ts}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400 italic text-3xs">
                          {isVi ? 'Không có truy cập' : 'No activity'}
                        </span>
                      )}
                    </div>

                    {/* Evening */}
                    <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                      <span className="font-semibold text-indigo-700 flex items-center gap-1">
                        <span>🌆</span> {t.time_evening}
                      </span>
                      {domainTimeClusters.evening.length > 0 ? (
                        <div className="flex gap-1 flex-wrap mt-0.5">
                          {domainTimeClusters.evening.map((ts, idx) => (
                            <span
                              key={idx}
                              className="bg-indigo-50 text-indigo-900 border border-indigo-200 px-1.5 py-0.5 rounded font-mono"
                            >
                              {ts}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400 italic text-3xs">
                          {isVi ? 'Không có truy cập' : 'No activity'}
                        </span>
                      )}
                    </div>

                    {/* Night */}
                    <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                      <span className="font-semibold text-purple-700 flex items-center gap-1">
                        <span>🌙</span> {t.time_night}
                      </span>
                      {domainTimeClusters.night.length > 0 ? (
                        <div className="flex gap-1 flex-wrap mt-0.5">
                          {domainTimeClusters.night.map((ts, idx) => (
                            <span
                              key={idx}
                              className="bg-purple-50 text-purple-900 border border-purple-200 px-1.5 py-0.5 rounded font-mono"
                            >
                              {ts}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400 italic text-3xs">
                          {isVi ? 'Không có truy cập' : 'No activity'}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              )}

              {/* 2.5. CHART.JS HOURLY FREQUENCY FOR DOMAIN */}
              <div className="bg-slate-50 border border-slate-200/80 rounded-xl p-3.5 flex flex-col gap-2">
                <div className="flex justify-between items-center text-xs">
                  <span className="font-bold text-slate-800 flex items-center gap-1.5">
                    <BarChart3 size={14} className="text-sky-600" />
                    <span>
                      {isVi
                        ? 'Biểu đồ tần suất truy vấn theo giờ của tên miền'
                        : 'Domain 24h Query Frequency Chart'}
                    </span>
                  </span>
                  <span className="text-2xs text-slate-500 font-mono">
                    {domainLogs.length} queries
                  </span>
                </div>
                <div className="bg-white rounded-lg p-2.5 border border-slate-200/60">
                  <DnsHourlyChart
                    hourlyData={domainHourlyData}
                    lang={lang}
                    peakHour={-1}
                    height={110}
                    allowChartTypeToggle={false}
                  />
                </div>
              </div>

              {/* 3. Query Log Table for Domain */}
              <div>
                <span className="text-xs font-bold text-slate-800 mb-2 block">
                  {isVi ? '📋 Lịch sử các truy vấn chi tiết' : '📋 Detailed Query Log History'}
                </span>
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
                  <div className="border border-slate-200 rounded-xl overflow-hidden">
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
                  </div>
                )}
              </div>
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

      {/* Reset Report Modal */}
      <ResetReportModal
        isOpen={showResetModal}
        onClose={() => setShowResetModal(false)}
        onSuccess={() => {
          fetchTelemetryData(false);
          onRefresh();
        }}
        onShowToast={onShowToast}
        lang={lang}
        defaultScope="dns"
      />
    </div>
  );
};
