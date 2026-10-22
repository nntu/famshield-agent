export interface StatusResponse {
  protection_active: boolean;
  monitor_mode: boolean;
  session_helper_running: boolean;
  session_helper_status: string;
  dns_proxy_addr: string;
  wfp_status: string;
  operating_mode: 'StandaloneFree' | 'OnlineManaged';
  device_id: string;
  active_foreground_app: string;
  daily_quota_minutes: number;
  used_today_minutes: number;
  blocked_queries_today: number;
  total_queries_today: number;
  rules_count: number;
  evidence_count: number;
  app_version?: string;
}

export interface LocalRuleRecord {
  rule_id: string;
  rule_type: 'domain' | 'application';
  payload: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface QueryLogRecord {
  domain: string;
  qtype: string;
  action: 'ALLOW' | 'BLOCK' | 'MONITOR' | 'ERROR';
  reason: string;
  rule_id?: string | null;
  occurred_at: string;
}

export interface DomainQuerySummary {
  domain: string;
  total_queries: number;
  allowed_count: number;
  blocked_count: number;
  monitored_count: number;
  error_count: number;
  last_seen_at: string;
}

export interface EvidenceItem {
  evidence_id: string;
  event_id: string;
  occurred_at: string;
  trigger: string;
  size_bytes: number;
  expires_at: string;
  thumbnail_url: string;
}

export type TabType = 'dashboard' | 'activity' | 'rules' | 'logs' | 'evidence' | 'security';
export type Language = 'vi' | 'en';

export interface SetupStatusResponse {
  is_configured: boolean;
  operating_mode: string;
  device_id: string;
  app_version?: string;
}

export interface InitializeSetupRequest {
  pin: string;
  profile?: string;
  blocked_categories?: string[];
  operating_mode?: string;
  enrollment_code?: string;
  server_url?: string;
}

export interface InitializeSetupResponse {
  success: boolean;
  token: string;
  recovery_key: string;
  rules_created: number;
}

export interface ServerConfig {
  operating_mode: 'StandaloneFree' | 'OnlineManaged';
  server_url: string;
  tenant_id: string;
  device_id: string;
  device_name: string;
  enrollment_code?: string;
  sync_interval_secs: number;
  tls_verify: boolean;
  connection_status: 'Connected' | 'Disconnected' | 'Syncing' | 'Error';
  last_sync_time?: string | null;
  policy_version: number;
}

export interface UpdateServerConfigRequest {
  operating_mode?: 'StandaloneFree' | 'OnlineManaged';
  server_url?: string;
  tenant_id?: string;
  device_name?: string;
  enrollment_code?: string;
  sync_interval_secs?: number;
  tls_verify?: boolean;
}

export interface TestConnectionResponse {
  success: boolean;
  latency_ms: number;
  message: string;
}

export interface SyncResponse {
  success: boolean;
  synced_at: string;
  policies_received: number;
  message?: string;
}

export interface BackupSettings {
  operating_mode: string;
  server_url: string;
  tenant_id: string;
  device_name: string;
  sync_interval_secs: number;
  tls_verify: boolean;
  monitor_mode: boolean;
}

export interface BackupPayload {
  format_version: string;
  created_at: string;
  app_version: string;
  device_id: string;
  settings: BackupSettings;
  rules: LocalRuleRecord[];
  checksum: string;
}

export interface RestoreResponse {
  success: boolean;
  rules_restored: number;
  message: string;
}

export interface ProtectionModeResponse {
  success: boolean;
  monitor_mode: boolean;
}

export interface AppActivityItem {
  id: string;
  app_name: string;
  app_title: string;
  category: string;
  start_time: string;
  end_time: string;
  duration_minutes: number;
  duration_formatted: string;
  status: 'Active' | 'Normal' | 'QuotaWarning' | 'Exceeded';
}

export interface AppUsageSummary {
  app_name: string;
  app_title: string;
  category: string;
  total_duration_minutes: number;
  total_duration_formatted: string;
  percentage: number;
  session_count: number;
  first_seen_at: string;
  last_seen_at: string;
  is_active_now: boolean;
  status: 'Active' | 'QuotaWarning' | 'Normal';
}

export interface WebActivityItem {
  id: number;
  domain: string;
  category: string;
  action: 'ALLOW' | 'BLOCK' | 'MONITOR';
  reason: string;
  occurred_at: string;
  source_app: string;
}

export interface UnifiedTimelineItem {
  id: string;
  item_type: 'app' | 'web' | 'keyword';
  title: string;
  subtitle: string;
  category: string;
  action: 'ALLOW' | 'BLOCK' | 'MONITOR' | 'ACTIVE' | 'VIOLATION';
  timestamp: string;
  duration_minutes: number;
}

export interface KeywordRule {
  id: string;
  pattern: string;
  category: string;
  category_name: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  enabled: boolean;
  is_custom: boolean;
}

export interface KeywordViolation {
  id: string;
  timestamp: string;
  app_name: string;
  keyword_matched: string;
  category: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  snippet_masked: string;
}

export interface KeywordConfigResponse {
  keyboard_monitoring_enabled: boolean;
  total_rules_count: number;
  violations_today_count: number;
  rules: KeywordRule[];
  recent_violations: KeywordViolation[];
}

export interface ActivitySummaryResponse {
  date: string;
  total_screen_time_minutes: number;
  total_screen_time_formatted: string;
  active_apps_count: number;
  total_web_visits_count: number;
  blocked_events_count: number;
  monitored_events_count: number;
  keyword_violations_count: number;
  app_activities: AppActivityItem[];
  grouped_apps?: AppUsageSummary[];
  web_activities: WebActivityItem[];
  keyword_violations: KeywordViolation[];
  unified_timeline: UnifiedTimelineItem[];
}

export interface UpstreamDiagnostic {
  target: string;
  addr: string;
  latency_ms: number;
  total_queries: number;
  success_count: number;
  failure_count: number;
  consecutive_fails: number;
  is_optimal: boolean;
}

export interface DnsConfigResponse {
  detected_system_dns: string[];
  current_upstream: string;
  optimal_upstream?: string | null;
  upstream_diagnostics?: UpstreamDiagnostic[];
  cache_entries: number;
  cache_hits: number;
  cache_misses: number;
}

export interface HourlyFrequency {
  hour: number;
  total_count: number;
  allowed_count: number;
  blocked_count: number;
}

export interface CaptureBlurConfig {
  blur_mode: 'none' | 'reduced' | 'full';
}

export interface SetDnsConfigResponse {
  success: boolean;
  current_upstream: string;
  message: string;
}

export interface LoginResponse {
  success: boolean;
  token?: string;
  expires_at?: string;
  requires_totp?: boolean;
  temp_auth_token?: string;
  message?: string;
  error?: string;
}

export interface TotpStatusResponse {
  enabled: boolean;
  created_at: string | null;
  last_verified_at: string | null;
  unused_backup_codes: number;
}

export interface TotpSetupResponse {
  success: boolean;
  secret: string;
  url: string;
  qr_base64: string;
  backup_codes: string[];
}

export interface DnsRetentionConfig {
  retention_days: number;
  auto_prune: boolean;
  raw_query_logs_count: number;
  aggregated_domains_count: number;
  last_pruned_at: string | null;
}

export interface ResetReportStats {
  dns_cleared: number;
  apps_cleared: number;
  evidence_cleared: number;
  keywords_cleared: number;
}

export interface ResetReportsPayload {
  master_pin: string;
  otp_code?: string;
  scope?: string[];
}

export interface ResetReportsResponse {
  success: boolean;
  message: string;
  stats?: ResetReportStats;
  requires_otp?: boolean;
  invalid_otp?: boolean;
  locked?: boolean;
}
