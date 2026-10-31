import type React from 'react';
import { Keyboard, Check, Plus, Search, Trash2 } from 'lucide-react';
import type { KeywordRule } from '../../types';
import { SYSTEM_KEYWORD_CATEGORIES } from './rulesTypes';

interface KeywordsSubTabProps {
  isVi: boolean;
  isKeywordsEnabled: boolean;
  isTogglingKeywords: boolean;
  onToggleKeyboardMonitoring: () => Promise<void>;
  customKeywords: KeywordRule[];
  newKwPattern: string;
  setNewKwPattern: (val: string) => void;
  newKwCategory: 'bullying' | 'profanity' | 'adult' | 'self_harm' | 'custom';
  setNewKwCategory: (val: 'bullying' | 'profanity' | 'adult' | 'self_harm' | 'custom') => void;
  newKwSeverity: 'critical' | 'high' | 'medium' | 'low';
  setNewKwSeverity: (val: 'critical' | 'high' | 'medium' | 'low') => void;
  isAddingKw: boolean;
  onAddCustomKeyword: () => Promise<void>;
  kwSearch: string;
  setKwSearch: (val: string) => void;
  filteredKeywords: KeywordRule[];
  onToggleKeyword: (id: string, currentEnabled: boolean) => Promise<void>;
  onDeleteCustomKeyword: (id: string) => Promise<void>;
}

