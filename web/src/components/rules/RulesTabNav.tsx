import { Keyboard, Layers, Sliders } from 'lucide-react';
import type React from 'react';
import type { FilterSubTab } from './rulesTypes';

interface RulesTabNavProps {
  activeSubTab: FilterSubTab;
  setActiveSubTab: (tab: FilterSubTab) => void;
  blockedStandardCount: number;
  blacklistCount: number;
  whitelistCount: number;
  isKeywordsEnabled: boolean;
  isVi: boolean;
}

export const RulesTabNav: React.FC<RulesTabNavProps> = ({
  activeSubTab,
  setActiveSubTab,
  blockedStandardCount,
  blacklistCount,
  whitelistCount,
  isKeywordsEnabled,
  isVi,
}) => {
  return (
    <div className="bg-white rounded-2xl border border-slate-200/80 p-2 shadow-xs">
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
        {/* Tab 1: Categories */}
        <button
          type="button"
          onClick={() => setActiveSubTab('categories')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'categories'
              ? 'bg-sky-50 text-sky-900 border border-sky-300 ring-2 ring-sky-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'categories'
                ? 'bg-sky-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Layers className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '1. Danh Mục Lọc Web' : '1. Web Categories'}
              </span>
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                  activeSubTab === 'categories'
                    ? 'bg-sky-200/80 text-sky-900'
                    : 'bg-slate-100 text-slate-600'
                }`}
              >
                {blockedStandardCount}/5 {isVi ? 'Chặn' : 'Block'}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi
                ? '5 danh mục chuẩn & thẩm định tên miền'
                : '5 standard categories & domain inspector'}
            </p>
          </div>
        </button>

        {/* Tab 2: Custom Rules (Blacklist & Whitelist) */}
        <button
          type="button"
          onClick={() => setActiveSubTab('rules')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'rules'
              ? 'bg-sky-50 text-sky-900 border border-sky-300 ring-2 ring-sky-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'rules'
                ? 'bg-sky-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Sliders className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '2. Chặn & Ngoại Lệ' : '2. Blacklist & Whitelist'}
              </span>
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                  activeSubTab === 'rules'
                    ? 'bg-sky-200/80 text-sky-900'
                    : 'bg-slate-100 text-slate-600'
                }`}
              >
                {blacklistCount} {isVi ? 'Cấm' : 'Block'} • {whitelistCount}{' '}
                {isVi ? 'Cho phép' : 'Allow'}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi
                ? 'Blacklist, Whitelist & Giới hạn Ứng dụng'
                : 'Blacklist, Whitelist & App limits'}
            </p>
          </div>
        </button>

        {/* Tab 3: Keyword Monitoring */}
        <button
          type="button"
          onClick={() => setActiveSubTab('keywords')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'keywords'
              ? 'bg-indigo-50/80 text-indigo-950 border border-indigo-300 ring-2 ring-indigo-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'keywords'
                ? 'bg-indigo-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Keyboard className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '3. Giám Sát Bàn Phím' : '3. Keyboard Monitoring'}
              </span>
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                  isKeywordsEnabled
                    ? 'bg-emerald-100 text-emerald-800'
                    : 'bg-slate-200 text-slate-700'
                }`}
              >
                {isKeywordsEnabled ? (isVi ? 'Đang bật' : 'Active') : isVi ? 'Đã tắt' : 'Disabled'}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi
                ? 'Aho-Corasick scan & từ khóa nhạy cảm'
                : 'Aho-Corasick scan & sensitive keywords'}
            </p>
          </div>
        </button>
      </div>
    </div>
  );
};
