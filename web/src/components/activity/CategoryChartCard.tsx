import { PieChart } from 'lucide-react';
import type React from 'react';
import type { AppActivityItem, Language } from '../../types';
import { CategoryDoughnutChart } from '../charts/CategoryDoughnutChart';

interface CategoryChartCardProps {
  appActivities?: AppActivityItem[];
  lang: Language;
}

export const CategoryChartCard: React.FC<CategoryChartCardProps> = ({
  appActivities,
  lang,
}) => {
  const isVi = lang === 'vi';

  if (!appActivities || appActivities.length === 0) return null;

  return (
    <div className="bg-white border border-slate-200/80 rounded-2xl p-5 sm:p-6 shadow-xs flex flex-col gap-3.5">
      <div className="flex justify-between items-center flex-wrap gap-2">
        <h3 className="text-sm font-bold text-slate-900 flex items-center gap-2">
          <PieChart size={16} className="text-sky-600" />
          <span>
            {isVi
              ? 'Phân Bố Thời Lượng Theo Danh Mục Ứng Dụng'
              : 'Screen Time Distribution by Category'}
          </span>
        </h3>
        <span className="text-2xs text-slate-500 font-medium">
          {appActivities.length} {isVi ? 'ứng dụng ghi nhận' : 'recorded apps'}
        </span>
      </div>
      <CategoryDoughnutChart appActivities={appActivities} lang={lang} height={175} />
    </div>
  );
};
