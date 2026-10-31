import { Database, Trash2 } from 'lucide-react';
import type React from 'react';
import type { DnsRetentionConfig, Language } from '../../types';

interface StorageRetentionCardProps {
  retentionConfig: DnsRetentionConfig;
  selectedRetentionDays: number;
  autoPruneEnabled: boolean;
  isSavingRetention: boolean;
  isPruningNow: boolean;
  onSaveRetention: (days: number, auto: boolean) => void;
  onPruneNow: () => void;
  onChangeRetentionDays: (days: number) => void;
  lang: Language;
}

export const StorageRetentionCard: React.FC<StorageRetentionCardProps> = ({
  retentionConfig,
  selectedRetentionDays,
  autoPruneEnabled,
  isSavingRetention,
  isPruningNow,
  onSaveRetention,
  onPruneNow,
  onChangeRetentionDays,
  lang,
}) => {
  const isVi = lang === 'vi';

  return (
    <div className="bg-slate-50 border border-slate-200 rounded-2xl p-4 sm:p-5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 animate-in fade-in duration-200">
      <div>
        <h3 className="text-sm font-bold text-slate-900 flex items-center gap-2">
          <Database size={15} className="text-sky-600" />
          <span>{isVi ? 'Tối Ưu Hóa Bộ Nhớ SQLite' : 'SQLite Storage Retention'}</span>
        </h3>
        <p className="text-xs text-slate-500 mt-0.5">
          {isVi
            ? 'Tự động dọn dẹp các truy vấn thô cũ hơn số ngày quy định, giữ lại bảng tổng hợp tên miền nhẹ dưới 2MB.'
            : 'Raw query logs older than retention period are pruned into domain aggregates to keep DB lightweight.'}
        </p>
        <div className="flex gap-4 mt-2 text-2xs font-semibold text-slate-600 flex-wrap">
          <span>
            {isVi ? 'Truy vấn chi tiết hiện có:' : 'Raw query logs:'}{' '}
            <strong className="text-slate-800">
              {retentionConfig.raw_query_logs_count.toLocaleString()}
            </strong>
          </span>
          <span>
            {isVi ? 'Tên miền đã tổng hợp:' : 'Aggregated domains:'}{' '}
            <strong className="text-slate-800">
              {retentionConfig.aggregated_domains_count.toLocaleString()}
            </strong>
          </span>
          {retentionConfig.last_pruned_at && (
            <span className="text-slate-500">
              {isVi ? 'Dọn dẹp gần nhất:' : 'Last pruned:'}{' '}
              {new Date(retentionConfig.last_pruned_at).toLocaleTimeString()}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-3 flex-wrap">
        <div className="flex items-center gap-2">
          <label className="text-xs text-slate-600 font-semibold">
            {isVi ? 'Lưu chi tiết:' : 'Retain raw:'}
          </label>
          <select
            value={selectedRetentionDays}
            onChange={(e) => {
              const days = parseInt(e.target.value, 10);
              onChangeRetentionDays(days);
              onSaveRetention(days, autoPruneEnabled);
            }}
            disabled={isSavingRetention}
            className="bg-white border border-slate-200 text-slate-800 px-2.5 py-1.5 rounded-lg text-xs cursor-pointer outline-none focus:border-sky-500"
          >
            <option value={1}>{isVi ? '1 ngày (Tiết kiệm tối đa)' : '1 day (Max savings)'}</option>
            <option value={3}>{isVi ? '3 ngày' : '3 days'}</option>
            <option value={7}>{isVi ? '7 ngày (Khuyên dùng)' : '7 days (Recommended)'}</option>
            <option value={14}>{isVi ? '14 ngày' : '14 days'}</option>
            <option value={30}>{isVi ? '30 ngày' : '30 days'}</option>
            <option value={90}>{isVi ? '90 ngày' : '90 days'}</option>
          </select>
        </div>

        <button
          type="button"
          onClick={onPruneNow}
          disabled={isPruningNow}
          className="inline-flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 disabled:opacity-50 text-white border-0 px-3.5 py-1.5 rounded-lg text-xs font-semibold cursor-pointer shadow-xs transition-all"
        >
          <Trash2 size={13} className={isPruningNow ? 'animate-spin' : ''} />
          {isPruningNow
            ? isVi
              ? 'Đang dọn dẹp...'
              : 'Pruning...'
            : isVi
              ? 'Thu Gọn & Dọn Dẹp Ngay'
              : 'Prune Now'}
        </button>
      </div>
    </div>
  );
};
