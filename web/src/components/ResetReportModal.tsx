import { AlertTriangle, CheckSquare, RotateCcw, ShieldAlert, Square, X } from 'lucide-react';
import type React from 'react';
import { useEffect, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { Language } from '../types';

interface ResetReportModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  onShowToast?: (msg: string, type?: 'success' | 'error' | 'info') => void;
  lang: Language;
  defaultScope?: 'all' | 'dns' | 'apps' | 'evidence' | 'keywords';
}

export const ResetReportModal: React.FC<ResetReportModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
  onShowToast,
  lang,
  defaultScope = 'all',
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [masterPin, setMasterPin] = useState('');
  const [otpCode, setOtpCode] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [totpEnabled, setTotpEnabled] = useState(false);

  // Scopes state
  const [scopes, setScopes] = useState({
    all: defaultScope === 'all',
    dns: defaultScope === 'all' || defaultScope === 'dns',
    apps: defaultScope === 'all' || defaultScope === 'apps',
    evidence: defaultScope === 'all' || defaultScope === 'evidence',
    keywords: defaultScope === 'all' || defaultScope === 'keywords',
  });

  useEffect(() => {
    if (isOpen) {
      setMasterPin('');
      setOtpCode('');
      setError(null);
      setScopes({
        all: defaultScope === 'all',
        dns: defaultScope === 'all' || defaultScope === 'dns',
        apps: defaultScope === 'all' || defaultScope === 'apps',
        evidence: defaultScope === 'all' || defaultScope === 'evidence',
        keywords: defaultScope === 'all' || defaultScope === 'keywords',
      });

      api
        .getTotpStatus()
        .then((res) => {
          setTotpEnabled(res.enabled);
        })
        .catch(() => {});
    }
  }, [isOpen, defaultScope]);

  if (!isOpen) return null;

  const toggleAll = () => {
    const nextVal = !scopes.all;
    setScopes({
      all: nextVal,
      dns: nextVal,
      apps: nextVal,
      evidence: nextVal,
      keywords: nextVal,
    });
  };

  const toggleItem = (key: 'dns' | 'apps' | 'evidence' | 'keywords') => {
    const updated = { ...scopes, [key]: !scopes[key] };
    const allSelected = updated.dns && updated.apps && updated.evidence && updated.keywords;
    updated.all = allSelected;
    setScopes(updated);
  };

  const handleConfirm = async () => {
    if (!masterPin.trim()) {
      setError(isVi ? 'Vui lòng nhập Master PIN để xác nhận.' : 'Please enter Master PIN.');
      return;
    }

    const selectedList: string[] = [];
    if (scopes.all) {
      selectedList.push('all');
    } else {
      if (scopes.dns) selectedList.push('dns');
      if (scopes.apps) selectedList.push('apps');
      if (scopes.evidence) selectedList.push('evidence');
      if (scopes.keywords) selectedList.push('keywords');
    }

    if (selectedList.length === 0) {
      setError(
        isVi
          ? 'Vui lòng chọn ít nhất một mục báo cáo để đặt lại.'
          : 'Please select at least one report item to reset.'
      );
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      const res = await api.resetReports({
        master_pin: masterPin,
        otp_code: otpCode.trim() || undefined,
        scope: selectedList,
      });

      if (res.success) {
        onShowToast?.(
          res.message || t.reset_reports_success,
          'success'
        );
        onSuccess();
        onClose();
      } else {
        setError(res.message || (isVi ? 'Đặt lại báo cáo thất bại' : 'Reset failed'));
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg || (isVi ? 'Lỗi kết nối máy chủ' : 'Server connection error'));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/70 backdrop-blur-xs p-4 overflow-y-auto">
      <div className="bg-white border border-rose-200/90 rounded-2xl max-w-md w-full shadow-2xl overflow-hidden flex flex-col my-auto animate-in fade-in zoom-in-95 duration-200">
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-rose-100 bg-rose-50/50">
          <div className="flex items-center gap-2.5">
            <div className="w-9 h-9 rounded-xl bg-rose-100 flex items-center justify-center text-rose-600 shadow-2xs">
              <RotateCcw size={18} />
            </div>
            <div>
              <h3 className="text-sm font-bold text-slate-900 leading-tight">
                {t.reset_reports_title}
              </h3>
              <p className="text-2xs text-rose-600 font-medium">
                {isVi ? 'Khôi phục toàn bộ số liệu về con số 0' : 'Clean-slate activity reset'}
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg hover:bg-slate-100 cursor-pointer transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 flex flex-col gap-4 text-xs">
          <div className="bg-amber-50 border border-amber-200/80 rounded-xl p-3 flex gap-2.5 text-amber-800">
            <AlertTriangle size={16} className="text-amber-600 shrink-0 mt-0.5" />
            <div className="text-2xs leading-relaxed">
              {t.reset_reports_desc}
            </div>
          </div>

          {/* Scope Selection */}
          <div className="flex flex-col gap-2">
            <label className="font-semibold text-slate-700">
              {isVi ? 'Lựa chọn phạm vi đặt lại:' : 'Select reset scope:'}
            </label>

            <div className="space-y-1.5 bg-slate-50 border border-slate-200/80 rounded-xl p-3">
              {/* All */}
              <button
                type="button"
                onClick={toggleAll}
                className="w-full flex items-center gap-2.5 p-1.5 rounded-lg hover:bg-white text-left cursor-pointer transition-colors"
              >
                {scopes.all ? (
                  <CheckSquare size={16} className="text-rose-600 shrink-0" />
                ) : (
                  <Square size={16} className="text-slate-400 shrink-0" />
                )}
                <span className="font-bold text-slate-900">{t.reset_scope_all}</span>
              </button>

              <div className="border-t border-slate-200/60 my-1 pt-1 pl-6 space-y-1.5">
                <button
                  type="button"
                  onClick={() => toggleItem('dns')}
                  className="w-full flex items-center gap-2 p-1 rounded-md hover:bg-white text-left cursor-pointer transition-colors"
                >
                  {scopes.dns ? (
                    <CheckSquare size={14} className="text-sky-600 shrink-0" />
                  ) : (
                    <Square size={14} className="text-slate-400 shrink-0" />
                  )}
                  <span className="text-slate-700">{t.reset_scope_dns}</span>
                </button>

                <button
                  type="button"
                  onClick={() => toggleItem('apps')}
                  className="w-full flex items-center gap-2 p-1 rounded-md hover:bg-white text-left cursor-pointer transition-colors"
                >
                  {scopes.apps ? (
                    <CheckSquare size={14} className="text-sky-600 shrink-0" />
                  ) : (
                    <Square size={14} className="text-slate-400 shrink-0" />
                  )}
                  <span className="text-slate-700">{t.reset_scope_apps}</span>
                </button>

                <button
                  type="button"
                  onClick={() => toggleItem('evidence')}
                  className="w-full flex items-center gap-2 p-1 rounded-md hover:bg-white text-left cursor-pointer transition-colors"
                >
                  {scopes.evidence ? (
                    <CheckSquare size={14} className="text-sky-600 shrink-0" />
                  ) : (
                    <Square size={14} className="text-slate-400 shrink-0" />
                  )}
                  <span className="text-slate-700">{t.reset_scope_evidence}</span>
                </button>

                <button
                  type="button"
                  onClick={() => toggleItem('keywords')}
                  className="w-full flex items-center gap-2 p-1 rounded-md hover:bg-white text-left cursor-pointer transition-colors"
                >
                  {scopes.keywords ? (
                    <CheckSquare size={14} className="text-sky-600 shrink-0" />
                  ) : (
                    <Square size={14} className="text-slate-400 shrink-0" />
                  )}
                  <span className="text-slate-700">{t.reset_scope_keywords}</span>
                </button>
              </div>
            </div>
          </div>

          {/* Master PIN Input */}
          <div className="flex flex-col gap-1.5">
            <label className="font-semibold text-slate-700 flex items-center gap-1.5">
              <ShieldAlert size={14} className="text-rose-600" />
              <span>{t.lbl_enter_pin_to_reset}</span>
            </label>
            <input
              type="password"
              placeholder={isVi ? 'Nhập Master PIN 6 số' : 'Enter 6-digit Master PIN'}
              value={masterPin}
              onChange={(e) => setMasterPin(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleConfirm();
              }}
              className="bg-slate-50 border border-slate-300 focus:border-rose-500 focus:ring-1 focus:ring-rose-500 rounded-lg px-3 py-2 text-slate-900 outline-none font-mono text-center tracking-widest text-sm shadow-xs"
            />
          </div>

          {/* 2FA OTP Input (Conditional) */}
          {totpEnabled && (
            <div className="flex flex-col gap-1.5 animate-in fade-in duration-200">
              <label className="font-semibold text-sky-700">
                {t.lbl_enter_otp_to_reset}
              </label>
              <input
                type="text"
                placeholder={isVi ? 'Mã OTP 6 số hoặc ABCD-1234' : '6-digit OTP or ABCD-1234'}
                value={otpCode}
                onChange={(e) => setOtpCode(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleConfirm();
                }}
                className="bg-slate-50 border border-slate-300 focus:border-sky-500 focus:ring-1 focus:ring-sky-500 rounded-lg px-3 py-2 text-slate-900 outline-none font-mono text-center tracking-wider text-sm shadow-xs"
              />
            </div>
          )}

          {error && (
            <div className="bg-rose-50 border border-rose-200 text-rose-700 p-2.5 rounded-lg text-2xs font-medium">
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2.5 p-4 border-t border-slate-100 bg-slate-50/70">
          <button
            type="button"
            onClick={onClose}
            disabled={isSubmitting}
            className="px-4 py-2 rounded-lg text-xs font-semibold text-slate-600 hover:text-slate-800 hover:bg-slate-200/60 cursor-pointer transition-colors"
          >
            {isVi ? 'Hủy' : 'Cancel'}
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={isSubmitting}
            className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-bold text-white bg-rose-600 hover:bg-rose-700 disabled:opacity-50 cursor-pointer shadow-xs transition-colors"
          >
            {isSubmitting ? (
              <>
                <RotateCcw size={14} className="animate-spin" />
                <span>{t.reset_in_progress}</span>
              </>
            ) : (
              <>
                <RotateCcw size={14} />
                <span>{t.btn_confirm_reset}</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
