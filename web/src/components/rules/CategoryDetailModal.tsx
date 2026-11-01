import type React from 'react';
import { useState, useMemo, useRef } from 'react';
import {
  X,
  Upload,
  Download,
  Plus,
  Search,
  Trash2,
  FileText,
  ShieldAlert,
  ShieldCheck,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Database,
  Info,
} from 'lucide-react';
import type { CategorySourceFile, CloudPolicyData } from '../../types';

export interface CategoryDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  categoryId: string;
  categoryName: string;
  categoryNameEn: string;
  categoryIcon: string;
  categoryDesc: string;
  categoryDescEn: string;
  systemSeeds: string[];
  isBlocked: boolean;
  onToggleBlock: (catId: string, blocked: boolean) => Promise<void>;
  cloudPolicy: CloudPolicyData | null;
  onUpdateCategoryDomains: (
    catId: string,
    customDomains: string[],
    newFile?: CategorySourceFile,
    removedFileId?: string
  ) => Promise<void>;
  isVi: boolean;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

interface DomainListItem {
  domain: string;
  source: 'system' | 'file' | 'custom' | 'cloud';
  sourceFileName?: string;
}

export const CategoryDetailModal: React.FC<CategoryDetailModalProps> = ({
  isOpen,
  onClose,
  categoryId,
  categoryName,
  categoryNameEn,
  categoryIcon,
  categoryDesc,
  categoryDescEn,
  systemSeeds,
  isBlocked,
  onToggleBlock,
  cloudPolicy,
  onUpdateCategoryDomains,
  isVi,
  onShowToast,
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);

  // States
  const [searchTerm, setSearchTerm] = useState('');
  const [newDomainInput, setNewDomainInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);

  if (!isOpen) return null;

  // 1. Gather all custom domains and source files for this category
  const customDomains = useMemo(() => {
    if (!cloudPolicy?.category_custom_domains) return [];
    return cloudPolicy.category_custom_domains[categoryId] || [];
  }, [cloudPolicy, categoryId]);

  const sourceFiles = useMemo(() => {
    if (!cloudPolicy?.category_source_files) return [];
    return cloudPolicy.category_source_files.filter((f) => f.category_id === categoryId);
  }, [cloudPolicy, categoryId]);

  // 2. Build full list of items with proper source tagging
  const allItems: DomainListItem[] = useMemo(() => {
    const list: DomainListItem[] = [];
    const seen = new Set<string>();

    // Add system seeds first
    for (const d of systemSeeds) {
      const norm = d.trim().toLowerCase();
      if (norm && !seen.has(norm)) {
        seen.add(norm);
        list.push({ domain: norm, source: 'system' });
      }
    }

    // Add custom / imported domains
    const hasFiles = sourceFiles.length > 0;
    const latestFile = hasFiles ? sourceFiles[sourceFiles.length - 1] : undefined;

    for (const d of customDomains) {
      const norm = d.trim().toLowerCase();
      if (norm && !seen.has(norm)) {
        seen.add(norm);
        list.push({
          domain: norm,
          source: hasFiles ? 'file' : 'custom',
          sourceFileName: latestFile?.file_name,
        });
      }
    }

    return list;
  }, [systemSeeds, customDomains, sourceFiles]);

  // 3. Filter items by search keyword
  const filteredItems = useMemo(() => {
    const q = searchTerm.trim().toLowerCase();
    if (!q) return allItems;
    return allItems.filter((item) => item.domain.includes(q));
  }, [allItems, searchTerm]);

  // 4. Paginate items to prevent freezing UI
  const totalPages = Math.max(1, Math.ceil(filteredItems.length / pageSize));
  const safeCurrentPage = Math.min(currentPage, totalPages);
  const startIndex = (safeCurrentPage - 1) * pageSize;
  const paginatedItems = filteredItems.slice(startIndex, startIndex + pageSize);

  // Toggle blocking state
  const handleToggle = async () => {
    try {
      setIsProcessing(true);
      await onToggleBlock(categoryId, !isBlocked);
    } finally {
      setIsProcessing(false);
    }
  };

  // Add a single custom domain
  const handleAddDomain = async (e: React.FormEvent) => {
    e.preventDefault();
    const raw = newDomainInput.trim().toLowerCase();
    if (!raw) return;

    // Basic domain validation
    const cleaned = raw.replace(/^https?:\/\//i, '').split('/')[0].split(':')[0].trim();
    if (!cleaned || (!cleaned.includes('.') && cleaned !== 'localhost')) {
      onShowToast(isVi ? 'Định dạng tên miền không hợp lệ' : 'Invalid domain format', 'error');
      return;
    }

    if (allItems.some((item) => item.domain === cleaned)) {
      onShowToast(isVi ? 'Tên miền đã tồn tại trong danh mục này' : 'Domain already exists in category', 'info');
      return;
    }

    try {
      setIsProcessing(true);
      const updated = [...customDomains, cleaned];
      await onUpdateCategoryDomains(categoryId, updated);
      setNewDomainInput('');
      onShowToast(isVi ? `Đã thêm tên miền "${cleaned}" vào danh mục` : `Added "${cleaned}" to category`, 'success');
    } catch {
      onShowToast(isVi ? 'Không thể lưu tên miền mới' : 'Failed to save new domain', 'error');
    } finally {
      setIsProcessing(false);
    }
  };

  // Delete a custom domain
  const handleDeleteDomain = async (domain: string) => {
    try {
      setIsProcessing(true);
      const updated = customDomains.filter((d) => d.toLowerCase() !== domain.toLowerCase());
      await onUpdateCategoryDomains(categoryId, updated);
      onShowToast(isVi ? `Đã xoá tên miền "${domain}"` : `Deleted "${domain}"`, 'info');
    } catch {
      onShowToast(isVi ? 'Lỗi khi xoá tên miền' : 'Failed to delete domain', 'error');
    } finally {
      setIsProcessing(false);
    }
  };

  // Import domains from file (.txt, .list, .csv, .hosts)
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const fileName = file.name;
    const fileSize = file.size;
    const reader = new FileReader();

    reader.onload = async (evt) => {
      try {
        setIsProcessing(true);
        const text = evt.target?.result as string;
        if (!text) {
          onShowToast(isVi ? 'Tệp rỗng hoặc không có dữ liệu' : 'File is empty', 'error');
          return;
        }

        const lines = text.split(/\r?\n/);
        const extracted: string[] = [];
        const existingSet = new Set(allItems.map((i) => i.domain));

        for (let line of lines) {
          line = line.trim();
          if (!line || line.startsWith('#') || line.startsWith('//') || line.startsWith('!')) {
            continue; // Skip comments and empty lines
          }

          // Strip inline comments
          const hashIdx = line.indexOf('#');
          if (hashIdx !== -1) {
            line = line.slice(0, hashIdx).trim();
          }

          // Handle hosts file format: "0.0.0.0 domain.com" or "127.0.0.1 domain.com"
          const tokens = line.split(/\s+/);
          let target = tokens[tokens.length - 1];

          // Handle Adblock syntax: "||domain.com^"
          if (target.startsWith('||')) {
            target = target.slice(2);
          }
          if (target.endsWith('^')) {
            target = target.slice(0, -1);
          }

          // Strip protocol and path if url
          target = target.replace(/^https?:\/\//i, '').split('/')[0].split(':')[0].trim().toLowerCase();

          if (target && target.includes('.') && !existingSet.has(target)) {
            existingSet.add(target);
            extracted.push(target);
          }
        }

        if (extracted.length === 0) {
          onShowToast(
            isVi
              ? 'Không tìm thấy tên miền hợp lệ mới trong tệp (có thể đã tồn tại)'
              : 'No new valid domains found in file',
            'info'
          );
          return;
        }

        const newSourceFile: CategorySourceFile = {
          id: `file_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
          category_id: categoryId,
          file_name: fileName,
          uploaded_at: new Date().toISOString(),
          domain_count: extracted.length,
          size_bytes: fileSize,
        };

        const updated = [...customDomains, ...extracted];
        await onUpdateCategoryDomains(categoryId, updated, newSourceFile);

        onShowToast(
          isVi
            ? `Nạp thành công ${extracted.length.toLocaleString()} tên miền từ tệp "${fileName}"`
            : `Imported ${extracted.length.toLocaleString()} domains from "${fileName}"`,
          'success'
        );
      } catch (err) {
        onShowToast(isVi ? 'Lỗi khi xử lý tệp nạp vào' : 'Failed to parse file', 'error');
      } finally {
        setIsProcessing(false);
        if (fileInputRef.current) fileInputRef.current.value = '';
      }
    };

    reader.readAsText(file);
  };

  // Remove an imported source file
  const handleRemoveFile = async (fileId: string, fileName: string) => {
    if (!window.confirm(isVi ? `Bạn có chắc chắn muốn xoá tệp "${fileName}"?` : `Remove file "${fileName}"?`)) {
      return;
    }

    try {
      setIsProcessing(true);
      await onUpdateCategoryDomains(categoryId, customDomains, undefined, fileId);
      onShowToast(isVi ? `Đã xoá tệp "${fileName}" khỏi danh mục` : `Removed file "${fileName}"`, 'info');
    } catch {
      onShowToast(isVi ? 'Không thể xoá tệp' : 'Failed to remove file', 'error');
    } finally {
      setIsProcessing(false);
    }
  };

  // Export all domains of this category as .txt file
  const handleExportTxt = () => {
    const header = `# FamShield Domain Category Export: ${categoryName} (${categoryId})\n# Total: ${allItems.length} domains\n# Exported at: ${new Date().toISOString()}\n\n`;
    const body = allItems.map((item) => item.domain).join('\n');
    const blob = new Blob([header + body], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `famshield_${categoryId}_domains_${new Date().toISOString().slice(0, 10)}.txt`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    onShowToast(isVi ? 'Đã xuất danh sách tên miền thành tệp .txt' : 'Exported domain list to .txt', 'success');
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-slate-900/60 backdrop-blur-xs animate-fade-in">
      <div className="bg-white rounded-2xl shadow-2xl border border-slate-200/90 w-full max-w-4xl max-h-[92vh] flex flex-col overflow-hidden">
        {/* MODAL HEADER */}
        <div className="p-4 sm:p-5 border-b border-slate-200/80 bg-slate-50/50 flex items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <span className="text-3xl p-2 rounded-xl bg-white border border-slate-200/90 shadow-2xs">
              {categoryIcon}
            </span>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-base sm:text-lg font-bold text-slate-900">
                  {isVi ? categoryName : categoryNameEn}
                </h3>
                <span
                  className={`text-2xs font-bold px-2.5 py-0.5 rounded-full uppercase tracking-wider ${
                    isBlocked
                      ? 'bg-rose-100 text-rose-700 border border-rose-200'
                      : 'bg-emerald-100 text-emerald-700 border border-emerald-200'
                  }`}
                >
                  {isBlocked ? (isVi ? 'Đang Chặn' : 'Blocked') : (isVi ? 'Cho Phép' : 'Allowed')}
                </span>
              </div>
              <p className="text-xs text-slate-500 mt-0.5 line-clamp-1">
                {isVi ? categoryDesc : categoryDescEn}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {/* Quick Toggle Block Switch */}
            <button
              type="button"
              onClick={handleToggle}
              disabled={isProcessing}
              className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-semibold transition cursor-pointer shadow-xs disabled:opacity-50 ${
                isBlocked
                  ? 'bg-rose-600 hover:bg-rose-700 text-white'
                  : 'bg-emerald-600 hover:bg-emerald-700 text-white'
              }`}
            >
              {isBlocked ? (
                <>
                  <ShieldAlert className="w-3.5 h-3.5" />
                  <span>{isVi ? 'Tắt Chặn Danh Mục' : 'Unblock Category'}</span>
                </>
              ) : (
                <>
                  <ShieldCheck className="w-3.5 h-3.5" />
                  <span>{isVi ? 'Bật Chặn Danh Mục' : 'Block Category'}</span>
                </>
              )}
            </button>

            {/* Close Button */}
            <button
              type="button"
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-200/70 rounded-lg transition cursor-pointer"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* MODAL BODY */}
        <div className="p-4 sm:p-6 overflow-y-auto space-y-5 flex-1">
          {/* STATS & SUMMARY TILES */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
              <div className="text-[11px] font-medium text-slate-500">
                {isVi ? 'Tổng tên miền' : 'Total Domains'}
              </div>
              <div className="text-lg font-bold font-mono text-slate-900 mt-0.5">
                {allItems.length.toLocaleString()}
              </div>
            </div>

            <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
              <div className="text-[11px] font-medium text-slate-500">
                {isVi ? 'Hạt giống hệ thống' : 'System Seeds'}
              </div>
              <div className="text-lg font-bold font-mono text-slate-700 mt-0.5">
                {systemSeeds.length}
              </div>
            </div>

            <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
              <div className="text-[11px] font-medium text-slate-500">
                {isVi ? 'Tệp & Tuỳ biến' : 'Files & Custom'}
              </div>
              <div className="text-lg font-bold font-mono text-sky-700 mt-0.5">
                {customDomains.length.toLocaleString()}
              </div>
            </div>

            <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
              <div className="text-[11px] font-medium text-slate-500">
                {isVi ? 'Tệp nguồn đã nạp' : 'Source Files'}
              </div>
              <div className="text-lg font-bold font-mono text-indigo-700 mt-0.5">
                {sourceFiles.length}
              </div>
            </div>
          </div>

          {/* SOURCE FILES MANAGEMENT SECTION */}
          {sourceFiles.length > 0 && (
            <div className="p-4 rounded-xl bg-indigo-50/50 border border-indigo-200/70 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <FileText className="w-4 h-4 text-indigo-600" />
                  <h4 className="text-xs font-bold text-slate-900">
                    {isVi ? 'Tệp Danh Sách Nguồn Đã Nạp (Source Blocklists)' : 'Imported Source Files'}
                  </h4>
                </div>
                <span className="text-2xs text-slate-500">
                  {sourceFiles.length} {isVi ? 'tệp đang hoạt động' : 'files active'}
                </span>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                {sourceFiles.map((file) => (
                  <div
                    key={file.id}
                    className="p-2.5 rounded-lg bg-white border border-indigo-100 flex items-center justify-between shadow-2xs text-xs"
                  >
                    <div className="flex items-center gap-2 truncate">
                      <FileText className="w-3.5 h-3.5 text-indigo-500 shrink-0" />
                      <div className="truncate">
                        <div className="font-semibold text-slate-900 truncate">
                          {file.file_name}
                        </div>
                        <div className="text-[10px] text-slate-500">
                          {file.domain_count.toLocaleString()} {isVi ? 'tên miền' : 'domains'}
                          {file.size_bytes ? ` • ${(file.size_bytes / 1024).toFixed(1)} KB` : ''}
                        </div>
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={() => handleRemoveFile(file.id, file.file_name)}
                      disabled={isProcessing}
                      className="p-1 text-slate-400 hover:text-rose-600 transition cursor-pointer"
                      title={isVi ? 'Xoá tệp này' : 'Remove this file'}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* ACTIONS & TOOLBAR */}
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pt-2">
            {/* Search Input */}
            <div className="relative flex-1">
              <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => {
                  setSearchTerm(e.target.value);
                  setCurrentPage(1);
                }}
                placeholder={
                  isVi
                    ? 'Tìm kiếm tên miền trong danh mục...'
                    : 'Search domain in category...'
                }
                className="w-full pl-8 pr-8 py-2 rounded-xl bg-slate-50 border border-slate-300 focus:bg-white focus:border-sky-500 focus:ring-2 focus:ring-sky-100 text-xs font-mono text-slate-900 outline-none transition"
              />
              {searchTerm && (
                <button
                  type="button"
                  onClick={() => setSearchTerm('')}
                  className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 text-xs p-1"
                >
                  ✕
                </button>
              )}
            </div>

            {/* Action Buttons: Import, Export, Add */}
            <div className="flex items-center gap-2 shrink-0">
              {/* Hidden file input */}
              <input
                ref={fileInputRef}
                type="file"
                accept=".txt,.list,.csv,.hosts"
                onChange={handleFileUpload}
                className="hidden"
              />

              {/* Import from file button */}
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                disabled={isProcessing}
                className="inline-flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold bg-sky-50 hover:bg-sky-100 text-sky-700 border border-sky-200 transition cursor-pointer shadow-2xs"
                title={isVi ? 'Nạp danh sách tên miền từ tệp .txt / .list' : 'Import domain list from file'}
              >
                <Upload className="w-3.5 h-3.5" />
                <span>{isVi ? 'Nạp Tệp' : 'Import File'}</span>
              </button>

              {/* Export to .txt button */}
              <button
                type="button"
                onClick={handleExportTxt}
                className="inline-flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-200 transition cursor-pointer shadow-2xs"
                title={isVi ? 'Xuất toàn bộ tên miền ra tệp .txt' : 'Export all domains to .txt'}
              >
                <Download className="w-3.5 h-3.5" />
                <span>{isVi ? 'Xuất Tệp' : 'Export .txt'}</span>
              </button>
            </div>
          </div>

          {/* ADD SINGLE DOMAIN FORM */}
          <form onSubmit={handleAddDomain} className="flex gap-2">
            <input
              type="text"
              value={newDomainInput}
              onChange={(e) => setNewDomainInput(e.target.value)}
              placeholder={
                isVi
                  ? 'Thêm nhanh tên miền đơn lẻ (ví dụ: badsite.com)...'
                  : 'Add a single domain (e.g. badsite.com)...'
              }
              className="flex-1 px-3 py-2 rounded-xl bg-white border border-slate-300 focus:border-sky-500 focus:ring-2 focus:ring-sky-100 text-xs font-mono text-slate-900 outline-none transition"
            />
            <button
              type="submit"
              disabled={isProcessing || !newDomainInput.trim()}
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white transition cursor-pointer shadow-xs whitespace-nowrap"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>{isVi ? 'Thêm' : 'Add'}</span>
            </button>
          </form>

          {/* HIGH-PERFORMANCE PAGINATED DOMAIN TABLE (ANTI-FREEZE ARCHITECTURE) */}
          <div className="border border-slate-200/90 rounded-xl overflow-hidden bg-white shadow-2xs flex flex-col">
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="bg-slate-50/80 border-b border-slate-200 text-slate-600 font-semibold text-[11px]">
                    <th className="py-2.5 px-3 w-12 text-center">#</th>
                    <th className="py-2.5 px-4">{isVi ? 'Tên Miền' : 'Domain'}</th>
                    <th className="py-2.5 px-3">{isVi ? 'Nguồn Gốc' : 'Source'}</th>
                    <th className="py-2.5 px-3 text-center">{isVi ? 'Tình Trạng' : 'Status'}</th>
                    <th className="py-2.5 px-3 w-20 text-right">{isVi ? 'Thao Tác' : 'Action'}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {paginatedItems.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="py-8 text-center text-slate-400">
                        {searchTerm
                          ? isVi
                            ? 'Không tìm thấy tên miền nào phù hợp với từ khoá tìm kiếm.'
                            : 'No matching domains found.'
                          : isVi
                          ? 'Danh mục hiện tại chưa có tên miền nào.'
                          : 'No domains in this category.'}
                      </td>
                    </tr>
                  ) : (
                    paginatedItems.map((item, idx) => {
                      const absoluteIndex = startIndex + idx + 1;
                      const isSystem = item.source === 'system';

                      return (
                        <tr
                          key={item.domain}
                          className="hover:bg-slate-50/80 transition-colors"
                        >
                          <td className="py-2 px-3 text-center text-slate-400 font-mono text-2xs">
                            {absoluteIndex}
                          </td>
                          <td className="py-2 px-4 font-mono font-medium text-slate-800">
                            {item.domain}
                          </td>
                          <td className="py-2 px-3">
                            {isSystem ? (
                              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-slate-100 text-slate-600 border border-slate-200">
                                <Database className="w-3 h-3 text-slate-400" />
                                <span>{isVi ? 'Hệ thống chuẩn' : 'System Default'}</span>
                              </span>
                            ) : item.source === 'file' ? (
                              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-indigo-50 text-indigo-700 border border-indigo-200">
                                <FileText className="w-3 h-3 text-indigo-500" />
                                <span className="truncate max-w-[130px]">
                                  {item.sourceFileName || (isVi ? 'Tệp nạp' : 'File')}
                                </span>
                              </span>
                            ) : (
                              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-semibold bg-sky-50 text-sky-700 border border-sky-200">
                                <span>{isVi ? 'Tuỳ biến' : 'Custom'}</span>
                              </span>
                            )}
                          </td>
                          <td className="py-2 px-3 text-center">
                            <span
                              className={`inline-block px-2 py-0.5 rounded-full text-[10px] font-bold ${
                                isBlocked
                                  ? 'bg-rose-50 text-rose-600 border border-rose-200'
                                  : 'bg-emerald-50 text-emerald-600 border border-emerald-200'
                              }`}
                            >
                              {isBlocked ? (isVi ? 'Chặn' : 'Block') : (isVi ? 'Cho phép' : 'Allow')}
                            </span>
                          </td>
                          <td className="py-2 px-3 text-right">
                            {isSystem ? (
                              <span className="text-[10px] text-slate-400 italic">
                                {isVi ? 'Mặc định' : 'Fixed'}
                              </span>
                            ) : (
                              <button
                                type="button"
                                onClick={() => handleDeleteDomain(item.domain)}
                                disabled={isProcessing}
                                className="p-1 text-slate-400 hover:text-rose-600 rounded transition cursor-pointer"
                                title={isVi ? 'Xoá tên miền này' : 'Delete domain'}
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </button>
                            )}
                          </td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>

            {/* PAGINATION CONTROLLER (ANTI-FREEZE ARCHITECTURE) */}
            <div className="p-3 bg-slate-50/70 border-t border-slate-200 flex flex-wrap items-center justify-between gap-3 text-xs text-slate-600">
              <div className="flex items-center gap-2">
                <span>{isVi ? 'Hiển thị:' : 'Show:'}</span>
                <select
                  value={pageSize}
                  onChange={(e) => {
                    setPageSize(Number(e.target.value));
                    setCurrentPage(1);
                  }}
                  className="px-2 py-1 rounded-lg bg-white border border-slate-300 text-xs text-slate-800 outline-none"
                >
                  <option value={10}>10</option>
                  <option value={25}>25</option>
                  <option value={50}>50</option>
                  <option value={100}>100</option>
                </select>
                <span className="text-2xs text-slate-500">
                  {isVi
                    ? `(Đang hiển thị ${filteredItems.length.toLocaleString()} tên miền)`
                    : `(${filteredItems.length.toLocaleString()} domains total)`}
                </span>
              </div>

              <div className="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={() => setCurrentPage(1)}
                  disabled={safeCurrentPage <= 1}
                  className="p-1.5 rounded-lg border border-slate-300 bg-white hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white transition cursor-pointer"
                  title={isVi ? 'Trang đầu' : 'First page'}
                >
                  <ChevronsLeft className="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                  disabled={safeCurrentPage <= 1}
                  className="p-1.5 rounded-lg border border-slate-300 bg-white hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white transition cursor-pointer"
                  title={isVi ? 'Trang trước' : 'Previous page'}
                >
                  <ChevronLeft className="w-3.5 h-3.5" />
                </button>

                <span className="px-3 py-1 font-mono text-2xs font-semibold bg-white border border-slate-300 rounded-lg">
                  {safeCurrentPage} / {totalPages}
                </span>

                <button
                  type="button"
                  onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                  disabled={safeCurrentPage >= totalPages}
                  className="p-1.5 rounded-lg border border-slate-300 bg-white hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white transition cursor-pointer"
                  title={isVi ? 'Trang kế' : 'Next page'}
                >
                  <ChevronRight className="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={() => setCurrentPage(totalPages)}
                  disabled={safeCurrentPage >= totalPages}
                  className="p-1.5 rounded-lg border border-slate-300 bg-white hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white transition cursor-pointer"
                  title={isVi ? 'Trang cuối' : 'Last page'}
                >
                  <ChevronsRight className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2 text-2xs text-slate-400 bg-slate-50 p-2.5 rounded-xl border border-slate-200/60">
            <Info className="w-3.5 h-3.5 shrink-0 text-slate-500" />
            <span>
              {isVi
                ? 'Kiến trúc bảo vệ hiệu năng: Dữ liệu tên miền được phân trang an toàn trong RAM, không sinh DOM thừa giúp trình duyệt luôn đạt tốc độ tối đa ngay cả khi danh mục nạp hàng chục nghìn tên miền.'
                : 'Performance safe: Domain records are cleanly paginated to prevent DOM lag even with 50,000+ entries.'}
            </span>
          </div>
        </div>

        {/* MODAL FOOTER */}
        <div className="p-4 border-t border-slate-200/80 bg-slate-50/50 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="px-5 py-2 rounded-xl text-xs font-semibold bg-slate-200 hover:bg-slate-300 text-slate-700 transition cursor-pointer"
          >
            {isVi ? 'Đóng' : 'Close'}
          </button>
        </div>
      </div>
    </div>
  );
};
