import { AlertTriangle, Clock, Eye, Shield, X } from 'lucide-react';
import type React from 'react';
import { useEffect, useRef, useState } from 'react';
import { translations } from '../i18n';
import type { EvidenceItem, Language } from '../types';

interface EvidenceTabProps {
  evidenceList: EvidenceItem[];
  lang: Language;
  onRefresh: () => void;
}

export const EvidenceTab: React.FC<EvidenceTabProps> = ({ evidenceList, lang }) => {
  const t = translations[lang];
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceItem | null>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!selectedEvidence) return;
    closeButtonRef.current?.focus();
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setSelectedEvidence(null);
    };
    document.addEventListener('keydown', closeOnEscape);
    return () => document.removeEventListener('keydown', closeOnEscape);
  }, [selectedEvidence]);

  return (
    <div className="flex flex-col gap-6">
      {/* Privacy Notice Banner */}
      <div className="bg-sky-50 border border-sky-200 rounded-2xl p-4 sm:p-5 flex items-center gap-4 shadow-xs">
        <div className="w-11 h-11 rounded-xl bg-sky-100 flex items-center justify-center text-sky-600 shrink-0">
          <Shield size={24} />
        </div>
        <div>
          <h3 className="text-sm sm:text-base font-bold text-sky-900 mb-0.5">{t.privacy_title}</h3>
          <p className="text-xs sm:text-sm text-sky-700">{t.privacy_desc}</p>
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
                    className="w-full h-full object-cover blur-[3px] group-hover:blur-[2px] transition-all"
                  />
                  <div className="absolute top-2.5 right-2.5 bg-slate-900/80 backdrop-blur-xs text-amber-300 px-2 py-0.5 rounded-md text-2xs font-bold border border-amber-300/30">
                    🔒 Blurred 480p
                  </div>
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

      {/* Enlarged Evidence Modal */}
      {selectedEvidence && (
        <div className="fixed inset-0 bg-slate-900/50 backdrop-blur-xs flex items-center justify-center z-50 p-4">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-modal-title"
            className="bg-white border border-slate-200 rounded-2xl max-w-2xl w-full overflow-hidden shadow-2xl"
          >
            <div className="p-4 sm:p-5 border-b border-slate-200 flex justify-between items-center">
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

            <div className="p-5">
              <div className="rounded-xl overflow-hidden bg-slate-50 border border-slate-200 flex justify-center items-center">
                <img
                  src={selectedEvidence.thumbnail_url}
                  alt="Enlarged Visual Evidence"
                  className="max-w-full max-h-[420px] object-contain blur-[3px]"
                />
              </div>

              <div className="mt-3.5 text-xs text-slate-500 text-center">
                🔒 {t.evidence_modal_notice}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
