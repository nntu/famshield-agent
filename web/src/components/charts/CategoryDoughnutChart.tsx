import type React from 'react';
import { useMemo } from 'react';
import { Doughnut } from 'react-chartjs-2';
import type { AppActivityItem, Language } from '../../types';
import './chartSetup';

interface CategoryDoughnutChartProps {
  appActivities: AppActivityItem[];
  lang: Language;
  height?: number;
}

export const CategoryDoughnutChart: React.FC<CategoryDoughnutChartProps> = ({
  appActivities,
  lang,
  height = 190,
}) => {
  const isVi = lang === 'vi';

  const categoryAggregates = useMemo(() => {
    const map = new Map<string, number>();
    for (const app of appActivities) {
      const cat = app.category || (isVi ? 'Khác' : 'Other');
      const cur = map.get(cat) || 0;
      map.set(cat, cur + app.duration_minutes);
    }

    const items = Array.from(map.entries()).map(([category, minutes]) => ({
      category,
      minutes,
    }));
    items.sort((a, b) => b.minutes - a.minutes);
    return items;
  }, [appActivities, isVi]);

  const totalMinutes = useMemo(() => {
    return categoryAggregates.reduce((acc, curr) => acc + curr.minutes, 0);
  }, [categoryAggregates]);

  const palette = useMemo(
    () => [
      '#0284c7', // Sky-600
      '#10b981', // Emerald-500
      '#f59e0b', // Amber-500
      '#8b5cf6', // Violet-500
      '#f43f5e', // Rose-500
      '#06b6d4', // Cyan-500
      '#64748b', // Slate-500
    ],
    []
  );

  const chartData = useMemo(() => {
    return {
      labels: categoryAggregates.map((c) => c.category),
      datasets: [
        {
          data: categoryAggregates.map((c) => c.minutes),
          backgroundColor: palette.slice(0, categoryAggregates.length),
          borderColor: '#ffffff',
          borderWidth: 2,
          hoverOffset: 4,
        },
      ],
    };
  }, [categoryAggregates, palette]);

  const options = useMemo(() => {
    return {
      responsive: true,
      maintainAspectRatio: false,
      cutout: '68%',
      plugins: {
        legend: {
          display: false, // Custom legend below for better responsive UI
        },
        tooltip: {
          callbacks: {
            label: (context: any) => {
              const val = context.parsed || 0;
              const pct = totalMinutes > 0 ? Math.round((val / totalMinutes) * 100) : 0;
              return ` ${val} ${isVi ? 'phút' : 'mins'} (${pct}%)`;
            },
          },
        },
      },
    };
  }, [totalMinutes, isVi]);

  if (categoryAggregates.length === 0 || totalMinutes === 0) {
    return (
      <div className="flex items-center justify-center h-full text-slate-400 text-xs py-8">
        {isVi ? 'Chưa có dữ liệu thời gian ứng dụng hôm nay' : 'No screen time recorded today'}
      </div>
    );
  }

  return (
    <div className="flex flex-col sm:flex-row items-center gap-4 w-full">
      {/* Doughnut canvas with center label */}
      <div style={{ height: `${height}px`, width: `${height}px` }} className="relative shrink-0">
        <Doughnut data={chartData} options={options} />
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none text-center">
          <span className="text-xl font-bold text-slate-900 leading-tight">
            {Math.floor(totalMinutes / 60)}h{totalMinutes % 60}m
          </span>
          <span className="text-3xs text-slate-400 font-medium">
            {isVi ? 'Tổng thời gian' : 'Total time'}
          </span>
        </div>
      </div>

      {/* Legend list */}
      <div className="flex-1 grid grid-cols-1 sm:grid-cols-2 gap-2 w-full text-xs">
        {categoryAggregates.map((item, idx) => {
          const color = palette[idx % palette.length];
          const pct = totalMinutes > 0 ? Math.round((item.minutes / totalMinutes) * 100) : 0;
          return (
            <div
              key={item.category}
              className="flex items-center justify-between p-2 rounded-lg bg-slate-50 border border-slate-100"
            >
              <div className="flex items-center gap-2 truncate">
                <span
                  className="w-2.5 h-2.5 rounded-full shrink-0"
                  style={{ backgroundColor: color }}
                />
                <span className="font-medium text-slate-700 truncate">{item.category}</span>
              </div>
              <div className="text-right shrink-0 font-mono text-2xs">
                <span className="font-semibold text-slate-900">{item.minutes}m</span>
                <span className="text-slate-400 ml-1">({pct}%)</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
