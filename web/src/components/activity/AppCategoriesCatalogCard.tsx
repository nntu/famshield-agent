import type React from 'react';
import {
  Gamepad2,
  Globe,
  MessageCircle,
  BookOpen,
  Code2,
  Tv,
  Sliders,
  Cpu,
  Layers,
  Sparkles,
  CheckCircle2,
} from 'lucide-react';
import type { AppCategoryMetadata, Language } from '../../types';

interface AppCategoriesCatalogCardProps {
  categories: AppCategoryMetadata[];
  selectedCategory: string;
  onSelectCategory: (catId: string) => void;
  lang: Language;
}

export const AppCategoriesCatalogCard: React.FC<AppCategoriesCatalogCardProps> = ({
  categories,
  selectedCategory,
  onSelectCategory,
  lang,
}) => {
  const isVi = lang === 'vi';

  const getCategoryIcon = (iconName: string, isSystem: boolean) => {
    switch (iconName) {
      case 'Gamepad2':
        return <Gamepad2 size={16} className="text-amber-500" />;
      case 'Globe':
        return <Globe size={16} className="text-sky-500" />;
      case 'MessageCircle':
        return <MessageCircle size={16} className="text-emerald-500" />;
      case 'BookOpen':
        return <BookOpen size={16} className="text-indigo-500" />;
      case 'Code2':
        return <Code2 size={16} className="text-purple-500" />;
      case 'Tv':
        return <Tv size={16} className="text-rose-500" />;
      case 'Sliders':
        return <Sliders size={16} className="text-cyan-500" />;
      case 'Cpu':
        return <Cpu size={16} className="text-slate-400" />;
      default:
        return isSystem ? (
          <Cpu size={16} className="text-slate-400" />
        ) : (
          <Layers size={16} className="text-blue-500" />
        );
    }
  };

  const totalMinutes = categories.reduce((acc, c) => acc + c.total_duration_minutes, 0);

  return (
    <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col gap-4">
      <div className="flex justify-between items-center flex-wrap gap-2 pb-1 border-b border-slate-100">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-sky-50 border border-sky-200 flex items-center justify-center text-sky-600">
            <Sparkles size={16} />
          </div>
          <div>
            <h3 className="text-sm font-bold text-slate-900">
              {isVi ? 'Danh Mục Phân Loại Ứng Dụng' : 'Application Categories Catalog'}
            </h3>
            <p className="text-2xs text-slate-500">
              {isVi
                ? 'Phân nhóm ứng dụng thông minh, ưu tiên ứng dụng người dùng'
                : 'Smart categorization, prioritizing user interactive apps'}
            </p>
          </div>
        </div>

        {selectedCategory !== 'ALL' && (
          <button
            type="button"
            onClick={() => onSelectCategory('ALL')}
            className="text-2xs text-sky-600 hover:text-sky-800 font-semibold px-2.5 py-1 rounded-md bg-sky-50 hover:bg-sky-100 transition-colors cursor-pointer"
          >
            {isVi ? '✕ Xem tất cả danh mục' : '✕ View all categories'}
          </button>
        )}
      </div>

      {/* Categories Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        {categories.map((cat) => {
          const isSelected =
            selectedCategory === cat.id ||
            (selectedCategory !== 'ALL' && cat.name_vi.toLowerCase().includes(selectedCategory.toLowerCase()));
          const formattedTime =
            cat.total_duration_minutes >= 60
              ? `${Math.floor(cat.total_duration_minutes / 60)}h ${cat.total_duration_minutes % 60}m`
              : `${cat.total_duration_minutes} ${isVi ? 'phút' : 'mins'}`;

          return (
            <button
              key={cat.id}
              type="button"
              onClick={() => onSelectCategory(isSelected ? 'ALL' : cat.id)}
              className={`p-3 rounded-xl border text-left transition-all cursor-pointer relative overflow-hidden flex flex-col justify-between ${
                isSelected
                  ? 'border-sky-500 bg-sky-50/60 shadow-xs ring-1 ring-sky-400'
                  : 'border-slate-200/80 bg-slate-50/40 hover:bg-white hover:border-slate-300'
              }`}
            >
              <div className="flex items-start justify-between gap-2 mb-2">
                <div className="flex items-center gap-2">
                  <div
                    className={`w-7 h-7 rounded-lg flex items-center justify-center border ${
                      cat.is_system
                        ? 'bg-slate-100 border-slate-200'
                        : 'bg-white border-slate-200/90 shadow-2xs'
                    }`}
                  >
                    {getCategoryIcon(cat.icon, cat.is_system)}
                  </div>
                  <div>
                    <div className="text-xs font-bold text-slate-800 flex items-center gap-1.5">
                      <span>{isVi ? cat.name_vi : cat.name_en}</span>
                      {cat.is_system && (
                        <span className="text-[10px] px-1.5 py-0.2 rounded bg-slate-200 text-slate-600 font-normal">
                          {isVi ? 'Hệ thống' : 'System'}
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-slate-500">
                      {cat.app_count} {isVi ? 'ứng dụng' : 'apps'}
                    </div>
                  </div>
                </div>

                {isSelected && (
                  <CheckCircle2 size={15} className="text-sky-600 shrink-0 mt-0.5" />
                )}
              </div>

              {/* Progress and duration */}
              <div className="mt-1 pt-1.5 border-t border-slate-200/50">
                <div className="flex justify-between items-center text-2xs mb-1">
                  <span className="text-slate-500 font-medium">{isVi ? 'Thời lượng' : 'Usage'}:</span>
                  <span className="font-bold font-mono text-slate-900">{formattedTime}</span>
                </div>
                <div className="w-full bg-slate-200/70 rounded-full h-1.5 overflow-hidden">
                  <div
                    className={`h-1.5 rounded-full ${
                      cat.is_system ? 'bg-slate-400' : 'bg-sky-500'
                    }`}
                    style={{
                      width: `${Math.min(100, Math.max(cat.total_duration_minutes > 0 ? 5 : 0, cat.percentage))}%`,
                    }}
                  />
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
};
