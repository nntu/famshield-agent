export interface Device {
  id: string;
  device_name: string;
  os_version: string;
  status: 'online' | 'degraded' | 'offline';
  policy_version: number;
  last_seen: string;
}

export interface ChildProfile {
  id: string;
  name: string;
  daily_quota_minutes: number;
  used_minutes_today: number;
  assigned_devices: string[];
}

export interface VisualEvidence {
  id: string;
  event_id: string;
  device_id: string;
  trigger: string;
  occurred_at: string;
  expires_in_hours: number;
}
