import type React from 'react';
import { useState, useMemo } from 'react';
import { Keyboard, RefreshCw, Check, AlertTriangle, Plus, Trash2 } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { api } from '../../api';
import { translations } from '../../i18n';
import { DataTable } from '../DataTable';
import type { KeywordConfigResponse, KeywordRule, Language } from '../../types';

interface KeywordMonitoringCardProps {
  lang: Language;
  keywordConfig: KeywordConfigResponse | null;
  onUpdateKeywordConfig: (cfg: KeywordConfigResponse) => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const KeywordMonitoringCard: React.FC<KeywordMonitoringCardProps> = ({
  lang,
  keywordConfig,
  onUpdateKeywordConfig,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [newKwPattern, setNewKwPattern] = useState('');
  const [newKwCategory, setNewKwCategory] = useState(
    isVi ? 'Chửi bậy / Thô tục' : 'Profanity / Vulgarity'
  );
  const [newKwSeverity, setNewKwSeverity] = useState('medium');
  const [isAddingKw, setIsAddingKw] = useState(false);
  const [isTogglingKwMonitoring, setIsTogglingKwMonitoring] = useState(false);

  const handleToggleMonitoring = async () => {
    if (!keywordConfig) return;
    try {
      setIsTogglingKwMonitoring(true);
      const next = !keywordConfig.keyboard_monitoring_enabled;
      await api.toggleKeyboardMonitoring(next);
      onUpdateKeywordConfig({ ...keywordConfig, keyboard_monitoring_enabled: next });
      onShowToast(t.msg_kw_monitoring_toggled, 'success');
    } catch (e: any) {
      onShowToast(
        e.message ||
          (isVi ? 'Lỗi cập nhật giám sát bàn phím' : 'Failed to update keyboard monitoring'),
        'error'
      );
    } finally {
      setIsTogglingKwMonitoring(false);
    }
  };

  const handleAddKeyword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKwPattern.trim()) return;
    try {
      setIsAddingKw(true);
      const added = await api.addCustomKeyword(newKwPattern.trim(), newKwCategory, newKwSeverity);
      setNewKwPattern('');
      if (keywordConfig) {
        onUpdateKeywordConfig({
          ...keywordConfig,
          total_rules_count: keywordConfig.total_rules_count + 1,
          rules: [added, ...keywordConfig.rules],
        });
      }
      onShowToast(t.msg_kw_added, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi thêm từ khóa' : 'Failed to add keyword'), 'error');
    } finally {
      setIsAddingKw(false);
    }
  };

  const handleToggleKeyword = async (rule: KeywordRule) => {
    try {
      await api.toggleKeyword(rule.id, !rule.enabled);
      if (keywordConfig) {
        onUpdateKeywordConfig({
          ...keywordConfig,
          rules: keywordConfig.rules.map((r) =>
            r.id === rule.id ? { ...r, enabled: !r.enabled } : r
          ),
        });
      }
      onShowToast(t.msg_kw_toggled, 'success');
    } catch (e: any) {
      onShowToast(
        e.message || (isVi ? 'Lỗi cập nhật từ khóa' : 'Failed to toggle keyword'),
        'error'
      );
    }
  };

  const handleDeleteKeyword = async (ruleId: string) => {
    try {
      await api.deleteCustomKeyword(ruleId);
      if (keywordConfig) {
        onUpdateKeywordConfig({
          ...keywordConfig,
          total_rules_count: Math.max(0, keywordConfig.total_rules_count - 1),
          rules: keywordConfig.rules.filter((r) => r.id !== ruleId),
        });
      }
      onShowToast(t.msg_kw_deleted, 'success');
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi xóa từ khóa' : 'Failed to delete keyword'), 'error');
    }
  };

  const keywordRuleColumns = useMemo<ColumnDef<KeywordRule>[]>(
    () => [
      {
        accessorKey: 'pattern',
        header: t.th_kw_pattern,
        cell: (info) => (
          <span className="font-bold text-slate-900 font-mono">"{info.getValue() as string}"</span>
        ),
      },
      {
        accessorKey: 'category_name',
        header: t.th_kw_category,
        cell: (info) => (
          <span className="bg-slate-100 text-slate-700 border border-slate-200 px-2 py-0.5 rounded text-2xs font-medium">
            {info.getValue() as string}
          </span>
        ),
      },
      {
        accessorKey: 'severity',
        header: t.th_kw_severity,
        cell: (info) => {
          const sev = ((info.getValue() as string) || '').toLowerCase();
          return (
            <span
              className={`px-2 py-0.5 rounded text-2xs font-bold uppercase border ${
                sev === 'critical'
                  ? 'bg-rose-50 text-rose-700 border-rose-200'
                  : sev === 'high'
                    ? 'bg-orange-50 text-orange-700 border-orange-200'
                    : 'bg-amber-50 text-amber-700 border-amber-200'
              }`}
            >
              {sev}
            </span>
          );
        },
      },
      {
        accessorKey: 'is_custom',
        header: t.th_kw_type,
        cell: (info) => {
          const isCustom = info.getValue() as boolean;
          return isCustom ? (
            <span className="text-sky-700 font-semibold text-xs">
              {isVi ? 'Phụ huynh' : 'Custom'}
            </span>
          ) : (
            <span className="text-slate-500 text-xs">{isVi ? 'Hệ thống' : 'Built-in'}</span>
          );
        },
      },
      {
        accessorKey: 'enabled',
        header: t.th_kw_status,
        cell: ({ row }) => {
          const rule = row.original;
          return (
            <button
              type="button"
              onClick={() => handleToggleKeyword(rule)}
              className={`px-2.5 py-1 rounded text-2xs font-bold cursor-pointer transition-colors border ${
                rule.enabled
                  ? 'bg-emerald-50 text-emerald-700 border-emerald-200 hover:bg-emerald-100'
                  : 'bg-slate-100 text-slate-500 border-slate-200 hover:bg-slate-200'
              }`}
            >
              {rule.enabled ? (isVi ? 'BẬT' : 'ON') : isVi ? 'TẮT' : 'OFF'}
            </button>
          );
        },
      },
      {
        id: 'actions',
        header: () => <div className="text-right">{isVi ? 'Hành động' : 'Action'}</div>,
        cell: ({ row }) => {
          const rule = row.original;
          if (!rule.is_custom) return null;
          return (
            <div className="text-right">
              <button
                type="button"
                onClick={() => handleDeleteKeyword(rule.id)}
                className="text-rose-500 hover:text-rose-700 p-1 rounded hover:bg-rose-50 cursor-pointer transition-colors"
                title={isVi ? 'Xóa từ khóa' : 'Delete keyword'}
              >
                <Trash2 size={15} />
              </button>
            </div>
          );
        },
      },
    ],
    [t, isVi, keywordConfig]
  );

  return (
    <div
      id="section-keywords-monitor"
      style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: 24,
        boxShadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          flexWrap: 'wrap',
          gap: 16,
          marginBottom: 20,
          borderBottom: '1px solid #1e293b',
          paddingBottom: 16,
        }}
      >
        <div>
          <h3
            style={{
              fontSize: '1.25rem',
              fontWeight: 800,
              color: '#f8fafc',
              margin: 0,
              display: 'flex',
              alignItems: 'center',
              gap: 10,
            }}
          >
            <Keyboard size={24} color="#f59e0b" />
            {t.title_keyword_monitor}
          </h3>
          <p
            style={{
              fontSize: '0.85rem',
              color: '#94a3b8',
              marginTop: 6,
              marginBottom: 0,
              maxWidth: 800,
              lineHeight: 1.5,
            }}
          >
            {t.desc_keyword_monitor}
          </p>
        </div>

