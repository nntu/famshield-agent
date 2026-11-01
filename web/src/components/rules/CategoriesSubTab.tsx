import type React from 'react';
import {
  Search,
  Globe,
  XCircle,
  CheckCircle2,
  Plus,
  Layers,
  FileText,
  ChevronRight,
  ShieldAlert,
  ShieldCheck,
} from 'lucide-react';
import type { CloudPolicyData, ResolvePolicyResult } from '../../types';
import { STANDARD_CATEGORIES } from './rulesTypes';

interface CategoriesSubTabProps {
  isVi: boolean;
  blockedStandardCount: number;
  whitelistCount: number;
  blacklistCount: number;
  cloudPolicy: CloudPolicyData | null;
  inspectInput: string;
  setInspectInput: (val: string) => void;
  isInspecting: boolean;
  inspectResult: ResolvePolicyResult | null;
  handleInspectDomain: () => Promise<void>;
  handleAddAllowedDomain: (domain: string) => Promise<void>;
  handleAddBlockedDomain: (domain: string) => Promise<void>;
  onOpenCategoryDetail: (catId: string) => void;
  onToggleCategoryBlock: (catId: string, blocked: boolean) => Promise<void>;
}

export const CategoriesSubTab: React.FC<CategoriesSubTabProps> = ({
  isVi,
  blockedStandardCount,
  whitelistCount,
  blacklistCount,
  cloudPolicy,
  inspectInput,
  setInspectInput,
  isInspecting,
  inspectResult,
  handleInspectDomain,
  handleAddAllowedDomain,
  handleAddBlockedDomain,
  onOpenCategoryDetail,
  onToggleCategoryBlock,
}) => {
  return (
    <div className="space-y-6">
      {/* Status Badges */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div className="flex items-center gap-3 p-3.5 rounded-xl bg-white border border-slate-200/80 shadow-xs">
          <div className="w-9 h-9 rounded-lg bg-rose-50 border border-rose-200 flex items-center justify-center text-rose-600 font-bold text-sm">
            {blockedStandardCount}
          </div>
          <div>
            <div className="text-xs font-semibold text-slate-900">
              {isVi ? 'Danh Mục Chuẩn Đang Chặn' : 'Standard Categories Blocked'}
            </div>
            <div className="text-[11px] text-slate-500">
              {blockedStandardCount} / 5 {isVi ? 'danh mục hệ thống đang bật chặn' : 'system categories active'}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3 p-3.5 rounded-xl bg-white border border-slate-200/80 shadow-xs">
          <div className="w-9 h-9 rounded-lg bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-600 font-bold text-sm">
            {whitelistCount}
          </div>
          <div>
            <div className="text-xs font-semibold text-slate-900">
              {isVi ? 'Ngoại Lệ Whitelist' : 'Whitelist Exceptions'}
            </div>
            <div className="text-[11px] text-slate-500">
              {whitelistCount} {isVi ? 'website luôn được phép truy cập' : 'domains always allowed'}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3 p-3.5 rounded-xl bg-white border border-slate-200/80 shadow-xs">
          <div className="w-9 h-9 rounded-lg bg-sky-50 border border-sky-200 flex items-center justify-center text-sky-600 font-bold text-sm">
            {blacklistCount}
          </div>
          <div>
            <div className="text-xs font-semibold text-slate-900">
              {isVi ? 'Tên Miền Chặn Riêng (Blacklist)' : 'Custom Blacklist'}
            </div>
            <div className="text-[11px] text-slate-500">
              {blacklistCount} {isVi ? 'website cấm tuyệt đối' : 'domains strictly blocked'}
            </div>
          </div>
        </div>
      </div>

      {/* DOMAIN POLICY INSPECTOR / SMART LOOKUP */}
      <div className="rounded-2xl bg-gradient-to-br from-sky-50/70 via-indigo-50/40 to-white border border-sky-200/80 p-5 sm:p-6 shadow-xs space-y-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <div className="flex items-center gap-2">
              <Search className="w-5 h-5 text-sky-700" />
              <h3 className="text-sm font-bold text-slate-900 tracking-tight">
                {isVi
                  ? 'Tra Cứu & Điều Phối Quy Tắc Tên Miền (Domain Policy Inspector)'
                  : 'Domain Policy Inspector'}
              </h3>
            </div>
            <p className="text-xs text-slate-600 mt-1 max-w-2xl leading-relaxed">
              {isVi
                ? 'Kiểm tra tức thời một trang web đang bị Chặn hay Cho phép, và trực tiếp thêm nhanh vào Blacklist hoặc Whitelist.'
                : 'Instantly verify if a domain is blocked or allowed, and quickly add to Blacklist or Whitelist.'}
            </p>
          </div>
          <span className="text-[11px] font-semibold px-2.5 py-1 rounded-full bg-sky-100 text-sky-800 border border-sky-200">
            {isVi ? 'Mô phỏng phán quyết chính sách' : 'Real-time policy verdict'}
          </span>
        </div>

        <div className="flex flex-col sm:flex-row gap-2.5">
          <div className="relative flex-1">
            <input
              type="text"
              placeholder={
                isVi
                  ? 'Nhập tên miền kiểm tra (ví dụ: tiktok.com, google.com, pornhub.com)...'
                  : 'Enter domain (e.g. tiktok.com, google.com)...'
              }
              value={inspectInput}
              onChange={(e) => setInspectInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleInspectDomain()}
              className="w-full rounded-xl bg-white border border-slate-300 focus:border-sky-600 focus:ring-2 focus:ring-sky-100 pl-4 pr-10 py-2.5 text-xs text-slate-900 placeholder:text-slate-400 outline-none transition font-mono"
            />
            {inspectInput && (
              <button
                type="button"
                onClick={() => setInspectInput('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 p-1 cursor-pointer"
              >
                ✕
              </button>
            )}
          </div>
          <button
            type="button"
            onClick={handleInspectDomain}
            disabled={isInspecting || !inspectInput.trim()}
            className="inline-flex items-center justify-center gap-2 rounded-xl bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white font-semibold px-6 py-2.5 text-xs transition cursor-pointer shadow-xs whitespace-nowrap"
          >
            <Search className="w-4 h-4" />
            <span>{isInspecting ? (isVi ? 'Đang Tra Cứu...' : 'Inspecting...') : (isVi ? 'Tra Cứu Quy Tắc' : 'Inspect Domain')}</span>
          </button>
        </div>

        {/* Inspection Result Box */}
        {inspectResult && (
          <div className="rounded-xl bg-white border border-slate-200/90 p-4 shadow-2xs space-y-3 animate-fade-in">
            <div className="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-slate-100">
              <div className="flex items-center gap-3">
                <Globe className="w-5 h-5 text-slate-400" />
                <div>
                  <div className="font-mono font-bold text-sm text-slate-900">
                    {inspectResult.domain}
                  </div>
                  <div className="text-[11px] text-slate-500">
                    {isVi ? 'Phân loại:' : 'Category:'} <strong>{inspectResult.effective_category}</strong>
                  </div>
                </div>
              </div>

              {/* Status Badge */}
              <div className="flex items-center gap-2">
                <span
                  className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold ${
                    inspectResult.action === 'block'
                      ? 'bg-rose-100 text-rose-800 border border-rose-200'
                      : 'bg-emerald-100 text-emerald-800 border border-emerald-200'
                  }`}
                >
                  {inspectResult.action === 'block' ? (
                    <>
                      <XCircle className="w-3.5 h-3.5 text-rose-600" />
                      <span>{isVi ? 'KẾT QUẢ: BỊ CHẶN' : 'VERDICT: BLOCKED'}</span>
                    </>
                  ) : (
                    <>
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600" />
                      <span>{isVi ? 'KẾT QUẢ: ĐƯỢC CHO PHÉP' : 'VERDICT: ALLOWED'}</span>
                    </>
                  )}
                </span>
              </div>
            </div>

            <div className="text-xs text-slate-600 leading-relaxed">
              <span className="font-semibold text-slate-700">{isVi ? 'Căn cứ phán quyết:' : 'Reason:'} </span>
              {inspectResult.reason}
            </div>

            {/* Quick Action buttons */}
            <div className="flex items-center gap-2 pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => handleAddAllowedDomain(inspectResult.domain)}
                className="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-50 hover:bg-emerald-100 text-emerald-700 border border-emerald-200 transition cursor-pointer"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>{isVi ? 'Thêm vào Whitelist (Cho phép)' : 'Add to Whitelist'}</span>
              </button>
              <button
                type="button"
                onClick={() => handleAddBlockedDomain(inspectResult.domain)}
                className="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-semibold bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 transition cursor-pointer"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>{isVi ? 'Thêm vào Blacklist (Cấm)' : 'Add to Blacklist'}</span>
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Standard Categories Grid */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-slate-900 tracking-tight flex items-center gap-2">
            <Layers className="w-4 h-4 text-sky-600" />
            <span>{isVi ? '5 Danh Mục Kiểm Soát Tiêu Chuẩn' : '5 Standard Protection Categories'}</span>
          </h3>
          <span className="text-xs text-slate-500 font-medium">
            {isVi ? 'Áp dụng tức thời trên toàn Endpoint' : 'Real-time endpoint enforcement'}
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {STANDARD_CATEGORIES.map((cat) => {
            const isBlocked =
              cat.id === 'adult'
                ? !!cloudPolicy?.block_adult
                : cat.id === 'gambling'
                ? !!cloudPolicy?.block_gambling
                : cat.id === 'violence'
                ? !!cloudPolicy?.block_violence
                : cat.id === 'social'
                ? !!cloudPolicy?.block_social
                : cat.id === 'gaming'
                ? !!cloudPolicy?.block_gaming
                : false;

            const customCount =
              cloudPolicy?.category_custom_domains?.[cat.id]?.length || 0;
            const totalDomains = cat.seeds.length + customCount;
            const fileCount =
              cloudPolicy?.category_source_files?.filter(
                (f) => f.category_id === cat.id
              ).length || 0;

            const visibleSeeds = cat.seeds.slice(0, 3);
            const remainingCount = totalDomains - visibleSeeds.length;

            return (
              <div
                key={cat.id}
                className={`rounded-2xl p-5 border transition flex flex-col justify-between shadow-xs ${
                  isBlocked
                    ? 'bg-rose-50/40 border-rose-300 ring-2 ring-rose-100/80'
                    : 'bg-white border-slate-200/90 hover:border-slate-300'
                }`}
              >
                <div>
                  {/* Card Header: Icon, Name, and Block Toggle */}
                  <div className="flex items-start justify-between gap-2 mb-3">
                    <div className="flex items-center gap-2.5">
                      <span className="text-2xl p-1.5 rounded-xl bg-slate-50 border border-slate-200/80 shadow-2xs">
                        {cat.icon}
                      </span>
                      <div>
                        <span className="font-bold text-xs text-slate-900 block">
                          {isVi ? cat.name : cat.nameEn}
                        </span>
                        <div className="flex items-center gap-2 mt-0.5">
                          <span className="text-[11px] font-mono text-slate-500 font-semibold">
                            {totalDomains.toLocaleString()} {isVi ? 'tên miền' : 'domains'}
                          </span>
                          {fileCount > 0 && (
                            <span className="inline-flex items-center gap-0.5 px-1.5 py-0.2 rounded text-[10px] font-semibold bg-indigo-50 text-indigo-700 border border-indigo-200">
                              <FileText className="w-2.5 h-2.5" />
                              <span>{fileCount} {isVi ? 'tệp' : 'files'}</span>
                            </span>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Quick Toggle Button */}
                    <button
                      type="button"
                      onClick={() => onToggleCategoryBlock(cat.id, !isBlocked)}
                      className={`text-2xs font-bold px-2.5 py-1 rounded-full uppercase tracking-wider transition cursor-pointer flex items-center gap-1 ${
                        isBlocked
                          ? 'bg-rose-600 hover:bg-rose-700 text-white shadow-xs'
                          : 'bg-slate-200 hover:bg-slate-300 text-slate-700'
                      }`}
                      title={isVi ? 'Bật/Tắt chặn danh mục này' : 'Toggle block for this category'}
                    >
                      {isBlocked ? (
                        <>
                          <ShieldAlert className="w-3 h-3" />
                          <span>{isVi ? 'Đang Chặn' : 'Blocked'}</span>
                        </>
                      ) : (
                        <>
                          <ShieldCheck className="w-3 h-3" />
                          <span>{isVi ? 'Cho Phép' : 'Allowed'}</span>
                        </>
                      )}
                    </button>
                  </div>

                  <p className="text-xs text-slate-500 mb-4 leading-relaxed line-clamp-2">
                    {isVi ? cat.desc : cat.descEn}
                  </p>
                </div>

                <div className="space-y-3 pt-2 border-t border-slate-100">
                  {/* Representative Sample Pills (Clean summary, max 3 items to avoid DOM overload) */}
                  <div>
                    <div className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                      {isVi ? 'Mẫu tên miền tiêu biểu:' : 'Sample domains:'}
                    </div>
                    <div className="flex flex-wrap items-center gap-1">
                      {visibleSeeds.map((s) => (
                        <span
                          key={s}
                          className="text-[10px] font-mono bg-white/90 border border-slate-200 px-1.5 py-0.5 rounded text-slate-600 shadow-2xs"
                        >
                          {s}
                        </span>
                      ))}
                      {remainingCount > 0 && (
                        <button
                          type="button"
                          onClick={() => onOpenCategoryDetail(cat.id)}
                          className="text-[10px] font-medium text-sky-600 hover:text-sky-800 bg-sky-50 hover:bg-sky-100 border border-sky-200/80 px-1.5 py-0.5 rounded transition cursor-pointer"
                        >
                          +{remainingCount.toLocaleString()} {isVi ? 'khác...' : 'more...'}
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Primary Action: View Details & Manage Files */}
                  <button
                    type="button"
                    onClick={() => onOpenCategoryDetail(cat.id)}
                    className="w-full inline-flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold bg-sky-50/70 hover:bg-sky-100 text-sky-700 border border-sky-200 transition cursor-pointer group shadow-2xs"
                  >
                    <span className="flex items-center gap-1.5">
                      <FileText className="w-3.5 h-3.5 text-sky-600" />
                      <span>{isVi ? 'Chi Tiết & Quản Lý Tệp' : 'Details & File Management'}</span>
                    </span>
                    <ChevronRight className="w-3.5 h-3.5 text-sky-500 group-hover:translate-x-0.5 transition-transform" />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
