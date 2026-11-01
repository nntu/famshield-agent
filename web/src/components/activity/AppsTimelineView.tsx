import type { ColumnDef } from '@tanstack/react-table';
import { BarChart3, Clock, Smartphone } from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type { AppActivityItem, AppUsageSummary, Language } from '../../types';
import { DataTable } from '../DataTable';
import { AppUsageRankingChart } from '../charts/AppUsageRankingChart';

interface AppsTimelineViewProps {
  appViewMode: 'grouped' | 'timeline';
  setAppViewMode: (mode: 'grouped' | 'timeline') => void;
  appFilterMode?: 'user_only' | 'all' | 'system_only';
  setAppFilterMode?: (mode: 'user_only' | 'all' | 'system_only') => void;
  filteredGroupedApps: AppUsageSummary[];
  filteredApps: AppActivityItem[];
  appColumns: ColumnDef<AppUsageSummary>[];
  getCategoryIcon: (category: string) => React.ReactNode;
  lang: Language;
}

export const AppsTimelineView: React.FC<AppsTimelineViewProps> = ({
  appViewMode,
  setAppViewMode,
  appFilterMode = 'user_only',
  setAppFilterMode,
  filteredGroupedApps,
  filteredApps,
  appColumns,
  getCategoryIcon,
  lang,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  return (
    <div className="flex flex-col gap-4">
      {/* View Mode & System Filter Header */}
      <div className="flex justify-between items-center flex-wrap gap-2 pb-1">
        <div className="flex items-center gap-2 flex-wrap">
          {/* Mode Switcher */}
          <div className="inline-flex bg-slate-100 p-0.5 rounded-lg border border-slate-200">
            <button
              type="button"
              onClick={() => setAppViewMode('grouped')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                appViewMode === 'grouped'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <BarChart3
                size={14}
                className={appViewMode === 'grouped' ? 'text-sky-600' : 'text-slate-500'}
              />
              <span>{t.view_mode_grouped_apps}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${
                  appViewMode === 'grouped'
                    ? 'bg-sky-100 text-sky-800'
                    : 'bg-slate-200 text-slate-700'
                }`}
              >
                {filteredGroupedApps.length}
              </span>
            </button>
            <button
              type="button"
              onClick={() => setAppViewMode('timeline')}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                appViewMode === 'timeline'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              <Clock
                size={14}
                className={appViewMode === 'timeline' ? 'text-sky-600' : 'text-slate-500'}
              />
              <span>{t.view_mode_raw_apps}</span>
              <span
                className={`px-1.5 py-0.2 rounded-full text-2xs font-bold ${
                  appViewMode === 'timeline'
                    ? 'bg-sky-100 text-sky-800'
                    : 'bg-slate-200 text-slate-700'
                }`}
              >
                {filteredApps.length}
              </span>
            </button>
          </div>

          {/* System vs User Apps Filter Switcher */}
          {setAppFilterMode && (
            <div className="inline-flex bg-slate-100 p-0.5 rounded-lg border border-slate-200">
              <button
                type="button"
                onClick={() => setAppFilterMode('user_only')}
                className={`inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                  appFilterMode === 'user_only'
                    ? 'bg-sky-600 text-white shadow-2xs'
                    : 'text-slate-600 hover:text-slate-900'
                }`}
                title={isVi ? 'Ưu tiên hiển thị ứng dụng người dùng' : 'Prioritize user apps'}
              >
                <Smartphone size={13} />
                <span>{isVi ? 'Ứng dụng người dùng' : 'User Apps'}</span>
              </button>
              <button
                type="button"
                onClick={() => setAppFilterMode('all')}
                className={`inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                  appFilterMode === 'all'
                    ? 'bg-white text-slate-900 shadow-2xs'
                    : 'text-slate-600 hover:text-slate-900'
                }`}
              >
                <span>{isVi ? 'Tất cả' : 'All'}</span>
              </button>
              <button
                type="button"
                onClick={() => setAppFilterMode('system_only')}
                className={`inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                  appFilterMode === 'system_only'
                    ? 'bg-white text-slate-900 shadow-2xs'
                    : 'text-slate-600 hover:text-slate-900'
                }`}
              >
                <span>{isVi ? 'Hệ thống' : 'System'}</span>
              </button>
            </div>
          )}
        </div>

        <div className="text-2xs text-slate-500 font-medium">
          {appFilterMode === 'user_only'
            ? isVi
              ? '★ Đang ưu tiên ứng dụng người dùng'
              : '★ User applications prioritized'
            : isVi
              ? 'Bao gồm cả tiến trình nền hệ điều hành'
              : 'Including operating system services'}
        </div>
      </div>

      {/* Mode 1: Grouped Summary */}
      {appViewMode === 'grouped' && (
        <div className="flex flex-col gap-4">
          {/* Ranking Chart */}
          {filteredGroupedApps.length > 0 && (
            <div className="bg-slate-50/80 border border-slate-200/80 rounded-xl p-4 sm:p-5 flex flex-col gap-3">
              <div className="flex justify-between items-center flex-wrap gap-2">
                <div>
                  <h4 className="text-xs font-bold text-slate-900 flex items-center gap-1.5">
                    <BarChart3 size={15} className="text-sky-600" />
                    {t.app_ranking_title}
                  </h4>
                  <p className="text-2xs text-slate-500">{t.app_ranking_desc}</p>
                </div>
                <span className="text-2xs text-slate-500 font-medium">
                  {filteredGroupedApps.length} {isVi ? 'ứng dụng' : 'apps'}
                </span>
              </div>
              <AppUsageRankingChart
                apps={filteredGroupedApps}
                lang={lang}
                height={Math.min(320, Math.max(160, filteredGroupedApps.length * 36 + 40))}
              />
            </div>
          )}

          {/* Grouped Table */}
          <DataTable
            data={filteredGroupedApps}
            columns={appColumns}
            pageSize={10}
            searchPlaceholder={isVi ? 'Tìm ứng dụng...' : 'Search apps...'}
            emptyText={t.no_activity_found}
            lang={lang}
            enableExport={true}
            exportFilename="famshield_app_usage"
          />
        </div>
      )}

      {/* Mode 2: Timeline Segments */}
      {appViewMode === 'timeline' && (
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
                          <Smartphone size={16} className="text-sky-600 shrink-0" />
                          <div>
                            <div className="font-semibold text-slate-900">{app.app_title}</div>
                            <div className="text-2xs text-slate-400 font-mono">{app.app_name}</div>
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
    </div>
  );
};
