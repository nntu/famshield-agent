import {
  AlertTriangle,
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
  MessageCircle,
  RefreshCw,
  Search,
  ShieldAlert,
  Smartphone,
  Tv,
} from 'lucide-react';
import type React from 'react';
import { useCallback, useEffect, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { ActivitySummaryResponse, Language } from '../types';

interface ActivityTabProps {
  lang: Language;
  onRefresh?: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

type ViewSubTab = 'unified' | 'apps' | 'web' | 'keywords';

export const ActivityTab: React.FC<ActivityTabProps> = ({ lang, onRefresh, onShowToast }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [data, setData] = useState<ActivitySummaryResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [activeSubTab, setActiveSubTab] = useState<ViewSubTab>('unified');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedCategory, setSelectedCategory] = useState<string>('ALL');
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);

  const fetchActivity = useCallback(async () => {
    try {
      setLoading(true);
      const res = await api.getActivitySummary();
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
  }, [isVi, onShowToast]);

  useEffect(() => {
    fetchActivity();
  }, [fetchActivity]);

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
      return <Gamepad2 size={16} color="#ec4899" />;
    }
    if (lower.includes('giải trí') || lower.includes('video') || lower.includes('entertainment')) {
      return <Tv size={16} color="#38bdf8" />;
    }
    if (lower.includes('mạng xã hội') || lower.includes('social') || lower.includes('liên lạc')) {
      return <MessageCircle size={16} color="#a855f7" />;
    }
    if (lower.includes('học tập') || lower.includes('giáo dục') || lower.includes('education')) {
      return <BookOpen size={16} color="#34d399" />;
    }
    if (lower.includes('lập trình') || lower.includes('development')) {
      return <Code2 size={16} color="#f59e0b" />;
    }
    return <Globe size={16} color="#94a3b8" />;
  };

  // Filtered lists
  const filteredTimeline = (data?.unified_timeline || []).filter((item) => {
    const matchSearch =
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.subtitle.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat =
      selectedCategory === 'ALL' ||
      item.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredApps = (data?.app_activities || []).filter((app) => {
    const matchSearch =
      app.app_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      app.app_title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      app.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat =
      selectedCategory === 'ALL' ||
      app.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredWeb = (data?.web_activities || []).filter((web) => {
    const matchSearch =
      web.domain.toLowerCase().includes(searchQuery.toLowerCase()) ||
      web.category.toLowerCase().includes(searchQuery.toLowerCase()) ||
      web.source_app.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat =
      selectedCategory === 'ALL' ||
      web.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredKeywords = (data?.keyword_violations || []).filter((kv) => {
    const matchSearch =
      kv.keyword_matched.toLowerCase().includes(searchQuery.toLowerCase()) ||
      kv.app_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      kv.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat =
      selectedCategory === 'ALL' ||
      kv.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

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

          {/* Export Report */}
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all border-0"
          >
            <Download size={14} />
            <span>{t.btn_export_activity}</span>
          </button>
        </div>
      </div>

      {/* 5 KPI Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
        {/* Card 1: Total Screen Time */}
        <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div className="flex justify-between items-start mb-2">
            <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
              <Clock size={15} className="text-sky-600" />
              {t.stat_screen_time}
            </span>
            <span className="bg-sky-50 text-sky-700 px-2 py-0.5 rounded text-2xs font-mono font-bold border border-sky-100">
              {data?.date || new Date().toISOString().slice(0, 10)}
            </span>
          </div>
          <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
            {data?.total_screen_time_formatted || '0 phút'}
          </div>
          <div className="text-2xs text-slate-400">{t.stat_screen_time_sub}</div>
        </div>

        {/* Card 2: Applications Used */}
        <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div className="flex justify-between items-start mb-2">
            <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
              <Smartphone size={15} className="text-indigo-600" />
              {t.stat_apps_opened}
            </span>
            <span className="bg-indigo-50 text-indigo-700 px-2 py-0.5 rounded text-2xs font-bold border border-indigo-100">
              {data?.active_apps_count || 0} apps
            </span>
          </div>
          <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
            {data?.active_apps_count || 0}
          </div>
          <div className="text-2xs text-slate-400">{t.stat_apps_opened_sub}</div>
        </div>

        {/* Card 3: Websites Visited */}
        <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div className="flex justify-between items-start mb-2">
            <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
              <Globe size={15} className="text-emerald-600" />
              {t.stat_web_visits}
            </span>
            <span className="bg-emerald-50 text-emerald-700 px-2 py-0.5 rounded text-2xs font-bold border border-emerald-100">
              DNS
            </span>
          </div>
          <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
            {data?.total_web_visits_count || 0}
          </div>
          <div className="text-2xs text-slate-400">{t.stat_web_visits_sub}</div>
        </div>

        {/* Card 4: Blocked & Monitored Violations */}
        <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div className="flex justify-between items-start mb-2">
            <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
              <ShieldAlert size={15} className="text-rose-600" />
              {t.stat_violations}
            </span>
            <span className="bg-rose-50 text-rose-700 px-2 py-0.5 rounded text-2xs font-bold border border-rose-100">
              {data?.blocked_events_count || 0} Block
            </span>
          </div>
          <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
            {(data?.blocked_events_count || 0) + (data?.monitored_events_count || 0)}
          </div>
          <div className="text-2xs text-slate-400">{t.stat_violations_sub}</div>
        </div>

        {/* Card 5: Sensitive Keyword Violations */}
        <div
          onClick={() => setActiveSubTab('keywords')}
          className={`bg-white border rounded-2xl p-5 shadow-xs flex flex-col justify-between cursor-pointer transition-all ${
            activeSubTab === 'keywords'
              ? 'border-amber-400 ring-1 ring-amber-400'
              : 'border-slate-200/80 hover:border-slate-300'
          }`}
        >
          <div className="flex justify-between items-start mb-2">
            <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
              <AlertTriangle size={15} className="text-amber-600" />
              {t.stat_keywords}
            </span>
            <span className="bg-amber-50 text-amber-700 px-2 py-0.5 rounded text-2xs font-bold border border-amber-100">
              RAM
            </span>
          </div>
          <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
            {data?.keyword_violations_count || 0}
          </div>
          <div className="text-2xs text-slate-400">{t.stat_keywords_sub}</div>
        </div>
      </div>

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
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${activeSubTab === 'apps' ? 'bg-indigo-100 text-indigo-800' : 'bg-slate-200 text-slate-700'}`}
              >
                {filteredApps.length}
              </span>
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
              <option value="Trò chơi">🎮 Trò chơi (Gaming)</option>
              <option value="Giải trí">📺 Giải trí (Entertainment)</option>
              <option value="Mạng xã hội">💬 Mạng xã hội (Social)</option>
              <option value="Học tập">📚 Học tập (Education)</option>
              <option value="Trình duyệt">🌐 Trình duyệt (Browser)</option>
            </select>
          </div>
        </div>

        {/* VIEW 1: UNIFIED TIMELINE */}
        {activeSubTab === 'unified' && (
          <div>
            {filteredTimeline.length === 0 ? (
              <div className="text-center py-10 text-slate-500 text-sm">{t.no_activity_found}</div>
            ) : (
              <div className="flex flex-col gap-2.5">
                {filteredTimeline.map((item) => {
                  const isApp = item.item_type === 'app';
                  const isKeyword = item.item_type === 'keyword';
                  const dateObj = new Date(item.timestamp);
                  const timeFormatted = dateObj.toLocaleTimeString([], {
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit',
                  });

                  return (
                    <div
                      key={item.id}
                      className={`border rounded-xl p-3.5 flex items-center justify-between flex-wrap gap-3 transition-colors ${
                        isKeyword
                          ? 'bg-amber-50/50 border-amber-200/80 hover:bg-amber-50'
                          : 'bg-slate-50/60 border-slate-200/80 hover:bg-slate-100/60'
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className={`w-9 h-9 rounded-xl flex items-center justify-center shrink-0 ${
                            isKeyword
                              ? 'bg-amber-100 text-amber-700'
                              : isApp
                                ? 'bg-indigo-50 text-indigo-700'
                                : 'bg-sky-50 text-sky-700'
                          }`}
                        >
                          {isKeyword ? (
                            <AlertTriangle size={17} />
                          ) : isApp ? (
                            <Smartphone size={17} />
                          ) : (
                            <Globe size={17} />
                          )}
                        </div>

                        <div>
                          <div className="flex items-center gap-2 mb-0.5">
                            <span className="font-bold text-slate-900 text-sm">{item.title}</span>
                            <span className="bg-white text-slate-600 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1 shadow-2xs">
                              {getCategoryIcon(item.category)}
                              {item.category}
                            </span>
                          </div>

                          <div className="text-xs text-slate-500">{item.subtitle}</div>
                        </div>
                      </div>

                      <div className="flex items-center gap-3.5">
                        <div className="text-right">
                          <div className="text-xs font-mono font-bold text-slate-800">
                            {timeFormatted}
                          </div>
                          <div className="text-2xs text-slate-400">
                            {dateObj.toLocaleDateString()}
                          </div>
                        </div>

                        {/* Action Badge */}
                        <span
                          className={`px-2.5 py-0.5 rounded-full font-bold text-2xs border ${
                            item.action === 'VIOLATION'
                              ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                              : item.action === 'BLOCK'
                                ? 'bg-rose-50 text-rose-700 border-rose-200/80'
                                : item.action === 'MONITOR'
                                  ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                                  : item.action === 'ACTIVE'
                                    ? 'bg-indigo-50 text-indigo-700 border-indigo-200/80'
                                    : 'bg-emerald-50 text-emerald-700 border-emerald-200/80'
                          }`}
                        >
                          {item.action === 'VIOLATION'
                            ? isVi
                              ? '⚠️ TỪ KHÓA VI PHẠM'
                              : '⚠️ KEYWORD VIOLATION'
                            : item.action === 'BLOCK'
                              ? t.tag_block
                              : item.action === 'MONITOR'
                                ? isVi
                                  ? 'GIÁM SÁT'
                                  : 'MONITOR'
                                : item.action === 'ACTIVE'
                                  ? isVi
                                    ? 'ĐANG CHẠY'
                                    : 'ACTIVE'
                                  : t.tag_allow}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}

        {/* VIEW 2: APPLICATION TIMELINE */}
        {activeSubTab === 'apps' && (
          <div className="overflow-x-auto rounded-xl border border-slate-200 bg-white">
            <table className="w-full border-collapse text-left min-w-[650px] text-xs">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50 text-slate-600 font-semibold">
                  <th className="py-2.5 px-3.5">{t.th_app_col}</th>
                  <th className="py-2.5 px-3.5">{t.th_category_col}</th>
                  <th className="py-2.5 px-3.5">{t.th_time_window}</th>
                  <th className="py-2.5 px-3.5">{t.th_duration}</th>
                  <th className="py-2.5 px-3.5">{t.th_status_col}</th>
                </tr>
              </thead>
              <tbody>
                {filteredApps.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="text-center py-8 text-slate-500 text-sm">
                      {t.no_activity_found}
                    </td>
                  </tr>
                ) : (
                  filteredApps.map((app) => {
                    const startStr = new Date(app.start_time).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                    });
                    const endStr = new Date(app.end_time).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                    });

                    return (
                      <tr
                        key={app.id}
                        className="border-b border-slate-100 hover:bg-slate-50/50 transition-colors"
                      >
                        <td className="py-2.5 px-3.5">
                          <div className="flex items-center gap-2.5">
                            <Smartphone size={16} className="text-indigo-600 shrink-0" />
                            <div>
                              <div className="font-semibold text-slate-900">{app.app_title}</div>
                              <div className="text-2xs text-slate-400 font-mono">
                                {app.app_name}
                              </div>
                            </div>
                          </div>
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1.5">
                            {getCategoryIcon(app.category)}
                            {app.category}
                          </span>
                        </td>
                        <td className="py-2.5 px-3.5 text-slate-600 font-mono">
                          {startStr} &rarr; {endStr}
                        </td>
                        <td className="py-2.5 px-3.5 text-sky-700 font-bold font-mono">
                          {app.duration_formatted}
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span
                            className={`px-2 py-0.5 rounded-full font-bold text-2xs border ${
                              app.status === 'Active'
                                ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                                : app.status === 'QuotaWarning'
                                  ? 'bg-amber-50 text-amber-700 border-amber-200'
                                  : 'bg-slate-100 text-slate-700 border-slate-200'
                            }`}
                          >
                            {app.status === 'Active'
                              ? isVi
                                ? 'Đang chạy'
                                : 'Active'
                              : app.status === 'QuotaWarning'
                                ? isVi
                                  ? 'Cảnh báo hạn mức'
                                  : 'Quota Warning'
                                : isVi
                                  ? 'Bình thường'
                                  : 'Normal'}
                          </span>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        )}

        {/* VIEW 3: WEB & DOMAIN HISTORY */}
        {activeSubTab === 'web' && (
          <div className="overflow-x-auto rounded-xl border border-slate-200 bg-white">
            <table className="w-full border-collapse text-left min-w-[650px] text-xs">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50 text-slate-600 font-semibold">
                  <th className="py-2.5 px-3.5">{t.th_time}</th>
                  <th className="py-2.5 px-3.5">{t.th_domain_col}</th>
                  <th className="py-2.5 px-3.5">{t.th_category_col}</th>
                  <th className="py-2.5 px-3.5">{t.th_source_app}</th>
                  <th className="py-2.5 px-3.5">{t.th_action_col}</th>
                </tr>
              </thead>
              <tbody>
                {filteredWeb.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="text-center py-8 text-slate-500 text-sm">
                      {t.no_activity_found}
                    </td>
                  </tr>
                ) : (
                  filteredWeb.map((web) => {
                    const timeStr = new Date(web.occurred_at).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                      second: '2-digit',
                    });

                    return (
                      <tr
                        key={web.id}
                        className="border-b border-slate-100 hover:bg-slate-50/50 transition-colors"
                      >
                        <td className="py-2.5 px-3.5 text-slate-500 font-mono text-xs">
                          {timeStr}
                        </td>
                        <td className="py-2.5 px-3.5">
                          <div className="flex items-center gap-2">
                            <Globe size={14} className="text-sky-600 shrink-0" />
                            <span className="font-semibold text-slate-900 break-all">
                              {web.domain}
                            </span>
                          </div>
                          {web.reason && (
                            <div className="text-2xs text-slate-500 mt-0.5">{web.reason}</div>
                          )}
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1.5">
                            {getCategoryIcon(web.category)}
                            {web.category}
                          </span>
                        </td>
                        <td className="py-2.5 px-3.5 text-slate-500 font-mono text-xs">
                          {web.source_app}
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span
                            className={`px-2 py-0.5 rounded-full font-bold text-2xs border ${
                              web.action === 'BLOCK'
                                ? 'bg-rose-50 text-rose-700 border-rose-200'
                                : web.action === 'MONITOR'
                                  ? 'bg-amber-50 text-amber-700 border-amber-200'
                                  : 'bg-emerald-50 text-emerald-700 border-emerald-200'
                            }`}
                          >
                            {web.action === 'BLOCK'
                              ? t.tag_block
                              : web.action === 'MONITOR'
                                ? isVi
                                  ? 'GIÁM SÁT'
                                  : 'MONITOR'
                                : t.tag_allow}
                          </span>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        )}

        {/* VIEW 4: KEYWORD VIOLATIONS */}
        {activeSubTab === 'keywords' && (
          <div>
            {filteredKeywords.length === 0 ? (
              <div className="text-center py-10 text-slate-500 text-sm">
                {isVi ? 'Không phát hiện từ khóa vi phạm nào.' : 'No keyword violations recorded.'}
              </div>
            ) : (
              <div className="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                <table className="w-full border-collapse text-left text-xs">
                  <thead>
                    <tr className="border-b border-slate-200 bg-slate-50 text-slate-600 font-semibold uppercase text-2xs">
                      <th className="py-2.5 px-3.5">{t.th_time}</th>
                      <th className="py-2.5 px-3.5">{t.th_kw_pattern}</th>
                      <th className="py-2.5 px-3.5">{t.th_kw_category}</th>
                      <th className="py-2.5 px-3.5">{t.th_kw_severity}</th>
                      <th className="py-2.5 px-3.5">{t.th_kw_app}</th>
                      <th className="py-2.5 px-3.5">{t.th_kw_snippet}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredKeywords.map((kv) => (
                      <tr
                        key={kv.id}
                        className="border-b border-slate-100 hover:bg-slate-50/50 transition-colors"
                      >
                        <td className="py-2.5 px-3.5 text-slate-500 font-mono">
                          {new Date(kv.timestamp).toLocaleTimeString([], {
                            hour: '2-digit',
                            minute: '2-digit',
                            second: '2-digit',
                          })}
                        </td>
                        <td className="py-2.5 px-3.5 font-bold text-rose-600">
                          "{kv.keyword_matched}"
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium">
                            {kv.category}
                          </span>
                        </td>
                        <td className="py-2.5 px-3.5">
                          <span
                            className={`px-2 py-0.5 rounded text-2xs font-bold uppercase border ${
                              kv.severity === 'critical'
                                ? 'bg-rose-50 text-rose-700 border-rose-200'
                                : 'bg-amber-50 text-amber-700 border-amber-200'
                            }`}
                          >
                            {kv.severity}
                          </span>
                        </td>
                        <td className="py-2.5 px-3.5 text-indigo-700 font-medium">{kv.app_name}</td>
                        <td className="py-2.5 px-3.5 text-slate-600 italic font-mono text-2xs">
                          {kv.snippet_masked}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
