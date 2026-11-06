import {
  AlertTriangle,
  BarChart3,
  BookOpen,
  Calendar,
  CheckCircle,
  Clock,
  Code2,
  Download,
  ExternalLink,
  Filter,
  Gamepad2,
  Globe,
  Layers,
  ListFilter,
  MessageCircle,
  PieChart,
  RefreshCw,
  RotateCcw,
  Search,
  ShieldAlert,
  Smartphone,
  Tv,
  CloudUpload,
} from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import type React from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type {
  ActivitySummaryResponse,
  AppCategoryMetadata,
  AppUsageSummary,
  KeywordViolation,
  Language,
  TimeAggregatedReport,
  WebActivityItem,
} from '../types';
import { DataTable } from './DataTable';
import { ResetReportModal } from './ResetReportModal';
import { ActivityKpiCards } from './activity/ActivityKpiCards';
import { AppCategoriesCatalogCard } from './activity/AppCategoriesCatalogCard';
import { AppsTimelineView } from './activity/AppsTimelineView';
import { CategoryChartCard } from './activity/CategoryChartCard';
import { TimeAggregatedReportCard } from './activity/TimeAggregatedReportCard';
import { UnifiedTimelineView } from './activity/UnifiedTimelineView';

interface ActivityTabProps {
  lang: Language;
  onRefresh?: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

type ViewSubTab = 'unified' | 'apps' | 'report' | 'web' | 'keywords';

export const ActivityTab: React.FC<ActivityTabProps> = ({ lang, onRefresh, onShowToast }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [data, setData] = useState<ActivitySummaryResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [activeSubTab, setActiveSubTab] = useState<ViewSubTab>('unified');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedCategory, setSelectedCategory] = useState<string>('ALL');
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  const [appViewMode, setAppViewMode] = useState<'grouped' | 'timeline'>('grouped');
  const [showResetModal, setShowResetModal] = useState<boolean>(false);
  const [appFilterMode, setAppFilterMode] = useState<'user_only' | 'all' | 'system_only'>('user_only');
  const [selectedPeriod, setSelectedPeriod] = useState<'today' | '7d' | '30d' | 'all'>('today');
  const [categories, setCategories] = useState<AppCategoryMetadata[]>([]);
  const [timeReport, setTimeReport] = useState<TimeAggregatedReport | null>(null);
  const [isSyncing, setIsSyncing] = useState<boolean>(false);

  const fetchCategories = useCallback(async () => {
    try {
      const cats = await api.getAppCategories();
      setCategories(cats);
    } catch {}
  }, []);

  const fetchTimeReport = useCallback(async () => {
    try {
      const rep = await api.getTimeAggregatedReport({
        period: selectedPeriod,
        excludeSystem: appFilterMode === 'user_only',
      });
      setTimeReport(rep);
    } catch {}
  }, [selectedPeriod, appFilterMode]);

  const handleSyncAppsToServer = async () => {
    try {
      setIsSyncing(true);
      const res = await api.syncAppsToServer();
      if (res.success) {
        onShowToast(
          isVi
            ? `Đã đồng bộ ${res.apps_synced} ứng dụng lên Cloud Server thành công!`
            : `Successfully synced ${res.apps_synced} apps to Cloud Server!`,
          'success'
        );
      } else {
        onShowToast(res.message || 'Lỗi đồng bộ', 'error');
      }
    } catch (err: unknown) {
      onShowToast(String(err), 'error');
    } finally {
      setIsSyncing(false);
    }
  };

  const fetchActivity = useCallback(async () => {
    try {
      setLoading(true);
      const [res] = await Promise.all([
        api.getActivitySummary(),
        fetchCategories(),
        fetchTimeReport(),
      ]);
      setData(res);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      onShowToast(
        msg || (isVi ? 'Lỗi tải nhật ký hoạt động' : 'Failed to load activity logs'),
        'error'
      );
    } finally {
      setLoading(false);
    }
  }, [isVi, onShowToast, fetchCategories, fetchTimeReport]);

