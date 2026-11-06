import type React from 'react';
import { FileSpreadsheet, X } from 'lucide-react';
import type { ParsedImportRule } from './rulesTypes';

interface ExcelImportModalProps {
  isOpen: boolean;
  onClose: () => void;
  importFileName: string;
  previewRules: ParsedImportRule[];
  isImporting: boolean;
  onConfirmImport: () => Promise<void>;
  isVi: boolean;
}

export const ExcelImportModal: React.FC<ExcelImportModalProps> = ({
  isOpen,
  onClose,
  importFileName,
  previewRules,
  isImporting,
  onConfirmImport,
  isVi,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-xs animate-fade-in">
      <div className="bg-white rounded-2xl border border-slate-200 max-w-2xl w-full p-6 shadow-xl flex flex-col gap-4 max-h-[90vh]">
        <div className="flex justify-between items-start">
          <div>
            <h3 className="text-base font-bold text-slate-900 flex items-center gap-2">
              <FileSpreadsheet className="text-emerald-600" size={18} />
              <span>{isVi ? 'Xem Trước Dữ Liệu Nhập Từ Excel' : 'Excel Import Preview'}</span>
            </h3>
            <p className="text-xs text-slate-500 mt-0.5">
              {isVi ? `Tệp: ${importFileName} • Phát hiện ` : `File: ${importFileName} • Detected `}
              <strong className="text-slate-700">{previewRules.length}</strong>{' '}
              {isVi ? 'quy tắc' : 'rules'}
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 p-1 rounded-lg cursor-pointer"
          >
            <X size={18} />
          </button>
        </div>

        {/* List preview */}
        <div className="overflow-y-auto max-h-[350px] border border-slate-200 rounded-xl divide-y divide-slate-100">
          {previewRules.map((r, idx) => (
            <div key={idx} className="p-3 text-xs flex justify-between items-center gap-3">
              <div>
                <span className="font-mono font-bold text-slate-900">{r.rule_id}</span>
                {r.reason && <div className="text-slate-500 text-2xs mt-0.5">{r.reason}</div>}
              </div>
              <div className="flex items-center gap-2">
                <span
                  className={`px-2 py-0.5 rounded text-2xs font-mono font-bold ${
                    r.action === 'ALLOW'
                      ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                      : r.rule_type === 'application'
                      ? 'bg-amber-50 text-amber-700 border border-amber-200'
                      : 'bg-rose-50 text-rose-700 border border-rose-200'
                  }`}
                >
                  {r.action === 'ALLOW'
                    ? (isVi ? 'CHO PHÉP' : 'ALLOW')
                    : r.rule_type === 'application'
                    ? (isVi ? 'GIỚI HẠN' : 'LIMIT')
                    : (isVi ? 'CHẶN' : 'BLOCK')}
                </span>
                <span className="bg-slate-100 text-slate-700 px-2 py-0.5 rounded text-2xs font-mono uppercase font-semibold">
                  {r.rule_type}
                </span>
                {r.quota && (
                  <span className="bg-sky-50 text-sky-700 px-2 py-0.5 rounded text-2xs font-mono font-bold">
                    {r.quota} {isVi ? 'phút' : 'min'}
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="flex justify-end gap-2.5 pt-2">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-600 hover:bg-slate-100 cursor-pointer"
          >
            {isVi ? 'Hủy' : 'Cancel'}
          </button>
          <button
            type="button"
            disabled={isImporting}
            onClick={onConfirmImport}
            className="px-5 py-2 rounded-xl text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white cursor-pointer shadow-xs transition"
          >
            {isImporting ? (isVi ? 'Đang nhập...' : 'Importing...') : (isVi ? 'Xác Nhận Nhập' : 'Confirm Import')}
          </button>
        </div>
      </div>
    </div>
  );
};
