import {
  BookOpen,
  Check,
  Code2,
  Copy,
  Cpu,
  ExternalLink,
  Globe,
  Heart,
  Mail,
  Scale,
  Shield,
  ShieldCheck,
  Sparkles,
  Terminal,
  X,
} from 'lucide-react';
import type React from 'react';
import { useState } from 'react';
import type { Language } from '../types';

interface AuthorModalProps {
  isOpen: boolean;
  onClose: () => void;
  lang: Language;
  appVersion?: string;
}

export type AuthorTab = 'author' | 'license' | 'architecture' | 'contact';

export const AuthorModal: React.FC<AuthorModalProps> = ({
  isOpen,
  onClose,
  lang,
  appVersion = '1.0.3',
}) => {
  const [activeTab, setActiveTab] = useState<AuthorTab>('author');
  const [copiedEmail, setCopiedEmail] = useState(false);
  const [copiedClone, setCopiedClone] = useState(false);

  const isVi = lang === 'vi';
  const authorEmail = 'ngoctuct@gmail.com';
  const authorGithub = 'https://github.com/nntu/';
  const authorFacebook = 'https://www.facebook.com/ngoctuct';
  const agentRepo = 'https://github.com/nntu/famshield-agent';
  const cloneCmd = 'git clone https://github.com/nntu/famshield-agent.git';

  if (!isOpen) return null;

  const handleCopyEmail = async () => {
    try {
      await navigator.clipboard.writeText(authorEmail);
      setCopiedEmail(true);
      setTimeout(() => setCopiedEmail(false), 2000);
    } catch {
      // fallback
    }
  };

  const handleCopyClone = async () => {
    try {
      await navigator.clipboard.writeText(cloneCmd);
      setCopiedClone(true);
      setTimeout(() => setCopiedClone(false), 2000);
    } catch {
      // fallback
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="author-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/70 backdrop-blur-xs animate-in fade-in duration-200"
    >
      <div className="relative w-full max-w-3xl max-h-[90vh] bg-white rounded-2xl shadow-2xl border border-slate-200 overflow-hidden flex flex-col">
        {/* Header with gradient banner */}
        <div className="relative bg-gradient-to-r from-slate-900 via-sky-950 to-blue-950 px-6 py-5 text-white">
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-3.5">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-sky-400 to-blue-600 flex items-center justify-center text-white font-black text-xl shadow-md ring-2 ring-white/20 shrink-0">
                NT
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <h2 id="author-modal-title" className="text-lg font-bold text-white tracking-tight">
                    Nguyễn Ngọc Tú
                  </h2>
                  <span className="inline-flex items-center gap-1 text-[10px] font-semibold font-mono bg-sky-500/20 text-sky-200 border border-sky-400/30 px-2 py-0.5 rounded-full">
                    <Sparkles className="w-3 h-3 text-sky-300" />
                    {isVi ? 'Tác Giả & Sáng Lập' : 'Author & Creator'}
                  </span>
                </div>
                <p className="text-xs text-sky-200/80 mt-0.5">
                  {isVi
                    ? 'Kỹ sư Phần mềm & Kiến trúc sư Hệ thống An ninh mạng'
                    : 'Software Engineer & Cybersecurity Systems Architect'}
                </p>
              </div>
            </div>

            <button
              type="button"
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-white/10 transition cursor-pointer"
              aria-label={isVi ? 'Đóng cửa sổ' : 'Close modal'}
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Sub Navigation Tabs */}
          <div className="flex items-center gap-2 mt-5 border-t border-white/10 pt-3 overflow-x-auto text-xs font-semibold">
            <button
              type="button"
              onClick={() => setActiveTab('author')}
              className={`px-3 py-1.5 rounded-lg transition cursor-pointer flex items-center gap-1.5 whitespace-nowrap ${
                activeTab === 'author'
                  ? 'bg-sky-500 text-white shadow-xs'
                  : 'text-slate-300 hover:text-white hover:bg-white/10'
              }`}
            >
              <ShieldCheck className="w-3.5 h-3.5" />
              <span>{isVi ? 'Về Tác Giả & Sứ Mệnh' : 'Author & Mission'}</span>
            </button>

            <button
              type="button"
              onClick={() => setActiveTab('license')}
              className={`px-3 py-1.5 rounded-lg transition cursor-pointer flex items-center gap-1.5 whitespace-nowrap ${
                activeTab === 'license'
                  ? 'bg-sky-500 text-white shadow-xs'
                  : 'text-slate-300 hover:text-white hover:bg-white/10'
              }`}
            >
              <Scale className="w-3.5 h-3.5" />
              <span>{isVi ? 'Mã Nguồn Mở & Giấy Phép MIT' : 'Open Source & MIT'}</span>
            </button>

            <button
              type="button"
              onClick={() => setActiveTab('architecture')}
              className={`px-3 py-1.5 rounded-lg transition cursor-pointer flex items-center gap-1.5 whitespace-nowrap ${
                activeTab === 'architecture'
                  ? 'bg-sky-500 text-white shadow-xs'
                  : 'text-slate-300 hover:text-white hover:bg-white/10'
              }`}
            >
              <Cpu className="w-3.5 h-3.5" />
              <span>{isVi ? 'Kiến Trúc & Bảo Vệ Riêng Tư' : 'Privacy Architecture'}</span>
            </button>

            <button
              type="button"
              onClick={() => setActiveTab('contact')}
              className={`px-3 py-1.5 rounded-lg transition cursor-pointer flex items-center gap-1.5 whitespace-nowrap ${
                activeTab === 'contact'
                  ? 'bg-sky-500 text-white shadow-xs'
                  : 'text-slate-300 hover:text-white hover:bg-white/10'
              }`}
            >
              <Mail className="w-3.5 h-3.5" />
              <span>{isVi ? 'Liên Hệ & Đóng Góp' : 'Contact & Support'}</span>
            </button>
          </div>
        </div>

        {/* Modal Body */}
        <div className="p-6 overflow-y-auto space-y-5 text-slate-700 text-xs sm:text-sm leading-relaxed flex-1">
          {/* TAB 1: Author & Mission */}
          {activeTab === 'author' && (
            <div className="space-y-4">
              <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                <h3 className="font-bold text-slate-900 text-sm flex items-center gap-2">
                  <Heart className="w-4 h-4 text-rose-500" />
                  <span>{isVi ? 'Sứ Mệnh Bảo Vệ Trẻ Em Trên Không Gian Mạng' : 'Mission for Child Online Safety'}</span>
                </h3>
                <p className="text-slate-600 text-xs leading-relaxed">
                  {isVi
                    ? 'FamShield được sáng lập và phát triển bởi Nguyễn Ngọc Tú với tâm huyết xây dựng một giải pháp an toàn số "Make in Vietnam", mang lại môi trường mạng trong sạch, lành mạnh cho các thế hệ trẻ em Việt Nam trong thời đại số hóa mạnh mẽ.'
                    : 'FamShield was founded and architected by Nguyen Ngoc Tu to build an open, trustworthy parental control and cybersecurity platform, empowering parents to protect their children online.'}
                </p>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                <div className="p-3.5 rounded-xl border border-slate-200 bg-white space-y-1.5">
                  <div className="font-bold text-xs text-slate-900 flex items-center gap-1.5">
                    <Globe className="w-3.5 h-3.5 text-sky-600" />
                    <span>{isVi ? 'Tự Chủ Công Nghệ Việt Nam' : 'Sovereign Technology'}</span>
                  </div>
                  <p className="text-xs text-slate-500">
                    {isVi
                      ? 'Làm chủ 100% công nghệ lõi từ driver lọc mạng Windows (WFP), DNS Sinkhole đến hạ tầng phân tán máy chủ, không phụ thuộc dịch vụ nước ngoài.'
                      : 'Full ownership of core technologies: WFP network filters, loopback DNS sinkholes, and resilient distributed agents.'}
                  </p>
                </div>

                <div className="p-3.5 rounded-xl border border-slate-200 bg-white space-y-1.5">
                  <div className="font-bold text-xs text-slate-900 flex items-center gap-1.5">
                    <Shield className="w-3.5 h-3.5 text-emerald-600" />
                    <span>{isVi ? 'Tôn Trọng Quyền Riêng Tư' : 'Privacy-First Design'}</span>
                  </div>
                  <p className="text-xs text-slate-500">
                    {isVi
                      ? 'Tuân thủ Nghị định 13/2023/NĐ-CP và tiêu chuẩn COPPA. Không ghi phím thô bạo (keylogger), tự làm mờ ảnh bằng chứng 480p và tự hủy sau 48 giờ.'
                      : 'Strictly complies with Decree 13/2023/ND-CP and COPPA. No invasive keylogging, localized 480p blurring, auto-expiring in 48 hours.'}
                  </p>
                </div>
              </div>

              {/* Quick Info Box */}
              <div className="p-4 rounded-xl bg-sky-50/70 border border-sky-200 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs">
                <div>
                  <div className="font-bold text-slate-900">
                    {isVi ? 'Kho mã nguồn chính thức của dự án:' : 'Official Project Repository:'}
                  </div>
                  <a
                    href={agentRepo}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sky-700 hover:text-sky-900 font-mono underline inline-flex items-center gap-1 mt-0.5"
                  >
                    <span>{agentRepo}</span>
                    <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
                <div className="flex items-center gap-2">
                  <span className="bg-white px-2.5 py-1 rounded-md border border-sky-300 font-mono font-bold text-sky-800 text-xs">
                    MIT License
                  </span>
                  <span className="bg-sky-600 text-white px-2.5 py-1 rounded-md font-mono font-bold text-xs">
                    v{appVersion}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: License & Source Code */}
          {activeTab === 'license' && (
            <div className="space-y-4">
              <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                <div className="flex items-center justify-between">
                  <h3 className="font-bold text-slate-900 text-sm flex items-center gap-2">
                    <Scale className="w-4 h-4 text-emerald-600" />
                    <span>Giấy Phép Mã Nguồn Mở MIT (MIT License)</span>
                  </h3>
                  <span className="text-[10px] font-mono bg-emerald-100 text-emerald-800 px-2 py-0.5 rounded-full font-bold">
                    Open Source
                  </span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  {isVi
                    ? 'FamShield Agent là phần mềm mã nguồn mở được cấp phép theo các điều khoản của Giấy phép MIT. Mọi phụ huynh, nhà nghiên cứu và lập trình viên đều có quyền tự do kiểm tra mã nguồn, đóng góp tính năng hoặc triển khai cho gia đình mà không mất bất kỳ chi phí bản quyền nào.'
                    : 'FamShield Agent is open-source software distributed under the permissive MIT License. Anyone is free to audit, study, modify, and deploy the software.'}
                </p>
              </div>

              {/* Git Clone Box */}
              <div className="space-y-1.5">
                <label className="text-xs font-bold text-slate-700">
                  {isVi ? 'Lệnh sao chép mã nguồn qua Git:' : 'Clone repository via Git:'}
                </label>
                <div className="flex items-center justify-between p-3 rounded-xl bg-slate-900 text-sky-300 font-mono text-xs">
                  <span className="truncate mr-2">{cloneCmd}</span>
                  <button
                    type="button"
                    onClick={handleCopyClone}
                    className="inline-flex items-center gap-1 bg-slate-800 hover:bg-slate-700 text-white px-2.5 py-1 rounded-lg text-xs transition cursor-pointer shrink-0"
                  >
                    {copiedClone ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    <span>{copiedClone ? (isVi ? 'Đã chép' : 'Copied') : isVi ? 'Sao chép' : 'Copy'}</span>
                  </button>
                </div>
              </div>

              {/* License Notice Snippet */}
              <div className="p-4 rounded-xl bg-slate-100/80 border border-slate-200 font-mono text-[11px] text-slate-600 space-y-2 leading-normal">
                <div className="font-bold text-slate-800">
                  Copyright (c) 2026 Nguyen Ngoc Tu (ngoctuct@gmail.com)
                </div>
                <div>
                  Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction...
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: Architecture & Privacy */}
          {activeTab === 'architecture' && (
            <div className="space-y-4">
              <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                <h3 className="font-bold text-slate-900 text-sm flex items-center gap-2">
                  <Cpu className="w-4 h-4 text-sky-600" />
                  <span>{isVi ? 'Kiến Trúc Kỹ Thuật Agent Cục Bộ' : 'Local Agent Architecture'}</span>
                </h3>
                <p className="text-xs text-slate-600">
                  {isVi
                    ? 'FamShield Windows Agent được lập trình bằng ngôn ngữ Rust tối tân, vận hành như một Windows Service (famshield-service) độc lập tại Session 0, đem lại độ ổn định tuyệt đối và khả năng bảo vệ liên tục.'
                    : 'FamShield Windows Agent is engineered in Rust as a hardened Windows Service running in Session 0, delivering zero-latency protection.'}
                </p>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                <div className="p-3 rounded-xl border border-slate-200 bg-white space-y-1">
                  <div className="font-bold text-slate-900 flex items-center gap-1.5">
                    <Code2 className="w-3.5 h-3.5 text-indigo-600" />
                    <span>Windows Filtering Platform (WFP)</span>
                  </div>
                  <p className="text-slate-500 text-[11px]">
                    {isVi
                      ? 'Lọc gói tin cổng 53 và ngăn chặn rò rỉ DNS bằng driver WFP chuẩn của hệ điều hành Windows.'
                      : 'Filters Port 53 UDP/TCP traffic via native WFP subsystem to eliminate DNS leak vulnerabilities.'}
                  </p>
                </div>

                <div className="p-3 rounded-xl border border-slate-200 bg-white space-y-1">
                  <div className="font-bold text-slate-900 flex items-center gap-1.5">
                    <Terminal className="w-3.5 h-3.5 text-emerald-600" />
                    <span>DNS Loopback Sinkhole (0ms)</span>
                  </div>
                  <p className="text-slate-500 text-[11px]">
                    {isVi
                      ? 'Lắng nghe tại 127.0.0.2:53, chặn đứng tức thì các tên miền độc hại trong danh mục cấm.'
                      : 'Listens at 127.0.0.2:53 loopback, instantly sinking blacklisted domains in 0ms without external roundtrips.'}
                  </p>
                </div>
              </div>

              <div className="p-3.5 rounded-xl bg-amber-50 border border-amber-200 text-xs text-amber-900">
                <span className="font-bold">Quyền Riêng Tư (Privacy-by-Design):</span> Agent chỉ phân tích tên miền phân giải DNS và thời gian chạy ứng dụng tiền cảnh để thực thi hạn ngạch. Mọi bằng chứng vi phạm hình ảnh được làm mờ 480p tại chỗ và tự động xóa sau 48 giờ.
              </div>
            </div>
          )}

          {/* TAB 4: Contact & Social */}
          {activeTab === 'contact' && (
            <div className="space-y-4">
              <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                <h3 className="font-bold text-slate-900 text-sm flex items-center gap-2">
                  <Mail className="w-4 h-4 text-sky-600" />
                  <span>{isVi ? 'Kết Nối Trực Tiếp Với Tác Giả' : 'Connect Directly With Author'}</span>
                </h3>
                <p className="text-xs text-slate-600">
                  {isVi
                    ? 'Mọi phản hồi, góp ý báo lỗi kỹ thuật hoặc đề xuất hợp tác bảo vệ an toàn cho trường học và gia đình đều được tác giả trân trọng tiếp nhận:'
                    : 'Any feedback, bug reports, or safety collaboration inquiries are welcomed directly by the creator:'}
                </p>
              </div>

              <div className="space-y-2.5">
                {/* Email row */}
                <div className="flex items-center justify-between p-3.5 rounded-xl border border-slate-200 bg-white">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-sky-50 text-sky-600 flex items-center justify-center font-bold">
                      @
                    </div>
                    <div>
                      <div className="text-xs text-slate-500">Email Phụ Trách</div>
                      <div className="font-semibold text-slate-900 text-xs font-mono">{authorEmail}</div>
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={handleCopyEmail}
                    className="inline-flex items-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-medium transition cursor-pointer"
                  >
                    {copiedEmail ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
                    <span>{copiedEmail ? (isVi ? 'Đã chép' : 'Copied') : isVi ? 'Sao chép' : 'Copy'}</span>
                  </button>
                </div>

                {/* GitHub row */}
                <div className="flex items-center justify-between p-3.5 rounded-xl border border-slate-200 bg-white">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-slate-100 text-slate-800 flex items-center justify-center font-bold">
                      GH
                    </div>
                    <div>
                      <div className="text-xs text-slate-500">GitHub Profile</div>
                      <div className="font-semibold text-slate-900 text-xs font-mono">{authorGithub}</div>
                    </div>
                  </div>
                  <a
                    href={authorGithub}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1.5 bg-slate-900 hover:bg-slate-800 text-white px-3 py-1.5 rounded-lg text-xs font-medium transition cursor-pointer"
                  >
                    <span>Mở GitHub</span>
                    <ExternalLink className="w-3.5 h-3.5" />
                  </a>
                </div>

                {/* Facebook row */}
                <div className="flex items-center justify-between p-3.5 rounded-xl border border-slate-200 bg-white">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-blue-50 text-blue-600 flex items-center justify-center font-bold">
                      FB
                    </div>
                    <div>
                      <div className="text-xs text-slate-500">Facebook Cá Nhân</div>
                      <div className="font-semibold text-slate-900 text-xs font-mono">{authorFacebook}</div>
                    </div>
                  </div>
                  <a
                    href={authorFacebook}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1.5 bg-blue-600 hover:bg-blue-700 text-white px-3 py-1.5 rounded-lg text-xs font-medium transition cursor-pointer"
                  >
                    <span>Mở Facebook</span>
                    <ExternalLink className="w-3.5 h-3.5" />
                  </a>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Modal Footer */}
        <div className="px-6 py-4 border-t border-slate-100 bg-slate-50 flex flex-col sm:flex-row items-center justify-between gap-3 text-xs text-slate-500">
          <div className="flex items-center gap-2">
            <span>FamShield Agent</span>
            <span className="font-mono text-slate-700 bg-white px-2 py-0.5 rounded border border-slate-200 font-semibold">
              v{appVersion}
            </span>
            <span>•</span>
            <span className="font-medium text-emerald-700">MIT Open Source License</span>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="w-full sm:w-auto px-5 py-2 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-semibold text-xs transition cursor-pointer"
          >
            {isVi ? 'Đóng' : 'Close'}
          </button>
        </div>
      </div>
    </div>
  );
};
