import type React from 'react';
import { useState } from 'react';
import { Sparkles, ExternalLink, Copy, Check, Mail } from 'lucide-react';
import type { Language } from '../../types';

interface AuthorOpenSourceCardProps {
  lang: Language;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const AuthorOpenSourceCard: React.FC<AuthorOpenSourceCardProps> = ({
  lang,
  onShowToast,
}) => {
  const isVi = lang === 'vi';
  const [copiedRepoClone, setCopiedRepoClone] = useState(false);
  const [copiedAuthorEmail, setCopiedAuthorEmail] = useState(false);

  const agentRepo = 'https://github.com/nntu/famshield-agent';
  const repoCloneCmd = 'git clone https://github.com/nntu/famshield-agent.git';
  const authorEmail = 'ngoctuct@gmail.com';

  return (
    <div
      id="section-author-opensource"
      style={{
        background: 'linear-gradient(135deg, rgba(15, 23, 42, 0.95) 0%, rgba(8, 47, 73, 0.4) 100%)',
        border: '1px solid rgba(56, 189, 248, 0.25)',
        borderRadius: 14,
        padding: 24,
        marginBottom: 32,
        boxShadow: '0 10px 25px -5px rgba(0, 0, 0, 0.2)',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 12,
          marginBottom: 16,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div
            style={{
              width: 42,
              height: 42,
              borderRadius: 10,
              background: 'linear-gradient(135deg, #0284c7, #2563eb)',
              color: '#fff',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontWeight: 900,
              fontSize: '1.1rem',
              boxShadow: '0 4px 12px rgba(2, 132, 199, 0.3)',
            }}
          >
            NT
          </div>
          <div>
            <h3
              style={{
                fontSize: '1.1rem',
                fontWeight: 800,
                color: '#f8fafc',
                margin: 0,
                display: 'flex',
                alignItems: 'center',
                gap: 8,
              }}
            >
              <Sparkles size={18} color="#38bdf8" />
              {isVi ? 'Tác Giả & Mã Nguồn Mở' : 'Author & Open Source'}
            </h3>
            <p style={{ fontSize: '0.82rem', color: '#94a3b8', margin: 0, marginTop: 2 }}>
              {isVi
                ? 'FamShield Parental Control & Endpoint Protection • Tác giả: Nguyễn Ngọc Tú'
                : 'FamShield Parental Control & Endpoint Protection • Created by Nguyen Ngoc Tu'}
            </p>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span
            style={{
              background: 'rgba(52, 211, 153, 0.1)',
              color: '#34d399',
              border: '1px solid rgba(52, 211, 153, 0.3)',
              padding: '4px 10px',
              borderRadius: 9999,
              fontSize: '0.75rem',
              fontWeight: 700,
              fontFamily: 'monospace',
            }}
          >
            MIT LICENSE
          </span>
          <span
            style={{
              background: 'rgba(56, 189, 248, 0.1)',
              color: '#38bdf8',
              border: '1px solid rgba(56, 189, 248, 0.3)',
              padding: '4px 10px',
              borderRadius: 9999,
              fontSize: '0.75rem',
              fontWeight: 700,
              fontFamily: 'monospace',
            }}
          >
            v1.0.3
          </span>
        </div>
      </div>

      <p style={{ fontSize: '0.85rem', color: '#cbd5e1', lineHeight: 1.6, marginBottom: 16 }}>
        {isVi
          ? 'FamShield được kiến trúc và phát triển bởi Nguyễn Ngọc Tú với triết lý mã nguồn mở, tôn trọng tối đa quyền riêng tư (Zero-Trust, Local Blur 480p, tự hủy 48h) và tối ưu hóa tài nguyên phần cứng máy tính con bằng công nghệ Rust & Windows Filtering Platform (WFP).'
          : 'FamShield is architected and built by Nguyen Ngoc Tu under the permissive open-source MIT License, prioritizing child privacy (Zero-Trust, localized 480p blur, 48h retention) and lightweight Rust performance.'}
      </p>

      {/* Repository & Clone Info */}
      <div
        style={{
          background: '#070b14',
          border: '1px solid #1e293b',
          borderRadius: 10,
          padding: 14,
          marginBottom: 16,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 10,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ fontSize: '0.8rem', color: '#64748b', fontWeight: 600 }}>GitHub:</span>
          <a
            href={agentRepo}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              color: '#38bdf8',
              textDecoration: 'none',
              fontFamily: 'monospace',
              fontSize: '0.85rem',
              fontWeight: 600,
              display: 'flex',
              alignItems: 'center',
              gap: 5,
            }}
          >
            <span>{agentRepo}</span>
            <ExternalLink size={13} />
          </a>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: '0.75rem', color: '#94a3b8', fontFamily: 'monospace' }}>
            {repoCloneCmd}
          </span>
          <button
            type="button"
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(repoCloneCmd);
                setCopiedRepoClone(true);
                setTimeout(() => setCopiedRepoClone(false), 2000);
                onShowToast(isVi ? 'Đã chép lệnh git clone!' : 'Git clone command copied!', 'success');
              } catch {
                // fallback
              }
            }}
            style={{
              background: '#1e293b',
              color: '#e2e8f0',
              border: '1px solid #334155',
              padding: '4px 10px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 5,
            }}
          >
            {copiedRepoClone ? <Check size={12} color="#34d399" /> : <Copy size={12} />}
            <span>{copiedRepoClone ? (isVi ? 'Đã chép' : 'Copied') : isVi ? 'Chép lệnh' : 'Copy'}</span>
          </button>
        </div>
      </div>

      {/* Contacts row */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap', fontSize: '0.8rem', color: '#94a3b8' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <Mail size={14} color="#38bdf8" />
          <span>Email: </span>
          <span style={{ color: '#f8fafc', fontFamily: 'monospace' }}>{authorEmail}</span>
          <button
            type="button"
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(authorEmail);
                setCopiedAuthorEmail(true);
                setTimeout(() => setCopiedAuthorEmail(false), 2000);
                onShowToast(isVi ? 'Đã chép email tác giả!' : 'Author email copied!', 'success');
              } catch {
                // fallback
              }
            }}
            style={{
              background: 'transparent',
              border: 'none',
              color: '#38bdf8',
              cursor: 'pointer',
              padding: '2px 4px',
            }}
            title={isVi ? 'Chép email' : 'Copy email'}
          >
            {copiedAuthorEmail ? <Check size={12} color="#34d399" /> : <Copy size={12} />}
          </button>
        </div>

        <span>•</span>

        <a
          href="https://github.com/nntu/"
          target="_blank"
          rel="noopener noreferrer"
          style={{ color: '#38bdf8', textDecoration: 'none', display: 'flex', alignItems: 'center', gap: 4 }}
        >
          <span>GitHub Profile</span>
          <ExternalLink size={12} />
        </a>

        <span>•</span>

        <a
          href="https://www.facebook.com/ngoctuct"
          target="_blank"
          rel="noopener noreferrer"
          style={{ color: '#38bdf8', textDecoration: 'none', display: 'flex', alignItems: 'center', gap: 4 }}
        >
          <span>Facebook Profile</span>
          <ExternalLink size={12} />
        </a>
      </div>
    </div>
  );
};
