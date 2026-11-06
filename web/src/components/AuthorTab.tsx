import {
  BookOpen,
  Check,
  Code2,
  Copy,
  Cpu,
  ExternalLink,
  GitBranch,
  Globe,
  Heart,
  Layers,
  Lock,
  Mail,
  Scale,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Terminal,
  Zap,
} from 'lucide-react';
import type React from 'react';
import { useState } from 'react';
import type { Language } from '../types';

interface AuthorTabProps {
  lang: Language;
  appVersion?: string;
  onShowToast?: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export type AuthorSubSection = 'mission' | 'architecture' | 'license' | 'contact';

export const AuthorTab: React.FC<AuthorTabProps> = ({
  lang,
  appVersion = '1.0.5',
  onShowToast,
}) => {
  const isVi = lang === 'vi';
  const [activeSection, setActiveSection] = useState<AuthorSubSection>('mission');
  const [copiedEmail, setCopiedEmail] = useState(false);
  const [copiedClone, setCopiedClone] = useState(false);

  const authorEmail = 'ngoctuct@gmail.com';
  const authorGithub = 'https://github.com/nntu/';
  const authorFacebook = 'https://www.facebook.com/ngoctuct';
  const agentRepo = 'https://github.com/nntu/famshield-agent';
  const serverRepo = 'https://github.com/nntu/famshield-server';
  const cloneCmd = 'git clone https://github.com/nntu/famshield-agent.git';

  const handleCopyEmail = async () => {
    try {
      await navigator.clipboard.writeText(authorEmail);
      setCopiedEmail(true);
      onShowToast?.(
        isVi ? 'Đã sao chép địa chỉ email tác giả!' : 'Author email copied to clipboard!',
        'success'
      );
      setTimeout(() => setCopiedEmail(false), 2000);
    } catch {
      // fallback
    }
  };

  const handleCopyClone = async () => {
    try {
      await navigator.clipboard.writeText(cloneCmd);
      setCopiedClone(true);
      onShowToast?.(
        isVi ? 'Đã sao chép lệnh git clone!' : 'Git clone command copied to clipboard!',
        'success'
      );
      setTimeout(() => setCopiedClone(false), 2000);
    } catch {
      // fallback
    }
  };

  return (
    <div className="space-y-6">
      {/* Hero Header Banner */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-slate-900 via-sky-950 to-blue-950 p-6 sm:p-8 text-white shadow-lg border border-sky-900/50">
        <div className="absolute top-0 right-0 w-96 h-96 bg-sky-500/10 rounded-full blur-3xl pointer-events-none -mr-20 -mt-20" />
        <div className="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
          <div className="flex items-center gap-5">
            <div className="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-gradient-to-br from-sky-400 via-blue-600 to-indigo-700 flex items-center justify-center text-white font-black text-2xl sm:text-3xl shadow-xl ring-4 ring-white/20 shrink-0">
              NT
            </div>
            <div>
              <div className="flex items-center gap-2.5 flex-wrap">
                <h1 className="text-xl sm:text-2xl font-black text-white tracking-tight">
                  Nguyễn Ngọc Tú
                </h1>
                <span className="inline-flex items-center gap-1.5 text-xs font-semibold bg-sky-500/20 text-sky-200 border border-sky-400/30 px-3 py-1 rounded-full shadow-xs">
                  <Sparkles className="w-3.5 h-3.5 text-sky-300" />
                  {isVi ? 'Tác Giả & Sáng Lập' : 'Author & Creator'}
                </span>
                <span className="inline-flex items-center gap-1 text-xs font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-400/30 px-2.5 py-1 rounded-full">
                  🇻🇳 Make in Vietnam
                </span>
              </div>
              <p className="text-sm text-sky-200/90 mt-1.5 font-medium">
                {isVi
                  ? 'Kỹ sư Phần mềm & Kiến trúc sư Hệ thống An ninh mạng'
                  : 'Software Engineer & Cybersecurity Systems Architect'}
              </p>
              <div className="flex items-center gap-3 mt-3 text-xs text-sky-300/80 flex-wrap">
                <span className="flex items-center gap-1.5">
                  <ShieldCheck className="w-4 h-4 text-emerald-400" />
                  FamShield Windows Agent
                </span>
                <span>•</span>
                <span className="font-mono text-white/90 bg-white/10 px-2 py-0.5 rounded">
                  v{appVersion}
                </span>
                <span>•</span>
                <span className="font-semibold text-emerald-300">Giấy phép MIT</span>
              </div>
            </div>
          </div>

          <div className="flex flex-row md:flex-col gap-2.5 shrink-0">
            <a
              href={authorGithub}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 bg-white/10 hover:bg-white/20 text-white border border-white/15 px-4 py-2 rounded-xl text-xs font-semibold transition cursor-pointer backdrop-blur-xs shadow-xs"
            >
              <span>GitHub Profile</span>
              <ExternalLink className="w-3.5 h-3.5 text-sky-300" />
            </a>
            <button
              type="button"
              onClick={handleCopyEmail}
              className="inline-flex items-center justify-center gap-2 bg-sky-500 hover:bg-sky-400 text-white px-4 py-2 rounded-xl text-xs font-semibold transition cursor-pointer shadow-xs"
            >
              {copiedEmail ? <Check className="w-3.5 h-3.5" /> : <Mail className="w-3.5 h-3.5" />}
              <span>{copiedEmail ? (isVi ? 'Đã sao chép' : 'Copied') : isVi ? 'Email tác giả' : 'Author Email'}</span>
            </button>
          </div>
        </div>

        {/* Section Navigation Tabs */}
        <div className="flex items-center gap-2 mt-6 pt-4 border-t border-white/15 overflow-x-auto text-xs sm:text-sm font-semibold">
          <button
            type="button"
            onClick={() => setActiveSection('mission')}
            className={`px-4 py-2 rounded-xl transition cursor-pointer flex items-center gap-2 whitespace-nowrap ${
              activeSection === 'mission'
                ? 'bg-sky-500 text-white shadow-md'
                : 'text-slate-300 hover:text-white hover:bg-white/10'
            }`}
          >
            <Heart className="w-4 h-4 text-rose-400" />
            <span>{isVi ? '1. Về Tác Giả & Sứ Mệnh' : '1. Author & Mission'}</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSection('architecture')}
            className={`px-4 py-2 rounded-xl transition cursor-pointer flex items-center gap-2 whitespace-nowrap ${
              activeSection === 'architecture'
                ? 'bg-sky-500 text-white shadow-md'
                : 'text-slate-300 hover:text-white hover:bg-white/10'
            }`}
          >
            <Cpu className="w-4 h-4 text-sky-300" />
            <span>{isVi ? '2. Kiến Trúc & Bảo Vệ Riêng Tư' : '2. Architecture & Privacy'}</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSection('license')}
            className={`px-4 py-2 rounded-xl transition cursor-pointer flex items-center gap-2 whitespace-nowrap ${
              activeSection === 'license'
                ? 'bg-sky-500 text-white shadow-md'
                : 'text-slate-300 hover:text-white hover:bg-white/10'
            }`}
          >
            <Scale className="w-4 h-4 text-emerald-400" />
            <span>{isVi ? '3. Mã Nguồn Mở & Giấy Phép MIT' : '3. Open Source & License'}</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSection('contact')}
            className={`px-4 py-2 rounded-xl transition cursor-pointer flex items-center gap-2 whitespace-nowrap ${
              activeSection === 'contact'
                ? 'bg-sky-500 text-white shadow-md'
                : 'text-slate-300 hover:text-white hover:bg-white/10'
            }`}
          >
            <Mail className="w-4 h-4 text-amber-400" />
            <span>{isVi ? '4. Liên Hệ & Đóng Góp' : '4. Contact & Support'}</span>
          </button>
        </div>
      </div>

