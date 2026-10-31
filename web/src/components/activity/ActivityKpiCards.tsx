import { AlertTriangle, Clock, Globe, ShieldAlert, Smartphone } from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type { ActivitySummaryResponse, Language } from '../../types';

interface ActivityKpiCardsProps {
  data: ActivitySummaryResponse | null;
  activeSubTab: 'unified' | 'apps' | 'web' | 'keywords';
  onSelectKeywordsTab: () => void;
  lang: Language;
}

export const ActivityKpiCards: React.FC<ActivityKpiCardsProps> = ({
  data,
  activeSubTab,
  onSelectKeywordsTab,
  lang,
}) => {
  const t = translations[lang];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
      {/* Card 1: Total Screen Time */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
        <div className="flex justify-between items-start mb-2">
          <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
            <Clock size={15} className="text-sky-600" />
            {t.stat_screen_time}
          </span>
          <span className="bg-sky-50 text-sky-700 px-2 py-0.5 rounded text-2xs font-mono font-bold border border-sky-100">
            {data?.date || new Date().toISOString().slice(0, 10)}
          </span>
        </div>
        <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
          {data?.total_screen_time_formatted || '0 phút'}
        </div>
        <div className="text-2xs text-slate-400">{t.stat_screen_time_sub}</div>
      </div>

      {/* Card 2: Applications Used */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
        <div className="flex justify-between items-start mb-2">
          <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
            <Smartphone size={15} className="text-sky-600" />
            {t.stat_apps_opened}
          </span>
          <span className="bg-sky-50 text-sky-700 px-2 py-0.5 rounded text-2xs font-bold border border-sky-100">
            {data?.active_apps_count || 0} apps
          </span>
        </div>
        <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
          {data?.active_apps_count || 0}
        </div>
        <div className="text-2xs text-slate-400">{t.stat_apps_opened_sub}</div>
      </div>

      {/* Card 3: Websites Visited */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
        <div className="flex justify-between items-start mb-2">
          <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
            <Globe size={15} className="text-emerald-600" />
            {t.stat_web_visits}
          </span>
          <span className="bg-emerald-50 text-emerald-700 px-2 py-0.5 rounded text-2xs font-bold border border-emerald-100">
            DNS
          </span>
        </div>
        <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
          {data?.total_web_visits_count || 0}
        </div>
        <div className="text-2xs text-slate-400">{t.stat_web_visits_sub}</div>
      </div>

      {/* Card 4: Blocked & Monitored Violations */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
        <div className="flex justify-between items-start mb-2">
          <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
            <ShieldAlert size={15} className="text-rose-600" />
            {t.stat_violations}
          </span>
          <span className="bg-rose-50 text-rose-700 px-2 py-0.5 rounded text-2xs font-bold border border-rose-100">
            {data?.blocked_events_count || 0} Block
          </span>
        </div>
        <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
          {(data?.blocked_events_count || 0) + (data?.monitored_events_count || 0)}
        </div>
        <div className="text-2xs text-slate-400">{t.stat_violations_sub}</div>
      </div>

      {/* Card 5: Sensitive Keyword Violations */}
      <div
        onClick={onSelectKeywordsTab}
        className={`bg-white border rounded-2xl p-5 shadow-xs flex flex-col justify-between cursor-pointer transition-all ${
          activeSubTab === 'keywords'
            ? 'border-amber-400 ring-1 ring-amber-400'
            : 'border-slate-200/80 hover:border-slate-300'
        }`}
      >
        <div className="flex justify-between items-start mb-2">
          <span className="text-xs text-slate-500 font-semibold flex items-center gap-1.5">
            <AlertTriangle size={15} className="text-amber-600" />
            {t.stat_keywords}
          </span>
          <span className="bg-amber-50 text-amber-700 px-2 py-0.5 rounded text-2xs font-bold border border-amber-100">
            RAM
          </span>
        </div>
        <div className="text-2xl font-bold text-slate-900 tracking-tight my-1">
          {data?.keyword_violations_count || 0}
        </div>
        <div className="text-2xs text-slate-400">{t.stat_keywords_sub}</div>
      </div>
    </div>
  );
};
