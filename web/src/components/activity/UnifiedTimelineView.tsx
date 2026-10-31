import { AlertTriangle, Globe, Smartphone } from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type { Language, UnifiedTimelineItem } from '../../types';

interface UnifiedTimelineViewProps {
  filteredTimeline: UnifiedTimelineItem[];
  getCategoryIcon: (category: string) => React.ReactNode;
  lang: Language;
}

export const UnifiedTimelineView: React.FC<UnifiedTimelineViewProps> = ({
  filteredTimeline,
  getCategoryIcon,
  lang,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  if (filteredTimeline.length === 0) {
    return <div className="text-center py-10 text-slate-500 text-sm">{t.no_activity_found}</div>;
  }

  return (
    <div className="flex flex-col gap-2.5">
      {filteredTimeline.map((item) => {
        const isApp = item.item_type === 'app';
        const isKeyword = item.item_type === 'keyword';
        const dateObj = new Date(item.timestamp);
        const timeFormatted = dateObj.toLocaleTimeString([], {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
        });

        return (
          <div
            key={item.id}
            className={`border rounded-xl p-3.5 flex items-center justify-between flex-wrap gap-3 transition-colors ${
              isKeyword
                ? 'bg-amber-50/50 border-amber-200/80 hover:bg-amber-50'
                : 'bg-slate-50/60 border-slate-200/80 hover:bg-slate-100/60'
            }`}
          >
            <div className="flex items-center gap-3">
              <div
                className={`w-9 h-9 rounded-xl flex items-center justify-center shrink-0 ${
                  isKeyword
                    ? 'bg-amber-100 text-amber-700'
                    : isApp
                      ? 'bg-sky-50 text-sky-700'
                      : 'bg-sky-50 text-sky-700'
                }`}
              >
                {isKeyword ? (
                  <AlertTriangle size={17} />
                ) : isApp ? (
                  <Smartphone size={17} />
                ) : (
                  <Globe size={17} />
                )}
              </div>

              <div>
                <div className="flex items-center gap-2 mb-0.5">
                  <span className="font-bold text-slate-900 text-sm">{item.title}</span>
                  <span className="bg-white text-slate-600 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium inline-flex items-center gap-1 shadow-2xs">
                    {getCategoryIcon(item.category)}
                    {item.category}
                  </span>
                </div>

                <div className="text-xs text-slate-500">{item.subtitle}</div>
              </div>
            </div>

            <div className="flex items-center gap-3.5">
              <div className="text-right">
                <div className="text-xs font-mono font-bold text-slate-800">{timeFormatted}</div>
                <div className="text-2xs text-slate-400">{dateObj.toLocaleDateString()}</div>
              </div>

              {/* Action Badge */}
              <span
                className={`px-2.5 py-0.5 rounded-full font-bold text-2xs border ${
                  item.action === 'VIOLATION'
                    ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                    : item.action === 'BLOCK'
                      ? 'bg-rose-50 text-rose-700 border-rose-200/80'
                      : item.action === 'MONITOR'
                        ? 'bg-amber-50 text-amber-700 border-amber-200/80'
                        : item.action === 'ACTIVE'
                          ? 'bg-sky-50 text-sky-700 border-sky-200/80'
                          : 'bg-emerald-50 text-emerald-700 border-emerald-200/80'
                }`}
              >
                {item.action === 'VIOLATION'
                  ? isVi
                    ? '⚠️ TỪ KHÓA VI PHẠM'
                    : '⚠️ KEYWORD VIOLATION'
                  : item.action === 'BLOCK'
                    ? t.tag_block
                    : item.action === 'MONITOR'
                      ? isVi
                        ? 'GIÁM SÁT'
                        : 'MONITOR'
                      : item.action === 'ACTIVE'
                        ? isVi
                          ? 'ĐANG CHẠY'
                          : 'ACTIVE'
                        : t.tag_allow}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
};
