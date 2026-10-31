import type React from 'react';
import { useState, useEffect } from 'react';
import { Globe, RefreshCw, Check, Activity } from 'lucide-react';
import { api } from '../../api';
import type { DnsConfigResponse, Language } from '../../types';

interface DnsConfigCardProps {
  lang: Language;
  dnsConfig: DnsConfigResponse | null;
  onRefreshDns: () => Promise<void>;
  onShowToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
}

export const DnsConfigCard: React.FC<DnsConfigCardProps> = ({
  lang,
  dnsConfig,
  onRefreshDns,
  onShowToast,
}) => {
  const isVi = lang === 'vi';

  const [selectedUpstream, setSelectedUpstream] = useState('1.1.1.1');
  const [customUpstreamInput, setCustomUpstreamInput] = useState('');
  const [isSavingDns, setIsSavingDns] = useState(false);
  const [isFlushingCache, setIsFlushingCache] = useState(false);

  useEffect(() => {
    if (dnsConfig?.current_upstream) {
      setSelectedUpstream(dnsConfig.current_upstream);
    }
  }, [dnsConfig?.current_upstream]);

  const handleSaveDnsConfig = async () => {
    const target = selectedUpstream === 'custom' ? customUpstreamInput.trim() : selectedUpstream;
    if (!target) {
      onShowToast(
        isVi ? 'Vui lòng chọn hoặc nhập máy chủ DNS' : 'Please select or enter DNS server',
        'error'
      );
      return;
    }
    setIsSavingDns(true);
    try {
      const res = await api.setDnsConfig(target);
      onShowToast(
        res.message || (isVi ? 'Đã lưu cấu hình DNS' : 'DNS configured successfully'),
        'success'
      );
      await onRefreshDns();
    } catch (e: any) {
      onShowToast(
        e.message || (isVi ? 'Lỗi lưu cấu hình DNS' : 'Failed to save DNS configuration'),
        'error'
      );
    } finally {
      setIsSavingDns(false);
    }
  };

  const handleFlushCache = async () => {
    setIsFlushingCache(true);
    try {
      const res = await api.flushDnsCache();
      onShowToast(res.message || (isVi ? 'Đã xóa bộ nhớ đệm DNS' : 'DNS cache flushed'), 'success');
      await onRefreshDns();
    } catch (e: any) {
      onShowToast(e.message || (isVi ? 'Lỗi xóa cache DNS' : 'Failed to flush DNS cache'), 'error');
    } finally {
      setIsFlushingCache(false);
    }
  };

  return (
    <div
      id="section-dns-config"
      style={{
        background: 'rgba(30, 41, 59, 0.4)',
        border: '1px solid rgba(255, 255, 255, 0.08)',
        borderRadius: 14,
        padding: 24,
        marginBottom: 32,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 16,
          marginBottom: 20,
        }}
      >
        <div>
          <h3
            style={{
              fontSize: '1.05rem',
              fontWeight: 700,
              color: '#f8fafc',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              marginBottom: 4,
            }}
          >
            <Globe size={18} style={{ color: '#38bdf8' }} />
            {isVi
              ? 'Cấu Hình DNS Chuyển Tiếp & Bộ Nhớ Đệm (DNS Upstream & Cache)'
              : 'DNS Upstream & Cache Management'}
          </h3>
          <p style={{ fontSize: '0.85rem', color: '#94a3b8', margin: 0, maxWidth: 640 }}>
            {isVi
              ? 'FamShield chuyển tiếp truy vấn phân giải tên miền an toàn qua danh sách upstream dự phòng (failover) và cache phản hồi cục bộ trong RAM (≤ 2 MB) để tăng tốc độ lướt web.'
              : 'FamShield forwards DNS queries through fallback upstream servers and caches responses locally in RAM (≤ 2 MB) to accelerate browsing speed.'}
          </p>
        </div>
        <button
          type="button"
          onClick={handleFlushCache}
          disabled={isFlushingCache}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 8,
            background: 'rgba(56, 189, 248, 0.1)',
            color: '#38bdf8',
            border: '1px solid rgba(56, 189, 248, 0.3)',
            padding: '8px 16px',
            borderRadius: 8,
            fontSize: '0.85rem',
            fontWeight: 600,
            cursor: isFlushingCache ? 'not-allowed' : 'pointer',
          }}
        >
          <RefreshCw size={15} className={isFlushingCache ? 'animate-spin' : ''} />
          {isVi ? 'Xóa Bộ Nhớ Đệm DNS' : 'Flush DNS Cache'}
        </button>
      </div>

      {/* Cache Statistics Row */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
          gap: 12,
          marginBottom: 20,
        }}
      >
        <div
          style={{
            background: 'rgba(15, 23, 42, 0.6)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            borderRadius: 10,
            padding: 14,
          }}
        >
          <span
            style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
          >
            {isVi ? 'Bản Ghi Đang Cache' : 'Active Cached Records'}
          </span>
          <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#38bdf8' }}>
            {dnsConfig?.cache_entries ?? 0}{' '}
            <span style={{ fontSize: '0.8rem', color: '#64748b' }}>/ 2.000 (RAM ≤ 2MB)</span>
          </span>
        </div>
        <div
          style={{
            background: 'rgba(15, 23, 42, 0.6)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            borderRadius: 10,
            padding: 14,
          }}
        >
          <span
            style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
          >
            {isVi ? 'Tỷ Lệ Trúng Cache (Hits)' : 'Cache Hits'}
          </span>
          <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#34d399' }}>
            {dnsConfig?.cache_hits ?? 0}
          </span>
        </div>
        <div
          style={{
            background: 'rgba(15, 23, 42, 0.6)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            borderRadius: 10,
            padding: 14,
          }}
        >
          <span
            style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
          >
            {isVi ? 'Yêu Cầu Ra Upstream (Misses)' : 'Upstream Queries (Misses)'}
          </span>
          <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#fbbf24' }}>
            {dnsConfig?.cache_misses ?? 0}
          </span>
        </div>
        <div
          style={{
            background: 'rgba(15, 23, 42, 0.6)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            borderRadius: 10,
            padding: 14,
          }}
        >
          <span
            style={{ fontSize: '0.75rem', color: '#94a3b8', display: 'block', marginBottom: 4 }}
          >
            {isVi ? 'DNS Gốc Hệ Thống Nhận Diện' : 'Detected System DNS'}
          </span>
          <span style={{ fontSize: '0.95rem', fontWeight: 600, color: '#a78bfa' }}>
            {dnsConfig?.detected_system_dns && dnsConfig.detected_system_dns.length > 0
              ? dnsConfig.detected_system_dns.join(', ')
              : isVi
                ? 'Không phát hiện'
                : 'None detected'}
          </span>
        </div>
      </div>

      {/* Upstream Selector */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        <label style={{ fontSize: '0.85rem', fontWeight: 600, color: '#cbd5e1' }}>
          {isVi
            ? 'Chọn Máy Chủ DNS Chuyển Tiếp Ưu Tiên (Primary Upstream):'
            : 'Select Primary Upstream DNS Server:'}
        </label>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
            gap: 10,
          }}
        >
          {/* Detected System DNS Option (if any) */}
          {dnsConfig?.detected_system_dns?.map((sysDns) => (
            <div
              key={sysDns}
              onClick={() => setSelectedUpstream(sysDns)}
              style={{
                padding: '12px 14px',
                borderRadius: 10,
                cursor: 'pointer',
                border: `1px solid ${selectedUpstream === sysDns ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                background:
                  selectedUpstream === sysDns
                    ? 'rgba(56, 189, 248, 0.12)'
                    : 'rgba(15, 23, 42, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div>
                <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                  {isVi ? 'DNS Gốc Của Máy / Router' : 'System / Router DNS'}
                </div>
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>{sysDns}</div>
              </div>
              {selectedUpstream === sysDns && <Check size={16} color="#38bdf8" />}
            </div>
          ))}

          {/* Cloudflare 1.1.1.1 */}
          <div
            onClick={() => setSelectedUpstream('1.1.1.1')}
            style={{
              padding: '12px 14px',
              borderRadius: 10,
              cursor: 'pointer',
              border: `1px solid ${selectedUpstream === '1.1.1.1' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
              background:
                selectedUpstream === '1.1.1.1'
                  ? 'rgba(56, 189, 248, 0.12)'
                  : 'rgba(15, 23, 42, 0.4)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                Cloudflare DNS
              </div>
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                1.1.1.1 (Tốc độ cao & Riêng tư)
              </div>
            </div>
            {selectedUpstream === '1.1.1.1' && <Check size={16} color="#38bdf8" />}
          </div>

          {/* Google DNS 8.8.8.8 */}
          <div
            onClick={() => setSelectedUpstream('8.8.8.8')}
            style={{
              padding: '12px 14px',
              borderRadius: 10,
              cursor: 'pointer',
              border: `1px solid ${selectedUpstream === '8.8.8.8' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
              background:
                selectedUpstream === '8.8.8.8'
                  ? 'rgba(56, 189, 248, 0.12)'
                  : 'rgba(15, 23, 42, 0.4)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                Google Public DNS
              </div>
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                8.8.8.8 (Độ ổn định toàn cầu)
              </div>
            </div>
            {selectedUpstream === '8.8.8.8' && <Check size={16} color="#38bdf8" />}
          </div>

          {/* Quad9 9.9.9.9 */}
          <div
            onClick={() => setSelectedUpstream('9.9.9.9')}
            style={{
              padding: '12px 14px',
              borderRadius: 10,
              cursor: 'pointer',
              border: `1px solid ${selectedUpstream === '9.9.9.9' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
              background:
                selectedUpstream === '9.9.9.9'
                  ? 'rgba(56, 189, 248, 0.12)'
                  : 'rgba(15, 23, 42, 0.4)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                Quad9 DNS
              </div>
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                9.9.9.9 (Lọc mã độc đám mây)
              </div>
            </div>
            {selectedUpstream === '9.9.9.9' && <Check size={16} color="#38bdf8" />}
          </div>

          {/* Custom Option */}
          <div
            onClick={() => setSelectedUpstream('custom')}
            style={{
              padding: '12px 14px',
              borderRadius: 10,
              cursor: 'pointer',
              border: `1px solid ${selectedUpstream === 'custom' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
              background:
                selectedUpstream === 'custom'
                  ? 'rgba(56, 189, 248, 0.12)'
                  : 'rgba(15, 23, 42, 0.4)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f8fafc' }}>
                {isVi ? 'Tùy Chỉnh Khác...' : 'Custom Upstream...'}
              </div>
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>IP:Port hoặc DNS nội bộ</div>
            </div>
            {selectedUpstream === 'custom' && <Check size={16} color="#38bdf8" />}
          </div>
        </div>

        {selectedUpstream === 'custom' && (
          <div style={{ marginTop: 8 }}>
            <input
              type="text"
              value={customUpstreamInput}
              onChange={(e) => setCustomUpstreamInput(e.target.value)}
              placeholder={
                isVi
                  ? 'Nhập địa chỉ DNS upstream (ví dụ: 192.168.1.1 hoặc 10.0.0.1:53)'
                  : 'Enter DNS upstream address (e.g., 192.168.1.1 or 10.0.0.1:53)'
              }
              style={{
                width: '100%',
                background: 'rgba(15, 23, 42, 0.6)',
                border: '1px solid rgba(255, 255, 255, 0.15)',
                borderRadius: 8,
                padding: '10px 14px',
                color: '#ffffff',
                fontSize: '0.85rem',
              }}
            />
          </div>
        )}

        <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 12 }}>
          <button
            type="button"
            onClick={handleSaveDnsConfig}
            disabled={isSavingDns}
            style={{
              background: '#0284c7',
              color: '#ffffff',
              border: 'none',
              padding: '9px 20px',
              borderRadius: 8,
              fontSize: '0.85rem',
              fontWeight: 600,
              cursor: isSavingDns ? 'not-allowed' : 'pointer',
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
            }}
          >
            {isSavingDns
              ? isVi
                ? 'Đang lưu...'
                : 'Saving...'
              : isVi
                ? 'Áp Dụng Cấu Hình DNS'
                : 'Apply DNS Settings'}
          </button>
        </div>
      </div>

      {/* NUMA Dynamic Learning DNS Balancer Diagnostics */}
      {dnsConfig?.upstream_diagnostics && dnsConfig.upstream_diagnostics.length > 0 && (
        <div
          style={{
            marginTop: 18,
            padding: 16,
            borderRadius: 12,
            background: 'rgba(15, 23, 42, 0.5)',
            border: '1px solid rgba(56, 189, 248, 0.2)',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              flexWrap: 'wrap',
              gap: 8,
              marginBottom: 12,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Activity size={16} color="#38bdf8" />
              <span style={{ fontSize: '0.85rem', fontWeight: 700, color: '#f8fafc' }}>
                {isVi
                  ? 'Bộ Cân Bằng Tải Tự Học NUMA (Dynamic DNS Balancer)'
                  : 'NUMA Dynamic Learning DNS Balancer'}
              </span>
            </div>
            {dnsConfig.optimal_upstream && (
              <span
                style={{
                  fontSize: '0.75rem',
                  fontWeight: 600,
                  background: 'rgba(56, 189, 248, 0.15)',
                  color: '#38bdf8',
                  border: '1px solid rgba(56, 189, 248, 0.3)',
                  padding: '3px 8px',
                  borderRadius: 6,
                }}
              >
                🚀 {isVi ? 'Đang chọn tối ưu:' : 'Active Optimal:'} {dnsConfig.optimal_upstream}
              </span>
            )}
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
              gap: 10,
            }}
          >
            {dnsConfig.upstream_diagnostics.map((diag) => (
              <div
                key={diag.addr}
                style={{
                  padding: 12,
                  borderRadius: 8,
                  background: diag.is_optimal
                    ? 'rgba(56, 189, 248, 0.08)'
                    : 'rgba(255, 255, 255, 0.03)',
                  border: `1px solid ${
                    diag.is_optimal ? 'rgba(56, 189, 248, 0.4)' : 'rgba(255, 255, 255, 0.06)'
                  }`,
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    marginBottom: 6,
                  }}
                >
                  <span style={{ fontSize: '0.8rem', fontWeight: 700, color: '#f8fafc' }}>
                    {diag.target}
                  </span>
                  {diag.is_optimal && (
                    <span
                      style={{
                        fontSize: '0.65rem',
                        fontWeight: 700,
                        color: '#38bdf8',
                        background: 'rgba(56, 189, 248, 0.2)',
                        padding: '1px 6px',
                        borderRadius: 4,
                      }}
                    >
                      OPTIMAL
                    </span>
                  )}
                </div>
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    fontSize: '0.75rem',
                    color: '#94a3b8',
                    marginBottom: 3,
                  }}
                >
                  <span>{isVi ? 'Độ trễ (RTT):' : 'Latency (RTT):'}</span>
                  <strong
                    style={{
                      color:
                        diag.latency_ms < 25
                          ? '#4ade80'
                          : diag.latency_ms < 80
                            ? '#facc15'
                            : '#f87171',
                    }}
                  >
                    {diag.latency_ms} ms
                  </strong>
                </div>
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    fontSize: '0.7rem',
                    color: '#64748b',
                  }}
                >
                  <span>{isVi ? 'Thành công:' : 'Success:'}</span>
                  <span>
                    {diag.success_count} / {diag.total_queries}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
