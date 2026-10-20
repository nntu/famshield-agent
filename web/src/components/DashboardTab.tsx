import React, { useState, useEffect } from 'react';
import { ShieldCheck, AlertOctagon, Layers, ArrowRight, RefreshCw, Smartphone, Play } from 'lucide-react';
import { Language, QueryLogRecord, StatusResponse, TabType } from '../types';
import { translations } from '../i18n';
import { api } from '../api';

interface DashboardTabProps {
  status: StatusResponse | null;
  recentLogs: QueryLogRecord[];
  lang: Language;
  onNavigateTab: (tab: TabType) => void;
  onRefresh: () => void;
}

export const DashboardTab: React.FC<DashboardTabProps> = ({
  status,
  recentLogs,
  lang,
  onNavigateTab,
  onRefresh,
}) => {
  const t = translations[lang];
  const [startingHelper, setStartingHelper] = useState(false);
  const [totpStatus, setTotpStatus] = useState<{ enabled: boolean } | null>(null);

  useEffect(() => {
    api.getTotpStatus()
      .then((s) => setTotpStatus(s))
      .catch(() => {});
  }, []);

  const handleStartHelper = async () => {
    try {
      setStartingHelper(true);
      const res = await api.restartSessionHelper();
      alert(res.message);
      onRefresh();
    } catch (e: any) {
      alert(e.message || (lang === 'vi' ? 'Lỗi khi khởi chạy lại trợ lý phiên' : 'Failed to start session helper'));
    } finally {
      setStartingHelper(false);
    }
  };

  const quotaTotal = status?.daily_quota_minutes || 60;
  const quotaUsed = status?.used_today_minutes || 0;
  const quotaPct = Math.min(100, Math.round((quotaUsed / quotaTotal) * 100));

  return (
    <div className="flex flex-col gap-6">
      {/* Session Helper Alert Banner */}
      {status && !status.session_helper_running && (
        <div className="bg-red-50 border border-red-200 rounded-2xl p-4 sm:p-5 flex items-center justify-between gap-4 shadow-xs">
          <div className="flex items-center gap-3.5">
            <span className="text-2xl">⚠️</span>
            <div>
              <div className="font-bold text-red-700 text-sm sm:text-base mb-0.5">
                {lang === 'vi' ? 'Cảnh Báo: Trợ Lý Phiên Người Dùng (famshield-session.exe) Chưa Chạy' : 'Warning: User Session Helper (famshield-session.exe) Not Running'}
              </div>
              <div className="text-red-600/90 text-xs sm:text-sm">
                {lang === 'vi'
                  ? 'Biểu tượng khay hệ thống (Tray icon), hạn mức thời gian ứng dụng và giám sát từ khóa nhạy cảm đang bị gián đoạn. Dịch vụ ngầm (Windows Service) vẫn đang bảo vệ DNS.'
                  : 'System tray icon, real-time app limits, and keystroke monitoring are interrupted. Windows Service is still guarding DNS.'}
              </div>
            </div>
          </div>
          <button
            disabled={startingHelper}
            onClick={handleStartHelper}
            className="bg-red-600 hover:bg-red-700 disabled:bg-red-400 text-white px-4 py-2 rounded-xl text-xs sm:text-sm font-semibold flex items-center gap-2 whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            <Play size={14} />
            {startingHelper
              ? (lang === 'vi' ? 'Đang khởi chạy...' : 'Starting...')
              : (lang === 'vi' ? 'Khởi Chạy Lại' : 'Restart Helper')}
          </button>
        </div>
      )}

      {/* Monitor Mode Alert Banner */}
      {status?.monitor_mode && (
        <div className="bg-amber-50 border border-amber-200 rounded-2xl p-4 sm:p-5 flex items-center justify-between gap-4 shadow-xs">
          <div className="flex items-center gap-3.5">
            <span className="text-2xl">👁️</span>
            <div>
              <div className="font-bold text-amber-800 text-sm sm:text-base mb-0.5">
                {t.mode_monitor}
              </div>
              <div className="text-amber-700 text-xs sm:text-sm">
                {t.banner_monitor_mode}
              </div>
            </div>
          </div>
          <button
            onClick={() => onNavigateTab('security')}
            className="bg-amber-600 hover:bg-amber-700 text-white px-4 py-2 rounded-xl text-xs sm:text-sm font-semibold whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            {t.nav_security} &rarr;
          </button>
        </div>
      )}

      {/* 2FA Disabled Security Banner */}
      {totpStatus && !totpStatus.enabled && (
        <div className="bg-sky-50 border border-sky-200 rounded-2xl p-4 sm:p-5 flex items-center justify-between flex-wrap gap-4 shadow-xs">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🔐</span>
            <div>
              <div className="font-bold text-sky-900 text-sm sm:text-base mb-0.5">
                {lang === 'vi' ? 'Khuyến Nghị Bảo Mật: Chưa Kích Hoạt Xác Thực 2 Lớp (2FA / OTP)' : 'Security Notice: Two-Factor Authentication (2FA) is Disabled'}
              </div>
              <div className="text-sky-700 text-xs sm:text-sm">
                {lang === 'vi'
                  ? 'Bảo vệ bảng điều khiển chống nhìn lén Master PIN bằng mã OTP 6 số sinh từ Google/Microsoft Authenticator (100% ngoại tuyến).'
                  : 'Protect your Master PIN with 6-digit OTP codes from Google/Microsoft Authenticator (100% offline).'}
              </div>
            </div>
          </div>
          <button
            type="button"
            onClick={() => onNavigateTab('security')}
            className="bg-sky-600 hover:bg-sky-700 text-white px-4 py-2 rounded-xl text-xs sm:text-sm font-bold whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            {lang === 'vi' ? '👉 Kích Hoạt 2FA Ngay' : '👉 Enable 2FA Now'}
          </button>
        </div>
      )}

      {/* 4 Stat Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
        {/* Card 1: Active App */}
        <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div>
            <div className="flex justify-between items-start mb-2.5">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-2">
                <Smartphone size={16} className="text-sky-600" />
                {t.card_app_title}
              </span>
              <span
                className={`px-2 py-0.5 rounded-full text-xs font-semibold border ${
                  status?.session_helper_running
                    ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                    : 'bg-red-50 text-red-700 border-red-200'
                }`}
              >
                {status?.session_helper_running
                  ? (lang === 'vi' ? '🟢 Trợ lý online' : '🟢 Active')
                  : (lang === 'vi' ? '🔴 Chưa chạy' : '🔴 Stopped')}
              </span>
            </div>

            <div className="text-2xl font-extrabold text-slate-900 mb-1 truncate">
              {status?.active_foreground_app || 'Roblox.exe'}
            </div>

            <div className="text-xs text-slate-500 mb-3">
              {t.dash_quota_text.replace('{used}', quotaUsed.toString()).replace('{total}', quotaTotal.toString())}
            </div>

            {/* Quota Progress Bar */}
            <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-500 ${
                  quotaPct > 80
                    ? 'bg-gradient-to-r from-amber-500 to-red-500'
                    : 'bg-gradient-to-r from-sky-500 to-emerald-500'
                }`}
                style={{ width: `${quotaPct}%` }}
              />
            </div>
          </div>

          <button
            onClick={() => onNavigateTab('activity')}
            className="mt-3 text-xs font-semibold text-sky-600 hover:text-sky-700 flex items-center gap-1 cursor-pointer"
          >
            <span>{lang === 'vi' ? 'Xem chi tiết hoạt động' : 'View full activity'} &rarr;</span>
          </button>
        </div>

        {/* Card 2: DNS & WFP Filter */}
        <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div>
            <div className="flex justify-between items-start mb-2.5">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-2">
                <ShieldCheck size={16} className={status?.monitor_mode ? 'text-amber-500' : 'text-emerald-600'} />
                {t.card_dns_title}
              </span>
              <span className="bg-sky-50 text-sky-700 border border-sky-200 px-2 py-0.5 rounded-full text-xs font-bold">
                {status?.dns_proxy_addr || '127.0.0.2:53'}
              </span>
            </div>

            <div className={`text-2xl font-extrabold mb-1 ${status?.monitor_mode ? 'text-amber-600' : 'text-emerald-600'}`}>
              {status?.monitor_mode ? t.mode_monitor : t.status_protecting}
            </div>

            <div className="text-xs text-slate-500">
              {t.dash_dns_sub}
            </div>
          </div>
        </div>

        {/* Card 3: Blocked Today */}
        <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div>
            <div className="flex justify-between items-start mb-2.5">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-2">
                <AlertOctagon size={16} className="text-red-500" />
                {t.card_blocked_today}
              </span>
              <span className="bg-red-50 text-red-700 border border-red-200 px-2 py-0.5 rounded-full text-xs font-bold">
                {t.tag_block}
              </span>
            </div>

            <div className="text-2xl font-extrabold text-red-600 mb-1">
              {status?.blocked_queries_today ?? 0}
            </div>

            <div className="text-xs text-slate-500">
              {t.dash_blocked_sub.replace('{total}', (status?.total_queries_today ?? 0).toString())}
            </div>
          </div>
        </div>

        {/* Card 4: Rules in effect */}
        <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-xs flex flex-col justify-between">
          <div>
            <div className="flex justify-between items-start mb-2.5">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-2">
                <Layers size={16} className="text-purple-600" />
                {t.card_rules_title}
              </span>
              <span className="bg-slate-100 text-slate-600 border border-slate-200 px-2 py-0.5 rounded-full text-xs font-semibold">
                {status?.rules_count ?? 0} quy tắc
              </span>
            </div>

            <div className="text-2xl font-extrabold text-slate-900 mb-1">
              {status?.rules_count ?? 0}
            </div>

            <div className="text-xs text-slate-500">
              {t.dash_rules_sub}
            </div>
          </div>
        </div>
      </div>

      {/* Mini Recent Query Feed */}
      <div className="bg-white border border-slate-200 rounded-2xl p-5 sm:p-6 shadow-xs">
        <div className="flex justify-between items-center mb-4 flex-wrap gap-3">
          <div>
            <h2 className="text-base sm:text-lg font-bold text-slate-900">
              {t.recent_logs_title}
            </h2>
            <p className="text-xs text-slate-500">
              Theo dõi trực tiếp các yêu cầu phân giải DNS loopback mới nhất
            </p>
          </div>

          <div className="flex items-center gap-2.5">
            <button
              onClick={onRefresh}
              className="flex items-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-200 px-3 py-1.5 rounded-xl text-xs font-semibold cursor-pointer transition-colors shadow-2xs"
            >
              <RefreshCw size={14} />
              {t.btn_refresh}
            </button>
            <button
              onClick={() => onNavigateTab('logs')}
              className="flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 text-white px-3 py-1.5 rounded-xl text-xs font-bold cursor-pointer transition-colors shadow-xs"
            >
              <span>{t.btn_view_all_logs}</span>
              <ArrowRight size={14} />
            </button>
          </div>
        </div>

        {recentLogs.length === 0 ? (
          <div className="text-center text-slate-400 py-9 text-sm">
            {t.no_logs}
          </div>
        ) : (
          <div className="overflow-x-auto rounded-xl border border-slate-200">
            <table className="w-full border-collapse text-xs sm:text-sm text-left">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50/80">
                  <th className="text-left px-3.5 py-2.5 text-slate-600 font-semibold">{t.th_time}</th>
                  <th className="text-left px-3.5 py-2.5 text-slate-600 font-semibold">{t.th_domain}</th>
                  <th className="text-left px-3.5 py-2.5 text-slate-600 font-semibold">{t.th_type}</th>
                  <th className="text-left px-3.5 py-2.5 text-slate-600 font-semibold">{t.th_action}</th>
                  <th className="text-left px-3.5 py-2.5 text-slate-600 font-semibold">{t.th_reason}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {recentLogs.slice(0, 5).map((log, idx) => {
                  const isBlock = log.action === 'BLOCK';
                  const timeStr = new Date(log.occurred_at).toLocaleTimeString();
                  return (
                    <tr key={idx} className="hover:bg-slate-50/80 transition-colors">
                      <td className="px-3.5 py-2.5 text-slate-500 whitespace-nowrap">{timeStr}</td>
                      <td className="px-3.5 py-2.5 font-semibold text-slate-800">{log.domain}</td>
                      <td className="px-3.5 py-2.5">
                        <span className="bg-slate-100 text-slate-600 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium">
                          {log.qtype}
                        </span>
                      </td>
                      <td className="px-3.5 py-2.5">
                        <span
                          className={`px-2 py-0.5 rounded-full font-bold text-2xs border ${
                            isBlock
                              ? 'bg-red-50 text-red-700 border-red-200'
                              : 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          }`}
                        >
                          {isBlock ? t.tag_block : t.tag_allow}
                        </span>
                      </td>
                      <td className="px-3.5 py-2.5 text-slate-500 text-xs">{log.reason}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