  // Initial parallel load on mount (single round-trip, no waterfall)
  useEffect(() => {
    let isMounted = true;
    (async () => {
      setLoading(true);
      try {
        const [res, cats, rep] = await Promise.all([
          api.getActivitySummary(),
          api.getAppCategories().catch(() => []),
          api.getTimeAggregatedReport({
            period: selectedPeriod,
            excludeSystem: appFilterMode === 'user_only',
          }).catch(() => null),
        ]);
        if (isMounted) {
          setData(res);
          setCategories(cats);
          setTimeReport(rep);
        }
      } catch (err: unknown) {
        if (isMounted) {
          const msg = err instanceof Error ? err.message : String(err);
          onShowToast(
            msg || (isVi ? 'Lỗi tải nhật ký hoạt động' : 'Failed to load activity logs'),
            'error'
          );
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    })();
    return () => {
      isMounted = false;
    };
  }, []);

  // Re-fetch time report only when period or filter mode changes
  const isFirstPeriodMount = useRef(true);
  useEffect(() => {
    if (isFirstPeriodMount.current) {
      isFirstPeriodMount.current = false;
      return;
    }
    fetchTimeReport();
  }, [fetchTimeReport]);

  useEffect(() => {
    if (!autoRefresh) return;
    const timer = setInterval(() => {
      api
        .getActivitySummary()
        .then(setData)
        .catch(() => {});
    }, 6000);
    return () => clearInterval(timer);
  }, [autoRefresh]);

  const handleExport = () => {
    if (!data) return;
    const jsonStr = JSON.stringify(data, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `famshield-activity-report-${data.date}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    onShowToast(
      isVi ? 'Đã xuất báo cáo hoạt động thành công!' : 'Activity report exported successfully!',
      'success'
    );
  };

  const getCategoryIcon = (category: string) => {
    const lower = category.toLowerCase();
    if (lower.includes('trò chơi') || lower.includes('gaming')) {
      return <Gamepad2 size={16} className="text-sky-600" />;
    }
    if (lower.includes('giải trí') || lower.includes('video') || lower.includes('entertainment')) {
      return <Tv size={16} className="text-sky-600" />;
    }
    if (lower.includes('mạng xã hội') || lower.includes('social') || lower.includes('liên lạc')) {
      return <MessageCircle size={16} className="text-sky-600" />;
    }
    if (lower.includes('học tập') || lower.includes('giáo dục') || lower.includes('education')) {
      return <BookOpen size={16} className="text-sky-600" />;
    }
    if (lower.includes('lập trình') || lower.includes('development')) {
      return <Code2 size={16} className="text-sky-600" />;
    }
    return <Globe size={16} className="text-slate-500" />;
  };

  // Filtered lists (memoized to prevent heavy UI recalculations on re-render)
  const filteredTimeline = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    const cat = selectedCategory.toLowerCase();
    return (data?.unified_timeline || []).filter((item) => {
      const matchSearch =
        !q ||
        item.title.toLowerCase().includes(q) ||
        item.subtitle.toLowerCase().includes(q) ||
        item.category.toLowerCase().includes(q);
      const matchCat =
        selectedCategory === 'ALL' ||
        item.category.toLowerCase().includes(cat);
      return matchSearch && matchCat;
    });
  }, [data?.unified_timeline, searchQuery, selectedCategory]);

  const filteredApps = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    const cat = selectedCategory.toLowerCase();
    return (data?.app_activities || []).filter((app) => {
      const matchSearch =
        !q ||
        app.app_name.toLowerCase().includes(q) ||
        app.app_title.toLowerCase().includes(q) ||
        app.category.toLowerCase().includes(q);
      const matchCat =
        selectedCategory === 'ALL' ||
        app.category.toLowerCase().includes(cat) ||
        (app.category_id && app.category_id.toLowerCase() === cat);
      const matchSystem =
        appFilterMode === 'all'
          ? true
          : appFilterMode === 'system_only'
            ? Boolean(app.is_system)
            : !app.is_system;
      return matchSearch && matchCat && matchSystem;
    });
  }, [data?.app_activities, searchQuery, selectedCategory, appFilterMode]);

  const filteredGroupedApps = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    const cat = selectedCategory.toLowerCase();
    return (data?.grouped_apps || []).filter((app) => {
      const matchSearch =
        !q ||
        app.app_name.toLowerCase().includes(q) ||
        app.app_title.toLowerCase().includes(q) ||
        app.category.toLowerCase().includes(q);
      const matchCat =
        selectedCategory === 'ALL' ||
        app.category.toLowerCase().includes(cat) ||
        (app.category_id && app.category_id.toLowerCase() === cat);
      const matchSystem =
        appFilterMode === 'all'
          ? true
          : appFilterMode === 'system_only'
            ? Boolean(app.is_system)
            : !app.is_system;
      return matchSearch && matchCat && matchSystem;
    });
  }, [data?.grouped_apps, searchQuery, selectedCategory, appFilterMode]);

  const filteredWeb = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    const cat = selectedCategory.toLowerCase();
    return (data?.web_activities || []).filter((web) => {
      const matchSearch =
        !q ||
        web.domain.toLowerCase().includes(q) ||
        web.category.toLowerCase().includes(q) ||
        web.source_app.toLowerCase().includes(q);
      const matchCat =
        selectedCategory === 'ALL' ||
        web.category.toLowerCase().includes(cat);
      return matchSearch && matchCat;
    });
  }, [data?.web_activities, searchQuery, selectedCategory]);

  const filteredKeywords = useMemo(() => {
    const q = searchQuery.toLowerCase().trim();
    const cat = selectedCategory.toLowerCase();
    return (data?.keyword_violations || []).filter((kv) => {
      const matchSearch =
        !q ||
        kv.keyword_matched.toLowerCase().includes(q) ||
        kv.app_name.toLowerCase().includes(q) ||
        kv.category.toLowerCase().includes(q);
      const matchCat =
        selectedCategory === 'ALL' ||
        kv.category.toLowerCase().includes(cat);
      return matchSearch && matchCat;
    });
  }, [data?.keyword_violations, searchQuery, selectedCategory]);

  const appColumns = useMemo<ColumnDef<AppUsageSummary>[]>(
    () => [
      {
        accessorKey: 'app_title',
        header: t.th_app_col,
        cell: ({ row }) => {
          const app = row.original;
          return (
            <div className="flex items-center gap-2.5">
              <Smartphone
                size={16}
                className={app.is_system ? 'text-slate-400 shrink-0' : 'text-sky-600 shrink-0'}
              />
              <div>
                <div className="font-semibold text-slate-900 flex items-center gap-1.5">
                  <span>{app.app_title}</span>
                  {app.is_system ? (
                    <span className="text-[10px] px-1.5 py-0.2 rounded bg-slate-100 text-slate-600 font-normal border border-slate-200">
                      {isVi ? 'Hệ thống' : 'System'}
                    </span>
                  ) : (
                    <span className="text-[10px] px-1.5 py-0.2 rounded bg-sky-50 text-sky-700 font-semibold border border-sky-200">
                      {isVi ? 'Ứng dụng' : 'User App'}
                    </span>
                  )}
                </div>
                <div className="text-2xs text-slate-400 font-mono">{app.app_name}</div>
              </div>
            </div>
          );
        },
      },
      {
        accessorKey: 'category',
        header: t.th_category_col,
        cell: (info) => {
          const cat = info.getValue() as string;
          return (
            <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1.5">
              {getCategoryIcon(cat)}
              {cat}
            </span>
          );
        },
      },
      {
        accessorKey: 'total_duration_minutes',
        header: t.th_duration,
        cell: ({ row }) => (
          <span className="text-sky-700 font-bold font-mono text-sm">
            {row.original.total_duration_formatted}
          </span>
        ),
      },
      {
        accessorKey: 'percentage',
        header: t.th_app_share,
        cell: (info) => {
          const pct = info.getValue() as number;
          return (
            <div className="flex items-center gap-2 min-w-[100px]">
              <div className="w-16 bg-slate-100 rounded-full h-2 overflow-hidden border border-slate-200/60">
                <div
                  className="bg-sky-500 h-2 rounded-full"
                  style={{ width: `${Math.min(100, Math.max(2, pct))}%` }}
                />
              </div>
              <span className="text-2xs font-mono font-semibold text-slate-700">{pct}%</span>
            </div>
          );
        },
      },
      {
        accessorKey: 'session_count',
        header: t.th_session_count,
        cell: (info) => (
          <span className="text-slate-700 font-mono">
            {info.getValue() as number} {isVi ? 'lần' : 'times'}
          </span>
        ),
      },
      {
        accessorKey: 'last_seen_at',
        header: t.th_last_used,
        cell: (info) => {
          const val = info.getValue() as string;
          return (
            <span className="text-slate-600 font-mono">
              {new Date(val).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </span>
          );
        },
      },
      {
        accessorKey: 'status',
        header: t.th_status_col,
        cell: (info) => {
          const s = info.getValue() as string;
          return (
            <span
              className={`px-2 py-0.5 rounded-full font-bold text-2xs border ${
                s === 'Active'
                  ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                  : s === 'QuotaWarning'
                    ? 'bg-amber-50 text-amber-700 border-amber-200'
                    : 'bg-slate-100 text-slate-700 border-slate-200'
              }`}
            >
              {s === 'Active'
                ? isVi
                  ? 'Đang mở'
                  : 'Active'
                : s === 'QuotaWarning'
                  ? isVi
                    ? 'Cảnh báo hạn mức'
                    : 'Quota Warning'
                  : isVi
                    ? 'Bình thường'
                    : 'Normal'}
            </span>
          );
        },
      },
    ],
    [t, isVi]
  );

  const webColumns = useMemo<ColumnDef<WebActivityItem>[]>(
    () => [
      {
        accessorKey: 'occurred_at',
        header: t.th_time,
        cell: (info) => (
          <span className="text-slate-500 font-mono text-xs">
            {new Date(info.getValue() as string).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit',
            })}
          </span>
        ),
      },
      {
        accessorKey: 'domain',
        header: t.th_domain_col,
        cell: ({ row }) => {
          const item = row.original;
          return (
            <div>
              <div className="flex items-center gap-2">
                <Globe size={14} className="text-sky-600 shrink-0" />
                <span className="font-semibold text-slate-900 break-all">{item.domain}</span>
              </div>
              {item.reason && <div className="text-2xs text-slate-500 mt-0.5">{item.reason}</div>}
            </div>
          );
        },
      },
      {
        accessorKey: 'category',
        header: t.th_category_col,
        cell: (info) => {
          const cat = info.getValue() as string;
          return (
            <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1.5">
              {getCategoryIcon(cat)}
              {cat}
            </span>
          );
        },
      },
      {
        accessorKey: 'source_app',
        header: t.th_source_app,
        cell: (info) => (
          <span className="font-mono text-2xs text-slate-600 bg-slate-50 border border-slate-200/80 px-2 py-0.5 rounded">
            {info.getValue() as string}
          </span>
        ),
      },
      {
        accessorKey: 'action',
        header: t.th_action_col,
        cell: (info) => {
          const act = ((info.getValue() as string) || '').toUpperCase();
          return (
            <span
              className={`px-2 py-0.5 rounded-full font-bold text-2xs border ${
                act === 'BLOCK'
                  ? 'bg-rose-50 text-rose-700 border-rose-200'
                  : 'bg-emerald-50 text-emerald-700 border-emerald-200'
              }`}
            >
              {act === 'BLOCK' ? t.tag_block : t.tag_allow}
            </span>
          );
        },
      },
    ],
    [t]
  );

  const keywordColumns = useMemo<ColumnDef<KeywordViolation>[]>(
    () => [
      {
        accessorKey: 'timestamp',
        header: t.th_time,
        cell: (info) => (
          <span className="text-slate-500 font-mono text-xs">
            {new Date(info.getValue() as string).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit',
            })}
          </span>
        ),
      },
      {
        accessorKey: 'keyword_matched',
        header: t.th_kw_pattern,
        cell: (info) => (
          <span className="font-bold text-rose-600">"{info.getValue() as string}"</span>
        ),
      },
      {
        accessorKey: 'category',
        header: t.th_kw_category,
        cell: (info) => (
          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium">
            {info.getValue() as string}
          </span>
        ),
      },
      {
        accessorKey: 'severity',
        header: t.th_kw_severity,
        cell: (info) => {
          const sev = ((info.getValue() as string) || '').toLowerCase();
          return (
            <span
              className={`px-2 py-0.5 rounded text-2xs font-bold uppercase border ${
                sev === 'critical'
                  ? 'bg-rose-50 text-rose-700 border-rose-200'
                  : 'bg-amber-50 text-amber-700 border-amber-200'
              }`}
            >
              {sev}
            </span>
          );
        },
      },
      {
        accessorKey: 'app_name',
        header: t.th_kw_app,
        cell: (info) => (
          <span className="font-mono text-2xs text-slate-700">{info.getValue() as string}</span>
        ),
      },
      {
        accessorKey: 'snippet_masked',
        header: t.th_kw_snippet,
        cell: (info) => (
          <span className="font-mono text-2xs text-slate-600 bg-slate-50 px-2 py-1 rounded border border-slate-200/80 block max-w-sm truncate">
            {info.getValue() as string}
          </span>
        ),
      },
    ],
    [t]
  );

  return (
    <div className="flex flex-col gap-6">
      {/* Header Banner */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-6 flex justify-between items-center flex-wrap gap-4 shadow-xs">
        <div>
          <h2 className="text-xl font-bold text-slate-900 flex items-center gap-2.5">
            <Clock size={22} className="text-sky-600" />
            {t.activity_title}
          </h2>
          <p className="text-xs text-slate-500 mt-1 max-w-3xl leading-relaxed">
            {t.activity_subtitle}
          </p>
        </div>

        <div className="flex items-center gap-2.5 flex-wrap">
          {/* Auto refresh button */}
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold border cursor-pointer flex items-center gap-1.5 transition-all ${
              autoRefresh
                ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                : 'bg-white text-slate-600 border-slate-200 hover:bg-slate-50'
            }`}
          >
            <RefreshCw size={13} className={autoRefresh ? 'animate-spin' : ''} />
            <span>
              {autoRefresh
                ? isVi
                  ? 'Tự làm mới: BẬT'
                  : 'Auto-refresh On'
                : isVi
                  ? 'Tự làm mới: TẮT'
                  : 'Auto-refresh Off'}
            </span>
          </button>

          {/* Refresh Manual */}
          <button
            onClick={fetchActivity}
            disabled={loading}
            className="flex items-center gap-1.5 bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all disabled:opacity-50"
          >
            <RefreshCw size={13} className={loading ? 'animate-spin' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Sync Apps to Server */}
          <button
            type="button"
            onClick={handleSyncAppsToServer}
            disabled={isSyncing}
            className="flex items-center gap-1.5 bg-emerald-600 hover:bg-emerald-700 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all border-0 disabled:opacity-50"
            title={
              isVi
                ? 'Đồng bộ dữ liệu sử dụng ứng dụng lên máy chủ đám mây'
                : 'Sync application usage to Cloud Server'
            }
          >
            <CloudUpload size={14} className={isSyncing ? 'animate-bounce' : ''} />
            <span>{isVi ? 'Đồng bộ Server' : 'Sync Server'}</span>
          </button>

          {/* Export Report */}
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all border-0"
          >
            <Download size={14} />
            <span>{t.btn_export_activity}</span>
          </button>

          {/* Reset Reports */}
          <button
            type="button"
            onClick={() => setShowResetModal(true)}
            className="flex items-center gap-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200/80 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-2xs cursor-pointer transition-all"
          >
            <RotateCcw size={13} />
            <span>{t.btn_reset_reports}</span>
          </button>
        </div>
      </div>

      {/* 5 KPI Summary Cards */}
      <ActivityKpiCards
        data={data}
        activeSubTab={activeSubTab}
        onSelectKeywordsTab={() => setActiveSubTab('keywords')}
        lang={lang}
      />

      {/* Category Screen Time Chart Card */}
      <CategoryChartCard appActivities={data?.app_activities} lang={lang} />

      {/* Main Content Area */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs flex flex-col gap-5">
        {/* Navigation Tabs for Views & Search Bar */}
        <div className="flex justify-between items-center flex-wrap gap-3.5 border-b border-slate-100 pb-4">
          {/* Subtabs */}
          <div className="inline-flex bg-slate-100 p-1 rounded-xl border border-slate-200/80 gap-1 flex-wrap">
            <button
              onClick={() => setActiveSubTab('unified')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                activeSubTab === 'unified'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <span>{t.tab_unified_timeline}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${activeSubTab === 'unified' ? 'bg-sky-100 text-sky-800' : 'bg-slate-200 text-slate-700'}`}
              >
                {filteredTimeline.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('apps')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                activeSubTab === 'apps'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <span>{t.tab_apps_timeline}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${activeSubTab === 'apps' ? 'bg-sky-100 text-sky-800' : 'bg-slate-200 text-slate-700'}`}
              >
                {filteredApps.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('report')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                activeSubTab === 'report'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <span>{isVi ? 'Báo cáo tổng hợp' : 'Time Analytics'}</span>
            </button>

            <button
              onClick={() => setActiveSubTab('web')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                activeSubTab === 'web'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <span>{t.tab_web_timeline}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${activeSubTab === 'web' ? 'bg-emerald-100 text-emerald-800' : 'bg-slate-200 text-slate-700'}`}
              >
                {filteredWeb.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('keywords')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                activeSubTab === 'keywords'
                  ? 'bg-white text-slate-900 shadow-xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <span>{t.tab_keywords_timeline}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${activeSubTab === 'keywords' ? 'bg-amber-100 text-amber-800' : 'bg-slate-200 text-slate-700'}`}
              >
                {filteredKeywords.length}
              </span>
            </button>
          </div>

          {/* Search & Filter */}
          <div className="flex gap-2.5 flex-wrap items-center">
            <div className="relative w-64">
              <Search
                size={14}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400"
              />
              <input
                type="text"
                placeholder={t.search_activity_placeholder}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full bg-white border border-slate-200 text-slate-900 pl-9 pr-3 py-1.5 rounded-lg text-xs outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
              />
            </div>

            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="bg-white border border-slate-200 text-slate-800 px-3 py-1.5 rounded-lg text-xs outline-none cursor-pointer focus:border-sky-500 shadow-xs"
            >
              <option value="ALL">{t.filter_all_cats}</option>
              <option value="gaming">🎮 Trò chơi (Gaming)</option>
              <option value="browsers">🌐 Trình duyệt (Browser)</option>
              <option value="social">💬 Mạng xã hội & Chat (Social)</option>
              <option value="education_work">📚 Học tập & Làm việc (Education/Work)</option>
              <option value="development">💻 Lập trình & Kỹ thuật (Development)</option>
              <option value="entertainment">📺 Giải trí & Media (Entertainment)</option>
              <option value="utilities">🛠️ Tiện ích & Công cụ (Utilities)</option>
              <option value="system">⚙️ Hệ thống Windows (System)</option>
            </select>
          </div>
        </div>

        {/* VIEW 1: UNIFIED TIMELINE */}
        {activeSubTab === 'unified' && (
          <UnifiedTimelineView
            filteredTimeline={filteredTimeline}
            getCategoryIcon={getCategoryIcon}
            lang={lang}
          />
        )}

        {/* VIEW 2: APPLICATION USAGE & TIMELINE */}
        {activeSubTab === 'apps' && (
          <div className="flex flex-col gap-6">
            {categories.length > 0 && (
              <AppCategoriesCatalogCard
                categories={categories}
                selectedCategory={selectedCategory}
                onSelectCategory={(catId) => setSelectedCategory(catId)}
                lang={lang}
              />
            )}
            <AppsTimelineView
              appViewMode={appViewMode}
              setAppViewMode={setAppViewMode}
              appFilterMode={appFilterMode}
              setAppFilterMode={setAppFilterMode}
              filteredGroupedApps={filteredGroupedApps}
              filteredApps={filteredApps}
              appColumns={appColumns}
              getCategoryIcon={getCategoryIcon}
              lang={lang}
            />
          </div>
        )}

        {/* VIEW 2B: TIME-AGGREGATED REPORT */}
        {activeSubTab === 'report' && (
          <TimeAggregatedReportCard
            report={timeReport}
            selectedPeriod={selectedPeriod}
            onSelectPeriod={setSelectedPeriod}
            onSyncToServer={handleSyncAppsToServer}
            isSyncing={isSyncing}
            lang={lang}
          />
        )}

        {/* VIEW 3: WEB & DOMAIN HISTORY */}
        {activeSubTab === 'web' && (
          <DataTable
            data={filteredWeb}
            columns={webColumns}
            pageSize={10}
            searchPlaceholder={isVi ? 'Tìm tên miền...' : 'Search domains...'}
            emptyText={t.no_activity_found}
            lang={lang}
            enableExport={true}
            exportFilename="famshield_web_history"
          />
        )}

        {/* VIEW 4: KEYWORD VIOLATIONS */}
        {activeSubTab === 'keywords' && (
          <DataTable
            data={filteredKeywords}
            columns={keywordColumns}
            pageSize={10}
            searchPlaceholder={isVi ? 'Tìm từ khóa...' : 'Search keywords...'}
            emptyText={
              isVi
                ? 'Không phát hiện từ khóa vi phạm nào.'
                : 'No keyword violations recorded.'
            }
            lang={lang}
            enableExport={true}
            exportFilename="famshield_keyword_violations"
          />
        )}
      </div>

      {/* Reset Report Modal */}
      <ResetReportModal
        isOpen={showResetModal}
        onClose={() => setShowResetModal(false)}
        onSuccess={fetchActivity}
        onShowToast={onShowToast}
        lang={lang}
        defaultScope="all"
      />
    </div>
  );
};