export const KeywordsSubTab: React.FC<KeywordsSubTabProps> = ({
  isVi,
  isKeywordsEnabled,
  isTogglingKeywords,
  onToggleKeyboardMonitoring,
  customKeywords,
  newKwPattern,
  setNewKwPattern,
  newKwCategory,
  setNewKwCategory,
  newKwSeverity,
  setNewKwSeverity,
  isAddingKw,
  onAddCustomKeyword,
  kwSearch,
  setKwSearch,
  filteredKeywords,
  onToggleKeyword,
  onDeleteCustomKeyword,
}) => {
  return (
    <div className="space-y-6">
      {/* Master Toggle Banner */}
      <div className="flex flex-wrap items-center justify-between gap-4 p-5 rounded-2xl bg-indigo-50/70 border border-indigo-200 shadow-xs">
        <div className="flex items-center gap-3.5">
          <div className="w-10 h-10 rounded-xl bg-indigo-600 text-white flex items-center justify-center shrink-0 shadow-2xs">
            <Keyboard className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-base font-bold text-slate-900">
              {isVi ? 'Bộ Lọc Nhận Diện Bàn Phím Trực Tiếp (Aho-Corasick RAM Scanner)' : 'Live Keystroke Scanner'}
            </h3>
            <p className="text-xs text-slate-600 mt-0.5 max-w-2xl leading-relaxed">
              {isVi
                ? 'Thuật toán Aho-Corasick chạy ngầm dưới RAM cực nhẹ (≤ 2 MB). Nhận diện tức thời từ khóa nhạy cảm không gây lag hệ thống.'
                : 'Aho-Corasick scanner runs in RAM (≤ 2 MB) with zero latency impact.'}
            </p>
          </div>
        </div>

        <button
          type="button"
          disabled={isTogglingKeywords}
          onClick={onToggleKeyboardMonitoring}
          className={`flex items-center gap-2 px-5 py-2.5 rounded-xl font-bold text-xs shadow-xs transition cursor-pointer ${
            isKeywordsEnabled
              ? 'bg-emerald-600 hover:bg-emerald-700 text-white'
              : 'bg-slate-300 hover:bg-slate-400 text-slate-800'
          }`}
        >
          <Check className="w-4 h-4" />
          <span>
            {isKeywordsEnabled
              ? (isVi ? 'Đang Bật Giám Sát' : 'Monitoring Active')
              : (isVi ? 'Đã Tắt Giám Sát' : 'Monitoring Off')}
          </span>
        </button>
      </div>

      {/* 4 System Keyword Categories */}
      <div className="space-y-3">
        <h4 className="text-xs font-bold text-slate-500 uppercase tracking-wider">
          {isVi ? '4 Danh Mục Từ Khóa Hệ Thống Tự Động' : '4 Built-in System Keyword Categories'}
        </h4>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {SYSTEM_KEYWORD_CATEGORIES.map((cat) => (
            <div
              key={cat.id}
              className="rounded-2xl bg-white border border-slate-200/90 p-4 shadow-xs flex flex-col justify-between"
            >
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-2xl">{cat.icon}</span>
                  <span
                    className={`text-[10px] font-bold px-2 py-0.5 rounded-full uppercase ${
                      cat.severity === 'critical'
                        ? 'bg-rose-100 text-rose-800'
                        : cat.severity === 'high'
                        ? 'bg-amber-100 text-amber-800'
                        : 'bg-sky-100 text-sky-800'
                    }`}
                  >
                    {cat.severity}
                  </span>
                </div>
                <div className="font-bold text-xs text-slate-900 mb-1">
                  {isVi ? cat.name : cat.nameEn}
                </div>
                <p className="text-[11px] text-slate-500 mb-3 leading-relaxed">
                  {isVi ? cat.desc : cat.descEn}
                </p>
              </div>

              <div className="pt-2 border-t border-slate-100">
                <div className="text-[10px] font-semibold text-slate-400 mb-1">
                  {isVi ? 'Từ mẫu:' : 'Samples:'}
                </div>
                <div className="flex flex-wrap gap-1">
                  {cat.samples.map((w) => (
                    <span
                      key={w}
                      className="text-[10px] font-mono bg-slate-50 border border-slate-200 px-1.5 py-0.5 rounded text-slate-600"
                    >
                      {w}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Custom Keywords Management */}
      <div className="rounded-2xl bg-white border border-slate-200/80 p-5 sm:p-6 shadow-xs space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-slate-100">
          <div>
            <h4 className="text-sm font-bold text-slate-900">
              {isVi ? 'Danh Sách Từ Khóa Tự Định Nghĩa (Custom Keywords)' : 'Custom Keywords'}
            </h4>
            <p className="text-xs text-slate-500 mt-0.5">
              {isVi
                ? 'Thêm các từ khóa riêng cần giám sát (ví dụ tên hội nhóm xấu, từ lóng...)'
                : 'Add specific phrases or slangs to monitor.'}
            </p>
          </div>
          <span className="text-xs font-bold px-2.5 py-0.5 rounded-full bg-indigo-100 text-indigo-800 border border-indigo-200">
            {customKeywords.length} {isVi ? 'từ khóa riêng' : 'custom words'}
          </span>
        </div>

        {/* Add Keyword Form */}
        <div className="flex flex-wrap gap-2.5">
          <input
            type="text"
            placeholder={isVi ? 'Nhập từ khóa theo dõi (ví dụ: bốc bát họ, trốn học...)' : 'Enter phrase...'}
            value={newKwPattern}
            onChange={(e) => setNewKwPattern(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && onAddCustomKeyword()}
            className="flex-3 min-w-[200px] rounded-xl bg-white border border-slate-300 focus:border-indigo-600 focus:ring-2 focus:ring-indigo-100 px-3.5 py-2 text-xs text-slate-900 placeholder:text-slate-400 outline-none transition font-mono"
          />
          <select
            value={newKwCategory}
            onChange={(e) => setNewKwCategory(e.target.value as any)}
            className="flex-1 min-w-[140px] rounded-xl bg-white border border-slate-300 px-3 py-2 text-xs text-slate-900 outline-none"
          >
            <option value="custom">{isVi ? 'Tùy chỉnh' : 'Custom'}</option>
            <option value="bullying">{isVi ? 'Bắt nạt / Đe dọa' : 'Bullying'}</option>
            <option value="profanity">{isVi ? 'Chửi bậy' : 'Profanity'}</option>
            <option value="adult">{isVi ? 'Người lớn 18+' : 'Adult'}</option>
            <option value="self_harm">{isVi ? 'Tự hại' : 'Self-harm'}</option>
          </select>
          <select
            value={newKwSeverity}
            onChange={(e) => setNewKwSeverity(e.target.value as any)}
            className="flex-1 min-w-[120px] rounded-xl bg-white border border-slate-300 px-3 py-2 text-xs text-slate-900 outline-none"
          >
            <option value="critical">{isVi ? 'Nguy cấp (Critical)' : 'Critical'}</option>
            <option value="high">{isVi ? 'Cao (High)' : 'High'}</option>
            <option value="medium">{isVi ? 'Vừa (Medium)' : 'Medium'}</option>
            <option value="low">{isVi ? 'Thấp (Low)' : 'Low'}</option>
          </select>
          <button
            type="button"
            disabled={isAddingKw || !newKwPattern.trim()}
            onClick={onAddCustomKeyword}
            className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white font-semibold px-5 py-2 text-xs transition cursor-pointer shadow-xs whitespace-nowrap"
          >
            <Plus className="w-4 h-4" />
            <span>{isVi ? 'Thêm Từ Khóa' : 'Add Keyword'}</span>
          </button>
        </div>

        {/* Keyword Search & List */}
        {customKeywords.length > 5 && (
          <div className="relative">
            <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder={isVi ? 'Tìm kiếm trong danh sách từ khóa...' : 'Search keywords...'}
              value={kwSearch}
              onChange={(e) => setKwSearch(e.target.value)}
              className="w-full pl-8 pr-3 py-1.5 rounded-lg border border-slate-200 text-xs font-mono outline-none"
            />
          </div>
        )}

        <div className="flex flex-wrap gap-2 max-h-72 overflow-y-auto">
          {filteredKeywords.length > 0 ? (
            filteredKeywords.map((r) => (
              <span
                key={r.id}
                className={`inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-mono font-medium shadow-2xs ${
                  r.enabled
                    ? 'bg-indigo-50 border-indigo-200 text-indigo-900'
                    : 'bg-slate-100 border-slate-200 text-slate-500 line-through'
                }`}
              >
                <span>{r.pattern}</span>
                <span
                  className={`text-[9px] px-1 py-0.2 rounded font-sans font-bold uppercase ${
                    r.severity === 'critical'
                      ? 'bg-rose-100 text-rose-800'
                      : r.severity === 'high'
                      ? 'bg-amber-100 text-amber-800'
                      : 'bg-slate-200 text-slate-700'
                  }`}
                >
                  {r.severity}
                </span>
                <button
                  type="button"
                  onClick={() => onToggleKeyword(r.id, r.enabled)}
                  className="text-slate-400 hover:text-indigo-600 cursor-pointer ml-1"
                  title={r.enabled ? 'Tạm tắt' : 'Bật lại'}
                >
                  {r.enabled ? '✓' : '✗'}
                </button>
                {r.category === 'custom' && (
                  <button
                    type="button"
                    onClick={() => onDeleteCustomKeyword(r.id)}
                    className="text-slate-400 hover:text-rose-600 cursor-pointer ml-0.5"
                    title="Xóa từ khóa"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                )}
              </span>
            ))
          ) : (
            <div className="text-xs text-slate-400 italic p-6 rounded-xl bg-slate-50/70 border border-dashed border-slate-200 text-center w-full">
              {isVi ? 'Chưa có từ khóa tùy chỉnh nào.' : 'No custom keywords configured.'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
