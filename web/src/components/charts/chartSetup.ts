import {
  ArcElement,
  BarElement,
  CategoryScale,
  Chart as ChartJS,
  Filler,
  Legend,
  LineElement,
  LinearScale,
  PointElement,
  Title,
  Tooltip,
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  PointElement,
  LineElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

// Configure Chart.js global defaults for clean modern light theme
ChartJS.defaults.font.family =
  'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
ChartJS.defaults.font.size = 11;
ChartJS.defaults.color = '#64748b'; // slate-500
ChartJS.defaults.plugins.tooltip.backgroundColor = '#0f172a'; // slate-900
ChartJS.defaults.plugins.tooltip.titleColor = '#f8fafc'; // slate-50
ChartJS.defaults.plugins.tooltip.bodyColor = '#e2e8f0'; // slate-200
ChartJS.defaults.plugins.tooltip.padding = 10;
ChartJS.defaults.plugins.tooltip.cornerRadius = 8;
ChartJS.defaults.plugins.tooltip.boxPadding = 4;

export { ChartJS };
