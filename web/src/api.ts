import type {
  ActivitySummaryResponse,
  AppUsageSummary,
  BackupPayload,
  CaptureBlurConfig,
  DnsConfigResponse,
  DnsRetentionConfig,
  DomainQuerySummary,
  EvidenceItem,
  HourlyFrequency,
  InitializeSetupRequest,
  InitializeSetupResponse,
  KeywordConfigResponse,
  KeywordRule,
  LocalRuleRecord,
  LoginResponse,
  ProtectionModeResponse,
  QueryLogRecord,
  ResetReportsPayload,
  ResetReportsResponse,
  RestoreResponse,
  ServerConfig,
  SetDnsConfigResponse,
  SetupStatusResponse,
  StatusResponse,
  SyncResponse,
  TestConnectionResponse,
  TotpSetupResponse,
  TotpStatusResponse,
  UpdateServerConfigRequest,
} from './types';

let sessionToken = localStorage.getItem('famshield_token') || '';

export const getSessionToken = () => sessionToken;

export const setSessionToken = (token: string) => {
  sessionToken = token;
  if (token) {
    localStorage.setItem('famshield_token', token);
  } else {
    localStorage.removeItem('famshield_token');
  }
};

async function apiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers || {});
  if (sessionToken) {
    headers.set('X-FamShield-Token', sessionToken);
  }

  const response = await fetch(endpoint, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    setSessionToken('');
    window.dispatchEvent(new CustomEvent('famshield_unauthorized'));
    throw new Error('Unauthorized');
  }

  if (!response.ok) {
    let errMessage = 'Request failed';
    try {
      const err = await response.json();
      if (err.error) errMessage = err.error;
    } catch {}
    throw new Error(errMessage);
  }

  return response.json();
}

