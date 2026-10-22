import type React from 'react';
import { useMemo } from 'react';
import { Bar } from 'react-chartjs-2';
import type { AppUsageSummary, Language } from '../../types';
import './chartSetup';

interface AppUsageRankingChartProps {
  apps: AppUsageSummary[];
  lang: Language;
  maxItems?: number;
  height?: number;
}

export const AppUsageRankingChart: React.FC<AppUsageRankingChartProps> = ({
  apps,
  lang,
  maxItems = 7,
  height = 200,
}) => {
  const isVi = lang === 'vi';

  const topApps = useMemo(() => {
    return [...apps]
      .sort((a, b) => b.total_duration_minutes - a.total_duration_minutes)
      .slice(0, maxItems);
  }, [apps, maxItems]);

  const labels = useMemo(() => {
    return topApps.map((a) => a.app_title || a.app_name);
  }, [topApps]);

  const chartData = useMemo(() => {
    return {
      labels,
      datasets: [
        {
          label: isVi ? 'Thời gian dùng (phút)' : 'Usage Time (mins)',
          data: topApps.map((a) => a.total_duration_minutes),
          backgroundColor: topApps.map((a) => {
            if (a.is_active_now) return 'rgba(16, 185, 129, 0.85)'; // Emerald for active
            if (a.status === 'QuotaWarning' || a.total_duration_minutes > 60)
              return 'rgba(245, 158, 11, 0.85)'; // Amber for warning
            return 'rgba(2, 132, 199, 0.85)'; // Sky for normal
          }),
          hoverBackgroundColor: topApps.map((a) => {
            if (a.is_active_now) return '#10b981';
            if (a.status === 'QuotaWarning' || a.total_duration_minutes > 60) return '#f59e0b';
            return '#0284c7';
          }),
          borderRadius: 4,
          borderSkipped: false,
          barThickness: 16,
        },
      ],
    };
  }, [topApps, labels, isVi]);

  const options = useMemo(() => {
    return {
      indexAxis: 'y' as const,
      responsive: true,
      maintainAspectRatio: false,
      animation: {
        duration: 300,
      },
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          callbacks: {
            label: (context: any) => {
              const app = topApps[context.dataIndex];
              if (!app) return '';
              const mins = app.total_duration_minutes;
              const formatted = app.total_duration_formatted;
              const pct = Math.round(app.percentage);
              return ` ${formatted} (${mins} ${isVi ? 'phút' : 'mins'}, ${pct}%)`;
            },
            afterLabel: (context: any) => {
              const app = topApps[context.dataIndex];
              if (!app) return '';
              const sessions = `${isVi ? 'Số lần mở' : 'Sessions'}: ${app.session_count}`;
              const activeText = app.is_active_now
                ? ` • ${isVi ? 'Đang chạy' : 'Active now'}`
                : '';
              return `${sessions}${activeText}`;
            },
          },
        },
      },
      scales: {
        x: {
          beginAtZero: true,
          grid: {
            color: 'rgba(226, 232, 240, 0.6)',
          },
          ticks: {
            font: {
              size: 10,
            },
            callback: (val: any) => `${val}m`,
          },
        },
        y: {
          grid: {
            display: false,
          },
          ticks: {
            font: {
              size: 11,
              weight: 'bold' as const,
            },
            color: '#1e293b',
          },
        },
      },
    };
  }, [topApps, isVi]);

  if (topApps.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-slate-400 text-xs py-8">
        {isVi ? 'Chưa có ứng dụng nào được sử dụng' : 'No application usage recorded'}
      </div>
    );
  }

  return (
    <div style={{ height: `${height}px` }} className="w-full relative">
      <Bar data={chartData} options={options} />
    </div>
  );
};
