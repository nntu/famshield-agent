import {
  AlertOctagon,
  ArrowRight,
  Layers,
  Play,
  RefreshCw,
  ShieldCheck,
  Smartphone,
} from 'lucide-react';
import type React from 'react';
import { useEffect, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { Language, QueryLogRecord, StatusResponse, TabType } from '../types';

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
    api
      .getTotpStatus()
      .then((s) => setTotpStatus(s))
      .catch(() => {});
  }, []);

  const handleStartHelper = async () => {
    try {
      setStartingHelper(true);
      const res = await api.restartSessionHelper();
      alert(res.message);
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      alert(
        msg ||
          (lang === 'vi' ? 'Lỗi khi khởi chạy lại trợ lý phiên' : 'Failed to start session helper')
      );
    } finally {
      setStartingHelper(false);
    }
  };

  const quotaTotal = status?.daily_quota_minutes || 60;
  const quotaUsed = status?.used_today_minutes || 0;
  const quotaPct = Math.min(100, Math.round((quotaUsed / quotaTotal) * 100));

  return (
    <div className="flex flex-col gap-6 w-full">
      {/* Session Helper Alert Banner */}
      {status && !status.session_helper_running && (
        <div className="bg-rose-50/70 border border-rose-200/80 rounded-xl p-4 sm:p-4.5 flex items-center justify-between gap-4 shadow-xs">
          <div className="flex items-center gap-3">
            <span className="text-xl">⚠️</span>
            <div>
              <div className="font-bold text-rose-900 text-sm mb-0.5">
                {lang === 'vi'
                  ? 'Cảnh Báo: Trợ Lý Phiên Người Dùng Chưa Chạy'
                  : 'Warning: User Session Helper Not Running'}
              </div>
              <div className="text-rose-700/90 text-xs">
                {lang === 'vi'
                  ? 'Biểu tượng khay hệ thống, hạn mức thời gian và giám sát từ khóa đang bị gián đoạn. Dịch vụ ngầm vẫn đang lọc DNS.'
                  : 'System tray, app limits, and keystroke monitoring are interrupted. Windows Service is guarding DNS.'}
              </div>
            </div>
          </div>
          <button
            disabled={startingHelper}
            onClick={handleStartHelper}
            className="bg-rose-600 hover:bg-rose-700 disabled:bg-rose-400 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            <Play size={13} />
            {startingHelper
              ? lang === 'vi'
                ? 'Đang khởi chạy...'
                : 'Starting...'
              : lang === 'vi'
                ? 'Khởi Chạy Lại'
                : 'Restart Helper'}
          </button>
        </div>
      )}

      {/* Monitor Mode Alert Banner */}
      {status?.monitor_mode && (
        <div className="bg-amber-50/70 border border-amber-200/80 rounded-xl p-4 sm:p-4.5 flex items-center justify-between gap-4 shadow-xs">
          <div className="flex items-center gap-3">
            <span className="text-xl">👁️</span>
            <div>
              <div className="font-bold text-amber-900 text-sm mb-0.5">{t.mode_monitor}</div>
              <div className="text-amber-800 text-xs">{t.banner_monitor_mode}</div>
            </div>
          </div>
          <button
            onClick={() => onNavigateTab('security')}
            className="bg-amber-700 hover:bg-amber-800 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            {t.nav_security} &rarr;
          </button>
        </div>
      )}

      {/* 2FA Notice Banner (Quiet, unified design) */}
      {totpStatus && !totpStatus.enabled && (
        <div className="bg-amber-50/60 border border-amber-200/80 rounded-xl p-4 sm:p-4.5 flex items-center justify-between flex-wrap gap-4 shadow-xs">
          <div className="flex items-center gap-3">
            <span className="text-xl text-amber-700">🔐</span>
            <div>
              <div className="font-bold text-slate-900 text-sm mb-0.5">
                {lang === 'vi'
                  ? 'Khuyến nghị bảo mật: Kích hoạt xác thực 2 lớp (2FA / OTP)'
                  : 'Security Notice: Two-Factor Authentication (2FA) is Disabled'}
              </div>
              <div className="text-slate-600 text-xs">
                {lang === 'vi'
                  ? 'Bảo vệ bảng điều khiển chống nhìn lén Master PIN bằng mã OTP 6 số (Google/Microsoft Authenticator ngoại tuyến).'
                  : 'Protect Master PIN with 6-digit OTP codes from Google/Microsoft Authenticator (100% offline).'}
              </div>
            </div>
          </div>
          <button
            type="button"
            onClick={() => onNavigateTab('security')}
            className="bg-slate-900 hover:bg-slate-800 text-white px-3.5 py-1.5 rounded-lg text-xs font-semibold whitespace-nowrap cursor-pointer transition-colors shadow-xs"
          >
            {lang === 'vi' ? 'Kích Hoạt 2FA' : 'Enable 2FA'} &rarr;
          </button>
        </div>
      )}

      {/* 4 Stat Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4.5 w-full">
        {/* Card 1: Active App */}
        <div className="bg-white border border-slate-200/80 rounded-xl p-5 shadow-[0_1px_3px_rgba(15,23,42,0.04)] flex flex-col justify-between min-h-[160px] min-w-0">
          <div>
            <div className="flex justify-between items-start mb-2 gap-2">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-1.5 truncate">
                <Smartphone size={15} className="text-sky-600 shrink-0" />
                <span>{t.card_app_title}</span>
              </span>
              <span
                className={`px-2 py-0.5 rounded-full text-2xs font-semibold border shrink-0 ${
                  status?.session_helper_running
                    ? 'bg-emerald-50 text-emerald-700 border-emerald-200/80'
                    : 'bg-rose-50 text-rose-700 border-rose-200/80'
                }`}
              >
                {status?.session_helper_running
                  ? lang === 'vi'
                    ? '🟢 Online'
                    : '🟢 Active'
                  : lang === 'vi'
                    ? '🔴 Chưa chạy'
                    : '🔴 Stopped'}
              </span>
            </div>

            <div
              className="text-xl font-bold text-slate-900 mb-1 truncate"
              title={status?.active_foreground_app || 'Roblox.exe'}
            >
              {status?.active_foreground_app || 'Roblox.exe'}
            </div>

            <div className="text-xs text-slate-500 mb-2.5 truncate">
              {t.dash_quota_text
                .replace('{used}', quotaUsed.toString())
                .replace('{total}', quotaTotal.toString())}
            </div>

            {/* Quota Progress Bar */}
            <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-500 ${
                  quotaPct > 80 ? 'bg-rose-500' : 'bg-sky-600'
                }`}
                style={{ width: `${quotaPct}%` }}
              />
            </div>
          </div>

          <button
            onClick={() => onNavigateTab('activity')}
            className="mt-3 text-xs font-semibold text-sky-600 hover:text-sky-700 flex items-center gap-1 cursor-pointer"
          >
            <span>{lang === 'vi' ? 'Chi tiết hoạt động' : 'View full activity'} &rarr;</span>
          </button>
        </div>

        {/* Card 2: DNS & WFP Filter */}
        <div className="bg-white border border-slate-200/80 rounded-xl p-5 shadow-[0_1px_3px_rgba(15,23,42,0.04)] flex flex-col justify-between min-h-[160px] min-w-0">
          <div>
            <div className="flex justify-between items-start mb-2 gap-2">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-1.5 truncate">
                <ShieldCheck
                  size={15}
                  className={status?.monitor_mode ? 'text-amber-500' : 'text-emerald-600'}
                />
                <span>{t.card_dns_title}</span>
              </span>
              <span className="bg-slate-100 text-slate-700 border border-slate-200/80 px-2 py-0.5 rounded-md text-2xs font-mono shrink-0">
                {status?.dns_proxy_addr || '127.0.0.2:53'}
              </span>
            </div>

            <div
              className={`text-xl font-bold mb-1 ${status?.monitor_mode ? 'text-amber-700' : 'text-slate-900'}`}
            >
              {status?.monitor_mode ? t.mode_monitor : t.status_protecting}
            </div>

            <div className="text-xs text-slate-500 line-clamp-2">{t.dash_dns_sub}</div>
          </div>

          <div className="mt-3 pt-2.5 border-t border-slate-100 flex items-center justify-between text-2xs text-slate-500">
            <span>{lang === 'vi' ? 'Phiên bản Agent' : 'Agent Version'}</span>
            <span className="font-mono font-semibold text-slate-700 bg-slate-100 px-1.5 py-0.5 rounded">
              v{status?.app_version || '1.0.3'}
            </span>
          </div>
        </div>

        {/* Card 3: Blocked Today */}
        <div className="bg-white border border-slate-200/80 rounded-xl p-5 shadow-[0_1px_3px_rgba(15,23,42,0.04)] flex flex-col justify-between min-h-[160px] min-w-0">
          <div>
            <div className="flex justify-between items-start mb-2 gap-2">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-1.5 truncate">
                <AlertOctagon size={15} className="text-rose-500" />
                <span>{t.card_blocked_today}</span>
              </span>
              <span className="bg-rose-50 text-rose-700 border border-rose-200/80 px-2 py-0.5 rounded-full text-2xs font-bold shrink-0">
                {t.tag_block}
              </span>
            </div>

            <div className="text-2xl font-bold text-slate-900 mb-1">
              {status?.blocked_queries_today ?? 0}
            </div>

            <div className="text-xs text-slate-500 truncate">
              {t.dash_blocked_sub.replace('{total}', (status?.total_queries_today ?? 0).toString())}
            </div>
          </div>
        </div>

        {/* Card 4: Rules in effect */}
        <div className="bg-white border border-slate-200/80 rounded-xl p-5 shadow-[0_1px_3px_rgba(15,23,42,0.04)] flex flex-col justify-between min-h-[160px] min-w-0">
          <div>
            <div className="flex justify-between items-start mb-2 gap-2">
              <span className="text-xs font-semibold text-slate-500 flex items-center gap-1.5 truncate">
                <Layers size={15} className="text-slate-600" />
                <span>{t.card_rules_title}</span>
              </span>
              <span className="bg-slate-100 text-slate-600 border border-slate-200/80 px-2 py-0.5 rounded-full text-2xs font-semibold shrink-0">
                {status?.rules_count ?? 0} {lang === 'vi' ? 'quy tắc' : 'rules'}
              </span>
            </div>

            <div className="text-2xl font-bold text-slate-900 mb-1">{status?.rules_count ?? 0}</div>

            <div className="text-xs text-slate-500 truncate">{t.dash_rules_sub}</div>
          </div>
        </div>
      </div>

      {/* Mini Recent Query Feed */}
      <div className="bg-white border border-slate-200/80 rounded-xl p-5 sm:p-6 shadow-[0_1px_3px_rgba(15,23,42,0.04)] w-full">
        <div className="flex justify-between items-center mb-4 flex-wrap gap-3">
          <div>
            <h2 className="text-base font-bold text-slate-900">{t.recent_logs_title}</h2>
            <p className="text-xs text-slate-500 mt-0.5">
              {lang === 'vi'
                ? 'Theo dõi trực tiếp các yêu cầu phân giải DNS loopback mới nhất'
                : 'Real-time DNS loopback query feed'}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={onRefresh}
              className="flex items-center gap-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-200/80 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-colors"
            >
              <RefreshCw size={13} />
              <span>{t.btn_refresh}</span>
            </button>
            <button
              onClick={() => onNavigateTab('logs')}
              className="flex items-center gap-1.5 bg-slate-900 hover:bg-slate-800 text-white px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-colors shadow-xs"
            >
              <span>{t.btn_view_all_logs}</span>
              <ArrowRight size={13} />
            </button>
          </div>
        </div>

        {recentLogs.length === 0 ? (
          <div className="text-center text-slate-400 py-8 text-sm">{t.no_logs}</div>
        ) : (
          <div className="w-full overflow-x-auto rounded-lg border border-slate-200/80">
            <table className="min-w-[640px] w-full border-collapse text-xs sm:text-sm text-left">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50">
                  <th className="px-3.5 py-2.5 text-slate-500 font-semibold text-xs uppercase tracking-wider">
                    {t.th_time}
                  </th>
                  <th className="px-3.5 py-2.5 text-slate-500 font-semibold text-xs uppercase tracking-wider">
                    {t.th_domain}
                  </th>
                  <th className="px-3.5 py-2.5 text-slate-500 font-semibold text-xs uppercase tracking-wider">
                    {t.th_type}
                  </th>
                  <th className="px-3.5 py-2.5 text-slate-500 font-semibold text-xs uppercase tracking-wider">
                    {t.th_action}
                  </th>
                  <th className="px-3.5 py-2.5 text-slate-500 font-semibold text-xs uppercase tracking-wider">
                    {t.th_reason}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {recentLogs.slice(0, 6).map((log, idx) => {
                  const isBlock = log.action === 'BLOCK';
                  const timeStr = new Date(log.occurred_at).toLocaleTimeString();
                  return (
                    <tr
                      key={`${log.domain}-${log.occurred_at}-${idx}`}
                      className="hover:bg-slate-50/70 transition-colors"
                    >
                      <td className="px-3.5 py-2.5 text-slate-500 font-mono text-xs whitespace-nowrap">
                        {timeStr}
                      </td>
                      <td className="px-3.5 py-2.5 font-medium text-slate-800">{log.domain}</td>
                      <td className="px-3.5 py-2.5">
                        <span className="bg-slate-100 text-slate-600 border border-slate-200/80 px-1.5 py-0.5 rounded text-2xs font-mono">
                          {log.qtype}
                        </span>
                      </td>
                      <td className="px-3.5 py-2.5">
                        <span
                          className={`px-2 py-0.5 rounded-full font-semibold text-2xs border ${
                            isBlock
                              ? 'bg-rose-50 text-rose-700 border-rose-200/70'
                              : 'bg-emerald-50 text-emerald-700 border-emerald-200/70'
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