        {/* Toggle monitoring switch button */}
        <button
          type="button"
          onClick={handleToggleMonitoring}
          disabled={isTogglingKwMonitoring}
          style={{
            background: keywordConfig?.keyboard_monitoring_enabled
              ? 'rgba(52, 211, 153, 0.15)'
              : 'rgba(239, 68, 68, 0.15)',
            color: keywordConfig?.keyboard_monitoring_enabled ? '#34d399' : '#f87171',
            border: `1px solid ${
              keywordConfig?.keyboard_monitoring_enabled
                ? 'rgba(52, 211, 153, 0.4)'
                : 'rgba(239, 68, 68, 0.4)'
            }`,
            padding: '9px 18px',
            borderRadius: 10,
            fontSize: '0.88rem',
            fontWeight: 700,
            cursor: isTogglingKwMonitoring ? 'not-allowed' : 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            transition: 'all 0.2s ease',
          }}
        >
          {isTogglingKwMonitoring ? (
            <RefreshCw size={14} className="animate-spin" />
          ) : keywordConfig?.keyboard_monitoring_enabled ? (
            <Check size={14} />
          ) : (
            <AlertTriangle size={14} />
          )}
          <span>
            {keywordConfig?.keyboard_monitoring_enabled
              ? isVi
                ? '🟢 Đang Giám Sát Bàn Phím (Bật)'
                : '🟢 Keystroke Monitor: Active'
              : isVi
                ? '⚪ Đã Tạm Dừng Giám Sát (Tắt)'
                : '⚪ Keystroke Monitor: Disabled'}
          </span>
        </button>
      </div>

