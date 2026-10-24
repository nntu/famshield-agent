import {
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  Clock,
  Download,
  Flame,
  Radio,
  RefreshCw,
  Search,
  Send,
  ShieldAlert,
  Trash2,
} from 'lucide-react';
import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { ErrorLogItem, Language, ListErrorsResponse } from '../types';

interface ErrorManagementViewProps {
  lang: Language;
  onShowToast?: (message: string, type?: 'success' | 'error' | 'info') => void;
}

export const ErrorManagementView: React.FC<ErrorManagementViewProps> = ({
  lang,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [data, setData] = useState<ListErrorsResponse | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [isPushingNats, setIsPushingNats] = useState<boolean>(false);
  const [isClearing, setIsClearing] = useState<boolean>(false);
  const [filterQuery, setFilterQuery] = useState<string>('');
  const [expandedTraceId, setExpandedTraceId] = useState<string | null>(null);

  const fetchErrors = useCallback(
    async (showLoading = true) => {
      if (showLoading) setIsLoading(true);
      try {
        const res = await api.getErrors();
        setData(res);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        onShowToast?.(
          msg || (isVi ? 'Không thể tải danh sách lỗi' : 'Failed to fetch error records'),
          'error'
        );
      } finally {
        setIsLoading(false);
      }
    },
    [isVi, onShowToast]
  );

  useEffect(() => {
    fetchErrors(true);
  }, [fetchErrors]);

  const totalOccurrences = useMemo(() => {
    if (!data?.errors) return 0;
    return data.errors.reduce((acc, err) => acc + (err.occurrence_count || 1), 0);
  }, [data]);

  const handlePushNats = async () => {
    setIsPushingNats(true);
    try {
      const res = await api.pushErrorsNats();
      if (res.success) {
        onShowToast?.(
          res.message ||
            (isVi
              ? `Đã đẩy thành công ${res.pushed_count} bản ghi lỗi lên NATS broker`
              : `Successfully pushed ${res.pushed_count} error events to NATS broker`),
          'success'
        );
        await fetchErrors(false);
      } else {
        onShowToast?.(
          res.message || (isVi ? 'Lỗi khi đẩy dữ liệu NATS' : 'Failed to push NATS events'),
          'error'
        );
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      onShowToast?.(msg, 'error');
    } finally {
      setIsPushingNats(false);
    }
  };

  const handleClearErrors = async () => {
    const confirmed = window.confirm(
      isVi
        ? 'Bạn có chắc chắn muốn xóa toàn bộ lịch sử lỗi này khỏi cơ sở dữ liệu không?'
        : 'Are you sure you want to clear all error records from local database?'
    );
    if (!confirmed) return;

    setIsClearing(true);
    try {
      const res = await api.clearErrors();
      if (res.success) {
        onShowToast?.(t.err_cleared, 'success');
        await fetchErrors(false);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      onShowToast?.(msg, 'error');
    } finally {
      setIsClearing(false);
    }
  };

  const handleSaveJson = () => {
    if (!data?.errors?.length) return;
    try {
      const exportPayload = {
        exported_at: new Date().toISOString(),
        total_errors: data.total_count,
        total_occurrences: totalOccurrences,
        unsynced_count: data.unsynced_count,
        errors: data.errors,
      };
      const blob = new Blob([JSON.stringify(exportPayload, null, 2)], {
        type: 'application/json',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `famshield-error-report-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      onShowToast?.(
        isVi ? 'Đã lưu tệp báo cáo JSON thành công!' : 'Saved JSON error report successfully!',
        'success'
      );
    } catch {
      onShowToast?.(isVi ? 'Lỗi khi lưu tệp báo cáo' : 'Failed to save report', 'error');
    }
  };

  const filteredErrors = useMemo(() => {
    if (!data?.errors) return [];
    if (!filterQuery.trim()) return data.errors;
    const q = filterQuery.toLowerCase();
    return data.errors.filter(
      (err: ErrorLogItem) =>
        err.message.toLowerCase().includes(q) ||
        err.module.toLowerCase().includes(q) ||
        err.severity.toLowerCase().includes(q) ||
        err.error_id.toLowerCase().includes(q)
    );
  }, [data, filterQuery]);

  const renderSeverityBadge = (severity: string) => {
    const s = severity.toUpperCase();
    if (s === 'CRITICAL') {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-2xs font-bold bg-rose-100 text-rose-800 border border-rose-200">
          <AlertCircle size={10} /> CRITICAL
        </span>
      );
    }
    if (s === 'ERROR') {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-2xs font-bold bg-rose-50 text-rose-700 border border-rose-200/80">
          <AlertCircle size={10} /> ERROR
        </span>
      );
    }
    if (s === 'WARN') {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-2xs font-bold bg-amber-50 text-amber-700 border border-amber-200/80">
          <AlertTriangle size={10} /> WARN
        </span>
      );
    }
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-2xs font-bold bg-slate-100 text-slate-700 border border-slate-200">
        {s}
      </span>
    );
  };

  return (
    <div className="flex flex-col gap-5">
      {/* Top Header & Actions */}
      <div className="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4 bg-slate-50/80 p-4 rounded-xl border border-slate-200/80">
        <div>
          <div className="flex items-center gap-2">
            <ShieldAlert size={20} className="text-rose-600" />
            <h3 className="text-base font-bold text-slate-900">{t.err_title}</h3>
          </div>
          <p className="text-xs text-slate-500 mt-0.5">{t.err_desc}</p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            onClick={() => fetchErrors(false)}
            disabled={isLoading}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
            title={t.btn_refresh}
          >
            <RefreshCw size={13} className={isLoading ? 'animate-spin text-sky-600' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          <button
            onClick={handleSaveJson}
            disabled={!data?.errors?.length}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
            title={t.err_save_json}
          >
            <Download size={13} />
            <span>{t.err_save_json}</span>
          </button>

          <button
            onClick={handleClearErrors}
            disabled={isClearing || !data?.errors?.length}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-lg bg-white border border-rose-200 text-rose-600 hover:bg-rose-50 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
            title={t.err_clear}
          >
            <Trash2 size={13} />
            <span>{t.err_clear}</span>
          </button>

          <button
            onClick={handlePushNats}
            disabled={isPushingNats || !data?.errors?.length}
            className="inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white transition-all shadow-xs cursor-pointer disabled:opacity-50"
            title={t.err_push_nats}
          >
            {isPushingNats ? (
              <RefreshCw size={13} className="animate-spin" />
            ) : (
              <Send size={13} />
            )}
            <span>{isPushingNats ? t.err_pushing : t.err_push_nats}</span>
          </button>
        </div>
      </div>

      {/* 3 Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3.5">
        <div className="bg-white border border-slate-200/80 rounded-xl p-4 flex items-center gap-3.5 shadow-2xs">
          <div className="w-10 h-10 rounded-xl bg-slate-100 flex items-center justify-center text-slate-700 shrink-0">
            <AlertTriangle size={20} />
          </div>
          <div>
            <span className="text-2xs font-semibold text-slate-500 uppercase tracking-wider block">
              {t.err_total}
            </span>
            <span className="text-xl font-bold text-slate-900 font-mono">
              {data ? data.total_count : '—'}
            </span>
          </div>
        </div>

        <div className="bg-white border border-slate-200/80 rounded-xl p-4 flex items-center gap-3.5 shadow-2xs">
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 ${
              (data?.unsynced_count || 0) > 0
                ? 'bg-amber-50 text-amber-600 border border-amber-200/80'
                : 'bg-emerald-50 text-emerald-600 border border-emerald-200/80'
            }`}
          >
            <Radio size={20} />
          </div>
          <div>
            <span className="text-2xs font-semibold text-slate-500 uppercase tracking-wider block">
              {t.err_unsynced}
            </span>
            <div className="flex items-center gap-2">
              <span className="text-xl font-bold text-slate-900 font-mono">
                {data ? data.unsynced_count : '—'}
              </span>
              {(data?.unsynced_count || 0) > 0 && (
                <span className="text-2xs bg-amber-100 text-amber-800 font-bold px-1.5 py-0.5 rounded-full">
                  {isVi ? 'Cần đẩy NATS' : 'Pending'}
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="bg-white border border-slate-200/80 rounded-xl p-4 flex items-center gap-3.5 shadow-2xs">
          <div className="w-10 h-10 rounded-xl bg-rose-50 text-rose-600 border border-rose-200/80 flex items-center justify-center shrink-0">
            <Flame size={20} />
          </div>
          <div>
            <span className="text-2xs font-semibold text-slate-500 uppercase tracking-wider block">
              {t.err_occurrences}
            </span>
            <span className="text-xl font-bold text-slate-900 font-mono">
              {data ? totalOccurrences : '—'}
            </span>
          </div>
        </div>
      </div>

      {/* Search Filter */}
      <div className="relative max-w-md">
        <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
        <input
          type="text"
          value={filterQuery}
          onChange={(e) => setFilterQuery(e.target.value)}
          placeholder={
            isVi
              ? 'Lọc theo thông điệp lỗi, module (dns, sync, storage...), mức độ...'
              : 'Filter by error message, module, severity...'
          }
          className="w-full pl-9 pr-3 py-1.5 bg-white border border-slate-200 rounded-lg text-xs text-slate-700 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-sky-500 focus:border-sky-500"
        />
      </div>

      {/* Error Records List */}
      {isLoading && !data ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-400 gap-2 bg-white border border-slate-200/80 rounded-xl">
          <RefreshCw size={24} className="animate-spin text-sky-500" />
          <p className="text-xs">{isVi ? 'Đang đọc danh sách lỗi...' : 'Loading error logs...'}</p>
        </div>
      ) : filteredErrors.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 px-4 text-center bg-white border border-slate-200/80 rounded-xl">
          <div className="w-12 h-12 rounded-full bg-emerald-50 text-emerald-600 flex items-center justify-center mb-3">
            <CheckCircle2 size={24} />
          </div>
          <p className="text-sm font-semibold text-slate-800">{t.err_empty}</p>
          <p className="text-xs text-slate-400 mt-1 max-w-md">
            {isVi
              ? 'Tất cả các thành phần dịch vụ FamShield DNS, WFP và Sync đang vận hành ổn định và không phát sinh lỗi bất thường.'
              : 'All FamShield system modules (DNS proxy, WFP filter, Sync daemon) are operating normally without recorded exceptions.'}
          </p>
        </div>
      ) : (
        <div className="border border-slate-200 rounded-xl overflow-hidden bg-white shadow-2xs">
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left text-xs">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50 text-slate-600 font-semibold">
                  <th className="py-2.5 px-3">{t.err_col_module}</th>
                  <th className="py-2.5 px-3">{t.err_col_severity}</th>
                  <th className="py-2.5 px-3">{t.err_col_msg}</th>
                  <th className="py-2.5 px-3 text-center">{t.err_col_count}</th>
                  <th className="py-2.5 px-3">{t.err_col_last_seen}</th>
                  <th className="py-2.5 px-3 text-center">{t.err_col_status}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {filteredErrors.map((err) => {
                  const isExpanded = expandedTraceId === err.error_id;
                  const hasTrace = Boolean(err.stack_trace && err.stack_trace.trim().length > 0);

                  return (
                    <React.Fragment key={err.error_id}>
                      <tr className="hover:bg-slate-50/60 transition-colors">
                        <td className="py-2.5 px-3">
                          <span className="px-2 py-0.5 rounded font-mono font-semibold bg-slate-100 text-slate-700 border border-slate-200 text-2xs">
                            {err.module}
                          </span>
                        </td>
                        <td className="py-2.5 px-3">{renderSeverityBadge(err.severity)}</td>
                        <td className="py-2.5 px-3 max-w-md">
                          <div className="font-medium text-slate-800 break-words">
                            {err.message}
                          </div>
                          <div className="flex items-center gap-2 mt-1">
                            <span className="font-mono text-2xs text-slate-400" title={err.error_id}>
                              ID: {err.error_id.slice(0, 16)}...
                            </span>
                            {hasTrace && (
                              <button
                                onClick={() =>
                                  setExpandedTraceId(isExpanded ? null : err.error_id)
                                }
                                className="inline-flex items-center gap-0.5 text-2xs font-semibold text-sky-600 hover:text-sky-700 cursor-pointer"
                              >
                                {isExpanded ? (
                                  <>
                                    <ChevronUp size={11} />{' '}
                                    {isVi ? 'Ẩn Stack Trace' : 'Hide Trace'}
                                  </>
                                ) : (
                                  <>
                                    <ChevronDown size={11} />{' '}
                                    {isVi ? 'Xem Stack Trace' : 'View Trace'}
                                  </>
                                )}
                              </button>
                            )}
                          </div>
                        </td>
                        <td className="py-2.5 px-3 text-center font-mono">
                          <span className="px-2 py-0.5 rounded-full text-2xs font-bold bg-rose-50 text-rose-700 border border-rose-200/80">
                            {err.occurrence_count}x
                          </span>
                        </td>
                        <td className="py-2.5 px-3 text-slate-600 font-mono text-2xs">
                          {new Date(err.last_seen_at).toLocaleTimeString()}{' '}
                          <span className="text-slate-400">
                            ({new Date(err.last_seen_at).toLocaleDateString()})
                          </span>
                        </td>
                        <td className="py-2.5 px-3 text-center">
                          {err.synced ? (
                            <span className="inline-flex items-center gap-1 text-2xs font-semibold text-emerald-700 bg-emerald-50 border border-emerald-200 px-2 py-0.5 rounded-full">
                              <CheckCircle2 size={11} />
                              {t.err_status_synced}
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 text-2xs font-semibold text-amber-700 bg-amber-50 border border-amber-200 px-2 py-0.5 rounded-full">
                              <Clock size={11} />
                              {t.err_status_pending}
                            </span>
                          )}
                        </td>
                      </tr>
                      {isExpanded && hasTrace && (
                        <tr className="bg-slate-900 text-slate-200 font-mono text-2xs">
                          <td colSpan={6} className="p-3">
                            <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 overflow-x-auto whitespace-pre leading-relaxed text-rose-300">
                              {err.stack_trace}
                            </div>
                          </td>
                        </tr>
                      )}
                    </React.Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
};
