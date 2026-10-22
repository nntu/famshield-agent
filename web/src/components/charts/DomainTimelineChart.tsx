import type React from 'react';
import { useMemo } from 'react';
import { Bar } from 'react-chartjs-2';
import type { Language, QueryLogRecord } from '../../types';
import './chartSetup';

interface DomainTimelineChartProps {
  logs: QueryLogRecord[];
  lang: Language;
  height?: number;
}

export const DomainTimelineChart: React.FC<DomainTimelineChartProps> = ({
  logs,
  lang,
  height = 130,
}) => {
  const isVi = lang === 'vi';

  const hourlyCounts = useMemo(() => {
    const buckets = Array.from({ length: 24 }, (_, i) => ({
      hour: i,
      allowed: 0,
      blocked: 0,
      monitored: 0,
      total: 0,
    }));

    for (const log of logs) {
      const dt = new Date(log.occurred_at);
      if (isNaN(dt.getTime())) continue;
      const h = dt.getHours();
      buckets[h].total++;
      if (log.action === 'BLOCK') buckets[h].blocked++;
      else if (log.action === 'MONITOR') buckets[h].monitored++;
      else buckets[h].allowed++;
    }

    return buckets;
  }, [logs]);

  const labels = useMemo(() => {
    return hourlyCounts.map((b) => `${b.hour.toString().padStart(2, '0')}:00`);
  }, [hourlyCounts]);

  const chartData = useMemo(() => {
    return {
      labels,
      datasets: [
        {
          label: isVi ? 'Cho phép' : 'Allowed',
          data: hourlyCounts.map((b) => b.allowed),
          backgroundColor: 'rgba(2, 132, 199, 0.8)',
          hoverBackgroundColor: '#0284c7',
          borderRadius: 2,
          stack: 'domain',
        },
        {
          label: isVi ? 'Bị chặn' : 'Blocked',
          data: hourlyCounts.map((b) => b.blocked),
          backgroundColor: 'rgba(225, 29, 72, 0.85)',
          hoverBackgroundColor: '#e11d48',
          borderRadius: 2,
          stack: 'domain',
        },
      ],
    };
  }, [hourlyCounts, labels, isVi]);

  const options = useMemo(() => {
    return {
      responsive: true,
      maintainAspectRatio: false,
      interaction: {
        mode: 'index' as const,
        intersect: false,
      },
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          callbacks: {
            title: (items: any[]) => {
              if (!items.length) return '';
              const h = items[0].dataIndex;
              const nextH = (h + 1) % 24;
              return `${h.toString().padStart(2, '0')}:00 - ${nextH.toString().padStart(2, '0')}:00`;
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
              size: 9,
              family: 'monospace',
            },
          },
        },
        y: {
          stacked: true,
          beginAtZero: true,
          grid: {
            color: 'rgba(226, 232, 240, 0.6)',
          },
          ticks: {
            precision: 0,
            font: {
              size: 9,
            },
          },
        },
      },
    };
  }, []);

  return (
    <div style={{ height: `${height}px` }} className="w-full relative">
      <Bar data={chartData} options={options} />
    </div>
  );
};
