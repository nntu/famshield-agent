import type React from 'react';
import {
  Filter,
  ShieldCheck,
  ShieldAlert,
  Upload,
  Plus,
  Search,
  XCircle,
  CheckCircle2,
  Trash2,
  Laptop,
} from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { DataTable } from '../DataTable';
import type { CloudPolicyData, Language, LocalRuleRecord } from '../../types';
import type { MergedDomainItem } from './rulesTypes';

interface RulesSubTabProps {
  isVi: boolean;
  t: any;
  lang: Language;
  mergedBlockedItems: MergedDomainItem[];
  filteredBlocked: MergedDomainItem[];
  newBlockedDomain: string;
  setNewBlockedDomain: (val: string) => void;
  isAddingBlocked: boolean;
  handleAddBlockedDomain: () => Promise<void>;
  showBulkBlacklist: boolean;
  setShowBulkBlacklist: (val: boolean) => void;
  bulkBlacklistText: string;
  setBulkBlacklistText: (val: string) => void;
  handleConfirmBatchBlacklist: () => Promise<void>;
  blacklistSearch: string;
  setBlacklistSearch: (val: string) => void;
  handleDeleteRule: (id: string) => Promise<void>;
  mergedAllowedItems: MergedDomainItem[];
  filteredAllowed: MergedDomainItem[];
  newAllowedDomain: string;
  setNewAllowedDomain: (val: string) => void;
  isAddingAllowed: boolean;
  handleAddAllowedDomain: () => Promise<void>;
  showBulkWhitelist: boolean;
  setShowBulkWhitelist: (val: boolean) => void;
  bulkWhitelistText: string;
  setBulkWhitelistText: (val: string) => void;
  handleConfirmBatchWhitelist: () => Promise<void>;
  whitelistSearch: string;
  setWhitelistSearch: (val: string) => void;
  cloudPolicy: CloudPolicyData | null;
  newApp: string;
  setNewApp: (val: string) => void;
  appQuota: string;
  setAppQuota: (val: string) => void;
  isAddingApp: boolean;
  handleAddApp: (e: React.FormEvent) => Promise<void>;
  appRules: LocalRuleRecord[];
  appColumns: ColumnDef<LocalRuleRecord>[];
}

