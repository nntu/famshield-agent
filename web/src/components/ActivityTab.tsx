import React, { useState, useEffect } from 'react';
import {
  Clock,
  Smartphone,
  Globe,
  ShieldAlert,
  Search,
  RefreshCw,
  Download,
  Filter,
  CheckCircle,
  AlertTriangle,
  Layers,
  Calendar,
  Gamepad2,
  Tv,
  MessageCircle,
  BookOpen,
  Code2,
  ExternalLink,
} from 'lucide-react';
import { ActivitySummaryResponse, Language } from '../types';
import { translations } from '../i18n';
import { api } from '../api';

interface ActivityTabProps {
  lang: Language;
  onRefresh?: () => void;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

type ViewSubTab = 'unified' | 'apps' | 'web' | 'keywords';

export const ActivityTab: React.FC<ActivityTabProps> = ({
  lang,
  onRefresh,
  onShowToast,
}) => {
  const t = translations[lang];
  const isVi = lang === 'vi';

  const [data, setData] = useState<ActivitySummaryResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [activeSubTab, setActiveSubTab] = useState<ViewSubTab>('unified');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedCategory, setSelectedCategory] = useState<string>('ALL');
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);

  const fetchActivity = async () => {
    try {
      setLoading(true);
      const res = await api.getActivitySummary();
      setData(res);
    } catch (err: any) {
      onShowToast(err.message || (isVi ? 'Lỗi tải nhật ký hoạt động' : 'Failed to load activity logs'), 'error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchActivity();
  }, []);

  useEffect(() => {
    if (!autoRefresh) return;
    const timer = setInterval(() => {
      api.getActivitySummary().then(setData).catch(() => {});
    }, 6000);
    return () => clearInterval(timer);
  }, [autoRefresh]);

  const handleExport = () => {
    if (!data) return;
    const jsonStr = JSON.stringify(data, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `famshield-activity-report-${data.date}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    onShowToast(isVi ? 'Đã xuất báo cáo hoạt động thành công!' : 'Activity report exported successfully!', 'success');
  };

  const getCategoryIcon = (category: string) => {
    const lower = category.toLowerCase();
    if (lower.includes('trò chơi') || lower.includes('gaming')) {
      return <Gamepad2 size={16} color="#ec4899" />;
    }
    if (lower.includes('giải trí') || lower.includes('video') || lower.includes('entertainment')) {
      return <Tv size={16} color="#38bdf8" />;
    }
    if (lower.includes('mạng xã hội') || lower.includes('social') || lower.includes('liên lạc')) {
      return <MessageCircle size={16} color="#a855f7" />;
    }
    if (lower.includes('học tập') || lower.includes('giáo dục') || lower.includes('education')) {
      return <BookOpen size={16} color="#34d399" />;
    }
    if (lower.includes('lập trình') || lower.includes('development')) {
      return <Code2 size={16} color="#f59e0b" />;
    }
    return <Globe size={16} color="#94a3b8" />;
  };

  // Filtered lists
  const filteredTimeline = (data?.unified_timeline || []).filter((item) => {
    const matchSearch =
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.subtitle.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat = selectedCategory === 'ALL' || item.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredApps = (data?.app_activities || []).filter((app) => {
    const matchSearch =
      app.app_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      app.app_title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      app.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat = selectedCategory === 'ALL' || app.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredWeb = (data?.web_activities || []).filter((web) => {
    const matchSearch =
      web.domain.toLowerCase().includes(searchQuery.toLowerCase()) ||
      web.category.toLowerCase().includes(searchQuery.toLowerCase()) ||
      web.source_app.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat = selectedCategory === 'ALL' || web.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  const filteredKeywords = (data?.keyword_violations || []).filter((kv) => {
    const matchSearch =
      kv.keyword_matched.toLowerCase().includes(searchQuery.toLowerCase()) ||
      kv.app_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      kv.category.toLowerCase().includes(searchQuery.toLowerCase());
    const matchCat = selectedCategory === 'ALL' || kv.category.toLowerCase().includes(selectedCategory.toLowerCase());
    return matchSearch && matchCat;
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
      {/* Header Banner */}
      <div style={{
        background: 'linear-gradient(135deg, rgba(15, 23, 42, 0.9) 0%, rgba(30, 41, 59, 0.7) 100%)',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: '24px 28px',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: 16,
        boxShadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
      }}>
        <div>
          <h2 style={{ fontSize: '1.4rem', fontWeight: 800, color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 12, margin: 0 }}>
            <Clock size={26} color="#38bdf8" />
            {t.activity_title}
          </h2>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginTop: 6, marginBottom: 0, maxWidth: 800, lineHeight: 1.5 }}>
            {t.activity_subtitle}
          </p>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
          {/* Auto refresh button */}
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            style={{
              background: autoRefresh ? 'rgba(52, 211, 153, 0.15)' : '#1e293b',
              color: autoRefresh ? '#34d399' : '#94a3b8',
              border: `1px solid ${autoRefresh ? 'rgba(52, 211, 153, 0.4)' : '#334155'}`,
              padding: '8px 14px',
              borderRadius: 8,
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
            }}
          >
            <RefreshCw size={14} className={autoRefresh ? 'animate-spin' : ''} />
            <span>{autoRefresh ? (isVi ? 'Đang tự làm mới' : 'Auto-refresh On') : (isVi ? 'Tự làm mới: Tắt' : 'Auto-refresh Off')}</span>
          </button>

          {/* Refresh Manual */}
          <button
            onClick={fetchActivity}
            disabled={loading}
            style={{
              background: '#1e293b',
              color: '#38bdf8',
              border: '1px solid #334155',
              padding: '8px 14px',
              borderRadius: 8,
              fontSize: '0.82rem',
              fontWeight: 600,
              cursor: loading ? 'not-allowed' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
            }}
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            <span>{t.btn_refresh}</span>
          </button>

          {/* Export Report */}
          <button
            onClick={handleExport}
            style={{
              background: 'linear-gradient(135deg, #0284c7, #38bdf8)',
              color: '#070b14',
              border: 'none',
              padding: '8px 16px',
              borderRadius: 8,
              fontSize: '0.85rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              boxShadow: '0 4px 12px rgba(56, 189, 248, 0.3)',
            }}
          >
            <Download size={15} />
            <span>{t.btn_export_activity}</span>
          </button>
        </div>
      </div>

      {/* 4 KPI Summary Cards */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))',
        gap: 18,
      }}>
        {/* Card 1: Total Screen Time */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
          position: 'relative',
          overflow: 'hidden',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.88rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Clock size={16} color="#38bdf8" />
              {t.stat_screen_time}
            </span>
            <span style={{
              background: 'rgba(56, 189, 248, 0.15)',
              color: '#38bdf8',
              padding: '2px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {data?.date || new Date().toISOString().slice(0, 10)}
            </span>
          </div>

          <div style={{ fontSize: '1.9rem', fontWeight: 800, color: '#38bdf8', marginBottom: 6 }}>
            {data?.total_screen_time_formatted || '0 phút'}
          </div>

          <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
            {t.stat_screen_time_sub}
          </div>
        </div>

        {/* Card 2: Applications Used */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.88rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Smartphone size={16} color="#a855f7" />
              {t.stat_apps_opened}
            </span>
            <span style={{
              background: 'rgba(168, 85, 247, 0.15)',
              color: '#c084fc',
              padding: '2px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {data?.active_apps_count || 0} apps
            </span>
          </div>

          <div style={{ fontSize: '1.9rem', fontWeight: 800, color: '#c084fc', marginBottom: 6 }}>
            {data?.active_apps_count || 0}
          </div>

          <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
            {t.stat_apps_opened_sub}
          </div>
        </div>

        {/* Card 3: Websites Visited */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.88rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <Globe size={16} color="#34d399" />
              {t.stat_web_visits}
            </span>
            <span style={{
              background: 'rgba(52, 211, 153, 0.15)',
              color: '#34d399',
              padding: '2px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              DNS Loopback
            </span>
          </div>

          <div style={{ fontSize: '1.9rem', fontWeight: 800, color: '#34d399', marginBottom: 6 }}>
            {data?.total_web_visits_count || 0}
          </div>

          <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
            {t.stat_web_visits_sub}
          </div>
        </div>

        {/* Card 4: Blocked & Monitored Violations */}
        <div style={{
          background: '#0f172a',
          border: '1px solid #1e293b',
          borderRadius: 16,
          padding: 22,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.88rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <ShieldAlert size={16} color="#f87171" />
              {t.stat_violations}
            </span>
            <span style={{
              background: 'rgba(248, 113, 113, 0.15)',
              color: '#f87171',
              padding: '2px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              {data?.blocked_events_count || 0} Block
            </span>
          </div>

          <div style={{ fontSize: '1.9rem', fontWeight: 800, color: '#f87171', marginBottom: 6 }}>
            {(data?.blocked_events_count || 0) + (data?.monitored_events_count || 0)}
          </div>

          <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
            {t.stat_violations_sub}
          </div>
        </div>

        {/* Card 5: Sensitive Keyword Violations */}
        <div
          onClick={() => setActiveSubTab('keywords')}
          style={{
            background: '#0f172a',
            border: activeSubTab === 'keywords' ? '1px solid #f59e0b' : '1px solid #1e293b',
            borderRadius: 16,
            padding: 22,
            cursor: 'pointer',
            transition: 'all 0.2s ease',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 10 }}>
            <span style={{ fontSize: '0.88rem', color: '#94a3b8', fontWeight: 600, display: 'flex', alignItems: 'center', gap: 8 }}>
              <AlertTriangle size={16} color="#fbbf24" />
              {t.stat_keywords}
            </span>
            <span style={{
              background: 'rgba(245, 158, 11, 0.15)',
              color: '#fbbf24',
              padding: '2px 8px',
              borderRadius: 6,
              fontSize: '0.75rem',
              fontWeight: 700,
            }}>
              RAM Buffer
            </span>
          </div>

          <div style={{ fontSize: '1.9rem', fontWeight: 800, color: '#fbbf24', marginBottom: 6 }}>
            {data?.keyword_violations_count || 0}
          </div>

          <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
            {t.stat_keywords_sub}
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <div style={{
        background: '#0f172a',
        border: '1px solid #1e293b',
        borderRadius: 20,
        padding: 24,
        boxShadow: '0 10px 30px rgba(0, 0, 0, 0.3)',
      }}>
        {/* Navigation Tabs for Views & Search Bar */}
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          flexWrap: 'wrap',
          gap: 16,
          borderBottom: '1px solid #1e293b',
          paddingBottom: 20,
          marginBottom: 20,
        }}>
          {/* Subtabs */}
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            <button
              onClick={() => setActiveSubTab('unified')}
              style={{
                background: activeSubTab === 'unified' ? 'rgba(56, 189, 248, 0.15)' : '#070b14',
                color: activeSubTab === 'unified' ? '#38bdf8' : '#94a3b8',
                border: `1px solid ${activeSubTab === 'unified' ? '#38bdf8' : '#1e293b'}`,
                padding: '9px 18px',
                borderRadius: 10,
                fontSize: '0.88rem',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s ease',
              }}
            >
              <span>{t.tab_unified_timeline}</span>
              <span style={{
                background: activeSubTab === 'unified' ? '#38bdf8' : '#1e293b',
                color: activeSubTab === 'unified' ? '#070b14' : '#cbd5e1',
                padding: '2px 7px',
                borderRadius: 12,
                fontSize: '0.75rem',
                fontWeight: 700,
              }}>
                {filteredTimeline.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('apps')}
              style={{
                background: activeSubTab === 'apps' ? 'rgba(168, 85, 247, 0.15)' : '#070b14',
                color: activeSubTab === 'apps' ? '#c084fc' : '#94a3b8',
                border: `1px solid ${activeSubTab === 'apps' ? '#a855f7' : '#1e293b'}`,
                padding: '9px 18px',
                borderRadius: 10,
                fontSize: '0.88rem',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s ease',
              }}
            >
              <span>{t.tab_apps_timeline}</span>
              <span style={{
                background: activeSubTab === 'apps' ? '#a855f7' : '#1e293b',
                color: activeSubTab === 'apps' ? '#070b14' : '#cbd5e1',
                padding: '2px 7px',
                borderRadius: 12,
                fontSize: '0.75rem',
                fontWeight: 700,
              }}>
                {filteredApps.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('web')}
              style={{
                background: activeSubTab === 'web' ? 'rgba(52, 211, 153, 0.15)' : '#070b14',
                color: activeSubTab === 'web' ? '#34d399' : '#94a3b8',
                border: `1px solid ${activeSubTab === 'web' ? '#34d399' : '#1e293b'}`,
                padding: '9px 18px',
                borderRadius: 10,
                fontSize: '0.88rem',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s ease',
              }}
            >
              <span>{t.tab_web_timeline}</span>
              <span style={{
                background: activeSubTab === 'web' ? '#34d399' : '#1e293b',
                color: activeSubTab === 'web' ? '#070b14' : '#cbd5e1',
                padding: '2px 7px',
                borderRadius: 12,
                fontSize: '0.75rem',
                fontWeight: 700,
              }}>
                {filteredWeb.length}
              </span>
            </button>

            <button
              onClick={() => setActiveSubTab('keywords')}
              style={{
                background: activeSubTab === 'keywords' ? 'rgba(245, 158, 11, 0.15)' : '#070b14',
                color: activeSubTab === 'keywords' ? '#fbbf24' : '#94a3b8',
                border: `1px solid ${activeSubTab === 'keywords' ? '#f59e0b' : '#1e293b'}`,
                padding: '9px 18px',
                borderRadius: 10,
                fontSize: '0.88rem',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                transition: 'all 0.2s ease',
              }}
            >
              <span>{t.tab_keywords_timeline}</span>
              <span style={{
                background: activeSubTab === 'keywords' ? '#f59e0b' : '#1e293b',
                color: activeSubTab === 'keywords' ? '#070b14' : '#cbd5e1',
                padding: '2px 7px',
                borderRadius: 12,
                fontSize: '0.75rem',
                fontWeight: 700,
              }}>
                {filteredKeywords.length}
              </span>
            </button>
          </div>

          {/* Search & Filter */}
          <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'center' }}>
            <div style={{ position: 'relative', width: 280 }}>
              <Search
                size={16}
                color="#64748b"
                style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)' }}
              />
              <input
                type="text"
                placeholder={t.search_activity_placeholder}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{
                  width: '100%',
                  background: '#070b14',
                  border: '1px solid #1e293b',
                  color: '#fff',
                  padding: '9px 12px 9px 36px',
                  borderRadius: 8,
                  fontSize: '0.85rem',
                  outline: 'none',
                }}
              />
            </div>

            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              style={{
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#cbd5e1',
                padding: '9px 14px',
                borderRadius: 8,
                fontSize: '0.85rem',
                outline: 'none',
                cursor: 'pointer',
              }}
            >
              <option value="ALL">{t.filter_all_cats}</option>
              <option value="Trò chơi">🎮 Trò chơi (Gaming)</option>
              <option value="Giải trí">📺 Giải trí (Entertainment)</option>
              <option value="Mạng xã hội">💬 Mạng xã hội (Social)</option>
              <option value="Học tập">📚 Học tập (Education)</option>
              <option value="Trình duyệt">🌐 Trình duyệt (Browser)</option>
            </select>
          </div>
        </div>

        {/* VIEW 1: UNIFIED TIMELINE */}
        {activeSubTab === 'unified' && (
          <div>
            {filteredTimeline.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '40px 20px', color: '#64748b', fontSize: '0.9rem' }}>
                {t.no_activity_found}
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12, position: 'relative' }}>
                {filteredTimeline.map((item) => {
                  const isApp = item.item_type === 'app';
                  const isKeyword = item.item_type === 'keyword';
                  const dateObj = new Date(item.timestamp);
                  const timeFormatted = dateObj.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

                  return (
                    <div
                      key={item.id}
                      style={{
                        background: '#070b14',
                        border: isKeyword ? '1px solid rgba(245, 158, 11, 0.4)' : '1px solid #1e293b',
                        borderRadius: 12,
                        padding: '14px 18px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        flexWrap: 'wrap',
                        gap: 12,
                        transition: 'border-color 0.2s ease',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                        <div style={{
                          width: 40,
                          height: 40,
                          borderRadius: 10,
                          background: isKeyword
                            ? 'rgba(245, 158, 11, 0.15)'
                            : isApp
                            ? 'rgba(168, 85, 247, 0.15)'
                            : 'rgba(56, 189, 248, 0.15)',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}>
                          {isKeyword ? (
                            <AlertTriangle size={20} color="#fbbf24" />
                          ) : isApp ? (
                            <Smartphone size={20} color="#c084fc" />
                          ) : (
                            <Globe size={20} color="#38bdf8" />
                          )}
                        </div>

                        <div>
                          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 4 }}>
                            <span style={{ fontWeight: 700, color: isKeyword ? '#fef08a' : '#f8fafc', fontSize: '0.95rem' }}>
                              {item.title}
                            </span>
                            <span style={{
                              background: '#1e293b',
                              color: '#94a3b8',
                              padding: '2px 8px',
                              borderRadius: 6,
                              fontSize: '0.72rem',
                              fontWeight: 600,
                              display: 'flex',
                              alignItems: 'center',
                              gap: 4,
                            }}>
                              {getCategoryIcon(item.category)}
                              {item.category}
                            </span>
                          </div>

                          <div style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
                            {item.subtitle}
                          </div>
                        </div>
                      </div>

                      <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                        <div style={{ textAlign: 'right' }}>
                          <div style={{ fontSize: '0.88rem', fontWeight: 700, color: '#cbd5e1', fontFamily: 'monospace' }}>
                            {timeFormatted}
                          </div>
                          <div style={{ fontSize: '0.75rem', color: '#64748b' }}>
                            {dateObj.toLocaleDateString()}
                          </div>
                        </div>

                        {/* Action Badge */}
                        <span style={{
                          padding: '4px 10px',
                          borderRadius: 6,
                          fontSize: '0.75rem',
                          fontWeight: 700,
                          background:
                            item.action === 'VIOLATION'
                              ? 'rgba(245, 158, 11, 0.2)'
                              : item.action === 'BLOCK'
                              ? 'rgba(239, 68, 68, 0.15)'
                              : item.action === 'MONITOR'
                              ? 'rgba(245, 158, 11, 0.15)'
                              : item.action === 'ACTIVE'
                              ? 'rgba(168, 85, 247, 0.15)'
                              : 'rgba(52, 211, 153, 0.15)',
                          color:
                            item.action === 'VIOLATION'
                              ? '#fbbf24'
                              : item.action === 'BLOCK'
                              ? '#ef4444'
                              : item.action === 'MONITOR'
                              ? '#f59e0b'
                              : item.action === 'ACTIVE'
                              ? '#c084fc'
                              : '#34d399',
                          border: `1px solid ${
                            item.action === 'VIOLATION'
                              ? 'rgba(245, 158, 11, 0.4)'
                              : item.action === 'BLOCK'
                              ? 'rgba(239, 68, 68, 0.3)'
                              : item.action === 'MONITOR'
                              ? 'rgba(245, 158, 11, 0.3)'
                              : item.action === 'ACTIVE'
                              ? 'rgba(168, 85, 247, 0.3)'
                              : 'rgba(52, 211, 153, 0.3)'
                          }`,
                        }}>
                          {item.action === 'VIOLATION'
                            ? (isVi ? '⚠️ TỪ KHÓA VI PHẠM' : '⚠️ KEYWORD VIOLATION')
                            : item.action === 'BLOCK'
                            ? t.tag_block
                            : item.action === 'MONITOR'
                            ? (isVi ? 'GIÁM SÁT' : 'MONITOR')
                            : item.action === 'ACTIVE'
                            ? (isVi ? 'ĐANG CHẠY' : 'ACTIVE')
                            : t.tag_allow}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}

        {/* VIEW 2: APPLICATION TIMELINE */}
        {activeSubTab === 'apps' && (
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: 650 }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #1e293b' }}>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_app_col}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_category_col}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_time_window}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_duration}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_status_col}</th>
                </tr>
              </thead>
              <tbody>
                {filteredApps.length === 0 ? (
                  <tr>
                    <td colSpan={5} style={{ textAlign: 'center', padding: '30px', color: '#64748b', fontSize: '0.88rem' }}>
                      {t.no_activity_found}
                    </td>
                  </tr>
                ) : (
                  filteredApps.map((app) => {
                    const startStr = new Date(app.start_time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
                    const endStr = new Date(app.end_time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

                    return (
                      <tr key={app.id} style={{ borderBottom: '1px solid #1e293b' }}>
                        <td style={{ padding: '14px 16px' }}>
                          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                            <Smartphone size={18} color="#c084fc" />
                            <div>
                              <div style={{ fontWeight: 700, color: '#f8fafc', fontSize: '0.9rem' }}>{app.app_title}</div>
                              <div style={{ fontSize: '0.75rem', color: '#64748b', fontFamily: 'monospace' }}>{app.app_name}</div>
                            </div>
                          </div>
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <span style={{
                            background: '#1e293b',
                            color: '#cbd5e1',
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.78rem',
                            fontWeight: 600,
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: 6,
                          }}>
                            {getCategoryIcon(app.category)}
                            {app.category}
                          </span>
                        </td>
                        <td style={{ padding: '14px 16px', color: '#cbd5e1', fontSize: '0.85rem', fontFamily: 'monospace' }}>
                          {startStr} &rarr; {endStr}
                        </td>
                        <td style={{ padding: '14px 16px', color: '#38bdf8', fontSize: '0.9rem', fontWeight: 700 }}>
                          {app.duration_formatted}
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <span style={{
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 700,
                            background:
                              app.status === 'Active'
                                ? 'rgba(52, 211, 153, 0.15)'
                                : app.status === 'QuotaWarning'
                                ? 'rgba(245, 158, 11, 0.15)'
                                : 'rgba(56, 189, 248, 0.15)',
                            color:
                              app.status === 'Active'
                                ? '#34d399'
                                : app.status === 'QuotaWarning'
                                ? '#f59e0b'
                                : '#38bdf8',
                          }}>
                            {app.status === 'Active'
                              ? (isVi ? 'Đang chạy' : 'Active')
                              : app.status === 'QuotaWarning'
                              ? (isVi ? 'Cảnh báo hạn mức' : 'Quota Warning')
                              : (isVi ? 'Bình thường' : 'Normal')}
                          </span>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        )}

        {/* VIEW 3: WEB & DOMAIN HISTORY */}
        {activeSubTab === 'web' && (
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: 650 }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #1e293b' }}>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_time}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_domain_col}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_category_col}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_source_app}</th>
                  <th style={{ padding: '12px 16px', color: '#64748b', fontSize: '0.8rem', fontWeight: 600 }}>{t.th_action_col}</th>
                </tr>
              </thead>
              <tbody>
                {filteredWeb.length === 0 ? (
                  <tr>
                    <td colSpan={5} style={{ textAlign: 'center', padding: '30px', color: '#64748b', fontSize: '0.88rem' }}>
                      {t.no_activity_found}
                    </td>
                  </tr>
                ) : (
                  filteredWeb.map((web) => {
                    const timeStr = new Date(web.occurred_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

                    return (
                      <tr key={web.id} style={{ borderBottom: '1px solid #1e293b' }}>
                        <td style={{ padding: '14px 16px', color: '#94a3b8', fontSize: '0.82rem', fontFamily: 'monospace' }}>
                          {timeStr}
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                            <Globe size={15} color="#38bdf8" />
                            <span style={{ fontWeight: 700, color: '#f8fafc', fontSize: '0.88rem' }}>{web.domain}</span>
                          </div>
                          {web.reason && (
                            <div style={{ fontSize: '0.75rem', color: '#64748b', marginTop: 2 }}>{web.reason}</div>
                          )}
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <span style={{
                            background: '#1e293b',
                            color: '#cbd5e1',
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 600,
                            display: 'inline-flex',
                            alignItems: 'center',
                            gap: 6,
                          }}>
                            {getCategoryIcon(web.category)}
                            {web.category}
                          </span>
                        </td>
                        <td style={{ padding: '14px 16px', color: '#94a3b8', fontSize: '0.82rem', fontFamily: 'monospace' }}>
                          {web.source_app}
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <span style={{
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                            fontWeight: 700,
                            background:
                              web.action === 'BLOCK'
                                ? 'rgba(239, 68, 68, 0.15)'
                                : web.action === 'MONITOR'
                                ? 'rgba(245, 158, 11, 0.15)'
                                : 'rgba(52, 211, 153, 0.15)',
                            color:
                              web.action === 'BLOCK'
                                ? '#ef4444'
                                : web.action === 'MONITOR'
                                ? '#f59e0b'
                                : '#34d399',
                            border: `1px solid ${
                              web.action === 'BLOCK'
                                ? 'rgba(239, 68, 68, 0.3)'
                                : web.action === 'MONITOR'
                                ? 'rgba(245, 158, 11, 0.3)'
                                : 'rgba(52, 211, 153, 0.3)'
                            }`,
                          }}>
                            {web.action === 'BLOCK'
                              ? t.tag_block
                              : web.action === 'MONITOR'
                              ? (isVi ? 'GIÁM SÁT' : 'MONITOR')
                              : t.tag_allow}
                          </span>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        )}

        {/* VIEW 4: KEYWORD VIOLATIONS */}
        {activeSubTab === 'keywords' && (
          <div>
            {filteredKeywords.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '40px 20px', color: '#64748b', fontSize: '0.9rem' }}>
                {isVi ? 'Không phát hiện từ khóa vi phạm nào.' : 'No keyword violations recorded.'}
              </div>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '0.88rem' }}>
                  <thead>
                    <tr style={{ borderBottom: '1px solid #1e293b', color: '#64748b', fontSize: '0.78rem', textTransform: 'uppercase' }}>
                      <th style={{ padding: '12px 16px' }}>{t.th_time}</th>
                      <th style={{ padding: '12px 16px' }}>{t.th_kw_pattern}</th>
                      <th style={{ padding: '12px 16px' }}>{t.th_kw_category}</th>
                      <th style={{ padding: '12px 16px' }}>{t.th_kw_severity}</th>
                      <th style={{ padding: '12px 16px' }}>{t.th_kw_app}</th>
                      <th style={{ padding: '12px 16px' }}>{t.th_kw_snippet}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredKeywords.map((kv) => (
                      <tr key={kv.id} style={{ borderBottom: '1px solid rgba(30, 41, 59, 0.6)' }}>
                        <td style={{ padding: '14px 16px', color: '#94a3b8', fontFamily: 'monospace' }}>
                          {new Date(kv.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                        </td>
                        <td style={{ padding: '14px 16px', fontWeight: 700, color: '#f87171' }}>
                          "{kv.keyword_matched}"
                        </td>
                        <td style={{ padding: '14px 16px', color: '#e2e8f0' }}>
                          <span style={{
                            background: '#1e293b',
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.75rem',
                          }}>
                            {kv.category}
                          </span>
                        </td>
                        <td style={{ padding: '14px 16px' }}>
                          <span style={{
                            padding: '3px 8px',
                            borderRadius: 6,
                            fontSize: '0.72rem',
                            fontWeight: 700,
                            textTransform: 'uppercase',
                            background: kv.severity === 'critical' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(245, 158, 11, 0.2)',
                            color: kv.severity === 'critical' ? '#ef4444' : '#fbbf24',
                            border: `1px solid ${kv.severity === 'critical' ? 'rgba(239, 68, 68, 0.4)' : 'rgba(245, 158, 11, 0.4)'}`,
                          }}>
                            {kv.severity}
                          </span>
                        </td>
                        <td style={{ padding: '14px 16px', color: '#c084fc', fontWeight: 600 }}>
                          {kv.app_name}
                        </td>
                        <td style={{ padding: '14px 16px', color: '#94a3b8', fontStyle: 'italic', fontFamily: 'monospace' }}>
                          {kv.snippet_masked}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
