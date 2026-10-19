import { TranslationKeys } from './vi';

export const en: Record<TranslationKeys, string> = {
  // Brand & Navigation
  brand: 'FamShield Cloud',
  brand_tagline: 'Child Safety & Endpoint Protection Platform',
  nav_overview: 'Overview',
  nav_children: 'Child Profiles & Quotas',
  nav_devices: 'Protected Devices',
  nav_evidence: 'Smart Evidence (48h)',

  // Language switcher
  language: 'Language',
  lang_vi: 'Tiếng Việt (Vietnamese)',
  lang_en: 'English (Default: Tiếng Việt)',

  // Overview Tab
  overview_title: 'Centralized Guardian Management Overview',
  overview_subtitle: 'Real-time telemetry, DNS resolution filtering, and endpoint protection.',
  stat_online_devices: 'Online Devices',
  stat_blocked_dns: 'DNS Domains Blocked Today',
  stat_pending_evidence: 'Verification Evidence Pending',

  // Children Tab
  children_title: 'Child Profiles & Screen Time Quotas',
  children_status_protected: 'Protected',
  children_used_today: 'Used today: {used} mins',
  children_quota: 'Daily quota: {quota} mins',
  children_add_profile: 'Add Child Profile',

  // Devices Tab
  devices_title: 'Protected Endpoint Devices List',
  devices_th_name: 'Device Name',
  devices_th_os: 'OS Version',
  devices_th_policy: 'Policy Version',
  devices_th_status: 'Status',
  devices_synced: 'v{version} (Synced)',
  devices_status_online: '● Online',
  devices_status_offline: '○ Offline',
  devices_status_degraded: '⚠ Degraded',
  devices_enroll_button: 'Enroll New Device',

  // Evidence Tab
  evidence_title: 'Smart Visual Verification Library',
  evidence_desc: 'Images are automatically blurred on-device to preserve privacy. Strictly compliant with COPPA / Decree 13/2023/ND-CP: Auto-deleted after 48 hours.',
  evidence_thumb_placeholder: '480p Blurred Thumbnail (Privacy Protected)',
  evidence_timestamp: 'Captured: {time}',
  evidence_expires: 'Expires in: {hours} hours',

  // Quick Action & Footer
  footer_status: 'FamShield active endpoint protection is operating normally.',

  // User Auth & Telegram Activation
  auth_register_btn: 'Guardian Sign Up',
  auth_register_title: 'Create Guardian Account',
  auth_email_label: 'Guardian Email',
  auth_password_label: 'Password',
  auth_telegram_label: 'Telegram Username (Optional)',
  auth_submit_register: 'Sign Up & Activate via Telegram',
  auth_activation_pending: 'Awaiting Telegram Activation',
  auth_activation_desc: 'FamShield activates accounts via Telegram Bot for instant activation without Gmail daily quotas, and real-time alerts.',
  auth_open_telegram_btn: '👉 Activate Now via Telegram (1-Click)',
  auth_or_otp_code: 'Or send this OTP code to Bot: {code}',
  auth_activated_success: '🎉 Account activated successfully! Welcome to FamShield.',
  auth_close: 'Close',
};