      {/* SECTION 1: Author & Mission */}
      {activeSection === 'mission' && (
        <div className="space-y-6">
          <div className="bg-white rounded-2xl border border-slate-200/80 p-6 sm:p-8 shadow-xs">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-rose-50 text-rose-600 flex items-center justify-center font-bold">
                <Heart className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-lg font-bold text-slate-900">
                  {isVi ? 'Sứ Mệnh Bảo Vệ Trẻ Em Trên Không Gian Mạng' : 'Mission for Child Online Safety'}
                </h2>
                <p className="text-xs text-slate-500">
                  {isVi
                    ? 'Giải pháp kiểm soát bảo vệ phụ huynh độc lập, vì thế hệ trẻ Việt Nam'
                    : 'Independent parental protection platform engineered for the next generation'}
                </p>
              </div>
            </div>

            <p className="text-slate-600 text-sm leading-relaxed mb-6">
              {isVi
                ? 'FamShield được sáng lập và phát triển bởi Nguyễn Ngọc Tú với tâm huyết xây dựng một giải pháp an toàn số "Make in Vietnam", mang lại môi trường mạng trong sạch, lành mạnh cho các thế hệ trẻ em Việt Nam trong thời đại số hóa mạnh mẽ. Chúng tôi tin rằng mỗi đứa trẻ đều xứng đáng được tiếp cận tri thức internet bao la mà không phải đối mặt với cạm bẫy từ cờ bạc, nội dung khiêu dâm, lừa đảo hoặc bạo lực học đường.'
                : 'FamShield was founded and architected by Nguyen Ngoc Tu to build an open, trustworthy parental control and cybersecurity platform, empowering parents to protect their children online without commercial bloat or invasive corporate snooping.'}
            </p>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="p-4 rounded-xl border border-slate-200 bg-slate-50/50 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-sky-100 text-sky-700 flex items-center justify-center">
                  <Globe className="w-4 h-4" />
                </div>
                <h3 className="font-bold text-sm text-slate-900">
                  {isVi ? 'Tự Chủ Công Nghệ Việt Nam' : 'Sovereign Technology'}
                </h3>
                <p className="text-xs text-slate-500 leading-relaxed">
                  {isVi
                    ? 'Làm chủ 100% công nghệ lõi từ driver lọc mạng Windows (WFP), DNS Loopback Sinkhole đến hạ tầng NATS phân tán, không phụ thuộc máy chủ nước ngoài.'
                    : 'Full ownership of core technologies: WFP network filters, loopback DNS sinkholes, and resilient distributed agents.'}
                </p>
              </div>

              <div className="p-4 rounded-xl border border-slate-200 bg-slate-50/50 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-emerald-100 text-emerald-700 flex items-center justify-center">
                  <ShieldCheck className="w-4 h-4" />
                </div>
                <h3 className="font-bold text-sm text-slate-900">
                  {isVi ? 'Tôn Trọng Quyền Riêng Tư' : 'Privacy-First Design'}
                </h3>
                <p className="text-xs text-slate-500 leading-relaxed">
                  {isVi
                    ? 'Tuân thủ Nghị định 13/2023/NĐ-CP và tiêu chuẩn COPPA. Không ghi phím thô bạo (keylogger), tự động làm mờ ảnh vi phạm trực tiếp trong RAM và tự hủy sau 48 giờ.'
                    : 'Strictly complies with Decree 13/2023/ND-CP and COPPA. No keylogger, localized RAM blurring, auto-expiring in 48 hours.'}
                </p>
              </div>

              <div className="p-4 rounded-xl border border-slate-200 bg-slate-50/50 space-y-2">
                <div className="w-8 h-8 rounded-lg bg-indigo-100 text-indigo-700 flex items-center justify-center">
                  <Zap className="w-4 h-4" />
                </div>
                <h3 className="font-bold text-sm text-slate-900">
                  {isVi ? 'Mô Hình Kép Linh Hoạt' : 'Dual-Engine Model'}
                </h3>
                <p className="text-xs text-slate-500 leading-relaxed">
                  {isVi
                    ? 'Hoạt động 100% độc lập ngoại tuyến (Standalone Free) vĩnh viễn miễn phí, hoặc ghép nối máy chủ đám mây (Online Managed) để quản lý tập trung từ xa.'
                    : 'Works 100% offline standalone for free forever, or seamlessly connects to cloud server for remote centralized management.'}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SECTION 2: Technical Architecture */}
      {activeSection === 'architecture' && (
        <div className="space-y-6">
          <div className="bg-white rounded-2xl border border-slate-200/80 p-6 sm:p-8 shadow-xs">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-10 h-10 rounded-xl bg-sky-50 text-sky-600 flex items-center justify-center font-bold">
                <Cpu className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-lg font-bold text-slate-900">
                  {isVi ? 'Kiến Trúc Kỹ Thuật Agent Cục Bộ' : 'Local Agent Architecture'}
                </h2>
                <p className="text-xs text-slate-500">
                  {isVi
                    ? 'Phát triển bằng Rust tối tân, vận hành ổn định tại Windows Session 0'
                    : 'High-performance Rust service engineered for zero latency and zero crashes'}
                </p>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
              <div className="p-4 rounded-xl border border-slate-200 bg-white space-y-2">
                <div className="flex items-center gap-2 font-bold text-sm text-slate-900">
                  <Code2 className="w-4 h-4 text-indigo-600" />
                  <span>Windows Filtering Platform (WFP)</span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  {isVi
                    ? 'Sử dụng Dynamic Sublayer lọc gói tin cổng 53 UDP/TCP. Tự động thu hồi filter khi service dừng, bảo đảm không bao giờ gây lock mạng hoặc BSOD màn hình xanh.'
                    : 'Registers dynamic sublayer filters for Port 53 UDP/TCP. Auto-cleans up on exit, guaranteeing zero network lockups or BSOD.'}
                </p>
              </div>

              <div className="p-4 rounded-xl border border-slate-200 bg-white space-y-2">
                <div className="flex items-center gap-2 font-bold text-sm text-slate-900">
                  <Terminal className="w-4 h-4 text-emerald-600" />
                  <span>DNS Loopback Sinkhole (0ms)</span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  {isVi
                    ? 'Lắng nghe cục bộ tại 127.0.0.2:53, chặn đứng tức thì tên miền xấu trong danh mục cấm bằng phản hồi 0.0.0.0 hoặc NXDOMAIN trong chưa đầy 1 mili-giây.'
                    : 'Listens locally at 127.0.0.2:53, sinking blacklisted domains with 0.0.0.0 or NXDOMAIN in sub-millisecond time.'}
                </p>
              </div>

              <div className="p-4 rounded-xl border border-slate-200 bg-white space-y-2">
                <div className="flex items-center gap-2 font-bold text-sm text-slate-900">
                  <Lock className="w-4 h-4 text-sky-600" />
                  <span>Windows Session 0 & DPAPI</span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  {isVi
                    ? 'Vận hành dưới tài khoản LocalSystem cách ly trong Session 0. Mật khẩu PIN và bí mật được mã hóa bằng DPAPI machine-scope, chống trích xuất bộ nhớ.'
                    : 'Runs as isolated LocalSystem in Session 0. PINs and tokens are hardware-bound and protected via Windows DPAPI.'}
                </p>
              </div>

              <div className="p-4 rounded-xl border border-slate-200 bg-white space-y-2">
                <div className="flex items-center gap-2 font-bold text-sm text-slate-900">
                  <ShieldCheck className="w-4 h-4 text-rose-600" />
                  <span>Privacy-by-Design & Tự Hủy Dữ Liệu</span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  {isVi
                    ? 'Chỉ phân tích tên miền phân giải DNS và ứng dụng tiền cảnh. Ảnh bằng chứng được nén 480p, làm mờ Gaussian Blur trực tiếp trong RAM và tự xóa sau 48 giờ.'
                    : 'Analyzes DNS domains and foreground app usage only. Visual evidence is blurred in RAM at 480p and pruned after 48h.'}
                </p>
              </div>
            </div>

            <div className="p-4 rounded-xl bg-amber-50 border border-amber-200 text-xs text-amber-900 leading-relaxed flex items-start gap-3">
              <ShieldAlert className="w-5 h-5 text-amber-600 shrink-0 mt-0.5" />
              <div>
                <span className="font-bold">Cam Kết Bảo Mật Tuyệt Đối:</span> FamShield không thu thập nội dung tin nhắn, mật khẩu gõ phím hay webcam cá nhân. Thiết kế tuân thủ các quy định khắt khe nhất của Nghị định 13/2023/NĐ-CP về bảo vệ dữ liệu cá nhân tại Việt Nam.
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SECTION 3: Open Source & MIT License */}
      {activeSection === 'license' && (
        <div className="space-y-6">
          <div className="bg-white rounded-2xl border border-slate-200/80 p-6 sm:p-8 shadow-xs">
            <div className="flex items-center justify-between flex-wrap gap-4 mb-6">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-emerald-50 text-emerald-600 flex items-center justify-center font-bold">
                  <Scale className="w-5 h-5" />
                </div>
                <div>
                  <h2 className="text-lg font-bold text-slate-900">
                    Giấy Phép Mã Nguồn Mở MIT (MIT License)
                  </h2>
                  <p className="text-xs text-slate-500">
                    {isVi
                      ? 'Tự do sử dụng, kiểm tra mã nguồn, tùy biến và phân phối'
                      : 'Free to audit, modify, fork, and distribute without license fees'}
                  </p>
                </div>
              </div>
              <span className="text-xs font-mono font-bold bg-emerald-100 text-emerald-800 px-3 py-1 rounded-full border border-emerald-300">
                OSI Approved • MIT
              </span>
            </div>

            <p className="text-slate-600 text-sm leading-relaxed mb-6">
              {isVi
                ? 'FamShield Agent là phần mềm mã nguồn mở được phát hành theo giấy phép MIT License. Mọi phụ huynh, trường học, kỹ sư an ninh mạng và cộng đồng lập trình viên đều có quyền tự do kiểm tra mã nguồn, đóng góp tính năng hoặc tự triển khai cho gia đình mà không mất bất kỳ chi phí bản quyền nào.'
                : 'FamShield Agent is open-source software distributed under the permissive MIT License. Anyone is free to audit, study, modify, and deploy the software for families or educational institutions.'}
            </p>

            {/* Git Clone Box */}
            <div className="space-y-2 mb-6">
              <label className="text-xs font-bold text-slate-700">
                {isVi ? 'Lệnh sao chép kho mã nguồn Agent qua Git:' : 'Clone Agent repository via Git:'}
              </label>
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-slate-900 text-sky-300 font-mono text-xs shadow-inner">
                <span className="truncate mr-3">{cloneCmd}</span>
                <button
                  type="button"
                  onClick={handleCopyClone}
                  className="inline-flex items-center gap-1.5 bg-slate-800 hover:bg-slate-700 text-white px-3 py-1.5 rounded-lg text-xs font-medium transition cursor-pointer shrink-0"
                >
                  {copiedClone ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copiedClone ? (isVi ? 'Đã sao chép' : 'Copied') : isVi ? 'Sao chép' : 'Copy'}</span>
                </button>
              </div>
            </div>

            {/* Repositories Links */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
              <a
                href={agentRepo}
                target="_blank"
                rel="noopener noreferrer"
                className="p-4 rounded-xl border border-slate-200 hover:border-sky-300 hover:bg-sky-50/30 transition flex items-center justify-between group"
              >
                <div>
                  <div className="font-bold text-sm text-slate-900 group-hover:text-sky-700">
                    famshield-agent
                  </div>
                  <div className="text-xs text-slate-500 mt-0.5">
                    Rust, WFP Windows Service, React, Vite Console
                  </div>
                </div>
                <ExternalLink className="w-4 h-4 text-slate-400 group-hover:text-sky-600" />
              </a>

              <a
                href={serverRepo}
                target="_blank"
                rel="noopener noreferrer"
                className="p-4 rounded-xl border border-slate-200 hover:border-sky-300 hover:bg-sky-50/30 transition flex items-center justify-between group"
              >
                <div>
                  <div className="font-bold text-sm text-slate-900 group-hover:text-sky-700">
                    famshield-server
                  </div>
                  <div className="text-xs text-slate-500 mt-0.5">
                    Go Cloud Server, PostgreSQL, NATS JetStream, Next.js Web
                  </div>
                </div>
                <ExternalLink className="w-4 h-4 text-slate-400 group-hover:text-sky-600" />
              </a>
            </div>

            {/* License Text */}
            <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 font-mono text-xs text-slate-600 space-y-2 leading-normal">
              <div className="font-bold text-slate-800">
                Copyright (c) 2026 Nguyen Ngoc Tu (ngoctuct@gmail.com)
              </div>
              <div className="text-slate-500">
                Permission is hereby granted, free of charge, to any person obtaining a copy
                of this software and associated documentation files (the "Software"), to deal
                in the Software without restriction, including without limitation the rights
                to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
                copies of the Software, and to permit persons to whom the Software is
                furnished to do so, subject to the following conditions:
              </div>
              <div className="text-slate-500">
                The above copyright notice and this permission notice shall be included in all
                copies or substantial portions of the Software.
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SECTION 4: Contact & Social */}
      {activeSection === 'contact' && (
        <div className="space-y-6">
          <div className="bg-white rounded-2xl border border-slate-200/80 p-6 sm:p-8 shadow-xs">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-10 h-10 rounded-xl bg-amber-50 text-amber-600 flex items-center justify-center font-bold">
                <Mail className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-lg font-bold text-slate-900">
                  {isVi ? 'Liên Hệ Trực Tiếp & Đóng Góp Ý Kiến' : 'Direct Contact & Community Support'}
                </h2>
                <p className="text-xs text-slate-500">
                  {isVi
                    ? 'Tác giả trân trọng tiếp nhận mọi đề xuất, báo cáo lỗi và cơ hội hợp tác'
                    : 'Open for feedback, security disclosures, and educational partnerships'}
                </p>
              </div>
            </div>

            <p className="text-slate-600 text-sm leading-relaxed mb-6">
              {isVi
                ? 'Mọi ý kiến đóng góp tính năng, phản hồi trải nghiệm, báo cáo lỗi kỹ thuật hoặc đề xuất tích hợp giải pháp an toàn thông tin cho trường học, gia đình đều được tác giả tiếp nhận và xử lý nhanh chóng:'
                : 'Any feedback, feature requests, vulnerability disclosures, or collaboration inquiries are welcomed directly by the creator:'}
            </p>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              {/* Email */}
              <div className="p-4 rounded-xl border border-slate-200 bg-white flex flex-col justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-sky-50 text-sky-600 flex items-center justify-center font-bold">
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
                  className="w-full inline-flex items-center justify-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 py-2 rounded-lg text-xs font-semibold transition cursor-pointer"
                >
                  {copiedEmail ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copiedEmail ? (isVi ? 'Đã sao chép' : 'Copied') : isVi ? 'Sao chép Email' : 'Copy Email'}</span>
                </button>
              </div>

              {/* GitHub */}
              <div className="p-4 rounded-xl border border-slate-200 bg-white flex flex-col justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-slate-100 text-slate-800 flex items-center justify-center font-bold">
                    GH
                  </div>
                  <div>
                    <div className="text-xs text-slate-500">GitHub Profile</div>
                    <div className="font-semibold text-slate-900 text-xs font-mono">github.com/nntu</div>
                  </div>
                </div>
                <a
                  href={authorGithub}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full inline-flex items-center justify-center gap-1.5 bg-slate-900 hover:bg-slate-800 text-white py-2 rounded-lg text-xs font-semibold transition cursor-pointer"
                >
                  <span>Mở GitHub</span>
                  <ExternalLink className="w-3.5 h-3.5" />
                </a>
              </div>

              {/* Facebook */}
              <div className="p-4 rounded-xl border border-slate-200 bg-white flex flex-col justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-blue-50 text-blue-600 flex items-center justify-center font-bold">
                    FB
                  </div>
                  <div>
                    <div className="text-xs text-slate-500">Facebook Cá Nhân</div>
                    <div className="font-semibold text-slate-900 text-xs font-mono">fb.com/ngoctuct</div>
                  </div>
                </div>
                <a
                  href={authorFacebook}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full inline-flex items-center justify-center gap-1.5 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-lg text-xs font-semibold transition cursor-pointer"
                >
                  <span>Mở Facebook</span>
                  <ExternalLink className="w-3.5 h-3.5" />
                </a>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
