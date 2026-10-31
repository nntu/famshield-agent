import {
  BarChart3,
  Calendar,
  ChevronDown,
  ChevronUp,
  Database,
  Download,
  Flame,
  Layers,
  List,
  RefreshCw,
  RotateCcw,
} from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type { Language } from '../../types';

export type TimePreset = 'today' | '24h' | '7d' | 'all' | 'custom';
export type FilterType = 'all' | 'blocked' | 'allowed' | 'monitored' | 'errors';

interface LogsFilterBarProps {
  filteredCount: number;
  viewMode: 'summary' | 'stream';
  setViewMode: (mode: 'summary' | 'stream') => void;
  filterType: FilterType;
  setFilterType: (filter: FilterType) => void;
  autoRefresh: boolean;
  onToggleAutoRefresh: () => void;
  onManualRefresh: () => void;
  isLoadingSummaries: boolean;
  isBackgroundRefreshing: boolean;
  onExportCsv: () => void;
  isExportDisabled: boolean;
  showHourlyChart: boolean;
  setShowHourlyChart: (show: boolean) => void;
  showRetentionSettings: boolean;
  setShowRetentionSettings: (show: boolean) => void;
  onShowResetModal: () => void;
  timePreset: TimePreset;
  setTimePreset: (preset: TimePreset) => void;
  customStartDate: string;
  setCustomStartDate: (d: string) => void;
  customEndDate: string;
  setCustomEndDate: (d: string) => void;
  totalHourlyQueries: number;
  peakHour: number;
  lang: Language;
}

export const LogsFilterBar: React.FC<LogsFilterBarProps> = ({
  filteredCount,
  viewMode,
  setViewMode,
  filterType,
  setFilterType,
  autoRefresh,
  onToggleAutoRefresh,
  onManualRefresh,
  isLoadingSummaries,
  isBackgroundRefreshing,
  onExportCsv,
  isExportDisabled,
  showHourlyChart,
  setShowHourlyChart,
  showRetentionSettings,
  setShowRetentionSettings,
  onShowResetModal,
  timePreset,
  setTimePreset,
  customStartDate,
  setCustomStartDate,
  customEndDate,
  setCustomEndDate,
  totalHourlyQueries,
  peakHour,
  lang,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  return (
    <>
      {/* Header & Controls */}
      <div className="flex justify-between items-center flex-wrap gap-3.5">
        <div>
          <div className="flex items-center gap-2.5">
            <h2 className="text-xl font-bold text-slate-900">{t.dns_logs_title}</h2>
            <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded-md text-xs font-semibold">
              {viewMode === 'summary'
                ? `${filteredCount} domains`
                : `${filteredCount} queries`}
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
            onClick={onManualRefresh}
            disabled={isLoadingSummaries}
            className="flex items-center gap-1.5 bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 hover:text-slate-900 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all disabled:opacity-60"
          >
            <RefreshCw
              size={13}
              className={isLoadingSummaries || isBackgroundRefreshing ? 'animate-spin' : ''}
            />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Export DNS Logs CSV */}
          <button
            type="button"
            onClick={onExportCsv}
            disabled={isExportDisabled}
            className="flex items-center gap-1.5 bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 hover:text-slate-900 px-3 py-1.5 rounded-lg text-xs font-semibold shadow-xs cursor-pointer transition-all disabled:opacity-50"
            title={t.btn_export_dns_csv}
          >
            <Download size={13} />
            <span>{t.btn_export_dns_csv}</span>
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
            onClick={onShowResetModal}
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
    </>
  );
};