      {/* 3 Metrics Badges */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: 14,
          marginBottom: 20,
        }}
      >
        <div
          style={{
            background: '#070b14',
            border: '1px solid #1e293b',
            borderRadius: 12,
            padding: '14px 16px',
          }}
        >
          <div
            style={{
              fontSize: '0.78rem',
              color: '#64748b',
              fontWeight: 600,
              textTransform: 'uppercase',
            }}
          >
            {isVi ? 'Tổng số từ khóa đang theo dõi' : 'Total Monitored Keywords'}
          </div>
          <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#38bdf8', marginTop: 4 }}>
            {keywordConfig?.total_rules_count || 0}
          </div>
        </div>

        <div
          style={{
            background: '#070b14',
            border: '1px solid #1e293b',
            borderRadius: 12,
            padding: '14px 16px',
          }}
        >
          <div
            style={{
              fontSize: '0.78rem',
              color: '#64748b',
              fontWeight: 600,
              textTransform: 'uppercase',
            }}
          >
            {isVi ? 'Vi phạm phát hiện hôm nay' : 'Violations Detected Today'}
          </div>
          <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#fbbf24', marginTop: 4 }}>
            {keywordConfig?.violations_today_count || 0}
          </div>
        </div>

        <div
          style={{
            background: '#070b14',
            border: '1px solid #1e293b',
            borderRadius: 12,
            padding: '14px 16px',
          }}
        >
          <div
            style={{
              fontSize: '0.78rem',
              color: '#64748b',
              fontWeight: 600,
              textTransform: 'uppercase',
            }}
          >
            {isVi ? 'Cơ chế bảo vệ riêng tư' : 'Privacy Mechanism'}
          </div>
          <div style={{ fontSize: '0.95rem', fontWeight: 700, color: '#34d399', marginTop: 8 }}>
            RAM Buffer 128 chars • 0 Disk Logs
          </div>
        </div>
      </div>

      {/* Add Custom Keyword Form */}
      <form
        onSubmit={handleAddKeyword}
        style={{
          background: '#070b14',
          border: '1px solid #1e293b',
          borderRadius: 14,
          padding: 16,
          marginBottom: 20,
          display: 'flex',
          flexWrap: 'wrap',
          gap: 12,
          alignItems: 'flex-end',
        }}
      >
        <div style={{ flex: '1 1 240px' }}>
          <label
            style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
          >
            {t.lbl_kw_pattern}
          </label>
          <input
            type="text"
            placeholder={isVi ? 'Ví dụ: đánh nhau, chửi thề...' : 'e.g., hate you, fighting...'}
            value={newKwPattern}
            onChange={(e) => setNewKwPattern(e.target.value)}
            style={{
              width: '100%',
              background: '#0f172a',
              border: '1px solid #334155',
              color: '#fff',
              padding: '9px 12px',
              borderRadius: 8,
              fontSize: '0.88rem',
              outline: 'none',
            }}
          />
        </div>

        <div style={{ flex: '0 1 200px' }}>
          <label
            style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
          >
            {t.lbl_kw_category}
          </label>
          <select
            value={newKwCategory}
            onChange={(e) => setNewKwCategory(e.target.value)}
            style={{
              width: '100%',
              background: '#0f172a',
              border: '1px solid #334155',
              color: '#fff',
              padding: '9px 12px',
              borderRadius: 8,
              fontSize: '0.88rem',
              outline: 'none',
            }}
          >
            <option value="Chửi bậy / Thô tục">{t.cat_profanity}</option>
            <option value="Bắt nạt / Đe dọa">{t.cat_bullying}</option>
            <option value="Tự hại / Nguy hiểm">{t.cat_self_harm}</option>
            <option value="Người lớn / Khiêu dâm">{t.cat_adult}</option>
            <option value="Tùy chỉnh phụ huynh">{t.cat_custom}</option>
          </select>
        </div>

        <div style={{ flex: '0 1 150px' }}>
          <label
            style={{ display: 'block', fontSize: '0.8rem', color: '#94a3b8', marginBottom: 6 }}
          >
            {t.lbl_kw_severity}
          </label>
          <select
            value={newKwSeverity}
            onChange={(e) => setNewKwSeverity(e.target.value)}
            style={{
              width: '100%',
              background: '#0f172a',
              border: '1px solid #334155',
              color: '#fff',
              padding: '9px 12px',
              borderRadius: 8,
              fontSize: '0.88rem',
              outline: 'none',
            }}
          >
            <option value="low">{t.sev_low}</option>
            <option value="medium">{t.sev_medium}</option>
            <option value="high">{t.sev_high}</option>
            <option value="critical">{t.sev_critical}</option>
          </select>
        </div>

        <button
          type="submit"
          disabled={isAddingKw || !newKwPattern.trim()}
          style={{
            background: '#f59e0b',
            color: '#070b14',
            border: 'none',
            padding: '9px 18px',
            borderRadius: 8,
            fontSize: '0.85rem',
            fontWeight: 700,
            cursor: isAddingKw || !newKwPattern.trim() ? 'not-allowed' : 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            height: 38,
          }}
        >
          {isAddingKw ? <RefreshCw size={14} className="animate-spin" /> : <Plus size={15} />}
          <span>{t.btn_add_keyword}</span>
        </button>
      </form>

      {/* Search & Keyword List */}
      <div>
        <DataTable
          data={keywordConfig?.rules || []}
          columns={keywordRuleColumns}
          pageSize={10}
          searchPlaceholder={
            isVi ? 'Tìm kiếm từ khóa trong danh sách...' : 'Search monitored keywords...'
          }
          emptyText={
            isVi ? 'Chưa có từ khóa nào được thiết lập' : 'No keyword rules configured'
          }
          lang={lang}
          enableExport={true}
          exportFilename="famshield_monitored_keywords"
        />
      </div>
    </div>
  );
};
