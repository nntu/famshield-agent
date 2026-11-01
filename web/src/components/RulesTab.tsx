import {
  Layers,
  Sliders,
  Keyboard,
  Trash2,
} from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import type React from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { api } from '../api';
import { translations } from '../i18n';
import type {
  CloudPolicyData,
  KeywordConfigResponse,
  KeywordRule,
  Language,
  LocalRuleRecord,
  ResolvePolicyResult,
} from '../types';
import type {
  FilterSubTab,
  RulesTabProps,
  ParsedImportRule,
} from './rules/rulesTypes';
import { STANDARD_CATEGORIES } from './rules/rulesTypes';
import { CategoriesSubTab } from './rules/CategoriesSubTab';
import { CategoryDetailModal } from './rules/CategoryDetailModal';
import { CloudPolicyBanner } from './rules/CloudPolicyBanner';
import { ExcelImportModal } from './rules/ExcelImportModal';
import { KeywordsSubTab } from './rules/KeywordsSubTab';
import { PauseFilterBanner } from './rules/PauseFilterBanner';
import { RulesSubTab } from './rules/RulesSubTab';
import { RulesTabNav } from './rules/RulesTabNav';
import {
  downloadRulesTemplate,
  exportRulesToExcel,
  parseRulesFromExcelFile,
} from './rules/excelUtils';

