import {
  ActivitySummaryResponse,
  BackupPayload,
  EvidenceItem,
  InitializeSetupRequest,
  InitializeSetupResponse,
  KeywordConfigResponse,
  KeywordRule,
  LocalRuleRecord,
  ProtectionModeResponse,
  QueryLogRecord,
  RestoreResponse,
  ServerConfig,
  SetupStatusResponse,
  StatusResponse,
  SyncResponse,
  TestConnectionResponse,
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

  async login(pin: string): Promise<{ success: boolean; token?: string; error?: string }> {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pin }),
    });

    const data = await res.json();
    if (res.ok && data.success && data.token) {
      setSessionToken(data.token);
      return data;
    }
    return { success: false, error: data.error || 'Invalid PIN' };
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

  async createRulesBulk(rules: {
    rule_id: string;
    rule_type: 'domain' | 'application';
    payload_json: string;
    enabled: boolean;
  }[]): Promise<{ success: boolean; imported: number }> {
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

  async getLogs(limit: number = 100): Promise<QueryLogRecord[]> {
    return apiRequest<QueryLogRecord[]>(`/api/logs/dns?limit=${limit}`);
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

  async enrollCloud(code: string): Promise<{ success: boolean; operating_mode: string; device_id: string }> {
    return apiRequest<{ success: boolean; operating_mode: string; device_id: string }>('/api/cloud/enroll', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enrollment_code: code }),
    });
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

  async getKeywordConfig(): Promise<KeywordConfigResponse> {
    return apiRequest<KeywordConfigResponse>('/api/keywords');
  },

  async addCustomKeyword(pattern: string, category?: string, severity?: string): Promise<KeywordRule> {
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

  async toggleKeyboardMonitoring(enabled: boolean): Promise<{ success: boolean; enabled: boolean }> {
    return apiRequest<{ success: boolean; enabled: boolean }>('/api/keywords/monitoring', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled }),
    });
  },
};
