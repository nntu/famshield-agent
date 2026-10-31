import type React from 'react';
import { Cloud, RefreshCw, Upload, FileText, Download } from 'lucide-react';
import type { CloudPolicyData } from '../../types';

interface CloudPolicyBannerProps {
  isVi: boolean;
  t: any;
  cloudPolicy: CloudPolicyData | null;
  isPullingCloud: boolean;
  isSyncingCloud: boolean;
  onPullFromCloud: () => Promise<void>;
  onSyncToCloud: () => Promise<void>;
  onDownloadTemplate: () => Promise<void>;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  onFileSelected: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onExportExcel: () => Promise<void>;
}

export const CloudPolicyBanner: React.FC<CloudPolicyBannerProps> = ({
  isVi,
  t,
  cloudPolicy,
  isPullingCloud,
  isSyncingCloud,
  onPullFromCloud,
  onSyncToCloud,
  onDownloadTemplate,
  fileInputRef,
  onFileSelected,
  onExportExcel,
}) => {
  return (
    <div className="bg-gradient-to-r from-sky-950 via-slate-900 to-indigo-950 text-white rounded-2xl p-5 sm:p-6 shadow-xs border border-sky-800/40 flex flex-col gap-4">
      <div className="flex justify-between items-start flex-wrap gap-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-sky-500/20 border border-sky-400/30 flex items-center justify-center text-sky-300 shrink-0">
            <Cloud size={22} />
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <span className="font-bold text-base tracking-tight text-white">
                {isVi ? 'Chính Sách Bảo Vệ Từ Máy Chủ (Cloud Policy)' : 'Cloud Server Security Policy'}
              </span>
              <span className="bg-sky-400/20 text-sky-200 border border-sky-400/30 px-2 py-0.5 rounded-md text-2xs font-mono font-bold">
                v{cloudPolicy?.version || 1}
              </span>
            </div>
            <p className="text-xs text-sky-200/80 mt-0.5">
              {isVi
                ? 'Quy tắc lọc web & chặn tên miền tập trung áp xuống từ Cloud Server qua kênh NATS WebSocket'
                : 'Centralized web filter policy pushed down from Cloud Server via NATS WebSocket'}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            type="button"
            onClick={onPullFromCloud}
            disabled={isPullingCloud}
            className="inline-flex items-center gap-1.5 bg-sky-500 hover:bg-sky-400 text-slate-950 font-bold px-3.5 py-2 rounded-xl text-xs cursor-pointer shadow-xs transition-all border-0 disabled:opacity-50"
          >
            <RefreshCw size={13} className={isPullingCloud ? 'animate-spin' : ''} />
            <span>{isVi ? 'Kéo từ Cloud' : 'Pull from Cloud'}</span>
          </button>
          <button
            type="button"
            onClick={onSyncToCloud}
            disabled={isSyncingCloud}
            className="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 text-white font-semibold px-3.5 py-2 rounded-xl text-xs cursor-pointer border border-white/20 transition-all disabled:opacity-50"
          >
            <Upload size={13} />
            <span>{isVi ? 'Đẩy quy tắc lên Cloud' : 'Sync to Cloud'}</span>
          </button>
          <button
            type="button"
            onClick={onDownloadTemplate}
            className="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 text-white font-semibold px-3 py-2 rounded-xl text-xs cursor-pointer border border-white/20 transition-all"
            title={t.btn_download_template}
          >
            <FileText size={13} className="text-sky-300" />
            <span>{isVi ? 'Mẫu Excel' : 'Excel Template'}</span>
          </button>
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="inline-flex items-center gap-1.5 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold px-3 py-2 rounded-xl text-xs cursor-pointer border-0 transition-all shadow-xs"
          >
            <Upload size={13} />
            <span>{t.btn_import_excel}</span>
          </button>
          <input
            type="file"
            ref={fileInputRef as any}
            onChange={onFileSelected}
            accept=".xlsx,.xls,.csv"
            style={{ display: 'none' }}
          />
          <button
            type="button"
            onClick={onExportExcel}
            className="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 text-white font-semibold px-3 py-2 rounded-xl text-xs cursor-pointer border border-white/20 transition-all"
          >
            <Download size={13} />
            <span>{t.btn_export_excel}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
