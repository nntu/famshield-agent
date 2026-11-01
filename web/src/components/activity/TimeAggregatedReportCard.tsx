import type React from 'react';
import { useMemo } from 'react';
import { Bar } from 'react-chartjs-2';
import {
  Calendar,
  Clock,
  Smartphone,
  Cpu,
  TrendingUp,
  CloudUpload,
  RefreshCw,
  Award,
} from 'lucide-react';
import type { TimeAggregatedReport, Language } from '../../types';
import '../charts/chartSetup';

interface TimeAggregatedReportCardProps {
  report: TimeAggregatedReport | null;
  selectedPeriod: 'today' | '7d' | '30d' | 'all';
  onSelectPeriod: (period: 'today' | '7d' | '30d' | 'all') => void;
  onSyncToServer: () => void;
  isSyncing: boolean;
  lang: Language;
}

export const TimeAggregatedReportCard: React.FC<TimeAggregatedReportCardProps> = ({
  report,
  selectedPeriod,
  onSelectPeriod,
  onSyncToServer,
  isSyncing,
  lang,
}) => {
  const isVi = lang === 'vi';

  const hourlyChartData = useMemo(() => {
    if (!report?.hourly_distribution) return null;
    const labels = report.hourly_distribution.map((h) => h.label);
    const userData = report.hourly_distribution.map((h) => h.user_minutes);
    const systemData = report.hourly_distribution.map((h) => h.system_minutes);

    return {
      labels,
      datasets: [
        {
          label: isVi ? 'Ứng dụng người dùng (phút)' : 'User Apps (mins)',
          data: userData,
          backgroundColor: 'rgba(2, 132, 199, 0.85)', // sky-600
          hoverBackgroundColor: '#0284c7',
          borderRadius: 4,
          stack: 'usage',
        },
        {
          label: isVi ? 'Tiến trình hệ thống (phút)' : 'System Services (mins)',
          data: systemData,
          backgroundColor: 'rgba(148, 163, 184, 0.65)', // slate-400
          hoverBackgroundColor: '#94a3b8',
          borderRadius: 4,
          stack: 'usage',
        },
      ],
    };
  }, [report, isVi]);

  const chartOptions = useMemo(() => {
    return {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        x: {
          grid: { display: false },
          ticks: {
            font: { size: 10 },
            maxRotation: 0,
            autoSkip: true,
            maxTicksLimit: 12,
          },
        },
        y: {
          beginAtZero: true,
          grid: { color: 'rgba(226, 232, 240, 0.8)' },
          ticks: {
            font: { size: 10 },
            callback: (v: any) => `${v}m`,
          },
        },
      },
      plugins: {
        legend: {
          position: 'top' as const,
          align: 'end' as const,
          labels: {
            boxWidth: 12,
            boxHeight: 12,
            font: { size: 11, weight: 'bold' as const },
          },
        },
        tooltip: {
          callbacks: {
            label: (ctx: any) => ` ${ctx.dataset.label}: ${ctx.parsed.y} ${isVi ? 'phút' : 'mins'}`,
          },
        },
      },
    };
  }, [isVi]);

  if (!report) return null;

  return (
    <div className="bg-white border border-slate-200/80 rounded-2xl p-5 sm:p-6 shadow-xs flex flex-col gap-5">
      {/* Header with Period Switcher & Sync Button */}
      <div className="flex justify-between items-center flex-wrap gap-3 pb-3 border-b border-slate-100">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-sky-50 border border-sky-200 flex items-center justify-center text-sky-600">
            <Calendar size={16} />
          </div>
          <div>
            <h3 className="text-sm font-bold text-slate-900">
              {isVi ? 'Báo Cáo Tổng Hợp Theo Thời Gian' : 'Time-Aggregated Summary Report'}
            </h3>
            <p className="text-2xs text-slate-500">
              {isVi
                ? 'Thống kê thời lượng ứng dụng người dùng, loại trừ tạp âm hệ thống'
                : 'Aggregated user apps usage, filtered from system noise'}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          {/* Period selector */}
          <div className="inline-flex bg-slate-100 p-0.5 rounded-lg border border-slate-200">
            <button
              type="button"
              onClick={() => onSelectPeriod('today')}
              className={`px-2.5 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                selectedPeriod === 'today'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {isVi ? 'Hôm nay' : 'Today'}
            </button>
            <button
              type="button"
              onClick={() => onSelectPeriod('7d')}
              className={`px-2.5 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                selectedPeriod === '7d'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {isVi ? '7 ngày qua' : '7 Days'}
            </button>
            <button
              type="button"
              onClick={() => onSelectPeriod('30d')}
              className={`px-2.5 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                selectedPeriod === '30d'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {isVi ? '30 ngày' : '30 Days'}
            </button>
            <button
              type="button"
              onClick={() => onSelectPeriod('all')}
              className={`px-2.5 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                selectedPeriod === 'all'
                  ? 'bg-white text-slate-900 shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {isVi ? 'Tất cả' : 'All Time'}
            </button>
          </div>

          {/* Sync to Server Button */}
          <button
            type="button"
            onClick={onSyncToServer}
            disabled={isSyncing}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-700 active:bg-sky-800 text-white text-xs font-semibold transition-all shadow-2xs disabled:opacity-50 cursor-pointer"
          >
            {isSyncing ? (
              <RefreshCw size={13} className="animate-spin" />
            ) : (
              <CloudUpload size={13} />
            )}
            <span>{isVi ? 'Đồng bộ lên Server' : 'Sync to Server'}</span>
          </button>
        </div>
      </div>

      {/* KPI Cards Row */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div className="p-3.5 rounded-xl bg-gradient-to-br from-sky-50 to-white border border-sky-100 flex flex-col justify-between">
          <div className="flex items-center justify-between text-2xs text-sky-700 font-semibold mb-1">
            <span>{isVi ? 'Dùng Thực Tế (User Apps)' : 'Active User Apps'}</span>
            <Smartphone size={14} className="text-sky-600" />
          </div>
          <div className="text-lg font-bold font-mono text-slate-900">
            {report.total_screen_time_formatted || '0 phút'}
          </div>
          <div className="text-[11px] text-slate-500 mt-1">
            {report.user_apps_count} {isVi ? 'ứng dụng hoạt động' : 'active applications'}
          </div>
        </div>

        <div className="p-3.5 rounded-xl bg-gradient-to-br from-slate-50 to-white border border-slate-200 flex flex-col justify-between">
          <div className="flex items-center justify-between text-2xs text-slate-600 font-semibold mb-1">
            <span>{isVi ? 'Nền Hệ Thống (System)' : 'System Services'}</span>
            <Cpu size={14} className="text-slate-500" />
          </div>
          <div className="text-lg font-bold font-mono text-slate-700">
            {report.total_system_minutes >= 60
              ? `${Math.floor(report.total_system_minutes / 60)}h ${report.total_system_minutes % 60}m`
              : `${report.total_system_minutes} ${isVi ? 'phút' : 'mins'}`}
          </div>
          <div className="text-[11px] text-slate-400 mt-1">
            {report.system_apps_count} {isVi ? 'tiến trình hệ thống' : 'system services'}
          </div>
        </div>

        <div className="p-3.5 rounded-xl bg-gradient-to-br from-emerald-50 to-white border border-emerald-100 flex flex-col justify-between">
          <div className="flex items-center justify-between text-2xs text-emerald-700 font-semibold mb-1">
            <span>{isVi ? 'Ưu Tiên Người Dùng' : 'User Apps Priority'}</span>
            <TrendingUp size={14} className="text-emerald-600" />
          </div>
          <div className="text-lg font-bold font-mono text-emerald-800">
            {report.total_active_minutes + report.total_system_minutes > 0
              ? `${Math.round(
                  (report.total_active_minutes /
                    (report.total_active_minutes + report.total_system_minutes)) *
                    100
                )}%`
              : '100%'}
          </div>
          <div className="text-[11px] text-emerald-600 mt-1">
            {isVi ? 'Đã lọc bỏ tiến trình nền' : 'Background services filtered'}
          </div>
        </div>

        <div className="p-3.5 rounded-xl bg-gradient-to-br from-purple-50 to-white border border-purple-100 flex flex-col justify-between">
          <div className="flex items-center justify-between text-2xs text-purple-700 font-semibold mb-1">
            <span>{isVi ? 'Top Ứng Dụng Hàng Đầu' : 'Primary Application'}</span>
            <Award size={14} className="text-purple-600" />
          </div>
          <div className="text-sm font-bold text-slate-900 truncate">
            {report.top_user_apps[0]?.app_title || (isVi ? 'Chưa ghi nhận' : 'No data')}
          </div>
          <div className="text-[11px] text-purple-700 font-mono mt-1">
            {report.top_user_apps[0]?.total_duration_formatted || '-'}
          </div>
        </div>
      </div>

      {/* Hourly Trend Chart */}
      <div className="flex flex-col gap-2 pt-2">
        <div className="flex justify-between items-center text-xs font-semibold text-slate-700">
          <span className="flex items-center gap-1.5">
            <Clock size={14} className="text-sky-600" />
            {isVi
              ? 'Phân Bổ Thời Gian Sử Dụng Theo Từng Khung Giờ Trong Ngày (24h)'
              : 'Hourly Usage Distribution Across 24h'}
          </span>
          <span className="text-2xs text-slate-400 font-normal">
            {isVi ? 'Đơn vị: phút sử dụng' : 'Unit: active minutes'}
          </span>
        </div>

        <div className="h-44 w-full">
          {hourlyChartData ? (
            <Bar data={hourlyChartData} options={chartOptions} />
          ) : (
            <div className="h-full flex items-center justify-center text-xs text-slate-400">
              {isVi ? 'Chưa có dữ liệu theo giờ' : 'No hourly data available'}
            </div>
          )}
        </div>
      </div>

      {/* Top 5 User Apps Quick List */}
      {report.top_user_apps.length > 0 && (
        <div className="pt-2 border-t border-slate-100 flex flex-col gap-2">
          <div className="text-xs font-bold text-slate-800 flex items-center gap-1.5">
            <Award size={14} className="text-amber-500" />
            <span>{isVi ? 'Top Ứng Dụng Người Dùng Dùng Nhiều Nhất' : 'Top User Applications'}</span>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-2">
            {report.top_user_apps.map((app, idx) => (
              <div
                key={app.app_name}
                className="p-2.5 rounded-lg border border-slate-200/80 bg-slate-50/50 flex flex-col justify-between"
              >
                <div className="flex items-center gap-1.5 mb-1">
                  <span className="w-5 h-5 rounded-full bg-sky-100 text-sky-800 text-2xs font-bold flex items-center justify-center shrink-0">
                    {idx + 1}
                  </span>
                  <div className="text-xs font-bold text-slate-900 truncate" title={app.app_title}>
                    {app.app_title}
                  </div>
                </div>
                <div className="flex justify-between items-center text-2xs mt-1">
                  <span className="text-slate-500">{app.category}</span>
                  <span className="font-bold font-mono text-sky-700">
                    {app.total_duration_formatted}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
