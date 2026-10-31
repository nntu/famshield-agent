import type { ColumnDef } from '@tanstack/react-table';
import { Ban, BarChart3, Clock, Globe, RefreshCw, X } from 'lucide-react';
import type React from 'react';
import { translations } from '../../i18n';
import type {
  DomainQuerySummary,
  HourlyFrequency,
  Language,
  QueryLogRecord,
} from '../../types';
import { DataTable } from '../DataTable';
import { DnsHourlyChart } from '../charts/DnsHourlyChart';

interface DomainDrilldownModalProps {
  selectedDomain: string;
  activeSummary?: DomainQuerySummary | null;
  domainLogs: QueryLogRecord[];
  domainHourlyData: HourlyFrequency[];
  domainTimeClusters: {
    morning: string[];
    afternoon: string[];
    evening: string[];
    night: string[];
  } | null;
  isLoadingDetails: boolean;
  isBlocking: boolean;
  rawLogColumns: ColumnDef<QueryLogRecord>[];
  onClose: () => void;
  onQuickBlock: (domain: string) => void;
  lang: Language;
}

export const DomainDrilldownModal: React.FC<DomainDrilldownModalProps> = ({
  selectedDomain,
  activeSummary,
  domainLogs,
  domainHourlyData,
  domainTimeClusters,
  isLoadingDetails,
  isBlocking,
  rawLogColumns,
  onClose,
  onQuickBlock,
  lang,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  return (
    <div
      className="fixed inset-0 bg-slate-900/40 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-150"
      onClick={onClose}
    >
      <div
        className="bg-white border border-slate-200 rounded-2xl w-full max-w-3xl max-h-[92vh] flex flex-col shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Modal Header */}
        <div className="p-5 sm:p-6 border-b border-slate-100 flex justify-between items-center flex-wrap gap-3 bg-white">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-sky-50 text-sky-600 flex items-center justify-center shrink-0">
              <Globe size={20} />
            </div>
            <div>
              <h3 className="text-lg font-bold text-slate-900 break-all">{selectedDomain}</h3>
              <p className="text-xs text-slate-500">
                {t.modal_domain_desc}:{' '}
                <span className="text-sky-600 font-semibold">{selectedDomain}</span>
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2.5">
            {/* Quick Block Button */}
            <button
              onClick={() => onQuickBlock(selectedDomain)}
              disabled={isBlocking}
              className="inline-flex items-center gap-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 px-3.5 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all disabled:opacity-50"
            >
              <Ban size={13} />
              <span>{isBlocking ? (isVi ? 'Đang chặn...' : 'Blocking...') : t.btn_quick_block}</span>
            </button>

            {/* Close Button */}
            <button
              onClick={onClose}
              className="w-8 h-8 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 flex items-center justify-center cursor-pointer transition-colors border-0"
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* Modal Subheader with Summary Stats */}
        {activeSummary && (
          <div className="px-6 py-2.5 bg-slate-50 border-b border-slate-100 flex gap-4 flex-wrap text-xs text-slate-600 font-medium">
            <div className="flex items-center gap-1.5">
              <span>{t.th_total_queries}:</span>
              <strong className="text-sky-700 font-bold">{activeSummary.total_queries}</strong>
            </div>
            <div className="flex items-center gap-1.5">
              <span>ALLOW:</span>
              <strong className="text-emerald-700 font-bold">{activeSummary.allowed_count}</strong>
            </div>
            <div className="flex items-center gap-1.5">
              <span>BLOCK:</span>
              <strong className="text-rose-700 font-bold">{activeSummary.blocked_count}</strong>
            </div>
            {activeSummary.monitored_count > 0 && (
              <div className="flex items-center gap-1.5">
                <span>MONITOR:</span>
                <strong className="text-amber-700 font-bold">{activeSummary.monitored_count}</strong>
              </div>
            )}
            <div className="flex items-center gap-1.5">
              <span>{t.th_last_seen}:</span>
              <strong className="text-slate-800 font-mono">
                {new Date(activeSummary.last_seen_at).toLocaleTimeString()}{' '}
                <span className="text-slate-400 font-normal">
                  ({new Date(activeSummary.last_seen_at).toLocaleDateString()})
                </span>
              </strong>
            </div>
          </div>
        )}

        {/* Modal Body: Timeline & Query Log Table for Domain */}
        <div className="flex-1 overflow-y-auto p-5 sm:p-6 flex flex-col gap-5">
          {/* 1. VISUAL 24-HOUR ACCESS TIMELINE RIBBON */}
          <div className="bg-sky-50/60 border border-sky-100 rounded-xl p-4 flex flex-col gap-2.5">
            <div className="flex justify-between items-center text-xs">
              <span className="font-bold text-slate-900 flex items-center gap-1.5">
                <Clock size={14} className="text-sky-600" />
                <span>{t.domain_timeline_title}</span>
              </span>
              <span className="text-2xs text-slate-500 font-medium font-mono">
                00:00 ─── 24:00
              </span>
            </div>

            {/* 24-segment timeline ribbon */}
            <div className="grid grid-cols-24 gap-0.5 h-6 bg-slate-200/60 p-0.5 rounded-md overflow-hidden">
              {domainHourlyData.map((h) => {
                const hasTraffic = h.total_count > 0;
                const hasBlock = h.blocked_count > 0;
                return (
                  <div
                    key={h.hour}
                    title={`${h.hour.toString().padStart(2, '0')}:00 - ${h.total_count} ${
                      isVi ? 'truy vấn' : 'queries'
                    }`}
                    className={`h-full rounded-xs transition-all ${
                      !hasTraffic
                        ? 'bg-slate-100'
                        : hasBlock
                          ? 'bg-rose-500 hover:brightness-110'
                          : 'bg-sky-500 hover:brightness-110'
                    }`}
                  />
                );
              })}
            </div>

            <div className="flex justify-between text-3xs font-mono text-slate-400 px-0.5">
              <span>00h</span>
              <span>06h</span>
              <span>12h</span>
              <span>18h</span>
              <span>24h</span>
            </div>
          </div>

          {/* 2. TIME-OF-DAY ACCESS CLUSTERS */}
          {domainTimeClusters && (
            <div className="bg-slate-50 border border-slate-200/80 rounded-xl p-3.5 flex flex-col gap-2">
              <span className="text-xs font-bold text-slate-800">
                {isVi
                  ? '📍 Các thời điểm truy cập trong ngày'
                  : '📍 Access intervals throughout the day'}
              </span>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-2xs">
                {/* Morning */}
                <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                  <span className="font-semibold text-amber-700 flex items-center gap-1">
                    <span>🌅</span> {t.time_morning}
                  </span>
                  {domainTimeClusters.morning.length > 0 ? (
                    <div className="flex gap-1 flex-wrap mt-0.5">
                      {domainTimeClusters.morning.map((ts, idx) => (
                        <span
                          key={idx}
                          className="bg-amber-50 text-amber-900 border border-amber-200 px-1.5 py-0.5 rounded font-mono"
                        >
                          {ts}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <span className="text-slate-400 italic text-3xs">
                      {isVi ? 'Không có truy cập' : 'No activity'}
                    </span>
                  )}
                </div>

                {/* Afternoon */}
                <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                  <span className="font-semibold text-sky-700 flex items-center gap-1">
                    <span>☀️</span> {t.time_afternoon}
                  </span>
                  {domainTimeClusters.afternoon.length > 0 ? (
                    <div className="flex gap-1 flex-wrap mt-0.5">
                      {domainTimeClusters.afternoon.map((ts, idx) => (
                        <span
                          key={idx}
                          className="bg-sky-50 text-sky-900 border border-sky-200 px-1.5 py-0.5 rounded font-mono"
                        >
                          {ts}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <span className="text-slate-400 italic text-3xs">
                      {isVi ? 'Không có truy cập' : 'No activity'}
                    </span>
                  )}
                </div>

                {/* Evening */}
                <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                  <span className="font-semibold text-indigo-700 flex items-center gap-1">
                    <span>🌆</span> {t.time_evening}
                  </span>
                  {domainTimeClusters.evening.length > 0 ? (
                    <div className="flex gap-1 flex-wrap mt-0.5">
                      {domainTimeClusters.evening.map((ts, idx) => (
                        <span
                          key={idx}
                          className="bg-indigo-50 text-indigo-900 border border-indigo-200 px-1.5 py-0.5 rounded font-mono"
                        >
                          {ts}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <span className="text-slate-400 italic text-3xs">
                      {isVi ? 'Không có truy cập' : 'No activity'}
                    </span>
                  )}
                </div>

                {/* Night */}
                <div className="bg-white border border-slate-200 p-2.5 rounded-lg flex flex-col gap-1">
                  <span className="font-semibold text-purple-700 flex items-center gap-1">
                    <span>🌙</span> {t.time_night}
                  </span>
                  {domainTimeClusters.night.length > 0 ? (
                    <div className="flex gap-1 flex-wrap mt-0.5">
                      {domainTimeClusters.night.map((ts, idx) => (
                        <span
                          key={idx}
                          className="bg-purple-50 text-purple-900 border border-purple-200 px-1.5 py-0.5 rounded font-mono"
                        >
                          {ts}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <span className="text-slate-400 italic text-3xs">
                      {isVi ? 'Không có truy cập' : 'No activity'}
                    </span>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* 2.5. CHART.JS HOURLY FREQUENCY FOR DOMAIN */}
          <div className="bg-slate-50 border border-slate-200/80 rounded-xl p-3.5 flex flex-col gap-2">
            <div className="flex justify-between items-center text-xs">
              <span className="font-bold text-slate-800 flex items-center gap-1.5">
                <BarChart3 size={14} className="text-sky-600" />
                <span>
                  {isVi
                    ? 'Biểu đồ tần suất truy vấn theo giờ của tên miền'
                    : 'Domain 24h Query Frequency Chart'}
                </span>
              </span>
              <span className="text-2xs text-slate-500 font-mono">
                {domainLogs.length} queries
              </span>
            </div>
            <div className="bg-white rounded-lg p-2.5 border border-slate-200/60">
              <DnsHourlyChart
                hourlyData={domainHourlyData}
                lang={lang}
                peakHour={-1}
                height={110}
                allowChartTypeToggle={false}
              />
            </div>
          </div>

          {/* 3. Query Log Table for Domain */}
          <div>
            <span className="text-xs font-bold text-slate-800 mb-2 block">
              {isVi ? '📋 Lịch sử các truy vấn chi tiết' : '📋 Detailed Query Log History'}
            </span>
            {isLoadingDetails ? (
              <div className="text-center py-10 text-slate-500">
                <RefreshCw size={24} className="animate-spin mx-auto mb-3 text-sky-600" />
                <p className="text-sm">
                  {isVi
                    ? 'Đang truy xuất chi tiết truy vấn của tên miền...'
                    : 'Loading query resolution details...'}
                </p>
              </div>
            ) : domainLogs.length === 0 ? (
              <div className="text-center py-10 text-slate-500 text-sm">
                <p>{t.no_matching_logs}</p>
              </div>
            ) : (
              <DataTable
                data={domainLogs}
                columns={rawLogColumns}
                pageSize={10}
                searchPlaceholder={t.placeholder_search_logs}
                emptyText={t.no_matching_logs}
                lang={lang}
                enableExport={true}
                exportFilename={`famshield_dns_${selectedDomain}`}
              />
            )}
          </div>
        </div>

        {/* Modal Footer */}
        <div className="px-6 py-3.5 border-t border-slate-100 flex justify-end bg-slate-50">
          <button
            onClick={onClose}
            className="bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 px-4 py-1.5 rounded-lg text-xs font-semibold cursor-pointer shadow-xs transition-all"
          >
            {isVi ? 'Đóng' : 'Close'}
          </button>
        </div>
      </div>
    </div>
  );
};
