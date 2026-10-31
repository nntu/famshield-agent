import type { ColumnDef } from '@tanstack/react-table';
import {
  Ban,
  BarChart3,
  Calendar,
  ChevronDown,
  ChevronUp,
  Clock,
  Database,
  Download,
  Eye,
  Flame,
  Globe,
  Layers,
  List,
  RefreshCw,
  RotateCcw,
  ShieldAlert,
  ShieldCheck,
  Terminal,
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
import { ErrorManagementView } from './ErrorManagementView';
import { ResetReportModal } from './ResetReportModal';
import { SystemLogsView } from './SystemLogsView';
import { DomainDrilldownModal } from './logs/DomainDrilldownModal';
import { LogsChartCard } from './logs/LogsChartCard';
import { LogsFilterBar } from './logs/LogsFilterBar';
import { StorageRetentionCard } from './logs/StorageRetentionCard';

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

  // Subtabs: 'dns' (DNS queries) | 'system' (famshield-service.log) | 'errors' (error telemetry & NATS)
  const [logSubTab, setLogSubTab] = useState<'dns' | 'system' | 'errors'>('dns');

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

  // Export DNS logs to CSV
  const handleExportDnsCsv = () => {
    const listToExport = viewMode === 'stream' ? activeStreamLogs : logs;
    if (!listToExport || listToExport.length === 0) {
      onShowToast?.(
        isVi ? 'Không có dữ liệu nhật ký DNS để xuất!' : 'No DNS query logs to export!',
        'info'
      );
      return;
    }
    const headers = ['Timestamp', 'Domain', 'QueryType', 'Action', 'Reason'];
    const rows = listToExport.map((l) => [
      new Date(l.occurred_at).toISOString(),
      `"${l.domain.replace(/"/g, '""')}"`,
      l.qtype,
      l.action,
      `"${(l.reason || '').replace(/"/g, '""')}"`,
    ]);
    const csvContent = '\uFEFF' + [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `famshield-dns-logs-${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    onShowToast?.(
      isVi ? 'Đã xuất nhật ký DNS (CSV) thành công!' : 'DNS logs exported to CSV successfully!',
      'success'
    );
  };

  return (
    <div className="bg-white border border-slate-200/80 rounded-2xl p-6 flex flex-col gap-5 shadow-xs relative">
      {/* Subtab Navigation Pills */}
      <div className="flex items-center gap-2 border-b border-slate-200/80 pb-4 flex-wrap">
        <button
          onClick={() => setLogSubTab('dns')}
          className={`inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer ${
            logSubTab === 'dns'
              ? 'bg-sky-600 text-white shadow-xs shadow-sky-500/20'
              : 'bg-slate-100 text-slate-600 hover:bg-slate-200/70 hover:text-slate-900'
          }`}
        >
          <Globe size={15} />
          <span>{t.logs_subtab_dns}</span>
        </button>

        <button
          onClick={() => setLogSubTab('system')}
          className={`inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer ${
            logSubTab === 'system'
              ? 'bg-sky-600 text-white shadow-xs shadow-sky-500/20'
              : 'bg-slate-100 text-slate-600 hover:bg-slate-200/70 hover:text-slate-900'
          }`}
        >
          <Terminal size={15} />
          <span>{t.logs_subtab_system}</span>
        </button>

        <button
          onClick={() => setLogSubTab('errors')}
          className={`inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer ${
            logSubTab === 'errors'
              ? 'bg-rose-600 text-white shadow-xs shadow-rose-500/20'
              : 'bg-slate-100 text-slate-600 hover:bg-slate-200/70 hover:text-slate-900'
          }`}
        >
          <ShieldAlert size={15} />
          <span>{t.logs_subtab_errors}</span>
        </button>
      </div>

      {logSubTab === 'system' && (
        <SystemLogsView lang={lang} onShowToast={onShowToast} />
      )}

      {logSubTab === 'errors' && (
        <ErrorManagementView lang={lang} onShowToast={onShowToast} />
      )}

      {logSubTab === 'dns' && (
        <>
          {/* Header, Controls, & Time Range Filter */}
          <LogsFilterBar
            filteredCount={
              viewMode === 'summary' ? filteredSummaries.length : activeStreamLogs.length
            }
            viewMode={viewMode}
            setViewMode={setViewMode}
            filterType={filterType}
            setFilterType={setFilterType}
            autoRefresh={autoRefresh}
            onToggleAutoRefresh={onToggleAutoRefresh}
            onManualRefresh={() => {
              fetchTelemetryData(false);
              onRefresh();
            }}
            isLoadingSummaries={isLoadingSummaries}
            isBackgroundRefreshing={isBackgroundRefreshing}
            onExportCsv={handleExportDnsCsv}
            isExportDisabled={activeStreamLogs.length === 0 && summaries.length === 0}
            showHourlyChart={showHourlyChart}
            setShowHourlyChart={setShowHourlyChart}
            showRetentionSettings={showRetentionSettings}
            setShowRetentionSettings={setShowRetentionSettings}
            onShowResetModal={() => setShowResetModal(true)}
            timePreset={timePreset}
            setTimePreset={setTimePreset}
            customStartDate={customStartDate}
            setCustomStartDate={setCustomStartDate}
            customEndDate={customEndDate}
            setCustomEndDate={setCustomEndDate}
            totalHourlyQueries={totalHourlyQueries}
            peakHour={peakHour}
            lang={lang}
          />

      {/* 24-HOUR HOURLY ACCESS FREQUENCY CHART */}
      {showHourlyChart && (
        <LogsChartCard
          hourlyData={hourlyData}
          lang={lang}
          peakHour={peakHour}
          totalHourlyQueries={totalHourlyQueries}
          isInitialLoading={isInitialLoading}
          isBackgroundRefreshing={isBackgroundRefreshing}
        />
      )}

      {/* Storage Retention & Pruning Drawer */}
      {showRetentionSettings && retentionConfig && (
        <StorageRetentionCard
          retentionConfig={retentionConfig}
          selectedRetentionDays={selectedRetentionDays}
          autoPruneEnabled={autoPruneEnabled}
          isSavingRetention={isSavingRetention}
          isPruningNow={isPruningNow}
          onSaveRetention={handleSaveRetention}
          onPruneNow={handlePruneNow}
          onChangeRetentionDays={setSelectedRetentionDays}
          lang={lang}
        />
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
        <DomainDrilldownModal
          selectedDomain={selectedDomain}
          activeSummary={activeSummary}
          domainLogs={domainLogs}
          domainHourlyData={domainHourlyData}
          domainTimeClusters={domainTimeClusters}
          isLoadingDetails={isLoadingDetails}
          isBlocking={isBlocking}
          rawLogColumns={rawLogColumns}
          onClose={handleCloseDomainDetail}
          onQuickBlock={handleQuickBlockDomain}
          lang={lang}
        />
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
        </>
      )}
    </div>
  );
};
