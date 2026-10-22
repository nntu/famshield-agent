import { AlertTriangle, Clock, Eye, EyeOff, Shield, SlidersHorizontal, X } from 'lucide-react';
import type React from 'react';
import { useEffect, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { EvidenceItem, Language } from '../types';

interface EvidenceTabProps {
  evidenceList: EvidenceItem[];
  lang: Language;
  onRefresh: () => void;
  onShowToast?: (message: string, type?: 'success' | 'error' | 'info') => void;
}

export type BlurLevel = 'none' | 'reduced' | 'full';

export const EvidenceTab: React.FC<EvidenceTabProps> = ({
  evidenceList,
  lang,
  onRefresh,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceItem | null>(null);

  // Blur level: 'none' (tắt mờ), 'reduced' (giảm mờ / mờ nhẹ), 'full' (mờ bảo vệ)
  const [blurLevel, setBlurLevel] = useState<BlurLevel>(() => {
    return (localStorage.getItem('famshield_evidence_blur') as BlurLevel) || 'none';
  });
  const [modalBlurLevel, setModalBlurLevel] = useState<BlurLevel>(blurLevel);
  const [isSavingCaptureBlur, setIsSavingCaptureBlur] = useState<boolean>(false);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  const handleBlurChange = (lvl: BlurLevel) => {
    setBlurLevel(lvl);
    localStorage.setItem('famshield_evidence_blur', lvl);
  };

  const getBlurClass = (lvl: BlurLevel) => {
    switch (lvl) {
      case 'none':
        return 'blur-none';
      case 'reduced':
        return 'blur-[2.5px]';
      case 'full':
        return 'blur-md';
    }
  };

  useEffect(() => {
    if (!selectedEvidence) return;
    setModalBlurLevel(blurLevel);
    closeButtonRef.current?.focus();
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setSelectedEvidence(null);
    };
    document.addEventListener('keydown', closeOnEscape);
    return () => document.removeEventListener('keydown', closeOnEscape);
  }, [selectedEvidence, blurLevel]);

  const handleSaveAsAgentDefault = async (lvl: BlurLevel) => {
    setIsSavingCaptureBlur(true);
    try {
      const res = await api.setCaptureBlur(lvl);
      onShowToast?.(res.message, 'success');
    } catch {
      onShowToast?.(
        isVi ? 'Không thể lưu cài đặt làm mờ máy trạm' : 'Failed to save capture blur setting',
        'error'
      );
    } finally {
      setIsSavingCaptureBlur(false);
    }
  };

  return (
    <div className="flex flex-col gap-6">
      {/* Privacy Notice Banner & Toolbar */}
      <div className="bg-sky-50 border border-sky-200 rounded-2xl p-4 sm:p-5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 shadow-xs">
        <div className="flex items-center gap-3.5">
          <div className="w-11 h-11 rounded-xl bg-sky-100 flex items-center justify-center text-sky-600 shrink-0">
            <Shield size={24} />
          </div>
          <div>
            <h3 className="text-sm sm:text-base font-bold text-sky-900 mb-0.5">{t.privacy_title}</h3>
            <p className="text-xs sm:text-sm text-sky-700">{t.privacy_desc}</p>
          </div>
        </div>

        {/* Global Blur Level Segmented Control */}
        <div className="flex items-center gap-2 self-stretch sm:self-auto justify-end bg-white/80 border border-sky-200/80 p-1 rounded-xl">
          <span className="text-2xs font-bold text-sky-900 px-2 flex items-center gap-1">
            <SlidersHorizontal size={12} />
            <span>{t.blur_selector_label}</span>
          </span>

          <div className="inline-flex gap-0.5">
            <button
              onClick={() => handleBlurChange('none')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                blurLevel === 'none'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              👁️ {t.blur_mode_none}
            </button>
            <button
              onClick={() => handleBlurChange('reduced')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                blurLevel === 'reduced'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              👓 {t.blur_mode_reduced}
            </button>
            <button
              onClick={() => handleBlurChange('full')}
              className={`px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-all ${
                blurLevel === 'full'
                  ? 'bg-sky-600 text-white shadow-2xs'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              🔒 {t.blur_mode_full}
            </button>
          </div>
        </div>
      </div>

      {/* Grid of Evidence */}
      {evidenceList.length === 0 ? (
        <div className="bg-white border border-slate-200 rounded-2xl p-12 text-center text-slate-500 shadow-xs">
          <AlertTriangle size={36} className="text-slate-400 mx-auto mb-3" />
          <div className="text-sm font-semibold">{t.no_evidence}</div>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
          {evidenceList.map((item) => {
            const timeStr = new Date(item.occurred_at).toLocaleString();
            const expHours = Math.max(
              0,
              Math.round((new Date(item.expires_at).getTime() - Date.now()) / (1000 * 60 * 60))
            );

            return (
              <div
                key={item.evidence_id}
                className="bg-white border border-slate-200 rounded-2xl overflow-hidden flex flex-col hover:border-sky-500 hover:shadow-md transition-all"
              >
                {/* Thumbnail */}
                <button
                  type="button"
                  className="relative w-full h-44 bg-slate-100 overflow-hidden flex items-center justify-center cursor-pointer group"
                  onClick={() => setSelectedEvidence(item)}
                  aria-label={
                    lang === 'vi'
                      ? `Xem bằng chứng: ${item.trigger}`
                      : `View evidence: ${item.trigger}`
                  }
                >
                  <img
                    src={item.thumbnail_url}
                    alt="Violation Capture"
                    className={`w-full h-full object-cover transition-all duration-200 ${getBlurClass(
                      blurLevel
                    )}`}
                  />
                  <div className="absolute top-2.5 right-2.5 bg-slate-900/80 backdrop-blur-xs text-sky-300 px-2 py-0.5 rounded-md text-2xs font-bold border border-sky-300/30">
                    📸 480p
                  </div>
                  {blurLevel !== 'none' && (
                    <div className="absolute bottom-2.5 left-2.5 bg-slate-900/80 backdrop-blur-xs text-amber-300 px-2 py-0.5 rounded-md text-3xs font-semibold border border-amber-300/30">
                      {blurLevel === 'reduced' ? '👓 Giảm mờ' : '🔒 Mờ bảo vệ'}
                    </div>
                  )}
                </button>

                {/* Details */}
                <div className="p-4 flex flex-col flex-1 justify-between gap-3">
                  <div>
                    <div className="text-sm font-bold text-slate-800 mb-1">⚠️ {item.trigger}</div>
                    <div className="text-xs text-slate-500">
                      {timeStr} • {(item.size_bytes / 1024).toFixed(1)} KB
                    </div>
                  </div>

                  <div className="flex items-center justify-between pt-2.5 border-t border-slate-100">
                    <div className="flex items-center gap-1 text-xs text-sky-600 font-semibold">
                      <Clock size={13} />
                      <span>
                        {t.evidence_purge_countdown.replace('{hours}', expHours.toString())}
                      </span>
                    </div>

                    <button
                      onClick={() => setSelectedEvidence(item)}
                      className="text-slate-500 hover:text-sky-600 cursor-pointer flex items-center gap-1 text-xs font-semibold transition-colors"
                    >
                      <Eye size={13} />
                      <span>{t.btn_view_detail}</span>
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Enlarged Evidence Modal with Interactive 3-Level Blur Controls */}
      {selectedEvidence && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-150">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-modal-title"
            className="bg-white border border-slate-200 rounded-2xl max-w-2xl w-full overflow-hidden shadow-2xl"
          >
            <div className="p-4 sm:p-5 border-b border-slate-200 flex justify-between items-center bg-white">
              <div>
                <h3 id="evidence-modal-title" className="text-base font-bold text-slate-900">
                  {t.evidence_modal_title}
                </h3>
                <div className="text-xs text-slate-500">
                  {lang === 'vi' ? 'Sự kiện' : 'Event'}: {selectedEvidence.trigger}
                </div>
              </div>
              <button
                ref={closeButtonRef}
                type="button"
                onClick={() => setSelectedEvidence(null)}
                aria-label={lang === 'vi' ? 'Đóng' : 'Close'}
                className="bg-slate-100 hover:bg-slate-200 text-slate-600 p-1.5 rounded-lg cursor-pointer transition-colors"
              >
                <X size={18} />
              </button>
            </div>

            <div className="p-5 flex flex-col gap-4">
              {/* Image Preview Box */}
              <div className="rounded-xl overflow-hidden bg-slate-900/90 border border-slate-200 flex justify-center items-center min-h-[280px]">
                <img
                  src={selectedEvidence.thumbnail_url}
                  alt="Enlarged Visual Evidence"
                  className={`max-w-full max-h-[480px] object-contain transition-all duration-200 ${getBlurClass(
                    modalBlurLevel
                  )}`}
                />
              </div>

              {/* Blur Level Switcher & Save Setting */}
              <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 pt-1 border-t border-slate-100">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-semibold text-slate-700">
                    {t.blur_selector_label}
                  </span>
                  <div className="inline-flex bg-slate-100 p-0.5 rounded-lg border border-slate-200">
                    <button
                      type="button"
                      onClick={() => setModalBlurLevel('none')}
                      className={`px-2.5 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                        modalBlurLevel === 'none'
                          ? 'bg-white text-slate-900 shadow-2xs'
                          : 'text-slate-600 hover:text-slate-900'
                      }`}
                    >
                      👁️ {t.blur_mode_none}
                    </button>
                    <button
                      type="button"
                      onClick={() => setModalBlurLevel('reduced')}
                      className={`px-2.5 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                        modalBlurLevel === 'reduced'
                          ? 'bg-white text-slate-900 shadow-2xs'
                          : 'text-slate-600 hover:text-slate-900'
                      }`}
                    >
                      👓 {t.blur_mode_reduced}
                    </button>
                    <button
                      type="button"
                      onClick={() => setModalBlurLevel('full')}
                      className={`px-2.5 py-1 rounded-md text-xs font-semibold cursor-pointer transition-all ${
                        modalBlurLevel === 'full'
                          ? 'bg-white text-slate-900 shadow-2xs'
                          : 'text-slate-600 hover:text-slate-900'
                      }`}
                    >
                      🔒 {t.blur_mode_full}
                    </button>
                  </div>
                </div>

                {/* Optional default save button */}
                <button
                  type="button"
                  onClick={() => handleSaveAsAgentDefault(modalBlurLevel)}
                  disabled={isSavingCaptureBlur}
                  className="text-2xs text-sky-700 hover:text-sky-800 bg-sky-50 hover:bg-sky-100 border border-sky-200 px-2.5 py-1 rounded-md font-semibold cursor-pointer transition-colors"
                >
                  {isSavingCaptureBlur
                    ? isVi
                      ? 'Đang lưu...'
                      : 'Saving...'
                    : isVi
                      ? 'Lưu làm mặc định chụp'
                      : 'Set as agent capture default'}
                </button>
              </div>

              <div className="text-2xs text-slate-400">
                <span>🔒 {t.evidence_modal_notice}</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
