import {
  Copy,
  Download,
  FileText,
  RefreshCw,
  Search,
  Terminal,
} from 'lucide-react';
import type React from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { Language, SystemLogsResponse } from '../types';

interface SystemLogsViewProps {
  lang: Language;
  onShowToast?: (message: string, type?: 'success' | 'error' | 'info') => void;
}

export const SystemLogsView: React.FC<SystemLogsViewProps> = ({ lang, onShowToast }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [logData, setLogData] = useState<SystemLogsResponse | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [filterQuery, setFilterQuery] = useState<string>('');
  const [autoScroll, setAutoScroll] = useState<boolean>(true);
  const [limit, setLimit] = useState<number>(300);

  const terminalRef = useRef<HTMLDivElement>(null);

  const fetchLogs = useCallback(
    async (showLoading = true) => {
      if (showLoading) setIsLoading(true);
      try {
        const res = await api.getSystemLogs(limit);
        setLogData(res);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        onShowToast?.(
          msg || (isVi ? 'Không thể tải nhật ký dịch vụ' : 'Failed to fetch service logs'),
          'error'
        );
      } finally {
        setIsLoading(false);
      }
    },
    [limit, isVi, onShowToast]
  );

  useEffect(() => {
    fetchLogs(true);
  }, [fetchLogs]);

  useEffect(() => {
    if (autoScroll && terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [logData, autoScroll]);

  const filteredLines = useMemo(() => {
    if (!logData?.lines) return [];
    if (!filterQuery.trim()) return logData.lines;
    const query = filterQuery.toLowerCase();
    return logData.lines.filter((line: string) => line.toLowerCase().includes(query));
  }, [logData, filterQuery]);

  const handleCopyLogs = async () => {
    if (!logData?.lines?.length) return;
    try {
      await navigator.clipboard.writeText(logData.lines.join('\n'));
      onShowToast?.(t.logs_sys_copied, 'success');
    } catch {
      onShowToast?.(isVi ? 'Không thể sao chép nhật ký' : 'Failed to copy logs', 'error');
    }
  };

  const handleDownloadLogs = () => {
    const url = api.getSystemLogsDownloadUrl();
    window.open(url, '_blank');
  };

  const renderLogLine = (line: string, index: number) => {
    let levelClass = 'text-slate-300';
    if (line.includes('ERROR') || line.includes('error:')) {
      levelClass = 'text-rose-400 font-semibold';
    } else if (line.includes('WARN') || line.includes('warn:')) {
      levelClass = 'text-amber-400 font-medium';
    } else if (line.includes('INFO') || line.includes('info:')) {
      levelClass = 'text-emerald-400';
    } else if (line.includes('DEBUG') || line.includes('TRACE')) {
      levelClass = 'text-sky-400';
    }

    return (
      <div
        key={index}
        className="flex items-start hover:bg-slate-900/60 px-2 py-0.5 rounded font-mono text-xs transition-colors"
      >
        <span className="text-slate-600 select-none w-10 text-right pr-3 shrink-0 font-mono text-2xs pt-0.5">
          {index + 1}
        </span>
        <span className={`break-all leading-relaxed ${levelClass}`}>{line}</span>
      </div>
    );
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Top Header & Actions */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 bg-slate-50/80 p-4 rounded-xl border border-slate-200/80">
        <div>
          <div className="flex items-center gap-2">
            <Terminal size={18} className="text-sky-600" />
            <h3 className="text-base font-bold text-slate-900">{t.logs_sys_title}</h3>
            {logData?.total_lines !== undefined && (
              <span className="bg-slate-200 text-slate-700 text-xs px-2 py-0.5 rounded-full font-semibold font-mono">
                {filteredLines.length}/{logData.total_lines} {isVi ? 'dòng' : 'lines'}
              </span>
            )}
          </div>
          <p className="text-xs text-slate-500 mt-0.5">{t.logs_sys_desc}</p>
          {logData?.file_name && (
            <p className="text-2xs text-slate-400 font-mono mt-1 flex items-center gap-1">
              <FileText size={11} className="text-slate-400" />
              <span>{t.logs_sys_file}</span>
              <span className="text-slate-600 font-semibold">{logData.file_name}</span>
            </p>
          )}
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            onClick={() => fetchLogs(false)}
            disabled={isLoading}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
            title={t.btn_refresh}
          >
            <RefreshCw size={13} className={isLoading ? 'animate-spin text-sky-600' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          <button
            onClick={handleCopyLogs}
            disabled={!logData?.lines?.length}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
            title={t.logs_sys_copy}
          >
            <Copy size={13} />
            <span>{t.logs_sys_copy}</span>
          </button>

          <button
            onClick={handleDownloadLogs}
            className="inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-sky-600 hover:bg-sky-700 text-white transition-colors shadow-xs cursor-pointer"
            title={t.logs_sys_download}
          >
            <Download size={13} />
            <span>{t.logs_sys_download}</span>
          </button>
        </div>
      </div>

      {/* Filter and Limit Controls */}
      <div className="flex flex-wrap items-center justify-between gap-3 text-xs">
        <div className="relative flex-1 min-w-[240px] max-w-md">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            placeholder={
              isVi
                ? 'Tìm kiếm theo từ khóa trong log (INFO, WARN, socket, v.v.)...'
                : 'Filter log lines (INFO, WARN, socket, etc.)...'
            }
            className="w-full pl-9 pr-3 py-1.5 bg-white border border-slate-200 rounded-lg text-xs text-slate-700 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-sky-500 focus:border-sky-500"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="flex items-center gap-1.5 text-slate-600 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="rounded text-sky-600 focus:ring-sky-500 border-slate-300"
            />
            <span>{isVi ? 'Tự cuộn xuống đáy' : 'Auto-scroll'}</span>
          </label>

          <select
            value={limit}
            onChange={(e) => setLimit(Number(e.target.value))}
            className="bg-white border border-slate-200 text-slate-700 text-xs rounded-lg px-2.5 py-1.5 focus:outline-none focus:ring-1 focus:ring-sky-500 cursor-pointer"
          >
            <option value={100}>100 {isVi ? 'dòng' : 'lines'}</option>
            <option value={300}>300 {isVi ? 'dòng' : 'lines'}</option>
            <option value={500}>500 {isVi ? 'dòng' : 'lines'}</option>
            <option value={1000}>1000 {isVi ? 'dòng' : 'lines'}</option>
          </select>
        </div>
      </div>

      {/* Terminal Display */}
      <div className="relative bg-slate-950 border border-slate-800 rounded-xl overflow-hidden shadow-inner">
        {/* Terminal Header Bar */}
        <div className="flex items-center justify-between px-4 py-2 bg-slate-900 border-b border-slate-800 text-2xs text-slate-400">
          <div className="flex items-center gap-2">
            <span className="w-2.5 h-2.5 rounded-full bg-rose-500/80 inline-block"></span>
            <span className="w-2.5 h-2.5 rounded-full bg-amber-500/80 inline-block"></span>
            <span className="w-2.5 h-2.5 rounded-full bg-emerald-500/80 inline-block"></span>
            <span className="ml-2 font-mono text-slate-300 font-semibold">famshield-service.log</span>
          </div>
          <span className="font-mono text-slate-500">
            UTF-8 • {filteredLines.length} {isVi ? 'dòng hiển thị' : 'lines'}
          </span>
        </div>

        {/* Terminal Content Body */}
        <div
          ref={terminalRef}
          className="p-3 max-h-[560px] min-h-[300px] overflow-y-auto overflow-x-auto space-y-0.5 select-text"
        >
          {isLoading && !logData ? (
            <div className="flex flex-col items-center justify-center py-20 text-slate-400 gap-2">
              <RefreshCw size={24} className="animate-spin text-sky-400" />
              <p className="text-xs">{isVi ? 'Đang đọc tệp nhật ký...' : 'Reading service logs...'}</p>
            </div>
          ) : filteredLines.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 text-slate-500 gap-2">
              <Terminal size={28} className="text-slate-600" />
              <p className="text-xs">{t.logs_sys_empty}</p>
            </div>
          ) : (
            filteredLines.map((line: string, idx: number) => renderLogLine(line, idx))
          )}
        </div>
      </div>
    </div>
  );
};
