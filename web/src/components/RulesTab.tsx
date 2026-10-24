import type { ColumnDef } from '@tanstack/react-table';
import {
  AlertCircle,
  Check,
  CheckCircle2,
  Cloud,
  Download,
  FileSpreadsheet,
  FileText,
  Globe,
  Laptop,
  Plus,
  RefreshCw,
  ShieldCheck,
  Smartphone,
  Trash2,
  Upload,
  X,
  XCircle,
} from 'lucide-react';
import type React from 'react';
import { useMemo, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type { CloudPolicyData, Language, LocalRuleRecord } from '../types';
import { useEffect } from 'react';
import { DataTable } from './DataTable';

interface RulesTabProps {
  rules: LocalRuleRecord[];
  lang: Language;
  onRefresh: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

interface ParsedImportRule {
  rule_id: string;
  rule_type: 'domain' | 'application';
  reason: string;
  quota?: number;
  enabled: boolean;
}

export const RulesTab: React.FC<RulesTabProps> = ({ rules, lang, onRefresh, onShowToast }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [newDomain, setNewDomain] = useState('');
  const [domainReason, setDomainReason] = useState('');
  const [isAddingDomain, setIsAddingDomain] = useState(false);

  const [newApp, setNewApp] = useState('');
  const [appQuota, setAppQuota] = useState('60');
  const [isAddingApp, setIsAddingApp] = useState(false);

  // Excel Import Preview Modal State
  const [importModalOpen, setImportModalOpen] = useState(false);
  const [previewRules, setPreviewRules] = useState<ParsedImportRule[]>([]);
  const [importFileName, setImportFileName] = useState('');
  const [isImporting, setIsImporting] = useState(false);

  const [cloudPolicy, setCloudPolicy] = useState<CloudPolicyData | null>(null);
  const [isPullingCloud, setIsPullingCloud] = useState(false);
  const [filterOrigin, setFilterOrigin] = useState<'all' | 'cloud' | 'local'>('all');

  useEffect(() => {
    api.getCloudPolicy().then(setCloudPolicy);
  }, []);

  const handlePullFromCloud = async () => {
    try {
      setIsPullingCloud(true);
      onShowToast(
        isVi ? 'Đang kéo chính sách mới nhất từ Cloud Server...' : 'Pulling policy from Cloud Server...',
        'info'
      );
      const res = await api.pullRulesFromCloud();
      if (res.success) {
        onShowToast(res.message, 'success');
        api.getCloudPolicy().then(setCloudPolicy);
        onRefresh();
      } else {
        onShowToast(isVi ? 'Không thể kéo chính sách' : 'Pull failed', 'error');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi kết nối máy chủ' : 'Network error during pull'), 'error');
    } finally {
      setIsPullingCloud(false);
    }
  };

  const domainRules = rules.filter((r) => r.rule_type === 'domain');
  const appRules = rules.filter((r) => r.rule_type === 'application');

  const filteredDomainRules = useMemo(() => {
    if (filterOrigin === 'cloud') {
      return domainRules.filter((r) => r.source === 'cloud');
    }
    if (filterOrigin === 'local') {
      return domainRules.filter((r) => r.source !== 'cloud');
    }
    return domainRules;
  }, [domainRules, filterOrigin]);

  const handleAddDomain = async (e: React.FormEvent) => {
    e.preventDefault();
    const d = newDomain.trim().toLowerCase();
    if (!d) return;

    setIsAddingDomain(true);
    try {
      const payload = JSON.stringify({
        action: 'BLOCK',
        reason: domainReason.trim() || 'local_guardian_rule',
      });

      await api.createRule({
        rule_id: d,
        rule_type: 'domain',
        payload_json: payload,
        enabled: true,
      });

      onShowToast(isVi ? `Đã thêm chặn tên miền: ${d}` : `Domain blocked: ${d}`, 'success');
      setNewDomain('');
      setDomainReason('');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi thêm quy tắc' : 'Failed to add rule'), 'error');
    } finally {
      setIsAddingDomain(false);
    }
  };

  const [isSyncingCloud, setIsSyncingCloud] = useState(false);

  const handleSyncToCloud = async () => {
    try {
      setIsSyncingCloud(true);
      onShowToast(
        isVi ? 'Đang đồng bộ quy tắc lên Cloud Server...' : 'Syncing rules to Cloud Server...',
        'info'
      );
      const res = await api.syncRulesToCloud();
      if (res.success) {
        onShowToast(
          isVi ? res.message : `Successfully synced ${res.synced_domains_count} domains to Cloud`,
          'success'
        );
      } else {
        onShowToast(isVi ? 'Đồng bộ thất bại' : 'Sync failed', 'error');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi kết nối máy chủ' : 'Network error during sync'), 'error');
    } finally {
      setIsSyncingCloud(false);
    }
  };

  const handleAddApp = async (e: React.FormEvent) => {
    e.preventDefault();
    const app = newApp.trim().toLowerCase();
    const minutes = parseInt(appQuota, 10) || 60;
    if (!app) return;

    setIsAddingApp(true);
    try {
      const payload = JSON.stringify({
        max_minutes_per_day: minutes,
        action: 'LIMIT',
      });

      await api.createRule({
        rule_id: app,
        rule_type: 'application',
        payload_json: payload,
        enabled: true,
      });

      onShowToast(
        isVi
          ? `Đã đặt giới hạn cho ứng dụng: ${app} (${minutes} phút/ngày)`
          : `App limit set: ${app} (${minutes} min/day)`,
        'success'
      );
      setNewApp('');
      setAppQuota('60');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi thêm giới hạn' : 'Failed to set app limit'), 'error');
    } finally {
      setIsAddingApp(false);
    }
  };

  const handleDeleteRule = async (ruleId: string) => {
    if (!confirm(isVi ? `Xác nhận xóa quy tắc "${ruleId}"?` : `Delete rule "${ruleId}"?`)) return;
    try {
      await api.deleteRule(ruleId);
      onShowToast(isVi ? `Đã xóa quy tắc: ${ruleId}` : `Deleted rule: ${ruleId}`, 'info');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi khi xóa quy tắc' : 'Delete failed'), 'error');
    }
  };

  const domainColumns = useMemo<ColumnDef<LocalRuleRecord>[]>(
    () => [
      {
        accessorKey: 'rule_id',
        header: t.th_target,
        cell: ({ row }) => {
          let reason = '';
          try {
            const p = JSON.parse(row.original.payload);
            reason = p.reason || '';
          } catch {}
          return (
            <div>
              <div className="font-semibold text-slate-900">{row.original.rule_id}</div>
              {reason && <div className="text-xs text-slate-500">{reason}</div>}
            </div>
          );
        },
      },
      {
        accessorKey: 'rule_type',
        header: t.th_rule_type,
        cell: () => (
          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-mono font-medium">
            DOMAIN
          </span>
        ),
      },
      {
        accessorKey: 'source',
        header: isVi ? 'Nguồn Gốc' : 'Origin',
        cell: ({ row }) => {
          const isCloud = row.original.source === 'cloud';
          return isCloud ? (
            <span className="inline-flex items-center gap-1 bg-sky-50 text-sky-700 border border-sky-200/90 px-2 py-0.5 rounded-full text-2xs font-bold tracking-tight">
              <Cloud size={11} className="text-sky-600" />
              <span>{isVi ? 'MÁY CHỦ' : 'CLOUD'}</span>
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded-full text-2xs font-semibold tracking-tight">
              <Laptop size={11} className="text-slate-600" />
              <span>{isVi ? 'TẠI CHỖ' : 'LOCAL'}</span>
            </span>
          );
        },
      },
      {
        accessorKey: 'created_at',
        header: t.th_date_added,
        cell: ({ getValue }) => (
          <span className="text-slate-500 text-xs font-mono">
            {new Date(getValue<string>()).toLocaleDateString(isVi ? 'vi-VN' : 'en-US')}
          </span>
        ),
      },
      {
        accessorKey: 'enabled',
        header: t.th_status,
        cell: ({ getValue }) => {
          const enabled = getValue<boolean>();
          return enabled ? (
            <span className="inline-flex items-center gap-1.5 text-emerald-700 font-semibold text-xs">
              <CheckCircle2 size={13} /> {isVi ? 'BẬT' : 'ENABLED'}
            </span>
          ) : (
            <span className="inline-flex items-center gap-1.5 text-rose-700 font-semibold text-xs">
              <XCircle size={13} /> {isVi ? 'TẮT' : 'DISABLED'}
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: () => <div className="text-right">{t.th_action_col}</div>,
        cell: ({ row }) => {
          const isCloud = row.original.source === 'cloud';
          return (
            <div className="text-right">
              {isCloud ? (
                <span
                  title={
                    isVi
                      ? 'Quy tắc từ Máy chủ Cloud. Để xóa, vui lòng quản trị trên Cloud Console của phụ huynh.'
                      : 'Managed by Cloud Server. Manage via Parent Cloud Console.'
                  }
                  className="text-slate-400 p-1.5 inline-flex items-center text-2xs font-medium cursor-help"
                >
                  <ShieldCheck size={14} className="text-sky-500 mr-1" />
                  <span>{isVi ? 'Từ Server' : 'Cloud'}</span>
                </span>
              ) : (
                <button
                  onClick={() => handleDeleteRule(row.original.rule_id)}
                  className="text-rose-500 hover:text-rose-700 p-1.5 rounded-md hover:bg-rose-50 cursor-pointer border-0 bg-transparent transition-colors"
                  title={isVi ? 'Xóa quy tắc' : 'Delete rule'}
                >
                  <Trash2 size={15} />
                </button>
              )}
            </div>
          );
        },
      },
    ],
    [t, isVi]
  );

  const appColumns = useMemo<ColumnDef<LocalRuleRecord>[]>(
    () => [
      {
        accessorKey: 'rule_id',
        header: t.th_app_name,
        cell: ({ getValue }) => (
          <span className="font-semibold text-slate-900">{getValue<string>()}</span>
        ),
      },
      {
        id: 'limit',
        header: t.th_daily_limit,
        accessorFn: (row) => {
          try {
            const p = JSON.parse(row.payload);
            return p.max_minutes_per_day || 60;
          } catch {
            return 60;
          }
        },
        cell: ({ getValue }) => (
          <span className="text-sky-700 font-bold text-xs bg-sky-50 border border-sky-200/80 px-2 py-0.5 rounded-md">
            {getValue<number>()} {isVi ? 'phút' : 'mins'}
          </span>
        ),
      },
      {
        accessorKey: 'created_at',
        header: t.th_date_added,
        cell: ({ getValue }) => (
          <span className="text-slate-500 text-xs font-mono">
            {new Date(getValue<string>()).toLocaleDateString(isVi ? 'vi-VN' : 'en-US')}
          </span>
        ),
      },
      {
        accessorKey: 'enabled',
        header: t.th_status,
        cell: ({ getValue }) => {
          const enabled = getValue<boolean>();
          return enabled ? (
            <span className="inline-flex items-center gap-1.5 text-emerald-700 font-semibold text-xs">
              <CheckCircle2 size={13} /> {isVi ? 'BẬT' : 'ENABLED'}
            </span>
          ) : (
            <span className="inline-flex items-center gap-1.5 text-rose-700 font-semibold text-xs">
              <XCircle size={13} /> {isVi ? 'TẮT' : 'DISABLED'}
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: () => <div className="text-right">{t.th_action_col}</div>,
        cell: ({ row }) => (
          <div className="text-right">
            <button
              onClick={() => handleDeleteRule(row.original.rule_id)}
              className="text-rose-500 hover:text-rose-700 p-1.5 rounded-md hover:bg-rose-50 cursor-pointer border-0 bg-transparent transition-colors"
              title={isVi ? 'Xóa hạn mức' : 'Delete app limit'}
            >
              <Trash2 size={15} />
            </button>
          </div>
        ),
      },
    ],
    [t, isVi]
  );

  // ================= EXCEL EXPORT =================
  const handleExportExcel = async () => {
    if (rules.length === 0) {
      onShowToast(t.no_rules_to_export, 'info');
      return;
    }

    try {
      const XLSX = await import('xlsx');
      const wb = XLSX.utils.book_new();

      // 1. Domain Blocklist Sheet
      const domainData = domainRules.map((r) => {
        let reason = '';
        try {
          const p = JSON.parse(r.payload);
          reason = p.reason || '';
        } catch {}

        return {
          [isVi ? 'Tên Miền (Domain)' : 'Domain']: r.rule_id,
          [isVi ? 'Loại Quy Tắc' : 'Type']: 'DOMAIN',
          [isVi ? 'Lý Do Chặn' : 'Block Reason']: reason,
          [isVi ? 'Trạng Thái' : 'Status']: r.enabled
            ? isVi
              ? 'BẬT'
              : 'ENABLED'
            : isVi
              ? 'TẮT'
              : 'DISABLED',
          [isVi ? 'Ngày Tạo' : 'Created At']: new Date(r.created_at).toLocaleString(
            isVi ? 'vi-VN' : 'en-US'
          ),
        };
      });

      const wsDomains = XLSX.utils.json_to_sheet(domainData.length > 0 ? domainData : [{}]);
      XLSX.utils.book_append_sheet(
        wb,
        wsDomains,
        isVi ? 'Danh Sách Chặn Tên Miền' : 'Domain Blocklist'
      );

      // 2. App Limits Sheet
      const appData = appRules.map((r) => {
        let limit = 60;
        try {
          const p = JSON.parse(r.payload);
          limit = p.max_minutes_per_day || 60;
        } catch {}

        return {
          [isVi ? 'Tên Ứng Dụng (.exe)' : 'Application (.exe)']: r.rule_id,
          [isVi ? 'Loại Quy Tắc' : 'Type']: 'APPLICATION',
          [isVi ? 'Hạn Mức (Phút/Ngày)' : 'Daily Quota (Mins)']: limit,
          [isVi ? 'Trạng Thái' : 'Status']: r.enabled
            ? isVi
              ? 'BẬT'
              : 'ENABLED'
            : isVi
              ? 'TẮT'
              : 'DISABLED',
          [isVi ? 'Ngày Tạo' : 'Created At']: new Date(r.created_at).toLocaleString(
            isVi ? 'vi-VN' : 'en-US'
          ),
        };
      });

      const wsApps = XLSX.utils.json_to_sheet(appData.length > 0 ? appData : [{}]);
      XLSX.utils.book_append_sheet(wb, wsApps, isVi ? 'Hạn Mức Ứng Dụng' : 'App Limits');

      // Download file
      const dateStr = new Date().toISOString().slice(0, 10);
      XLSX.writeFile(wb, `FamShield_Blacklist_${dateStr}.xlsx`);
      onShowToast(t.export_success, 'success');
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi khi xuất file Excel' : 'Export failed'), 'error');
    }
  };

  // ================= DOWNLOAD SAMPLE TEMPLATE =================
  const handleDownloadTemplate = async () => {
    try {
      const XLSX = await import('xlsx');
      const wb = XLSX.utils.book_new();

      const sampleDomains = [
        {
          'Tên Miền (Domain)': 'tiktok.com',
          'Loại Quy Tắc': 'DOMAIN',
          'Lý Do Chặn': 'Mạng xã hội gây xao nhãng học tập',
          'Trạng Thái': 'BẬT',
        },
        {
          'Tên Miền (Domain)': 'roblox.com',
          'Loại Quy Tắc': 'DOMAIN',
          'Lý Do Chặn': 'Trang web game online',
          'Trạng Thái': 'BẬT',
        },
        {
          'Tên Miền (Domain)': 'bet365.com',
          'Loại Quy Tắc': 'DOMAIN',
          'Lý Do Chặn': 'Cờ bạc trực tuyến độc hại',
          'Trạng Thái': 'BẬT',
        },
      ];

      const sampleApps = [
        {
          'Tên Ứng Dụng (.exe)': 'roblox.exe',
          'Loại Quy Tắc': 'APPLICATION',
          'Hạn Mức (Phút/Ngày)': 60,
          'Trạng Thái': 'BẬT',
        },
        {
          'Tên Ứng Dụng (.exe)': 'genshinimpact.exe',
          'Loại Quy Tắc': 'APPLICATION',
          'Hạn Mức (Phút/Ngày)': 90,
          'Trạng Thái': 'BẬT',
        },
      ];

      const wsDom = XLSX.utils.json_to_sheet(sampleDomains);
      const wsApp = XLSX.utils.json_to_sheet(sampleApps);

      XLSX.utils.book_append_sheet(wb, wsDom, 'Danh Sách Chặn Tên Miền');
      XLSX.utils.book_append_sheet(wb, wsApp, 'Hạn Mức Ứng Dụng');

      XLSX.writeFile(wb, 'FamShield_Blacklist_Template.xlsx');
      onShowToast(isVi ? 'Đã tải xuống file mẫu Excel!' : 'Sample template downloaded!', 'success');
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi tải file mẫu' : 'Failed to download template'), 'error');
    }
  };

  // ================= EXCEL IMPORT =================
  const handleFileSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImportFileName(file.name);
    const reader = new FileReader();

    reader.onload = async (evt) => {
      try {
        const XLSX = await import('xlsx');
        const buffer = evt.target?.result as ArrayBuffer;
        const wb = XLSX.read(buffer, { type: 'array' });
        const parsed: ParsedImportRule[] = [];

        for (const sheetName of wb.SheetNames) {
          const sheet = wb.Sheets[sheetName];
          const rows: Record<string, unknown>[] = XLSX.utils.sheet_to_json(sheet);

          for (const row of rows) {
            // Find key case-insensitively
            const keys = Object.keys(row);
            const findVal = (keywords: string[]) => {
              const matchedKey = keys.find((k) =>
                keywords.some((w) => k.toLowerCase().includes(w))
              );
              return matchedKey ? row[matchedKey] : undefined;
            };

            const targetVal = findVal([
              'tên miền',
              'domain',
              'target',
              'ứng dụng',
              'application',
              'file',
              'app',
              'rule_id',
            ]);
            if (!targetVal) continue;

            const target = String(targetVal).trim().toLowerCase();
            if (!target || target === 'undefined') continue;

            const typeVal = findVal(['loại', 'type']);
            const reasonVal = findVal(['lý do', 'reason', 'ghi chú', 'note']);
            const quotaVal = findVal(['hạn mức', 'quota', 'phút', 'minute']);
            const statusVal = findVal(['trạng thái', 'status', 'enable']);

            const isApp =
              target.endsWith('.exe') || (typeVal && String(typeVal).toLowerCase().includes('app'));

            const ruleType = isApp ? 'application' : 'domain';
            const reason = reasonVal ? String(reasonVal).trim() : 'imported_from_excel';
            const quota = quotaVal ? parseInt(String(quotaVal), 10) || 60 : 60;
            const enabled =
              statusVal !== undefined
                ? !['tắt', 'tat', 'disabled', 'false', '0', 'off'].includes(
                    String(statusVal).trim().toLowerCase()
                  )
                : true;

            // Avoid duplicates in preview
            if (!parsed.some((p) => p.rule_id === target)) {
              parsed.push({
                rule_id: target,
                rule_type: ruleType,
                reason,
                quota: isApp ? quota : undefined,
                enabled,
              });
            }
          }
        }

        if (parsed.length === 0) {
          onShowToast(t.invalid_excel_file, 'error');
          return;
        }

        setPreviewRules(parsed);
        setImportModalOpen(true);
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        onShowToast(msg || t.invalid_excel_file, 'error');
      } finally {
        if (fileInputRef.current) fileInputRef.current.value = '';
      }
    };

    reader.readAsArrayBuffer(file);
  };

  const handleConfirmImport = async () => {
    if (previewRules.length === 0) return;

    setIsImporting(true);
    try {
      const bulkPayload = previewRules.map((r) => {
        let payload_json = '';
        if (r.rule_type === 'domain') {
          payload_json = JSON.stringify({
            action: 'BLOCK',
            reason: r.reason || 'excel_imported_rule',
          });
        } else {
          payload_json = JSON.stringify({
            max_minutes_per_day: r.quota || 60,
            action: 'LIMIT',
          });
        }

        return {
          rule_id: r.rule_id,
          rule_type: r.rule_type,
          payload_json,
          enabled: r.enabled,
        };
      });

      const res = await api.createRulesBulk(bulkPayload);
      onShowToast(
        t.import_success.replace('{count}', String(res.imported || previewRules.length)),
        'success'
      );
      setImportModalOpen(false);
      setPreviewRules([]);
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi nhập quy tắc' : 'Import failed'), 'error');
    } finally {
      setIsImporting(false);
    }
  };

  const previewDomainsCount = previewRules.filter((r) => r.rule_type === 'domain').length;
  const previewAppsCount = previewRules.filter((r) => r.rule_type === 'application').length;

  const cloudDomainCount = domainRules.filter((r) => r.source === 'cloud').length;
  const localDomainCount = domainRules.filter((r) => r.source !== 'cloud').length;

  return (
    <div className="flex flex-col gap-6">
      {/* 0. CLOUD SERVER POLICY OVERVIEW CARD */}
      <div className="bg-gradient-to-r from-sky-950 via-slate-900 to-indigo-950 text-white rounded-2xl p-5 sm:p-6 shadow-xs border border-sky-800/40 flex flex-col gap-4">
        <div className="flex justify-between items-start flex-wrap gap-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-sky-500/20 border border-sky-400/30 flex items-center justify-center text-sky-300 shrink-0">
              <Cloud size={22} />
            </div>
            <div>
              <div className="flex items-center gap-2 flex-wrap">
                <span className="font-bold text-base tracking-tight text-white">
                  {isVi ? 'Chính Sách Bảo Vệ Từ Máy Chủ (Cloud Policy)' : 'Cloud Server Security Policy'}
                </span>
                <span className="bg-sky-400/20 text-sky-200 border border-sky-400/30 px-2 py-0.5 rounded-md text-2xs font-mono font-bold">
                  v{cloudPolicy?.version || 1}
                </span>
              </div>
              <p className="text-xs text-sky-200/80 mt-0.5">
                {isVi
                  ? 'Quy tắc lọc web & chặn tên miền tập trung áp xuống từ Cloud Server qua kênh NATS WebSocket'
                  : 'Centralized web filter policy pushed down from Cloud Server via NATS WebSocket'}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 flex-wrap">
            <button
              type="button"
              onClick={handlePullFromCloud}
              disabled={isPullingCloud}
              className="inline-flex items-center gap-1.5 bg-sky-500 hover:bg-sky-400 text-slate-950 font-bold px-3.5 py-2 rounded-xl text-xs cursor-pointer shadow-xs transition-all border-0 disabled:opacity-50"
            >
              <RefreshCw size={13} className={isPullingCloud ? 'animate-spin' : ''} />
              <span>{isVi ? 'Kéo từ Cloud Server' : 'Pull from Cloud'}</span>
            </button>
            <button
              type="button"
              onClick={handleSyncToCloud}
              disabled={isSyncingCloud}
              className="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 text-white font-semibold px-3.5 py-2 rounded-xl text-xs cursor-pointer border border-white/20 transition-all disabled:opacity-50"
            >
              <Upload size={13} />
              <span>{isVi ? 'Đẩy quy tắc lên Cloud' : 'Sync to Cloud'}</span>
            </button>
          </div>
        </div>

        {/* Category Badges */}
        <div className="flex items-center gap-2 flex-wrap pt-3 border-t border-sky-800/50 text-2xs font-medium">
          <span className="text-sky-300 font-semibold">{isVi ? 'Danh mục bảo vệ:' : 'Protected categories:'}</span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.block_adult ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            🔞 {isVi ? 'Người Lớn' : 'Adult'} {cloudPolicy?.block_adult ? '✓' : '✗'}
          </span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.block_gambling ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            🎲 {isVi ? 'Cờ Bạc' : 'Gambling'} {cloudPolicy?.block_gambling ? '✓' : '✗'}
          </span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.block_violence ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            ⚔️ {isVi ? 'Bạo Lực' : 'Violence'} {cloudPolicy?.block_violence ? '✓' : '✗'}
          </span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.block_social ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            💬 {isVi ? 'Mạng Xã Hội' : 'Social'} {cloudPolicy?.block_social ? '✓' : '✗'}
          </span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.block_gaming ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            🎮 {isVi ? 'Game' : 'Gaming'} {cloudPolicy?.block_gaming ? '✓' : '✗'}
          </span>
          <span className={`px-2.5 py-1 rounded-lg border ${cloudPolicy?.safesearch_enabled ? 'bg-emerald-500/20 text-emerald-200 border-emerald-400/30 font-bold' : 'bg-white/5 text-slate-400 border-white/10'}`}>
            🔍 SafeSearch {cloudPolicy?.safesearch_enabled ? '✓' : '✗'}
          </span>
        </div>
      </div>

      {/* 1. EXCEL IMPORT / EXPORT ACTION BAR */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-5 sm:p-6 flex items-center justify-between flex-wrap gap-4 shadow-xs">
        <div className="flex items-center gap-3.5">
          <div className="bg-emerald-50 text-emerald-600 border border-emerald-100 p-2.5 rounded-xl flex items-center justify-center">
            <FileSpreadsheet size={22} className="text-emerald-600" />
          </div>
          <div>
            <div className="font-bold text-slate-900 text-base">
              {isVi ? 'Quản Lý Danh Sách Đen Bằng Excel' : 'Excel Blacklist Import & Export'}
            </div>
            <div className="text-xs text-slate-500 mt-0.5">
              {isVi
                ? 'Nhập & xuất danh sách tên miền chặn và ứng dụng giới hạn qua định dạng bảng tính (.xlsx, .csv)'
                : 'Bulk import & export domain blocklist and app limits via spreadsheets (.xlsx, .csv)'}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2.5 flex-wrap">
          {/* Download Sample Template */}
          <button
            type="button"
            onClick={handleDownloadTemplate}
            className="bg-white hover:bg-slate-50 text-slate-700 border border-slate-200 px-3.5 py-2 rounded-xl text-xs font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 transition-all"
          >
            <FileText size={14} className="text-sky-600" />
            <span>{t.btn_download_template}</span>
          </button>

          {/* Import Button */}
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="bg-emerald-600 hover:bg-emerald-700 text-white px-4 py-2 rounded-xl text-xs font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 transition-all border-0"
          >
            <Upload size={14} />
            <span>{t.btn_import_excel}</span>
          </button>
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileSelected}
            accept=".xlsx,.xls,.csv"
            style={{ display: 'none' }}
          />

          {/* Export Button */}
          <button
            type="button"
            onClick={handleExportExcel}
            className="bg-white hover:bg-slate-50 text-slate-700 border border-slate-200 px-4 py-2 rounded-xl text-xs font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 transition-all"
          >
            <Download size={14} className="text-slate-600" />
            <span>{t.btn_export_excel}</span>
          </button>
        </div>
      </div>

      {/* 2. DOMAIN BLOCKLIST SECTION */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs flex flex-col gap-5">
        <div className="flex justify-between items-start flex-wrap gap-3">
          <div>
            <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <Globe size={18} className="text-sky-600" />
              {t.title_domain_block}
            </h2>
            <p className="text-xs text-slate-500 mt-0.5">{t.desc_domain_block}</p>
          </div>
          <button
            onClick={handleSyncToCloud}
            disabled={isSyncingCloud}
            className="flex items-center gap-1.5 bg-sky-600 hover:bg-sky-700 text-white px-3.5 py-2 rounded-lg font-semibold text-xs cursor-pointer shadow-xs transition-all border-0 disabled:opacity-60"
          >
            <Cloud size={14} />
            {isSyncingCloud
              ? isVi
                ? 'Đang đồng bộ...'
                : 'Syncing...'
              : isVi
                ? 'Đồng bộ lên Cloud'
                : 'Sync to Cloud'}
          </button>
        </div>

        {/* Add Form */}
        <form onSubmit={handleAddDomain} className="flex gap-2.5 flex-wrap">
          <input
            type="text"
            placeholder={t.placeholder_domain}
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
            className="flex-2 min-w-[200px] bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
          />
          <input
            type="text"
            placeholder={t.placeholder_reason}
            value={domainReason}
            onChange={(e) => setDomainReason(e.target.value)}
            className="flex-2 min-w-[200px] bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
          />
          <button
            type="submit"
            disabled={isAddingDomain}
            className="bg-sky-600 hover:bg-sky-700 text-white px-4 py-2 rounded-lg text-sm font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 border-0 transition-all"
          >
            <Plus size={15} />
            <span>{t.btn_add_domain}</span>
          </button>
        </form>

        {/* Table */}
        <DataTable
          data={domainRules}
          columns={domainColumns}
          pageSize={10}
          searchPlaceholder={isVi ? 'Tìm kiếm tên miền bị chặn...' : 'Search blocked domains...'}
          emptyText={t.no_domain_rules}
          lang={lang}
        />
      </div>

      {/* 3. APPLICATION QUOTAS SECTION */}
      <div className="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs flex flex-col gap-5">
        <div>
          <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
            <Smartphone size={18} className="text-emerald-600" />
            {t.title_app_limits}
          </h2>
          <p className="text-xs text-slate-500 mt-0.5">{t.desc_app_limits}</p>
        </div>

        {/* Add Form */}
        <form onSubmit={handleAddApp} className="flex gap-2.5 flex-wrap">
          <input
            type="text"
            placeholder={t.placeholder_app}
            value={newApp}
            onChange={(e) => setNewApp(e.target.value)}
            className="flex-3 min-w-[200px] bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
          />
          <input
            type="number"
            placeholder={t.placeholder_quota}
            value={appQuota}
            onChange={(e) => setAppQuota(e.target.value)}
            className="w-32 bg-white border border-slate-200 text-slate-900 px-3.5 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 shadow-xs placeholder-slate-400"
          />
          <button
            type="submit"
            disabled={isAddingApp}
            className="bg-emerald-600 hover:bg-emerald-700 text-white px-4 py-2 rounded-lg text-sm font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 border-0 transition-all"
          >
            <Plus size={15} />
            <span>{t.btn_add_app}</span>
          </button>
        </form>

        {/* Table */}
        <DataTable
          data={appRules}
          columns={appColumns}
          pageSize={10}
          searchPlaceholder={isVi ? 'Tìm kiếm ứng dụng giới hạn...' : 'Search limited apps...'}
          emptyText={t.no_app_rules}
          lang={lang}
        />
      </div>

      {/* 4. EXCEL IMPORT PREVIEW MODAL */}
      {importModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="max-w-2xl w-full bg-white border border-slate-200 rounded-2xl p-6 shadow-2xl max-h-[85vh] flex flex-col">
            {/* Header */}
            <div className="flex justify-between items-start mb-4">
              <div>
                <h3 className="text-lg font-bold text-slate-900 flex items-center gap-2">
                  <FileSpreadsheet size={20} className="text-emerald-600" />
                  {t.import_modal_title}
                </h3>
                <p className="text-xs text-slate-500 mt-1">
                  {t.import_modal_desc
                    .replace('{domains}', String(previewDomainsCount))
                    .replace('{apps}', String(previewAppsCount))}
                  {importFileName && (
                    <span className="text-sky-600 font-medium"> ({importFileName})</span>
                  )}
                </p>
              </div>

              <button
                onClick={() => setImportModalOpen(false)}
                className="w-8 h-8 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 flex items-center justify-center cursor-pointer transition-colors border-0"
              >
                <X size={16} />
              </button>
            </div>

            {/* Preview Table Scrollable */}
            <div className="flex-1 overflow-y-auto border border-slate-200 rounded-xl mb-5">
              <table className="w-full border-collapse text-left text-xs">
                <thead className="sticky top-0 bg-slate-50 border-b border-slate-200 text-slate-700 font-semibold">
                  <tr>
                    <th className="py-2.5 px-3">{isVi ? 'Mục Tiêu' : 'Target'}</th>
                    <th className="py-2.5 px-3">{isVi ? 'Loại' : 'Type'}</th>
                    <th className="py-2.5 px-3">
                      {isVi ? 'Thông Số / Lý Do' : 'Details / Reason'}
                    </th>
                    <th className="py-2.5 px-3">{isVi ? 'Trạng Thái' : 'Status'}</th>
                  </tr>
                </thead>
                <tbody>
                  {previewRules.map((rule, idx) => (
                    <tr
                      key={rule.rule_id || `preview-${idx}`}
                      className="border-b border-slate-100 hover:bg-slate-50/50 text-slate-800"
                    >
                      <td className="py-2.5 px-3 font-semibold text-slate-900">{rule.rule_id}</td>
                      <td className="py-2.5 px-3">
                        <span
                          className={`px-2 py-0.5 rounded text-2xs font-bold border ${
                            rule.rule_type === 'domain'
                              ? 'bg-sky-50 text-sky-700 border-sky-200'
                              : 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          }`}
                        >
                          {rule.rule_type.toUpperCase()}
                        </span>
                      </td>
                      <td className="py-2.5 px-3 text-slate-600">
                        {rule.rule_type === 'application'
                          ? `${rule.quota || 60} ${isVi ? 'phút/ngày' : 'mins/day'}`
                          : rule.reason}
                      </td>
                      <td className="py-2.5 px-3">
                        <span
                          className={`text-xs font-semibold ${rule.enabled ? 'text-emerald-700' : 'text-rose-700'}`}
                        >
                          {rule.enabled ? (isVi ? 'BẬT' : 'ENABLED') : isVi ? 'TẮT' : 'DISABLED'}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Modal Actions */}
            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={() => setImportModalOpen(false)}
                disabled={isImporting}
                className="bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 px-4 py-2 rounded-xl text-xs font-semibold cursor-pointer shadow-xs transition-all"
              >
                {t.import_cancel_btn}
              </button>

              <button
                type="button"
                onClick={handleConfirmImport}
                disabled={isImporting}
                className="bg-emerald-600 hover:bg-emerald-700 text-white px-4 py-2 rounded-xl text-xs font-semibold cursor-pointer shadow-xs flex items-center gap-1.5 transition-all border-0 disabled:opacity-60"
              >
                <Check size={14} />
                <span>
                  {isImporting
                    ? isVi
                      ? 'Đang nhập...'
                      : 'Importing...'
                    : `${t.import_confirm_btn} (${previewRules.length})`}
                </span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
