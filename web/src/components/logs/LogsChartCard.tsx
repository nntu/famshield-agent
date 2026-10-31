import { BarChart3, RefreshCw } from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type { HourlyFrequency, Language } from '../../types';
import { DnsHourlyChart } from '../charts/DnsHourlyChart';

interface LogsChartCardProps {
  hourlyData: HourlyFrequency[];
  lang: Language;
  peakHour: number;
  totalHourlyQueries: number;
  isInitialLoading: boolean;
  isBackgroundRefreshing: boolean;
}

export const LogsChartCard: React.FC<LogsChartCardProps> = ({
  hourlyData,
  lang,
  peakHour,
  totalHourlyQueries,
  isInitialLoading,
  isBackgroundRefreshing,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  return (
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
  );
};
