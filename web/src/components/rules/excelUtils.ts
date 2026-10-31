import type { LocalRuleRecord } from '../../types';
import type { ParsedImportRule } from './rulesTypes';

export async function downloadRulesTemplate(isVi: boolean): Promise<void> {
  const XLSX = await import('xlsx');
  const sampleData = [
    {
      'Mã Quy Tắc (Domain/App)': 'tiktok.com',
      'Loại (domain/application)': 'domain',
      'Hành Động': 'BLOCK',
      'Lý Do': 'Xao nhãng học tập',
      'Hạn Mức Phút (App)': '',
    },
    {
      'Mã Quy Tắc (Domain/App)': 'edu.vn',
      'Loại (domain/application)': 'domain',
      'Hành Động': 'ALLOW',
      'Lý Do': 'Cổng học tập trực tuyến',
      'Hạn Mức Phút (App)': '',
    },
    {
      'Mã Quy Tắc (Domain/App)': 'discord.exe',
      'Loại (domain/application)': 'application',
      'Hành Động': 'LIMIT',
      'Lý Do': 'Giới hạn giờ trò chuyện',
      'Hạn Mức Phút (App)': 45,
    },
  ];

  const ws = XLSX.utils.json_to_sheet(sampleData);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, 'MauQuyTac');
  XLSX.writeFile(wb, 'FamShield_Rules_Template.xlsx');
}

export async function exportRulesToExcel(rules: LocalRuleRecord[]): Promise<void> {
  const XLSX = await import('xlsx');
  const exportData = rules.map((r) => {
    let action = 'UNKNOWN';
    let reason = '';
    let quota = '';
    try {
      const p = JSON.parse(r.payload || '{}');
      action = p.action || (r.rule_type === 'application' ? 'LIMIT' : 'BLOCK');
      reason = p.reason || '';
      if (p.quota_minutes) quota = p.quota_minutes;
    } catch {
      // ignore
    }
    return {
      'Mã Quy Tắc (Domain/App)': r.rule_id,
      'Loại (domain/application)': r.rule_type,
      'Hành Động': action,
      'Lý Do': reason,
      'Hạn Mức Phút (App)': quota,
      'Trạng Thái': r.enabled ? 'BẬT' : 'TẮT',
    };
  });

  const ws = XLSX.utils.json_to_sheet(exportData);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, 'DanhSachQuyTac');
  XLSX.writeFile(wb, `FamShield_Rules_Export_${new Date().toISOString().slice(0, 10)}.xlsx`);
}

export async function parseRulesFromExcelFile(file: File): Promise<ParsedImportRule[]> {
  const XLSX = await import('xlsx');
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (evt) => {
      try {
        const data = evt.target?.result;
        const wb = XLSX.read(data, { type: 'binary' });
        const sheetName = wb.SheetNames[0];
        const rows: any[] = XLSX.utils.sheet_to_json(wb.Sheets[sheetName]);

        const parsed: ParsedImportRule[] = [];
        for (const row of rows) {
          const ruleId = (
            row['Mã Quy Tắc (Domain/App)'] ||
            row['domain'] ||
            row['app'] ||
            row['rule_id'] ||
            ''
          )
            .toString()
            .trim();
          if (!ruleId) continue;

          let rType: 'domain' | 'application' = 'domain';
          const typeVal = (
            row['Loại (domain/application)'] ||
            row['rule_type'] ||
            ''
          )
            .toString()
            .toLowerCase();
          if (typeVal.includes('app') || ruleId.endsWith('.exe')) {
            rType = 'application';
          }

          const reason = (row['Lý Do'] || row['reason'] || 'import_from_excel').toString().trim();
          const quotaRaw = row['Hạn Mức Phút (App)'] || row['quota_minutes'];
          const quota = quotaRaw ? parseInt(quotaRaw, 10) : undefined;

          parsed.push({
            rule_id: ruleId,
            rule_type: rType,
            reason,
            quota,
            enabled: true,
          });
        }
        resolve(parsed);
      } catch (err) {
        reject(err);
      }
    };
    reader.onerror = (err) => reject(err);
    reader.readAsBinaryString(file);
  });
}
