import React, { useState } from 'react';
import { Shield, Clock, AlertTriangle, Eye, X } from 'lucide-react';
import { EvidenceItem, Language } from '../types';
import { translations } from '../i18n';

interface EvidenceTabProps {
  evidenceList: EvidenceItem[];
  lang: Language;
  onRefresh: () => void;
}

export const EvidenceTab: React.FC<EvidenceTabProps> = ({
  evidenceList,
  lang,
  onRefresh,
}) => {
  const t = translations[lang];
  const [selectedEvidence, setSelectedEvidence] = useState<EvidenceItem | null>(null);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Privacy Notice Banner */}
      <div style={{
        background: 'rgba(56, 189, 248, 0.08)',
        border: '1px solid rgba(56, 189, 248, 0.25)',
        borderRadius: 16,
        padding: '18px 24px',
        display: 'flex',
        alignItems: 'center',
        gap: 16,
      }}>
        <div style={{
          width: 44,
          height: 44,
          borderRadius: 12,
          background: 'rgba(56, 189, 248, 0.15)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#38bdf8',
          flexShrink: 0,
        }}>
          <Shield size={24} />
        </div>
        <div>
          <h3 style={{ fontSize: '1rem', fontWeight: 700, color: '#38bdf8', marginBottom: 2 }}>
            {t.privacy_title}
          </h3>
          <p style={{ fontSize: '0.84rem', color: '#94a3b8' }}>
            {t.privacy_desc}
          </p>
        </div>
      </div>

      {/* Grid of Evidence */}
      {evidenceList.length === 0 ? (
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: '60px 20px',
          textAlign: 'center',
          color: '#94a3b8',
        }}>
          <AlertTriangle size={36} color="#475569" style={{ margin: '0 auto 12px' }} />
          <div style={{ fontSize: '1rem', fontWeight: 600 }}>{t.no_evidence}</div>
        </div>
      ) : (
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
          gap: 20,
        }}>
          {evidenceList.map((item) => {
            const timeStr = new Date(item.occurred_at).toLocaleString();
            const expHours = Math.max(0, Math.round((new Date(item.expires_at).getTime() - Date.now()) / (1000 * 60 * 60)));

            return (
              <div
                key={item.evidence_id}
                style={{
                  background: '#0f172a',
                  border: '1px solid #1e293b',
                  borderRadius: 16,
                  overflow: 'hidden',
                  display: 'flex',
                  flexDirection: 'column',
                  transition: 'transform 0.15s, border-color 0.15s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-3px)';
                  e.currentTarget.style.borderColor = '#38bdf8';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'none';
                  e.currentTarget.style.borderColor = '#1e293b';
                }}
              >
                {/* Thumbnail */}
                <div style={{
                  position: 'relative',
                  width: '100%',
                  height: 170,
                  background: '#070b14',
                  overflow: 'hidden',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  cursor: 'pointer',
                }}
                  onClick={() => setSelectedEvidence(item)}
                >
                  <img
                    src={item.thumbnail_url}
                    alt="Violation Capture"
                    style={{
                      width: '100%',
                      height: '100%',
                      objectFit: 'cover',
                      filter: 'blur(3px)',
                      transition: 'filter 0.3s',
                    }}
                  />
                  <div style={{
                    position: 'absolute',
                    top: 10,
                    right: 10,
                    background: 'rgba(7, 11, 20, 0.85)',
                    backdropFilter: 'blur(4px)',
                    color: '#fbbf24',
                    padding: '3px 8px',
                    borderRadius: 6,
                    fontSize: '0.72rem',
                    fontWeight: 700,
                    border: '1px solid rgba(251, 191, 36, 0.3)',
                  }}>
                    🔒 Blurred 480p
                  </div>
                </div>

                {/* Details */}
                <div style={{ padding: 16, display: 'flex', flexDirection: 'column', flex: 1, justifyContent: 'space-between' }}>
                  <div>
                    <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#f8fafc', marginBottom: 4 }}>
                      ⚠️ {item.trigger}
                    </div>
                    <div style={{ fontSize: '0.78rem', color: '#94a3b8', marginBottom: 12 }}>
                      {timeStr} • {(item.size_bytes / 1024).toFixed(1)} KB
                    </div>
                  </div>

                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    paddingTop: 10,
                    borderTop: '1px solid #1e293b',
                  }}>
                    <div style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                      fontSize: '0.78rem',
                      color: '#38bdf8',
                      fontWeight: 600,
                    }}>
                      <Clock size={13} />
                      <span>{t.evidence_purge_countdown.replace('{hours}', expHours.toString())}</span>
                    </div>

                    <button
                      onClick={() => setSelectedEvidence(item)}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: '#94a3b8',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        gap: 4,
                        fontSize: '0.78rem',
                        fontWeight: 600,
                      }}
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
        <div style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(7, 11, 20, 0.85)',
          backdropFilter: 'blur(8px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 100,
          padding: 20,
        }}>
          <div style={{
            background: '#0f172a',
            border: '1px solid #1e293b',
            borderRadius: 16,
            maxWidth: 720,
            width: '100%',
            overflow: 'hidden',
            boxShadow: '0 25px 60px rgba(0,0,0,0.6)',
          }}>
            <div style={{
              padding: '16px 20px',
              borderBottom: '1px solid #1e293b',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}>
              <div>
                <h3 style={{ fontSize: '1.1rem', fontWeight: 700, color: '#f8fafc' }}>
                  {t.evidence_modal_title}
                </h3>
                <div style={{ fontSize: '0.78rem', color: '#94a3b8' }}>
                  Sự kiện: {selectedEvidence.trigger}
                </div>
              </div>
              <button
                onClick={() => setSelectedEvidence(null)}
                style={{
                  background: '#1e293b',
                  border: 'none',
                  color: '#94a3b8',
                  padding: 6,
                  borderRadius: 8,
                  cursor: 'pointer',
                }}
              >
                <X size={18} />
              </button>
            </div>

            <div style={{ padding: 20 }}>
              <div style={{
                borderRadius: 10,
                overflow: 'hidden',
                background: '#070b14',
                border: '1px solid #1e293b',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
              }}>
                <img
                  src={selectedEvidence.thumbnail_url}
                  alt="Enlarged Visual Evidence"
                  style={{ maxWidth: '100%', maxHeight: 420, objectFit: 'contain', filter: 'blur(3px)' }}
                />
              </div>

              <div style={{ marginTop: 14, fontSize: '0.8rem', color: '#94a3b8', textAlign: 'center' }}>
                🔒 {t.evidence_modal_notice}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
