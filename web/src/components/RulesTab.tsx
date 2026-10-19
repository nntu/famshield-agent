import React, { useState, useRef, useMemo } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import {
  Globe,
  Smartphone,
  Plus,
  Trash2,
  CheckCircle2,
  XCircle,
  FileSpreadsheet,
  Upload,
  Download,
  FileText,
  X,
  AlertCircle,
  Check,
} from 'lucide-react';
import * as XLSX from 'xlsx';
import { Language, LocalRuleRecord } from '../types';
import { translations } from '../i18n';
import { api } from '../api';
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

export const RulesTab: React.FC<RulesTabProps> = ({
  rules,
  lang,
  onRefresh,
  onShowToast,
}) => {
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

  const domainRules = rules.filter((r) => r.rule_type === 'domain');
  const appRules = rules.filter((r) => r.rule_type === 'application');

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
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi thêm quy tắc' : 'Failed to add rule'), 'error');
    } finally {
      setIsAddingDomain(false);
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
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi thêm giới hạn' : 'Failed to set app limit'), 'error');
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
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi khi xóa quy tắc' : 'Delete failed'), 'error');
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
              <div style={{ fontWeight: 600, color: '#f8fafc' }}>{row.original.rule_id}</div>
              {reason && <div style={{ fontSize: '0.75rem', color: '#64748b' }}>{reason}</div>}
            </div>
          );
        },
      },
      {
        accessorKey: 'rule_type',
        header: t.th_rule_type,
        cell: () => (
          <span style={{ background: '#1e293b', color: '#cbd5e1', padding: '3px 8px', borderRadius: 4, fontSize: '0.75rem', fontWeight: 700 }}>
            DOMAIN
          </span>
        ),
      },
      {
        accessorKey: 'created_at',
        header: t.th_date_added,
        cell: ({ getValue }) => (
          <span style={{ color: '#94a3b8' }}>
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
            <span style={{ color: '#34d399', display: 'flex', alignItems: 'center', gap: 4, fontWeight: 600 }}>
              <CheckCircle2 size={14} /> {isVi ? 'BẬT' : 'ENABLED'}
            </span>
          ) : (
            <span style={{ color: '#f87171', display: 'flex', alignItems: 'center', gap: 4, fontWeight: 600 }}>
              <XCircle size={14} /> {isVi ? 'TẮT' : 'DISABLED'}
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: () => <div style={{ textAlign: 'right' }}>{t.th_action_col}</div>,
        cell: ({ row }) => (
          <div style={{ textAlign: 'right' }}>
            <button
              onClick={() => handleDeleteRule(row.original.rule_id)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#f87171',
                cursor: 'pointer',
                padding: 4,
              }}
              title={isVi ? 'Xóa quy tắc' : 'Delete rule'}
            >
              <Trash2 size={16} />
            </button>
          </div>
        ),
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
          <span style={{ color: '#f8fafc', fontWeight: 600 }}>{getValue<string>()}</span>
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
          <span style={{ color: '#38bdf8', fontWeight: 700 }}>
            {getValue<number>()} {isVi ? 'phút' : 'mins'}
          </span>
        ),
      },
      {
        accessorKey: 'created_at',
        header: t.th_date_added,
        cell: ({ getValue }) => (
          <span style={{ color: '#94a3b8' }}>
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
            <span style={{ color: '#34d399', display: 'flex', alignItems: 'center', gap: 4, fontWeight: 600 }}>
              <CheckCircle2 size={14} /> {isVi ? 'BẬT' : 'ENABLED'}
            </span>
          ) : (
            <span style={{ color: '#f87171', display: 'flex', alignItems: 'center', gap: 4, fontWeight: 600 }}>
              <XCircle size={14} /> {isVi ? 'TẮT' : 'DISABLED'}
            </span>
          );
        },
      },
      {
        id: 'actions',
        header: () => <div style={{ textAlign: 'right' }}>{t.th_action_col}</div>,
        cell: ({ row }) => (
          <div style={{ textAlign: 'right' }}>
            <button
              onClick={() => handleDeleteRule(row.original.rule_id)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#f87171',
                cursor: 'pointer',
                padding: 4,
              }}
              title={isVi ? 'Xóa hạn mức' : 'Delete app limit'}
            >
              <Trash2 size={16} />
            </button>
          </div>
        ),
      },
    ],
    [t, isVi]
  );

  // ================= EXCEL EXPORT =================
  const handleExportExcel = () => {
    if (rules.length === 0) {
      onShowToast(t.no_rules_to_export, 'info');
      return;
    }

    try {
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
          [isVi ? 'Trạng Thái' : 'Status']: r.enabled ? (isVi ? 'BẬT' : 'ENABLED') : (isVi ? 'TẮT' : 'DISABLED'),
          [isVi ? 'Ngày Tạo' : 'Created At']: new Date(r.created_at).toLocaleString(isVi ? 'vi-VN' : 'en-US'),
        };
      });

      const wsDomains = XLSX.utils.json_to_sheet(domainData.length > 0 ? domainData : [{}]);
      XLSX.utils.book_append_sheet(wb, wsDomains, isVi ? 'Danh Sách Chặn Tên Miền' : 'Domain Blocklist');

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
          [isVi ? 'Trạng Thái' : 'Status']: r.enabled ? (isVi ? 'BẬT' : 'ENABLED') : (isVi ? 'TẮT' : 'DISABLED'),
          [isVi ? 'Ngày Tạo' : 'Created At']: new Date(r.created_at).toLocaleString(isVi ? 'vi-VN' : 'en-US'),
        };
      });

      const wsApps = XLSX.utils.json_to_sheet(appData.length > 0 ? appData : [{}]);
      XLSX.utils.book_append_sheet(wb, wsApps, isVi ? 'Hạn Mức Ứng Dụng' : 'App Limits');

      // Download file
      const dateStr = new Date().toISOString().slice(0, 10);
      XLSX.writeFile(wb, `FamShield_Blacklist_${dateStr}.xlsx`);
      onShowToast(t.export_success, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi khi xuất file Excel' : 'Export failed'), 'error');
    }
  };

  // ================= DOWNLOAD SAMPLE TEMPLATE =================
  const handleDownloadTemplate = () => {
    try {
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
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi tải file mẫu' : 'Failed to download template'), 'error');
    }
  };

  // ================= EXCEL IMPORT =================
  const handleFileSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImportFileName(file.name);
    const reader = new FileReader();

    reader.onload = (evt) => {
      try {
        const buffer = evt.target?.result as ArrayBuffer;
        const wb = XLSX.read(buffer, { type: 'array' });
        const parsed: ParsedImportRule[] = [];

        for (const sheetName of wb.SheetNames) {
          const sheet = wb.Sheets[sheetName];
          const rows: any[] = XLSX.utils.sheet_to_json(sheet);

          for (const row of rows) {
            // Find key case-insensitively
            const keys = Object.keys(row);
            const findVal = (keywords: string[]) => {
              const matchedKey = keys.find((k) =>
                keywords.some((w) => k.toLowerCase().includes(w))
              );
              return matchedKey ? row[matchedKey] : undefined;
            };

            const targetVal = findVal(['tên miền', 'domain', 'target', 'ứng dụng', 'application', 'file', 'app', 'rule_id']);
            if (!targetVal) continue;

            const target = String(targetVal).trim().toLowerCase();
            if (!target || target === 'undefined') continue;

            const typeVal = findVal(['loại', 'type']);
            const reasonVal = findVal(['lý do', 'reason', 'ghi chú', 'note']);
            const quotaVal = findVal(['hạn mức', 'quota', 'phút', 'minute']);
            const statusVal = findVal(['trạng thái', 'status', 'enable']);

            const isApp =
              target.endsWith('.exe') ||
              (typeVal && String(typeVal).toLowerCase().includes('app'));

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
      } catch (err: any) {
        onShowToast(err.message || t.invalid_excel_file, 'error');
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
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi nhập quy tắc' : 'Import failed'), 'error');
    } finally {
      setIsImporting(false);
    }
  };

  const previewDomainsCount = previewRules.filter((r) => r.rule_type === 'domain').length;
  const previewAppsCount = previewRules.filter((r) => r.rule_type === 'application').length;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
      {/* 1. EXCEL IMPORT / EXPORT ACTION BAR */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: '16px 22px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 16,
          boxShadow: '0 4px 20px rgba(0,0,0,0.2)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
          <div
            style={{
              background: 'rgba(16, 185, 129, 0.15)',
              border: '1px solid rgba(16, 185, 129, 0.35)',
              borderRadius: 12,
              padding: '10px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <FileSpreadsheet size={24} color="#10b981" />
          </div>
          <div>
            <div style={{ fontWeight: 800, fontSize: '1rem', color: '#f8fafc' }}>
              {isVi ? 'Quản Lý Danh Sách Đen Bằng Excel' : 'Excel Blacklist Import & Export'}
            </div>
            <div style={{ fontSize: '0.8rem', color: '#94a3b8', marginTop: 2 }}>
              {isVi
                ? 'Nhập & xuất danh sách tên miền chặn và ứng dụng giới hạn qua định dạng bảng tính (.xlsx, .csv)'
                : 'Bulk import & export domain blocklist and app limits via spreadsheets (.xlsx, .csv)'}
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
          {/* Download Sample Template */}
          <button
            type="button"
            onClick={handleDownloadTemplate}
            style={{
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              padding: '9px 14px',
              borderRadius: 10,
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              transition: 'all 0.2s',
            }}
          >
            <FileText size={15} />
            <span>{t.btn_download_template}</span>
          </button>

          {/* Import Button */}
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            style={{
              background: 'linear-gradient(135deg, #059669, #10b981)',
              color: '#fff',
              border: 'none',
              padding: '9px 16px',
              borderRadius: 10,
              fontSize: '0.85rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              boxShadow: '0 4px 12px rgba(16, 185, 129, 0.25)',
              transition: 'all 0.2s',
            }}
          >
            <Upload size={16} />
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
            style={{
              background: '#1e293b',
              color: '#f8fafc',
              border: '1px solid #334155',
              padding: '9px 16px',
              borderRadius: 10,
              fontSize: '0.85rem',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              transition: 'all 0.2s',
            }}
          >
            <Download size={16} />
            <span>{t.btn_export_excel}</span>
          </button>
        </div>
      </div>

      {/* 2. DOMAIN BLOCKLIST SECTION */}
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 24,
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
            <Globe size={20} color="#38bdf8" />
            {t.title_domain_block}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_domain_block}
          </p>
        </div>

        {/* Add Form */}
        <form onSubmit={handleAddDomain} style={{ display: 'flex', gap: 12, marginBottom: 24, flexWrap: 'wrap' }}>
          <input
            type="text"
            placeholder={t.placeholder_domain}
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
            style={{
              flex: 2,
              minWidth: 200,
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#fff',
              padding: '10px 14px',
              borderRadius: 8,
              fontSize: '0.9rem',
              outline: 'none',
            }}
          />
          <input
            type="text"
            placeholder={t.placeholder_reason}
            value={domainReason}
            onChange={(e) => setDomainReason(e.target.value)}
            style={{
              flex: 2,
              minWidth: 200,
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#fff',
              padding: '10px 14px',
              borderRadius: 8,
              fontSize: '0.9rem',
              outline: 'none',
            }}
          />
          <button
            type="submit"
            disabled={isAddingDomain}
            style={{
              background: '#38bdf8',
              color: '#070b14',
              border: 'none',
              padding: '10px 20px',
              borderRadius: 8,
              fontSize: '0.9rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
            }}
          >
            <Plus size={16} />
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
      <div
        style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 24,
        }}
      >
        <div style={{ marginBottom: 20 }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
            <Smartphone size={20} color="#34d399" />
            {t.title_app_limits}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
            {t.desc_app_limits}
          </p>
        </div>

        {/* Add Form */}
        <form onSubmit={handleAddApp} style={{ display: 'flex', gap: 12, marginBottom: 24, flexWrap: 'wrap' }}>
          <input
            type="text"
            placeholder={t.placeholder_app}
            value={newApp}
            onChange={(e) => setNewApp(e.target.value)}
            style={{
              flex: 3,
              minWidth: 200,
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#fff',
              padding: '10px 14px',
              borderRadius: 8,
              fontSize: '0.9rem',
              outline: 'none',
            }}
          />
          <input
            type="number"
            placeholder={t.placeholder_quota}
            value={appQuota}
            onChange={(e) => setAppQuota(e.target.value)}
            style={{
              flex: 1,
              minWidth: 120,
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#fff',
              padding: '10px 14px',
              borderRadius: 8,
              fontSize: '0.9rem',
              outline: 'none',
            }}
          />
          <button
            type="submit"
            disabled={isAddingApp}
            style={{
              background: '#34d399',
              color: '#070b14',
              border: 'none',
              padding: '10px 20px',
              borderRadius: 8,
              fontSize: '0.9rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
            }}
          >
            <Plus size={16} />
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
        <div
          style={{
            position: 'fixed',
            inset: 0,
            zIndex: 150,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'rgba(0, 0, 0, 0.75)',
            backdropFilter: 'blur(8px)',
            padding: 20,
          }}
        >
          <div
            style={{
              maxWidth: 750,
              width: '100%',
              background: '#0f172a',
              border: '1px solid #334155',
              borderRadius: 20,
              padding: 28,
              boxShadow: '0 25px 60px rgba(0,0,0,0.6)',
              maxHeight: '85vh',
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            {/* Header */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 16 }}>
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 10 }}>
                  <FileSpreadsheet size={22} color="#10b981" />
                  {t.import_modal_title}
                </h3>
                <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 4 }}>
                  {t.import_modal_desc
                    .replace('{domains}', String(previewDomainsCount))
                    .replace('{apps}', String(previewAppsCount))}
                  {importFileName && <span style={{ color: '#38bdf8' }}> ({importFileName})</span>}
                </p>
              </div>

              <button
                onClick={() => setImportModalOpen(false)}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: '#94a3b8',
                  cursor: 'pointer',
                  padding: 4,
                }}
              >
                <X size={20} />
              </button>
            </div>

            {/* Preview Table Scrollable */}
            <div style={{ flex: 1, overflowY: 'auto', border: '1px solid #1e293b', borderRadius: 12, marginBottom: 20 }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '0.84rem' }}>
                <thead style={{ position: 'sticky', top: 0, background: '#1e293b', color: '#cbd5e1' }}>
                  <tr>
                    <th style={{ padding: '10px 14px' }}>{isVi ? 'Mục Tiêu' : 'Target'}</th>
                    <th style={{ padding: '10px 14px' }}>{isVi ? 'Loại' : 'Type'}</th>
                    <th style={{ padding: '10px 14px' }}>{isVi ? 'Thông Số / Lý Do' : 'Details / Reason'}</th>
                    <th style={{ padding: '10px 14px' }}>{isVi ? 'Trạng Thái' : 'Status'}</th>
                  </tr>
                </thead>
                <tbody>
                  {previewRules.map((rule, idx) => (
                    <tr
                      key={idx}
                      style={{
                        borderBottom: '1px solid #1e293b',
                        background: idx % 2 === 0 ? 'transparent' : 'rgba(30, 41, 59, 0.3)',
                      }}
                    >
                      <td style={{ padding: '10px 14px', color: '#f8fafc', fontWeight: 600 }}>
                        {rule.rule_id}
                      </td>
                      <td style={{ padding: '10px 14px' }}>
                        <span
                          style={{
                            background: rule.rule_type === 'domain' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(52, 211, 153, 0.15)',
                            color: rule.rule_type === 'domain' ? '#38bdf8' : '#34d399',
                            padding: '2px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 700,
                          }}
                        >
                          {rule.rule_type.toUpperCase()}
                        </span>
                      </td>
                      <td style={{ padding: '10px 14px', color: '#94a3b8' }}>
                        {rule.rule_type === 'application'
                          ? `${rule.quota || 60} ${isVi ? 'phút/ngày' : 'mins/day'}`
                          : rule.reason}
                      </td>
                      <td style={{ padding: '10px 14px' }}>
                        <span style={{ color: rule.enabled ? '#34d399' : '#f87171', fontSize: '0.8rem', fontWeight: 600 }}>
                          {rule.enabled ? (isVi ? 'BẬT' : 'ENABLED') : (isVi ? 'TẮT' : 'DISABLED')}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Modal Actions */}
            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                type="button"
                onClick={() => setImportModalOpen(false)}
                disabled={isImporting}
                style={{
                  background: 'transparent',
                  border: '1px solid #475569',
                  color: '#94a3b8',
                  padding: '10px 18px',
                  borderRadius: 10,
                  fontSize: '0.88rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {t.import_cancel_btn}
              </button>

              <button
                type="button"
                onClick={handleConfirmImport}
                disabled={isImporting}
                style={{
                  background: 'linear-gradient(135deg, #059669, #10b981)',
                  color: '#fff',
                  border: 'none',
                  padding: '10px 22px',
                  borderRadius: 10,
                  fontSize: '0.88rem',
                  fontWeight: 700,
                  cursor: isImporting ? 'not-allowed' : 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  boxShadow: '0 4px 15px rgba(16, 185, 129, 0.3)',
                }}
              >
                <Check size={16} />
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