export const RulesSubTab: React.FC<RulesSubTabProps> = ({
  isVi,
  t,
  lang,
  mergedBlockedItems,
  filteredBlocked,
  newBlockedDomain,
  setNewBlockedDomain,
  isAddingBlocked,
  handleAddBlockedDomain,
  showBulkBlacklist,
  setShowBulkBlacklist,
  bulkBlacklistText,
  setBulkBlacklistText,
  handleConfirmBatchBlacklist,
  blacklistSearch,
  setBlacklistSearch,
  handleDeleteRule,
  mergedAllowedItems,
  filteredAllowed,
  newAllowedDomain,
  setNewAllowedDomain,
  isAddingAllowed,
  handleAddAllowedDomain,
  showBulkWhitelist,
  setShowBulkWhitelist,
  bulkWhitelistText,
  setBulkWhitelistText,
  handleConfirmBatchWhitelist,
  whitelistSearch,
  setWhitelistSearch,
  cloudPolicy,
  newApp,
  setNewApp,
  appQuota,
  setAppQuota,
  isAddingApp,
  handleAddApp,
  appRules,
  appColumns,
}) => {
  return (
    <div className="space-y-6">
      {/* Precedence Card */}
      <div className="p-4 rounded-2xl bg-gradient-to-r from-sky-50/70 via-indigo-50/40 to-slate-50 border border-sky-200/80 shadow-xs">
        <div className="flex items-center gap-2 mb-2">
          <Filter className="w-4 h-4 text-sky-700" />
          <span className="text-xs font-bold text-slate-900 uppercase tracking-wide">
            {isVi ? 'Thứ tự ưu tiên xử lý tên miền (Precedence Order)' : 'Domain Precedence Hierarchy'}
          </span>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
          <div className="p-3 rounded-xl bg-emerald-50/90 border border-emerald-200 text-emerald-950 flex items-start gap-2.5">
            <span className="font-bold text-emerald-700 text-base leading-none">1</span>
            <div>
              <div className="font-bold text-emerald-900 flex items-center gap-1.5">
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-600" />
                <span>{isVi ? 'Ngoại Lệ (Whitelist)' : 'Whitelist (Allowed)'}</span>
              </div>
              <p className="text-[11px] text-emerald-800/90 mt-0.5 leading-relaxed">
                <strong>{isVi ? 'Ưu tiên số 1:' : 'Priority 1:'}</strong>{' '}
                {isVi
                  ? 'Luôn cho phép truy cập, bỏ qua mọi lệnh chặn của danh mục.'
                  : 'Always allowed, bypasses all category and blacklist blocks.'}
              </p>
            </div>
          </div>

          <div className="p-3 rounded-xl bg-rose-50/90 border border-rose-200 text-rose-950 flex items-start gap-2.5">
            <span className="font-bold text-rose-700 text-base leading-none">2</span>
            <div>
              <div className="font-bold text-rose-900 flex items-center gap-1.5">
                <ShieldAlert className="w-3.5 h-3.5 text-rose-600" />
                <span>{isVi ? 'Chặn Riêng (Blacklist)' : 'Blacklist (Blocked)'}</span>
              </div>
              <p className="text-[11px] text-rose-800/90 mt-0.5 leading-relaxed">
                <strong>{isVi ? 'Ưu tiên số 2:' : 'Priority 2:'}</strong>{' '}
                {isVi
                  ? 'Luôn cấm truy cập triệt để, hỗ trợ chặn thương hiệu không dấu chấm.'
                  : 'Always strictly blocked, supports keyword brand pattern matches.'}
              </p>
            </div>
          </div>

          <div className="p-3 rounded-xl bg-white border border-slate-200 text-slate-900 flex items-start gap-2.5">
            <span className="font-bold text-slate-500 text-base leading-none">3</span>
            <div>
              <div className="font-bold text-slate-800">{isVi ? 'Danh Mục Phân Loại' : 'Categories'}</div>
              <p className="text-[11px] text-slate-600 mt-0.5 leading-relaxed">
                {isVi
                  ? 'Áp dụng quy tắc theo 5 nhóm chuẩn (Người lớn, Cờ bạc, Bạo lực...).'
                  : 'Applies rules from standard category feeds.'}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Side-by-side Dual Lists: Blacklist & Whitelist */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* LEFT COLUMN: CUSTOM BLACKLIST */}
        <div className="rounded-2xl bg-white border border-rose-200/90 p-5 sm:p-6 shadow-xs flex flex-col justify-between space-y-4">
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-slate-100">
              <div className="flex items-center gap-2">
                <ShieldAlert className="w-5 h-5 text-rose-600" />
                <h3 className="text-base font-bold text-slate-900 tracking-tight">
                  {isVi ? 'Tên Miền Chặn Riêng (Custom Blacklist)' : 'Custom Blacklist'}
                </h3>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setShowBulkBlacklist(!showBulkBlacklist)}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 transition cursor-pointer"
                >
                  <Upload className="w-3.5 h-3.5" />
                  <span>{showBulkBlacklist ? (isVi ? 'Đóng ô dán' : 'Close') : (isVi ? 'Dán hàng loạt' : 'Bulk Paste')}</span>
                </button>
                <span className="text-xs font-bold px-2.5 py-0.5 rounded-full bg-rose-100 text-rose-700 border border-rose-200">
                  {mergedBlockedItems.length} website
                </span>
              </div>
            </div>

            <p className="text-xs text-slate-600 leading-relaxed">
              {isVi
                ? 'Các website tại đây sẽ LUÔN BỊ CHẶN trên máy trạm với độ ưu tiên tuyệt đối.'
                : 'Domains listed here are ALWAYS BLOCKED on this endpoint.'}
            </p>

            {/* Bulk Blacklist Drawer */}
            {showBulkBlacklist && (
              <div className="p-3 bg-rose-50/70 border border-rose-200 rounded-xl space-y-2 animate-fade-in">
                <div className="text-[11px] font-semibold text-rose-900">
                  {isVi
                    ? 'Dán danh sách tên miền cần chặn (mỗi dòng hoặc cách nhau bởi dấu cách / phẩy):'
                    : 'Paste domain list to block (line or comma separated):'}
                </div>
                <textarea
                  rows={3}
                  value={bulkBlacklistText}
                  onChange={(e) => setBulkBlacklistText(e.target.value)}
                  placeholder="tiktok.com&#10;roblox.com&#10;thiendia.com"
                  className="w-full rounded-lg bg-white border border-rose-300 p-2.5 text-xs font-mono text-slate-900 outline-none"
                />
                <div className="flex justify-end gap-2">
                  <button
                    type="button"
                    onClick={() => setShowBulkBlacklist(false)}
                    className="px-3 py-1 rounded-md text-xs font-medium text-slate-600 hover:bg-slate-200 cursor-pointer"
                  >
                    {isVi ? 'Hủy' : 'Cancel'}
                  </button>
                  <button
                    type="button"
                    onClick={handleConfirmBatchBlacklist}
                    className="px-3 py-1 rounded-md text-xs font-semibold bg-rose-600 hover:bg-rose-700 text-white cursor-pointer shadow-xs"
                  >
                    {isVi ? 'Thêm Vào Blacklist' : 'Add to Blacklist'}
                  </button>
                </div>
              </div>
            )}

            {/* Add Input */}
            <div className="space-y-1.5">
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder={
                    isVi
                      ? 'Nhập tên miền (tiktok.com) hoặc thương hiệu (thiendia)...'
                      : 'Enter domain (e.g. tiktok.com)...'
                  }
                  value={newBlockedDomain}
                  onChange={(e) => setNewBlockedDomain(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddBlockedDomain()}
                  className="flex-1 rounded-xl bg-white border border-slate-300 focus:border-rose-600 focus:ring-2 focus:ring-rose-100 px-3.5 py-2 text-xs text-slate-900 placeholder:text-slate-400 outline-none transition font-mono"
                />
                <button
                  type="button"
                  disabled={isAddingBlocked || !newBlockedDomain.trim()}
                  onClick={() => handleAddBlockedDomain()}
                  className="inline-flex items-center gap-1.5 rounded-xl bg-rose-600 hover:bg-rose-700 disabled:opacity-50 text-white font-semibold px-4 py-2 text-xs transition cursor-pointer shadow-xs whitespace-nowrap"
                >
                  <Plus className="w-4 h-4" />
                  <span>{isVi ? 'Thêm Chặn' : 'Add Block'}</span>
                </button>
              </div>
            </div>

            {/* Search */}
            {mergedBlockedItems.length > 5 && (
              <div className="relative">
                <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
                <input
                  type="text"
                  placeholder={isVi ? 'Lọc trong danh sách blacklist...' : 'Filter blacklist...'}
                  value={blacklistSearch}
                  onChange={(e) => setBlacklistSearch(e.target.value)}
                  className="w-full pl-8 pr-3 py-1.5 rounded-lg border border-slate-200 text-xs text-slate-700 placeholder:text-slate-400 outline-none focus:border-rose-400 transition"
                />
              </div>
            )}

            {/* Tag Pills List */}
            <div>
              <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2">
                {isVi ? 'Danh sách website bị chặn' : 'Blocked domains'} ({filteredBlocked.length}
                {blacklistSearch ? ` / ${mergedBlockedItems.length}` : ''}):
              </div>

              {filteredBlocked.length > 0 ? (
                <div className="flex flex-wrap gap-2 max-h-64 overflow-y-auto pr-1">
                  {filteredBlocked.map((item) => (
                    <span
                      key={item.id}
                      className="inline-flex items-center gap-1.5 rounded-lg bg-rose-50 border border-rose-200 text-rose-700 px-2.5 py-1 text-xs font-mono font-medium shadow-2xs"
                    >
                      <XCircle className="w-3.5 h-3.5 text-rose-500" />
                      <span>{item.domain}</span>
                      <span
                        className={`text-[9px] px-1 py-0.2 rounded font-sans font-bold uppercase ${
                          item.source === 'cloud'
                            ? 'bg-sky-100 text-sky-800'
                            : 'bg-slate-200 text-slate-700'
                        }`}
                      >
                        {item.source}
                      </span>
                      {item.source === 'local' && (
                        <button
                          type="button"
                          onClick={() => handleDeleteRule(item.id)}
                          className="text-rose-400 hover:text-rose-700 cursor-pointer ml-0.5 p-0.5 rounded hover:bg-rose-100 transition"
                          title={`Xóa ${item.domain}`}
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </span>
                  ))}
                </div>
              ) : (
                <div className="text-xs text-slate-400 italic p-6 rounded-xl bg-slate-50/70 border border-dashed border-slate-200 text-center">
                  {blacklistSearch
                    ? (isVi ? 'Không tìm thấy tên miền nào khớp với từ khóa.' : 'No matching domains.')
                    : (isVi ? 'Chưa có tên miền nào trong Blacklist.' : 'No domains in Blacklist.')}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: CUSTOM WHITELIST */}
        <div className="rounded-2xl bg-white border border-emerald-200/90 p-5 sm:p-6 shadow-xs flex flex-col justify-between space-y-4">
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-slate-100">
              <div className="flex items-center gap-2">
                <ShieldCheck className="w-5 h-5 text-emerald-600" />
                <h3 className="text-base font-bold text-slate-900 tracking-tight">
                  {isVi ? 'Ngoại Lệ Cho Phép (Custom Whitelist)' : 'Custom Whitelist'}
                </h3>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setShowBulkWhitelist(!showBulkWhitelist)}
                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold bg-emerald-50 hover:bg-emerald-100 text-emerald-700 border border-emerald-200 transition cursor-pointer"
                >
                  <Upload className="w-3.5 h-3.5" />
                  <span>{showBulkWhitelist ? (isVi ? 'Đóng ô dán' : 'Close') : (isVi ? 'Dán hàng loạt' : 'Bulk Paste')}</span>
                </button>
                <span className="text-xs font-bold px-2.5 py-0.5 rounded-full bg-emerald-100 text-emerald-700 border border-emerald-200">
                  {mergedAllowedItems.length} website
                </span>
              </div>
            </div>

            <p className="text-xs text-slate-600 leading-relaxed">
              {isVi
                ? 'Các website tại đây LUÔN ĐƯỢC PHÉP TRUY CẬP với độ ưu tiên cao nhất.'
                : 'Domains listed here are ALWAYS ALLOWED with highest priority.'}
            </p>

            {/* Bulk Whitelist Drawer */}
            {showBulkWhitelist && (
              <div className="p-3 bg-emerald-50/70 border border-emerald-200 rounded-xl space-y-2 animate-fade-in">
                <div className="text-[11px] font-semibold text-emerald-900">
                  {isVi
                    ? 'Dán danh sách tên miền cho phép (mỗi dòng hoặc cách nhau bởi dấu cách / phẩy):'
                    : 'Paste domain list to allow (line or comma separated):'}
                </div>
                <textarea
                  rows={3}
                  value={bulkWhitelistText}
                  onChange={(e) => setBulkWhitelistText(e.target.value)}
                  placeholder="edu.vn&#10;zalo.me&#10;youtube.com"
                  className="w-full rounded-lg bg-white border border-emerald-300 p-2.5 text-xs font-mono text-slate-900 outline-none"
                />
                <div className="flex justify-end gap-2">
                  <button
                    type="button"
                    onClick={() => setShowBulkWhitelist(false)}
                    className="px-3 py-1 rounded-md text-xs font-medium text-slate-600 hover:bg-slate-200 cursor-pointer"
                  >
                    {isVi ? 'Hủy' : 'Cancel'}
                  </button>
                  <button
                    type="button"
                    onClick={handleConfirmBatchWhitelist}
                    className="px-3 py-1 rounded-md text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white cursor-pointer shadow-xs"
                  >
                    {isVi ? 'Thêm Vào Whitelist' : 'Add to Whitelist'}
                  </button>
                </div>
              </div>
            )}

            {/* Add Input */}
            <div className="space-y-1.5">
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder={
                    isVi
                      ? 'Nhập tên miền ngoại lệ (ví dụ: edu.vn, zalo.me, youtube.com)...'
                      : 'Enter allowed domain (e.g. edu.vn)...'
                  }
                  value={newAllowedDomain}
                  onChange={(e) => setNewAllowedDomain(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddAllowedDomain()}
                  className="flex-1 rounded-xl bg-white border border-slate-300 focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100 px-3.5 py-2 text-xs text-slate-900 placeholder:text-slate-400 outline-none transition font-mono"
                />
                <button
                  type="button"
                  disabled={isAddingAllowed || !newAllowedDomain.trim()}
                  onClick={() => handleAddAllowedDomain()}
                  className="inline-flex items-center gap-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white font-semibold px-4 py-2 text-xs transition cursor-pointer shadow-xs whitespace-nowrap"
                >
                  <Plus className="w-4 h-4" />
                  <span>{isVi ? 'Thêm Cho Phép' : 'Add Allow'}</span>
                </button>
              </div>
            </div>

            {/* Search */}
            {mergedAllowedItems.length > 5 && (
              <div className="relative">
                <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
                <input
                  type="text"
                  placeholder={isVi ? 'Lọc trong danh sách whitelist...' : 'Filter whitelist...'}
                  value={whitelistSearch}
                  onChange={(e) => setWhitelistSearch(e.target.value)}
                  className="w-full pl-8 pr-3 py-1.5 rounded-lg border border-slate-200 text-xs text-slate-700 placeholder:text-slate-400 outline-none focus:border-emerald-400 transition"
                />
              </div>
            )}

            {/* Tag Pills List */}
            <div>
              <div className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2">
                {isVi ? 'Danh sách website được ngoại lệ' : 'Allowed domains'} ({filteredAllowed.length}
                {whitelistSearch ? ` / ${mergedAllowedItems.length}` : ''}):
              </div>

              {filteredAllowed.length > 0 ? (
                <div className="flex flex-wrap gap-2 max-h-64 overflow-y-auto pr-1">
                  {filteredAllowed.map((item) => (
                    <span
                      key={item.id}
                      className="inline-flex items-center gap-1.5 rounded-lg bg-emerald-50 border border-emerald-200 text-emerald-800 px-2.5 py-1 text-xs font-mono font-medium shadow-2xs"
                    >
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600" />
                      <span>{item.domain}</span>
                      <span
                        className={`text-[9px] px-1 py-0.2 rounded font-sans font-bold uppercase ${
                          item.source === 'cloud'
                            ? 'bg-sky-100 text-sky-800'
                            : 'bg-slate-200 text-slate-700'
                        }`}
                      >
                        {item.source}
                      </span>
                      {item.source === 'local' && (
                        <button
                          type="button"
                          onClick={() => handleDeleteRule(item.id)}
                          className="text-emerald-500 hover:text-emerald-800 cursor-pointer ml-0.5 p-0.5 rounded hover:bg-emerald-100 transition"
                          title={`Xóa ${item.domain}`}
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </span>
                  ))}
                </div>
              ) : (
                <div className="text-xs text-slate-400 italic p-6 rounded-xl bg-slate-50/70 border border-dashed border-slate-200 text-center">
                  {whitelistSearch
                    ? (isVi ? 'Không tìm thấy tên miền nào khớp với từ khóa.' : 'No matching domains.')
                    : (isVi ? 'Chưa có tên miền nào trong Whitelist.' : 'No domains in Whitelist.')}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* SAFESEARCH & YOUTUBE RESTRICTED MODE CARDS */}
      <div className="space-y-3 pt-2">
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-sky-600" />
          <h3 className="text-sm font-bold text-slate-900 tracking-tight">
            {isVi
              ? 'Bộ Lọc Tìm Kiếm An Toàn & Video (SafeSearch & Restricted Mode)'
              : 'SafeSearch & YouTube Restricted Mode'}
          </h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            className={`flex items-center justify-between rounded-2xl p-5 border transition shadow-xs ${
              cloudPolicy?.safesearch_enabled
                ? 'bg-emerald-50/80 border-emerald-300 ring-2 ring-emerald-100'
                : 'bg-white border-slate-200'
            }`}
          >
            <div>
              <div
                className={`text-sm font-bold ${
                  cloudPolicy?.safesearch_enabled ? 'text-emerald-900' : 'text-slate-900'
                }`}
              >
                {isVi ? 'Ép Buộc Google & Bing SafeSearch' : 'Enforce SafeSearch'}
              </div>
              <p className="text-xs text-slate-500 mt-1">
                {isVi
                  ? 'Tự động ẩn mọi hình ảnh và kết quả tìm kiếm người lớn.'
                  : 'Automatically filter adult images and explicit search results.'}
              </p>
              <div className="text-[11px] text-slate-400 font-mono mt-1">
                forcesafesearch.google.com, strict.bing.com
              </div>
            </div>
            <span
              className={`text-xs font-bold px-3 py-1 rounded-full shrink-0 ${
                cloudPolicy?.safesearch_enabled
                  ? 'bg-emerald-600 text-white shadow-2xs'
                  : 'bg-slate-200 text-slate-700'
              }`}
            >
              {cloudPolicy?.safesearch_enabled ? (isVi ? 'BẬT' : 'ON') : (isVi ? 'TẮT' : 'OFF')}
            </span>
          </div>

          <div
            className={`flex items-center justify-between rounded-2xl p-5 border transition shadow-xs ${
              cloudPolicy?.youtube_restricted
                ? 'bg-emerald-50/80 border-emerald-300 ring-2 ring-emerald-100'
                : 'bg-white border-slate-200'
            }`}
          >
            <div>
              <div
                className={`text-sm font-bold ${
                  cloudPolicy?.youtube_restricted ? 'text-emerald-900' : 'text-slate-900'
                }`}
              >
                YouTube Restricted Mode
              </div>
              <p className="text-xs text-slate-500 mt-1">
                {isVi
                  ? 'Khóa các video có nội dung giới hạn độ tuổi trên YouTube.'
                  : 'Lock age-restricted videos and adult comments on YouTube.'}
              </p>
              <div className="text-[11px] text-slate-400 font-mono mt-1">
                restrictmoderate.youtube.com
              </div>
            </div>
            <span
              className={`text-xs font-bold px-3 py-1 rounded-full shrink-0 ${
                cloudPolicy?.youtube_restricted
                  ? 'bg-emerald-600 text-white shadow-2xs'
                  : 'bg-slate-200 text-slate-700'
              }`}
            >
              {cloudPolicy?.youtube_restricted ? (isVi ? 'BẬT' : 'ON') : (isVi ? 'TẮT' : 'OFF')}
            </span>
          </div>
        </div>
      </div>

      {/* APPLICATION QUOTA & USAGE LIMITS */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs flex flex-col gap-5 mt-4">
        <div>
          <h2 className="text-base font-bold text-slate-900 flex items-center gap-2">
            <Laptop size={18} className="text-sky-600" />
            {t.title_app_limits}
          </h2>
          <p className="text-xs text-slate-500 mt-0.5">{t.desc_app_limits}</p>
        </div>

        {/* Add App Form */}
        <form onSubmit={handleAddApp} className="flex gap-2.5 flex-wrap">
          <input
            type="text"
            placeholder={t.placeholder_app}
            value={newApp}
            onChange={(e) => setNewApp(e.target.value)}
            className="flex-3 min-w-[200px] bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400 font-mono"
          />
          <input
            type="number"
            min="5"
            max="1440"
            placeholder={t.placeholder_quota}
            value={appQuota}
            onChange={(e) => setAppQuota(e.target.value)}
            className="flex-1 min-w-[120px] bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
          />
          <button
            type="submit"
            disabled={isAddingApp}
            className="bg-sky-600 hover:bg-sky-700 text-white px-4 py-2 rounded-lg text-sm font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 border-0 transition-all"
          >
            <Plus size={15} />
            <span>{t.btn_add_app}</span>
          </button>
        </form>

        {/* App Table */}
        <DataTable
          data={appRules}
          columns={appColumns}
          pageSize={10}
          searchPlaceholder={isVi ? 'Tìm kiếm ứng dụng giới hạn...' : 'Search limited applications...'}
          emptyText={t.no_app_rules}
          lang={lang}
        />
      </div>
    </div>
  );
};