export const api = {
  async verifyAuth(): Promise<boolean> {
    if (!sessionToken) return false;
    try {
      const res = await apiRequest<{ authenticated: boolean }>('/api/auth/verify');
      return res.authenticated;
    } catch {
      return false;
    }
  },

  async login(pin: string): Promise<LoginResponse> {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pin }),
    });

    const data: LoginResponse = await res.json();
    if (res.ok && data.success) {
      if (data.token) {
        setSessionToken(data.token);
      }
      return data;
    }
    return { success: false, error: data.error || 'Mã PIN không chính xác' };
  },

  async loginTotp(tempAuthToken: string, code: string): Promise<LoginResponse> {
    const res = await fetch('/api/auth/login/totp', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ temp_auth_token: tempAuthToken, code }),
    });

    const data: LoginResponse = await res.json();
    if (res.ok && data.success && data.token) {
      setSessionToken(data.token);
      return data;
    }
    return { success: false, error: data.error || 'Mã xác thực không chính xác' };
  },

  async logout(): Promise<void> {
    try {
      await apiRequest('/api/auth/logout', { method: 'POST' });
    } catch {}
    setSessionToken('');
  },

  async getStatus(): Promise<StatusResponse> {
    return apiRequest<StatusResponse>('/api/status');
  },

  async getRules(): Promise<LocalRuleRecord[]> {
    return apiRequest<LocalRuleRecord[]>('/api/rules');
  },

  async createRule(rule: {
    rule_id: string;
    rule_type: 'domain' | 'application';
    payload_json: string;
    enabled: boolean;
  }): Promise<{ success: boolean }> {
    return apiRequest<{ success: boolean }>('/api/rules', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(rule),
    });
  },

  async createRulesBulk(
    rules: {
      rule_id: string;
      rule_type: 'domain' | 'application';
      payload_json: string;
      enabled: boolean;
    }[]
  ): Promise<{ success: boolean; imported: number }> {
    return apiRequest<{ success: boolean; imported: number }>('/api/rules/bulk', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(rules),
    });
  },

  async deleteRule(ruleId: string): Promise<{ success: boolean }> {
    return apiRequest<{ success: boolean }>(`/api/rules/${encodeURIComponent(ruleId)}`, {
      method: 'DELETE',
    });
  },

  async syncRulesToCloud(): Promise<{
    success: boolean;
    synced_domains_count: number;
    message: string;
  }> {
    return apiRequest<{ success: boolean; synced_domains_count: number; message: string }>(
      '/api/rules/sync-to-cloud',
      { method: 'POST' }
    );
  },

  async getDnsSummary(
    limit: number = 200,
    startTime?: string,
    endTime?: string
  ): Promise<DomainQuerySummary[]> {
    const params = new URLSearchParams();
    params.append('limit', limit.toString());
    if (startTime) params.append('start_time', startTime);
    if (endTime) params.append('end_time', endTime);
    return apiRequest<DomainQuerySummary[]>(`/api/logs/dns/summary?${params.toString()}`);
  },

  async getLogs(
    limit: number = 100,
    domain?: string,
    startTime?: string,
    endTime?: string,
    action?: string
  ): Promise<QueryLogRecord[]> {
    const params = new URLSearchParams();
    params.append('limit', limit.toString());
    if (domain) params.append('domain', domain);
    if (startTime) params.append('start_time', startTime);
    if (endTime) params.append('end_time', endTime);
    if (action) params.append('action', action);
    return apiRequest<QueryLogRecord[]>(`/api/logs/dns?${params.toString()}`);
  },

  async getDnsHourly(
    domain?: string,
    startTime?: string,
    endTime?: string
  ): Promise<HourlyFrequency[]> {
    const params = new URLSearchParams();
    if (domain) params.append('domain', domain);
    if (startTime) params.append('start_time', startTime);
    if (endTime) params.append('end_time', endTime);
    const qs = params.toString();
    return apiRequest<HourlyFrequency[]>(`/api/logs/dns/hourly${qs ? `?${qs}` : ''}`);
  },

  async getEvidence(limit: number = 50): Promise<EvidenceItem[]> {
    return apiRequest<EvidenceItem[]>(`/api/evidence?limit=${limit}`);
  },

  async changePin(payload: {
    current_pin?: string;
    recovery_key?: string;
    new_pin: string;
  }): Promise<{ success: boolean; recovery_key?: string }> {
    return apiRequest<{ success: boolean; recovery_key?: string }>('/api/settings/pin', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
  },

  async enrollCloud(
    code: string
  ): Promise<{ success: boolean; operating_mode: string; device_id: string }> {
    return apiRequest<{ success: boolean; operating_mode: string; device_id: string }>(
      '/api/cloud/enroll',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enrollment_code: code }),
      }
    );
  },

  async disenrollCloud(): Promise<{ success: boolean; operating_mode: string }> {
    return apiRequest<{ success: boolean; operating_mode: string }>('/api/cloud/disenroll', {
      method: 'POST',
    });
  },

  async getSetupStatus(): Promise<SetupStatusResponse> {
    const res = await fetch('/api/setup/status');
    if (!res.ok) throw new Error('Failed to get setup status');
    return res.json();
  },

  async initializeSetup(payload: InitializeSetupRequest): Promise<InitializeSetupResponse> {
    const res = await fetch('/api/setup/initialize', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const data = await res.json();
    if (!res.ok || !data.success) {
      throw new Error(data.error || 'Failed to initialize setup');
    }
    if (data.token) {
      setSessionToken(data.token);
    }
    return data;
  },

  async getServerConfig(): Promise<ServerConfig> {
    return apiRequest<ServerConfig>('/api/settings/server');
  },

  async updateServerConfig(req: UpdateServerConfigRequest): Promise<ServerConfig> {
    return apiRequest<ServerConfig>('/api/settings/server', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
  },

  async testServerConnection(serverUrl: string): Promise<TestConnectionResponse> {
    return apiRequest<TestConnectionResponse>('/api/settings/server/test', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ server_url: serverUrl }),
    });
  },

  async syncServer(): Promise<SyncResponse> {
    return apiRequest<SyncResponse>('/api/settings/server/sync', {
      method: 'POST',
    });
  },

  async getProtectionMode(): Promise<ProtectionModeResponse> {
    return apiRequest<ProtectionModeResponse>('/api/settings/protection-mode');
  },

  async setProtectionMode(monitorMode: boolean): Promise<ProtectionModeResponse> {
    return apiRequest<ProtectionModeResponse>('/api/settings/protection-mode', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ monitor_mode: monitorMode }),
    });
  },

  async exportBackup(): Promise<BackupPayload> {
    return apiRequest<BackupPayload>('/api/backup/export');
  },

  async restoreBackup(payload: BackupPayload): Promise<RestoreResponse> {
    return apiRequest<RestoreResponse>('/api/backup/restore', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
  },

  async getActivitySummary(): Promise<ActivitySummaryResponse> {
    return apiRequest<ActivitySummaryResponse>('/api/activity/summary');
  },

  async getAppUsageSummary(startTime?: string, endTime?: string): Promise<AppUsageSummary[]> {
    const params = new URLSearchParams();
    if (startTime) params.append('start_time', startTime);
    if (endTime) params.append('end_time', endTime);
    const query = params.toString() ? `?${params.toString()}` : '';
    return apiRequest<AppUsageSummary[]>(`/api/activity/apps/summary${query}`);
  },

  async getKeywordConfig(): Promise<KeywordConfigResponse> {
    return apiRequest<KeywordConfigResponse>('/api/keywords');
  },

  async addCustomKeyword(
    pattern: string,
    category?: string,
    severity?: string
  ): Promise<KeywordRule> {
    return apiRequest<KeywordRule>('/api/keywords', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pattern, category, severity }),
    });
  },

  async deleteCustomKeyword(id: string): Promise<void> {
    return apiRequest<void>(`/api/keywords/${id}`, {
      method: 'DELETE',
    });
  },

  async toggleKeyword(id: string, enabled: boolean): Promise<void> {
    return apiRequest<void>('/api/keywords/toggle', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, enabled }),
    });
  },

  async toggleKeyboardMonitoring(
    enabled: boolean
  ): Promise<{ success: boolean; enabled: boolean }> {
    return apiRequest<{ success: boolean; enabled: boolean }>('/api/keywords/monitoring', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled }),
    });
  },

  async restartSessionHelper(): Promise<{
    success: boolean;
    launched: boolean;
    running: boolean;
    message: string;
  }> {
    return apiRequest<{ success: boolean; launched: boolean; running: boolean; message: string }>(
      '/api/system/session-helper/start',
      {
        method: 'POST',
      }
    );
  },

  async uninstallSystem(
    masterPin: string,
    otpCode?: string
  ): Promise<{ success: boolean; message: string; requires_otp?: boolean }> {
    return apiRequest<{ success: boolean; message: string; requires_otp?: boolean }>(
      '/api/system/uninstall',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ master_pin: masterPin, otp_code: otpCode }),
      }
    );
  },

  async resetReports(payload: ResetReportsPayload): Promise<ResetReportsResponse> {
    return apiRequest<ResetReportsResponse>('/api/activity/reset', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
  },

  async checkSoftwareUpdate(): Promise<{
    current_version: string;
    latest_version: string;
    update_available: boolean;
    release_date?: string;
    changelog?: string[];
    download_url?: string;
    message?: string;
  }> {
    return apiRequest('/api/system/update/check');
  },

  async getTotpStatus(): Promise<TotpStatusResponse> {
    return apiRequest<TotpStatusResponse>('/api/auth/totp/status');
  },

  async setupTotp(): Promise<TotpSetupResponse> {
    return apiRequest<TotpSetupResponse>('/api/auth/totp/setup', {
      method: 'POST',
    });
  },

  async confirmTotp(code: string): Promise<{ success: boolean; message: string }> {
    return apiRequest<{ success: boolean; message: string }>('/api/auth/totp/confirm', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code }),
    });
  },

  async disableTotp(pin: string, code?: string): Promise<{ success: boolean; message: string }> {
    return apiRequest<{ success: boolean; message: string }>('/api/auth/totp/disable', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pin, code }),
    });
  },

  async getDnsConfig(): Promise<DnsConfigResponse> {
    return apiRequest<DnsConfigResponse>('/api/dns/config');
  },

  async setDnsConfig(upstream: string): Promise<SetDnsConfigResponse> {
    return apiRequest<SetDnsConfigResponse>('/api/dns/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ upstream }),
    });
  },

  async flushDnsCache(): Promise<{ success: boolean; message: string }> {
    return apiRequest<{ success: boolean; message: string }>('/api/dns/cache/flush', {
      method: 'POST',
    });
  },

  async getDnsRetention(): Promise<DnsRetentionConfig> {
    return apiRequest<DnsRetentionConfig>('/api/settings/dns-retention');
  },

  async setDnsRetention(
    retentionDays: number,
    autoPrune: boolean
  ): Promise<{ success: boolean; message: string }> {
    return apiRequest<{ success: boolean; message: string }>('/api/settings/dns-retention', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ retention_days: retentionDays, auto_prune: autoPrune }),
    });
  },

  async pruneDnsLogsNow(): Promise<{
    success: boolean;
    deleted_raw_logs: number;
    message: string;
  }> {
    return apiRequest<{ success: boolean; deleted_raw_logs: number; message: string }>(
      '/api/settings/dns-retention/prune-now',
      {
        method: 'POST',
      }
    );
  },

  async getCaptureBlur(): Promise<CaptureBlurConfig> {
    return apiRequest<CaptureBlurConfig>('/api/settings/capture-blur');
  },

  async setCaptureBlur(
    blur_mode: 'none' | 'reduced' | 'full'
  ): Promise<{ success: boolean; blur_mode: string; message: string }> {
    return apiRequest('/api/settings/capture-blur', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ blur_mode }),
    });
  },
};
