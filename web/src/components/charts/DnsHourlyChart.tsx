import type React from 'react';
import { useMemo, useState } from 'react';
import { Bar, Line } from 'react-chartjs-2';
import type { HourlyFrequency, Language } from '../../types';
import './chartSetup';
import { BarChart3, LineChart as LineChartIcon } from 'lucide-react';

interface DnsHourlyChartProps {
  hourlyData: HourlyFrequency[];
  lang: Language;
  peakHour: number;
  height?: number;
  allowChartTypeToggle?: boolean;
}

export const DnsHourlyChart: React.FC<DnsHourlyChartProps> = ({
  hourlyData,
  lang,
  peakHour,
  height = 160,
  allowChartTypeToggle = true,
}) => {
  const isVi = lang === 'vi';
  const [chartType, setChartType] = useState<'bar' | 'area'>('bar');

  const labels = useMemo(() => {
    return hourlyData.map((d) => `${d.hour.toString().padStart(2, '0')}:00`);
  }, [hourlyData]);

  const chartData = useMemo(() => {
    const allowedData = hourlyData.map((d) => d.allowed_count);
    const blockedData = hourlyData.map((d) => d.blocked_count);

    if (chartType === 'area') {
      return {
        labels,
        datasets: [
          {
            label: isVi ? 'Cho phép (ALLOW)' : 'Allowed',
            data: allowedData,
            fill: true,
            borderColor: '#0284c7', // sky-600
            backgroundColor: 'rgba(2, 132, 199, 0.18)',
            tension: 0.35,
            pointRadius: 2,
            pointHoverRadius: 5,
            pointBackgroundColor: '#0284c7',
          },
          {
            label: isVi ? 'Bị chặn (BLOCK)' : 'Blocked',
            data: blockedData,
            fill: true,
            borderColor: '#e11d48', // rose-600
            backgroundColor: 'rgba(225, 29, 72, 0.22)',
            tension: 0.35,
            pointRadius: 2,
            pointHoverRadius: 5,
            pointBackgroundColor: '#e11d48',
          },
        ],
      };
    }

    return {
      labels,
      datasets: [
        {
          label: isVi ? 'Cho phép' : 'Allowed',
          data: allowedData,
          backgroundColor: hourlyData.map((d) =>
            d.hour === peakHour ? 'rgba(2, 132, 199, 0.95)' : 'rgba(2, 132, 199, 0.75)'
          ),
          hoverBackgroundColor: '#0284c7',
          borderRadius: 3,
          borderSkipped: false,
          stack: 'dns',
        },
        {
          label: isVi ? 'Bị chặn' : 'Blocked',
          data: blockedData,
          backgroundColor: hourlyData.map((d) =>
            d.hour === peakHour ? 'rgba(225, 29, 72, 0.95)' : 'rgba(225, 29, 72, 0.8)'
          ),
          hoverBackgroundColor: '#e11d48',
          borderRadius: 3,
          borderSkipped: false,
          stack: 'dns',
        },
      ],
    };
  }, [hourlyData, labels, chartType, peakHour, isVi]);

  const options = useMemo(() => {
    return {
      responsive: true,
      maintainAspectRatio: false,
      animation: {
        duration: 250,
      },
      interaction: {
        mode: 'index' as const,
        intersect: false,
      },
      plugins: {
        legend: {
          display: false, // Custom legend in parent header
        },
        tooltip: {
          callbacks: {
            title: (items: any[]) => {
              if (!items.length) return '';
              const h = items[0].dataIndex;
              const nextH = (h + 1) % 24;
              const isPeak = h === peakHour;
              return `${h.toString().padStart(2, '0')}:00 - ${nextH.toString().padStart(2, '0')}:00${
                isPeak ? (isVi ? ' (Giờ cao điểm 🔥)' : ' (Peak Traffic 🔥)') : ''
              }`;
            },
            footer: (items: any[]) => {
              const total = items.reduce((acc, curr) => acc + (curr.parsed.y || 0), 0);
              return `${isVi ? 'Tổng truy vấn' : 'Total queries'}: ${total.toLocaleString()}`;
            },
          },
        },
      },
      scales: {
        x: {
          grid: {
            display: false,
          },
          ticks: {
            maxRotation: 0,
            autoSkip: true,
            maxTicksLimit: 12,
            font: {
              size: 10,
              family: 'monospace',
            },
            color: (context: any) => {
              const h = context.index;
              return h === peakHour ? '#b45309' : '#64748b';
            },
          },
        },
        y: {
          stacked: chartType === 'bar',
          beginAtZero: true,
          grid: {
            color: 'rgba(226, 232, 240, 0.6)', // slate-200
          },
          ticks: {
            precision: 0,
            font: {
              size: 10,
            },
          },
        },
      },
    };
  }, [chartType, peakHour, isVi]);

  return (
    <div className="w-full flex flex-col gap-2">
      {allowChartTypeToggle && (
        <div className="flex justify-end items-center gap-1.5 self-end">
          <span className="text-3xs text-slate-400 font-medium">
            {isVi ? 'Kiểu đồ thị:' : 'Chart Type:'}
          </span>
          <div className="inline-flex bg-white rounded-lg p-0.5 border border-slate-200 shadow-2xs">
            <button
              type="button"
              onClick={() => setChartType('bar')}
              className={`px-2 py-0.5 rounded text-2xs font-semibold flex items-center gap-1 transition-all cursor-pointer ${
                chartType === 'bar'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              <BarChart3 size={11} />
              <span>{isVi ? 'Cột' : 'Bar'}</span>
            </button>
            <button
              type="button"
              onClick={() => setChartType('area')}
              className={`px-2 py-0.5 rounded text-2xs font-semibold flex items-center gap-1 transition-all cursor-pointer ${
                chartType === 'area'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              <LineChartIcon size={11} />
              <span>{isVi ? 'Đường mượt' : 'Area'}</span>
            </button>
          </div>
        </div>
      )}

      <div style={{ height: `${height}px` }} className="w-full relative">
        {chartType === 'bar' ? (
          <Bar data={chartData} options={options} />
        ) : (
          <Line data={chartData} options={options} />
        )}
      </div>
    </div>
  );
};
