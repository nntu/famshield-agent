export const vi = {
  // Brand & Navigation
  brand: 'FamShield Cloud',
  brand_tagline: 'Hệ thống Quản trị An toàn Trẻ em',
  nav_overview: 'Tổng quan',
  nav_children: 'Hồ sơ trẻ & Giới hạn',
  nav_devices: 'Thiết bị bảo vệ',
  nav_evidence: 'Bằng chứng thông minh (48h)',

  // Language switcher
  language: 'Ngôn ngữ',
  lang_vi: 'Tiếng Việt (Mặc định)',
  lang_en: 'English (Tiếng Anh)',

  // Overview Tab
  overview_title: 'Tổng quan Hệ thống Quản trị Tập trung',
  overview_subtitle: 'Giám sát thời gian thực, lưu lượng phân giải tên miền và bảo vệ máy trạm.',
  stat_online_devices: 'Thiết bị trực tuyến',
  stat_blocked_dns: 'Tên miền DNS đã chặn hôm nay',
  stat_pending_evidence: 'Bằng chứng xác minh cần duyệt',

  // Children Tab
  children_title: 'Quản lý Hồ sơ & Giới hạn Thời gian',
  children_status_protected: 'Đang bảo vệ',
  children_used_today: 'Thời gian dùng hôm nay: {used} phút',
  children_quota: 'Hạn ngạch: {quota} phút',
  children_add_profile: 'Thêm hồ sơ trẻ',

  // Devices Tab
  devices_title: 'Danh sách Thiết bị Endpoint Bảo vệ',
  devices_th_name: 'Tên thiết bị',
  devices_th_os: 'Hệ điều hành',
  devices_th_policy: 'Phiên bản Policy',
  devices_th_status: 'Trạng thái',
  devices_synced: 'v{version} (Đã đồng bộ)',
  devices_status_online: '● Trực tuyến',
  devices_status_offline: '○ Ngoại tuyến',
  devices_status_degraded: '⚠ Cảnh báo',
  devices_enroll_button: 'Ghép đôi thiết bị mới',

  // Evidence Tab
  evidence_title: 'Thư viện Bằng chứng Xác minh Thông minh',
  evidence_desc: 'Hình ảnh được tự động làm mờ tại thiết bị để bảo vệ riêng tư. Tuân thủ tuyệt đối quy định COPPA / Nghị định 13/2023/NĐ-CP: Tự hủy sau 48 giờ.',
  evidence_thumb_placeholder: 'Ảnh 480p Đã Che Mờ (Bảo vệ Riêng tư)',
  evidence_timestamp: 'Thời điểm: {time}',
  evidence_expires: 'Tự hủy sau: {hours} giờ',

  // Quick Action & Footer
  footer_status: 'Hệ thống bảo vệ chủ động FamShield đang hoạt động bình thường.',
};

export type TranslationKeys = keyof typeof vi;