export const RulesTab: React.FC<RulesTabProps> = ({ rules, lang, onRefresh, onShowToast }) => {
  const t = translations[lang];
  const isVi = lang === 'vi';
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Active sub-tab
  const [activeSubTab, setActiveSubTab] = useState<FilterSubTab>('categories');

  // Numa Pause State
  const [nowSec, setNowSec] = useState(() => Math.floor(Date.now() / 1000));
  const [isPausing, setIsPausing] = useState(false);

  // Policy & Sync State
  const [cloudPolicy, setCloudPolicy] = useState<CloudPolicyData | null>(null);
  const [isPullingCloud, setIsPullingCloud] = useState(false);
  const [isSyncingCloud, setIsSyncingCloud] = useState(false);

  // Rule Form States
  const [newBlockedDomain, setNewBlockedDomain] = useState('');
  const [domainReason, setDomainReason] = useState('');
  const [isAddingBlocked, setIsAddingBlocked] = useState(false);

  const [newAllowedDomain, setNewAllowedDomain] = useState('');
  const [isAddingAllowed, setIsAddingAllowed] = useState(false);

  // Bulk Paste Drawers
  const [showBulkBlacklist, setShowBulkBlacklist] = useState(false);
  const [bulkBlacklistText, setBulkBlacklistText] = useState('');
  const [showBulkWhitelist, setShowBulkWhitelist] = useState(false);
  const [bulkWhitelistText, setBulkWhitelistText] = useState('');

  // Searches
  const [blacklistSearch, setBlacklistSearch] = useState('');
  const [whitelistSearch, setWhitelistSearch] = useState('');

  // App Limits Form State
  const [newApp, setNewApp] = useState('');
  const [appQuota, setAppQuota] = useState('60');
  const [isAddingApp, setIsAddingApp] = useState(false);

  // Domain Inspector State
  const [inspectInput, setInspectInput] = useState('');
  const [isInspecting, setIsInspecting] = useState(false);
  const [inspectResult, setInspectResult] = useState<ResolvePolicyResult | null>(null);

  // Keywords Monitoring State
  const [keywordConfig, setKeywordConfig] = useState<KeywordConfigResponse | null>(null);
  const [isKeywordsEnabled, setIsKeywordsEnabled] = useState(true);
  const [isTogglingKeywords, setIsTogglingKeywords] = useState(false);
  const [kwSearch, setKwSearch] = useState('');
  const [newKwPattern, setNewKwPattern] = useState('');
  const [newKwCategory, setNewKwCategory] = useState<'bullying' | 'profanity' | 'adult' | 'self_harm' | 'custom'>('custom');
  const [newKwSeverity, setNewKwSeverity] = useState<'critical' | 'high' | 'medium' | 'low'>('medium');
  const [isAddingKw, setIsAddingKw] = useState(false);

  // Excel Import Preview Modal State
  const [importModalOpen, setImportModalOpen] = useState(false);
  const [previewRules, setPreviewRules] = useState<ParsedImportRule[]>([]);
  const [importFileName, setImportFileName] = useState('');
  const [isImporting, setIsImporting] = useState(false);

  // Category Detail Modal State
  const [selectedCategoryModal, setSelectedCategoryModal] = useState<string | null>(null);

  // Timer for auto-resume countdown
  useEffect(() => {
    const timer = setInterval(() => {
      setNowSec(Math.floor(Date.now() / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Fetch initial policy & keyword data
  useEffect(() => {
    api.getCloudPolicy().then((p) => {
      if (p) setCloudPolicy(p);
    });
    api.getKeywordConfig().then((cfg) => {
      if (cfg) {
        setKeywordConfig(cfg);
        setIsKeywordsEnabled(cfg.keyboard_monitoring_enabled);
      }
    }).catch(() => {});
  }, []);

  const pausedUntil = cloudPolicy?.paused_until || 0;
  const isPaused = pausedUntil > nowSec;
  const remainingSec = Math.max(0, pausedUntil - nowSec);
  const remainingMin = Math.floor(remainingSec / 60);
  const remainingSecMod = remainingSec % 60;

  // Numa Pause & Unpause Handlers
  const handlePause = async (seconds: number) => {
    setIsPausing(true);
    try {
      const res = await api.pauseFilter(seconds);
      if (res.success) {
        setCloudPolicy((prev) => (prev ? { ...prev, paused_until: res.paused_until } : null));
        onShowToast(
          isVi
            ? `Đã tạm dừng bảo vệ web trong ${Math.floor(seconds / 60)} phút`
            : `Web filter paused for ${Math.floor(seconds / 60)} mins`,
          'info'
        );
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Failed to pause filter', 'error');
    } finally {
      setIsPausing(false);
    }
  };

  const handleUnpause = async () => {
    setIsPausing(true);
    try {
      const res = await api.unpauseFilter();
      if (res.success) {
        setCloudPolicy((prev) => (prev ? { ...prev, paused_until: 0 } : null));
        onShowToast(
          isVi ? 'Đã kích hoạt lại chế độ bảo vệ Web & DNS ngay lập tức' : 'Web protection resumed',
          'success'
        );
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Failed to unpause filter', 'error');
    } finally {
      setIsPausing(false);
    }
  };

  // Pull policy from Cloud
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

  // Sync rules to Cloud
  const handleSyncToCloud = async () => {
    try {
      setIsSyncingCloud(true);
      onShowToast(
        isVi ? 'Đang đẩy toàn bộ quy tắc Endpoint lên Cloud...' : 'Syncing local rules to Cloud...',
        'info'
      );
      const res = await api.syncRulesToCloud();
      if (res.success) {
        onShowToast(res.message, 'success');
        api.getCloudPolicy().then(setCloudPolicy);
      } else {
        onShowToast(isVi ? 'Không thể đồng bộ' : 'Sync failed', 'error');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi đồng bộ máy chủ' : 'Sync error'), 'error');
    } finally {
      setIsSyncingCloud(false);
    }
  };

  // Local rules categorized
  const localBlockedRules = useMemo(() => {
    return rules.filter((r) => {
      if (r.rule_type !== 'domain') return false;
      try {
        const payload = JSON.parse(r.payload || '{}');
        return payload.action === 'BLOCK';
      } catch {
        return true;
      }
    });
  }, [rules]);

  const localAllowedRules = useMemo(() => {
    return rules.filter((r) => {
      if (r.rule_type !== 'domain') return false;
      try {
        const payload = JSON.parse(r.payload || '{}');
        return payload.action === 'ALLOW';
      } catch {
        return false;
      }
    });
  }, [rules]);

  const appRules = useMemo(() => {
    return rules.filter((r) => r.rule_type === 'application');
  }, [rules]);

  // Cloud lists
  const cloudBlockedList = useMemo(() => {
    return cloudPolicy?.custom_blocked_domains || [];
  }, [cloudPolicy]);

  const cloudAllowedList = useMemo(() => {
    return cloudPolicy?.custom_allowed_domains || [];
  }, [cloudPolicy]);

  // Unified items for Blacklist
  const mergedBlockedItems = useMemo(() => {
    const list: { id: string; domain: string; source: 'local' | 'cloud'; enabled: boolean }[] = [];
    for (const r of localBlockedRules) {
      list.push({ id: r.rule_id, domain: r.rule_id, source: 'local', enabled: r.enabled });
    }
    for (const d of cloudBlockedList) {
      if (!list.some((item) => item.domain.toLowerCase() === d.toLowerCase())) {
        list.push({ id: `cloud_${d}`, domain: d, source: 'cloud', enabled: true });
      }
    }
    return list;
  }, [localBlockedRules, cloudBlockedList]);

  // Unified items for Whitelist
  const mergedAllowedItems = useMemo(() => {
    const list: { id: string; domain: string; source: 'local' | 'cloud'; enabled: boolean }[] = [];
    for (const r of localAllowedRules) {
      list.push({ id: r.rule_id, domain: r.rule_id, source: 'local', enabled: r.enabled });
    }
    for (const d of cloudAllowedList) {
      if (!list.some((item) => item.domain.toLowerCase() === d.toLowerCase())) {
        list.push({ id: `cloud_${d}`, domain: d, source: 'cloud', enabled: true });
      }
    }
    return list;
  }, [localAllowedRules, cloudAllowedList]);

  // Filtered lists by search
  const filteredBlocked = useMemo(() => {
    if (!blacklistSearch.trim()) return mergedBlockedItems;
    const q = blacklistSearch.trim().toLowerCase();
    return mergedBlockedItems.filter((i) => i.domain.toLowerCase().includes(q));
  }, [mergedBlockedItems, blacklistSearch]);

  const filteredAllowed = useMemo(() => {
    if (!whitelistSearch.trim()) return mergedAllowedItems;
    const q = whitelistSearch.trim().toLowerCase();
    return mergedAllowedItems.filter((i) => i.domain.toLowerCase().includes(q));
  }, [mergedAllowedItems, whitelistSearch]);

  // Add Single Blocked Domain
  const handleAddBlockedDomain = async (target?: string) => {
    const d = (target || newBlockedDomain)
      .trim()
      .toLowerCase()
      .replace(/^https?:\/\//, '')
      .replace(/\/.*$/, '');
    if (!d) return;

    setIsAddingBlocked(true);
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
      setNewBlockedDomain('');
      setDomainReason('');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi thêm tên miền chặn', 'error');
    } finally {
      setIsAddingBlocked(false);
    }
  };

  // Add Single Allowed Domain
  const handleAddAllowedDomain = async (target?: string) => {
    const d = (target || newAllowedDomain)
      .trim()
      .toLowerCase()
      .replace(/^https?:\/\//, '')
      .replace(/\/.*$/, '');
    if (!d) return;

    setIsAddingAllowed(true);
    try {
      const payload = JSON.stringify({
        action: 'ALLOW',
        reason: 'local_whitelist_exception',
      });

      await api.createRule({
        rule_id: d,
        rule_type: 'domain',
        payload_json: payload,
        enabled: true,
      });

      onShowToast(isVi ? `Đã thêm ngoại lệ: ${d}` : `Domain whitelisted: ${d}`, 'success');
      setNewAllowedDomain('');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi thêm tên miền ngoại lệ', 'error');
    } finally {
      setIsAddingAllowed(false);
    }
  };

  // Batch Add Blacklist
  const handleConfirmBatchBlacklist = async () => {
    const lines = bulkBlacklistText
      .split(/[\n,; ]+/)
      .map((s) => s.trim().toLowerCase().replace(/^https?:\/\//, '').replace(/\/.*$/, ''))
      .filter(Boolean);

    if (lines.length === 0) return;

    try {
      let count = 0;
      for (const d of lines) {
        await api.createRule({
          rule_id: d,
          rule_type: 'domain',
          payload_json: JSON.stringify({ action: 'BLOCK', reason: 'bulk_paste' }),
          enabled: true,
        });
        count++;
      }
      onShowToast(isVi ? `Đã thêm ${count} tên miền vào Blacklist!` : `Added ${count} domains!`, 'success');
      setBulkBlacklistText('');
      setShowBulkBlacklist(false);
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi thêm hàng loạt', 'error');
    }
  };

  // Batch Add Whitelist
  const handleConfirmBatchWhitelist = async () => {
    const lines = bulkWhitelistText
      .split(/[\n,; ]+/)
      .map((s) => s.trim().toLowerCase().replace(/^https?:\/\//, '').replace(/\/.*$/, ''))
      .filter(Boolean);

    if (lines.length === 0) return;

    try {
      let count = 0;
      for (const d of lines) {
        await api.createRule({
          rule_id: d,
          rule_type: 'domain',
          payload_json: JSON.stringify({ action: 'ALLOW', reason: 'bulk_paste_whitelist' }),
          enabled: true,
        });
        count++;
      }
      onShowToast(isVi ? `Đã thêm ${count} tên miền vào Whitelist!` : `Added ${count} domains!`, 'success');
      setBulkWhitelistText('');
      setShowBulkWhitelist(false);
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi thêm hàng loạt', 'error');
    }
  };

  // Delete Rule
  const handleDeleteRule = async (ruleId: string) => {
    try {
      await api.deleteRule(ruleId);
      onShowToast(isVi ? `Đã xóa quy tắc: ${ruleId}` : `Deleted rule: ${ruleId}`, 'info');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi xóa quy tắc', 'error');
    }
  };

  // Add Application Quota Rule
  const handleAddApp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newApp.trim()) return;

    setIsAddingApp(true);
    try {
      const appName = newApp.trim().toLowerCase().endsWith('.exe')
        ? newApp.trim().toLowerCase()
        : `${newApp.trim().toLowerCase()}.exe`;

      const payload = JSON.stringify({
        quota_minutes: parseInt(appQuota) || 60,
      });

      await api.createRule({
        rule_id: appName,
        rule_type: 'application',
        payload_json: payload,
        enabled: true,
      });

      onShowToast(isVi ? `Đã đặt giới hạn ứng dụng: ${appName}` : `App limit added: ${appName}`, 'success');
      setNewApp('');
      onRefresh();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || (isVi ? 'Lỗi đặt giới hạn ứng dụng' : 'Failed to set app limit'), 'error');
    } finally {
      setIsAddingApp(false);
    }
  };

  // Smart Domain Inspector
  const handleInspectDomain = async () => {
    const raw = inspectInput.trim().toLowerCase().replace(/^https?:\/\//, '').replace(/\/.*$/, '');
    if (!raw) return;

    setIsInspecting(true);
    try {
      const inAllow = mergedAllowedItems.some((i) => i.domain.toLowerCase() === raw);
      const inBlock = mergedBlockedItems.some((i) => i.domain.toLowerCase() === raw);

      let action: 'block' | 'allow' = 'allow';
      let reason = isVi
        ? 'Tên miền không thuộc danh sách chặn nào và các danh mục kiểm soát đang cho phép.'
        : 'Domain is clean and standard categories allow access.';
      let category = isVi ? 'Thông Thường / Cho Phép' : 'General / Allowed';

      if (inAllow) {
        action = 'allow';
        reason = isVi
          ? 'Ưu tiên số 1: Nằm trong Ngoại Lệ (Whitelist) do phụ huynh quy định.'
          : 'Priority 1: Matches Whitelist exception.';
        category = isVi ? 'Whitelist Ngoại Lệ' : 'Whitelist Exception';
      } else if (inBlock) {
        action = 'block';
        reason = isVi
          ? 'Ưu tiên số 2: Nằm trong Danh Sách Chặn Riêng (Blacklist).'
          : 'Priority 2: Matches Custom Blacklist.';
        category = isVi ? 'Blacklist Chặn Riêng' : 'Custom Blacklist';
      } else {
        for (const cat of STANDARD_CATEGORIES) {
          const matchSeed = cat.seeds.some((s) => raw === s || raw.endsWith(`.${s}`));
          if (matchSeed) {
            const isCatBlocked =
              cat.id === 'adult' ? !!cloudPolicy?.block_adult :
              cat.id === 'gambling' ? !!cloudPolicy?.block_gambling :
              cat.id === 'violence' ? !!cloudPolicy?.block_violence :
              cat.id === 'social' ? !!cloudPolicy?.block_social :
              cat.id === 'gaming' ? !!cloudPolicy?.block_gaming : false;

            category = isVi ? cat.name : cat.nameEn;
            if (isCatBlocked) {
              action = 'block';
              reason = isVi
                ? `Bị chặn bởi Danh Mục Tiêu Chuẩn: "${isVi ? cat.name : cat.nameEn}".`
                : `Blocked by standard category: "${cat.nameEn}".`;
            }
            break;
          }
        }
      }

      setInspectResult({
        domain: raw,
        action,
        reason,
        effective_category: category,
      });
    } catch {
      onShowToast(isVi ? 'Không thể tra cứu tên miền' : 'Inspection failed', 'error');
    } finally {
      setIsInspecting(false);
    }
  };

  // Keyboard Monitoring Master Toggle
  const handleToggleKeyboardMonitoring = async () => {
    try {
      setIsTogglingKeywords(true);
      const nextState = !isKeywordsEnabled;
      await api.toggleKeyboardMonitoring(nextState);
      setIsKeywordsEnabled(nextState);
      if (keywordConfig) {
        setKeywordConfig({ ...keywordConfig, keyboard_monitoring_enabled: nextState });
      }
      onShowToast(
        nextState
          ? (isVi ? 'Đã bật chế độ giám sát từ khóa trực tiếp' : 'Keystroke monitoring activated')
          : (isVi ? 'Đã tạm tắt giám sát từ khóa' : 'Keystroke monitoring disabled'),
        'info'
      );
    } catch {
      onShowToast(isVi ? 'Lỗi thay đổi trạng thái giám sát' : 'Toggle failed', 'error');
    } finally {
      setIsTogglingKeywords(false);
    }
  };

  // Add Custom Keyword
  const handleAddCustomKeyword = async () => {
    const raw = newKwPattern.trim().toLowerCase();
    if (!raw) return;

    try {
      setIsAddingKw(true);
      const res = await api.addCustomKeyword(raw, newKwCategory, newKwSeverity);
      setNewKwPattern('');
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          rules: [...keywordConfig.rules, res],
        });
      }
      onShowToast(isVi ? `Đã thêm từ khóa theo dõi: "${raw}"` : `Added keyword: "${raw}"`, 'success');
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      onShowToast(msg || 'Lỗi thêm từ khóa', 'error');
    } finally {
      setIsAddingKw(false);
    }
  };

  // Delete Custom Keyword
  const handleDeleteCustomKeyword = async (id: string) => {
    try {
      await api.deleteCustomKeyword(id);
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          rules: keywordConfig.rules.filter((r) => r.id !== id),
        });
      }
      onShowToast(isVi ? 'Đã xóa từ khóa theo dõi' : 'Keyword deleted', 'info');
    } catch {
      onShowToast(isVi ? 'Lỗi xóa từ khóa' : 'Delete failed', 'error');
    }
  };

  // Toggle Keyword Rule On/Off
  const handleToggleKeyword = async (id: string, currentEnabled: boolean) => {
    try {
      await api.toggleKeyword(id, !currentEnabled);
      if (keywordConfig) {
        setKeywordConfig({
          ...keywordConfig,
          rules: keywordConfig.rules.map((r) => (r.id === id ? { ...r, enabled: !currentEnabled } : r)),
        });
      }
    } catch {
      onShowToast(isVi ? 'Lỗi thay đổi trạng thái' : 'Toggle failed', 'error');
    }
  };

  // Toggle standard category blocking
  const handleToggleCategoryBlock = async (catId: string, blocked: boolean) => {
    try {
      const patch: Partial<CloudPolicyData> = {};
      if (catId === 'adult') patch.block_adult = blocked;
      else if (catId === 'gambling') patch.block_gambling = blocked;
      else if (catId === 'violence') patch.block_violence = blocked;
      else if (catId === 'social') patch.block_social = blocked;
      else if (catId === 'gaming') patch.block_gaming = blocked;

      const res = await api.updateCloudPolicy(patch);
      if (res?.policy) {
        setCloudPolicy(res.policy);
      } else if (cloudPolicy) {
        setCloudPolicy({ ...cloudPolicy, ...patch });
      }

      const catObj = STANDARD_CATEGORIES.find((c) => c.id === catId);
      const catName = catObj ? (isVi ? catObj.name : catObj.nameEn) : catId;
      onShowToast(
        blocked
          ? (isVi ? `Đã bật chặn danh mục: "${catName}"` : `Activated blocking for "${catName}"`)
          : (isVi ? `Đã cho phép truy cập danh mục: "${catName}"` : `Allowed access for "${catName}"`),
        blocked ? 'info' : 'success'
      );
    } catch {
      onShowToast(isVi ? 'Không thể cập nhật trạng thái danh mục' : 'Failed to update category state', 'error');
    }
  };

  // Update category custom domains & source files
  const handleUpdateCategoryDomains = async (
    catId: string,
    customDomains: string[],
    newFile?: import('../types').CategorySourceFile,
    removedFileId?: string
  ) => {
    try {
      const existingCustom = cloudPolicy?.category_custom_domains || {};
      const updatedCustom = { ...existingCustom, [catId]: customDomains };

      let updatedFiles = cloudPolicy?.category_source_files ? [...cloudPolicy.category_source_files] : [];
      if (newFile) {
        updatedFiles.push(newFile);
      }
      if (removedFileId) {
        updatedFiles = updatedFiles.filter((f) => f.id !== removedFileId);
      }

      const patch: Partial<CloudPolicyData> = {
        category_custom_domains: updatedCustom,
        category_source_files: updatedFiles,
      };

      const res = await api.updateCloudPolicy(patch);
      if (res?.policy) {
        setCloudPolicy(res.policy);
      } else if (cloudPolicy) {
        setCloudPolicy({
          ...cloudPolicy,
          category_custom_domains: updatedCustom,
          category_source_files: updatedFiles,
        });
      }
    } catch (e) {
      onShowToast(isVi ? 'Lỗi khi lưu danh sách tên miền của danh mục' : 'Failed to save category domains', 'error');
      throw e;
    }
  };

  // Excel Template Download
  const handleDownloadTemplate = async () => {
    try {
      await downloadRulesTemplate(isVi);
      onShowToast(isVi ? 'Đã tải file mẫu Excel!' : 'Template downloaded!', 'success');
    } catch {
      onShowToast(isVi ? 'Lỗi sinh file mẫu Excel' : 'Failed to generate template', 'error');
    }
  };

  // Export Excel
  const handleExportExcel = async () => {
    try {
      await exportRulesToExcel(rules);
      onShowToast(
        isVi ? `Đã xuất ${rules.length} quy tắc ra Excel!` : `Exported ${rules.length} rules!`,
        'success'
      );
    } catch {
      onShowToast(isVi ? 'Lỗi xuất file Excel' : 'Export failed', 'error');
    }
  };

  // Import Excel
  const handleFileSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImportFileName(file.name);
    try {
      const parsed = await parseRulesFromExcelFile(file);
      if (parsed.length === 0) {
        onShowToast(
          isVi ? 'Không tìm thấy dữ liệu quy tắc hợp lệ trong file' : 'No valid rules found',
          'error'
        );
        return;
      }
      setPreviewRules(parsed);
      setImportModalOpen(true);
    } catch {
      onShowToast(
        isVi ? 'Lỗi đọc file Excel. Vui lòng kiểm tra định dạng' : 'Failed to parse Excel file',
        'error'
      );
    } finally {
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  // Confirm Import
  const handleConfirmImport = async () => {
    setIsImporting(true);
    try {
      let successCount = 0;
      for (const r of previewRules) {
        let payload: any = { action: 'BLOCK', reason: r.reason };
        if (r.rule_type === 'application') {
          payload = { quota_minutes: r.quota || 60 };
        } else if (r.reason.toLowerCase().includes('allow') || r.reason.toLowerCase().includes('whitelist')) {
          payload = { action: 'ALLOW', reason: r.reason };
        }

        await api.createRule({
          rule_id: r.rule_id,
          rule_type: r.rule_type,
          payload_json: JSON.stringify(payload),
          enabled: r.enabled,
        });
        successCount++;
      }

      onShowToast(isVi ? `Đã nhập thành công ${successCount} quy tắc!` : `Imported ${successCount} rules!`, 'success');
      setImportModalOpen(false);
      setPreviewRules([]);
      onRefresh();
    } catch {
      onShowToast(isVi ? 'Lỗi khi nhập một số quy tắc' : 'Error importing rules', 'error');
    } finally {
      setIsImporting(false);
    }
  };

  // App table columns
  const appColumns = useMemo<ColumnDef<LocalRuleRecord>[]>(
    () => [
      {
        accessorKey: 'rule_id',
        header: t.th_app_name,
        cell: (info) => (
          <span className="font-bold text-slate-900 font-mono">{info.getValue() as string}</span>
        ),
      },
      {
        accessorKey: 'payload',
        header: isVi ? 'Hạn mức' : 'Quota',
        cell: (info) => {
          let quota = 60;
          try {
            const p = JSON.parse((info.getValue() as string) || '{}');
            quota = p.quota_minutes || 60;
          } catch {
            // ignore
          }
          return (
            <span className="bg-sky-50 text-sky-700 border border-sky-200 px-2 py-0.5 rounded text-2xs font-mono font-bold">
              {quota} {isVi ? 'phút / ngày' : 'mins / day'}
            </span>
          );
        },
      },
      {
        accessorKey: 'enabled',
        header: t.th_status,
        cell: (info) => (
          <span className="text-emerald-700 font-semibold text-xs flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-emerald-500 inline-block" />
            {info.getValue() ? (isVi ? 'Đang chặn' : 'Active') : (isVi ? 'Tắt' : 'Inactive')}
          </span>
        ),
      },
      {
        id: 'actions',
        header: () => <div className="text-right">{isVi ? 'Hành động' : 'Action'}</div>,
        cell: ({ row }) => (
          <div className="text-right">
            <button
              type="button"
              onClick={() => handleDeleteRule(row.original.rule_id)}
              className="text-rose-500 hover:text-rose-700 p-1 rounded hover:bg-rose-50 cursor-pointer transition-colors"
              title={t.btn_delete || (isVi ? 'Xóa' : 'Delete')}
            >
              <Trash2 size={15} />
            </button>
          </div>
        ),
      },
    ],
    [t, isVi]
  );

  // Counts
  const blockedStandardCount = useMemo(() => {
    if (!cloudPolicy) return 0;
    let count = 0;
    if (cloudPolicy.block_adult) count++;
    if (cloudPolicy.block_gambling) count++;
    if (cloudPolicy.block_violence) count++;
    if (cloudPolicy.block_social) count++;
    if (cloudPolicy.block_gaming) count++;
    return count;
  }, [cloudPolicy]);

  const whitelistCount = mergedAllowedItems.length;
  const blacklistCount = mergedBlockedItems.length;

  const customKeywords = keywordConfig?.rules?.filter((r) => r.category === 'custom') || [];
  const filteredKeywords = useMemo(() => {
    if (!kwSearch.trim()) return keywordConfig?.rules || [];
    const q = kwSearch.trim().toLowerCase();
    return (keywordConfig?.rules || []).filter((r) => r.pattern.toLowerCase().includes(q));
  }, [keywordConfig, kwSearch]);

  return (
    <div className="flex flex-col gap-6 animate-fade-in">
      {/* 1. NUMA AUTO-RESUME PAUSE BANNER */}
      <PauseFilterBanner
        isPaused={isPaused}
        remainingMin={remainingMin}
        remainingSecMod={remainingSecMod}
        isPausing={isPausing}
        onPause={handlePause}
        onUnpause={handleUnpause}
        isVi={isVi}
      />

      {/* 2. CLOUD POLICY & ACTION BAR */}
      <CloudPolicyBanner
        isVi={isVi}
        t={t}
        cloudPolicy={cloudPolicy}
        isPullingCloud={isPullingCloud}
        isSyncingCloud={isSyncingCloud}
        onPullFromCloud={handlePullFromCloud}
        onSyncToCloud={handleSyncToCloud}
        onDownloadTemplate={handleDownloadTemplate}
        fileInputRef={fileInputRef}
        onFileSelected={handleFileSelected}
        onExportExcel={handleExportExcel}
      />

      {/* 3. 3-TAB SEGMENTED NAVIGATION BAR */}
      <RulesTabNav
        activeSubTab={activeSubTab}
        setActiveSubTab={setActiveSubTab}
        blockedStandardCount={blockedStandardCount}
        blacklistCount={blacklistCount}
        whitelistCount={whitelistCount}
        isKeywordsEnabled={isKeywordsEnabled}
        isVi={isVi}
      />

      {/* ================= SUBTAB 1: WEB CATEGORIES ================= */}
      {activeSubTab === 'categories' && (
        <CategoriesSubTab
          isVi={isVi}
          blockedStandardCount={blockedStandardCount}
          whitelistCount={whitelistCount}
          blacklistCount={blacklistCount}
          cloudPolicy={cloudPolicy}
          inspectInput={inspectInput}
          setInspectInput={setInspectInput}
          isInspecting={isInspecting}
          inspectResult={inspectResult}
          handleInspectDomain={handleInspectDomain}
          handleAddAllowedDomain={handleAddAllowedDomain}
          handleAddBlockedDomain={handleAddBlockedDomain}
          onOpenCategoryDetail={(catId) => setSelectedCategoryModal(catId)}
          onToggleCategoryBlock={handleToggleCategoryBlock}
        />
      )}

      {/* ================= SUBTAB 2: BLACKLIST & WHITELIST ================= */}
      {activeSubTab === 'rules' && (
        <RulesSubTab
          isVi={isVi}
          t={t}
          lang={lang}
          mergedBlockedItems={mergedBlockedItems}
          filteredBlocked={filteredBlocked}
          newBlockedDomain={newBlockedDomain}
          setNewBlockedDomain={setNewBlockedDomain}
          isAddingBlocked={isAddingBlocked}
          handleAddBlockedDomain={handleAddBlockedDomain}
          showBulkBlacklist={showBulkBlacklist}
          setShowBulkBlacklist={setShowBulkBlacklist}
          bulkBlacklistText={bulkBlacklistText}
          setBulkBlacklistText={setBulkBlacklistText}
          handleConfirmBatchBlacklist={handleConfirmBatchBlacklist}
          blacklistSearch={blacklistSearch}
          setBlacklistSearch={setBlacklistSearch}
          handleDeleteRule={handleDeleteRule}
          mergedAllowedItems={mergedAllowedItems}
          filteredAllowed={filteredAllowed}
          newAllowedDomain={newAllowedDomain}
          setNewAllowedDomain={setNewAllowedDomain}
          isAddingAllowed={isAddingAllowed}
          handleAddAllowedDomain={handleAddAllowedDomain}
          showBulkWhitelist={showBulkWhitelist}
          setShowBulkWhitelist={setShowBulkWhitelist}
          bulkWhitelistText={bulkWhitelistText}
          setBulkWhitelistText={setBulkWhitelistText}
          handleConfirmBatchWhitelist={handleConfirmBatchWhitelist}
          whitelistSearch={whitelistSearch}
          setWhitelistSearch={setWhitelistSearch}
          cloudPolicy={cloudPolicy}
          newApp={newApp}
          setNewApp={setNewApp}
          appQuota={appQuota}
          setAppQuota={setAppQuota}
          isAddingApp={isAddingApp}
          handleAddApp={handleAddApp}
          appRules={appRules}
          appColumns={appColumns}
        />
      )}

      {/* ================= SUBTAB 3: KEYBOARD MONITORING ================= */}
      {activeSubTab === 'keywords' && (
        <KeywordsSubTab
          isVi={isVi}
          isKeywordsEnabled={isKeywordsEnabled}
          isTogglingKeywords={isTogglingKeywords}
          onToggleKeyboardMonitoring={handleToggleKeyboardMonitoring}
          customKeywords={customKeywords}
          newKwPattern={newKwPattern}
          setNewKwPattern={setNewKwPattern}
          newKwCategory={newKwCategory}
          setNewKwCategory={setNewKwCategory}
          newKwSeverity={newKwSeverity}
          setNewKwSeverity={setNewKwSeverity}
          isAddingKw={isAddingKw}
          onAddCustomKeyword={handleAddCustomKeyword}
          kwSearch={kwSearch}
          setKwSearch={setKwSearch}
          filteredKeywords={filteredKeywords}
          onToggleKeyword={handleToggleKeyword}
          onDeleteCustomKeyword={handleDeleteCustomKeyword}
        />
      )}

      {/* EXCEL IMPORT PREVIEW MODAL */}
      <ExcelImportModal
        isOpen={importModalOpen}
        onClose={() => setImportModalOpen(false)}
        importFileName={importFileName}
        previewRules={previewRules}
        isImporting={isImporting}
        onConfirmImport={handleConfirmImport}
        isVi={isVi}
      />

      {/* CATEGORY DRILLDOWN & HIGH-PERFORMANCE FILE/DOMAIN MANAGEMENT MODAL */}
      {selectedCategoryModal && (() => {
        const catObj = STANDARD_CATEGORIES.find((c) => c.id === selectedCategoryModal);
        if (!catObj) return null;
        const isBlocked =
          catObj.id === 'adult' ? !!cloudPolicy?.block_adult :
          catObj.id === 'gambling' ? !!cloudPolicy?.block_gambling :
          catObj.id === 'violence' ? !!cloudPolicy?.block_violence :
          catObj.id === 'social' ? !!cloudPolicy?.block_social :
          catObj.id === 'gaming' ? !!cloudPolicy?.block_gaming : false;

        return (
          <CategoryDetailModal
            isOpen={true}
            onClose={() => setSelectedCategoryModal(null)}
            categoryId={catObj.id}
            categoryName={catObj.name}
            categoryNameEn={catObj.nameEn}
            categoryIcon={catObj.icon}
            categoryDesc={catObj.desc}
            categoryDescEn={catObj.descEn}
            systemSeeds={catObj.seeds}
            isBlocked={isBlocked}
            onToggleBlock={handleToggleCategoryBlock}
            cloudPolicy={cloudPolicy}
            onUpdateCategoryDomains={handleUpdateCategoryDomains}
            isVi={isVi}
            onShowToast={onShowToast}
          />
        );
      })()}
    </div>
  );
};
