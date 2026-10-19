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

  // User Auth & Telegram Activation
  auth_register_btn: 'Đăng ký Phụ huynh',
  auth_register_title: 'Đăng Ký Tài Khoản FamShield',
  auth_email_label: 'Email phụ huynh',
  auth_password_label: 'Mật khẩu',
  auth_telegram_label: 'Telegram Username (Tùy chọn)',
  auth_submit_register: 'Đăng ký & Kích hoạt qua Telegram',
  auth_activation_pending: 'Chờ Kích Hoạt Qua Telegram',
  auth_activation_desc: 'FamShield kích hoạt tài khoản qua Telegram Bot để đảm bảo nhận thông báo vi phạm và yêu cầu xin thêm giờ tức thì, không bị giới hạn gửi thư của Gmail.',
  auth_open_telegram_btn: '👉 Kích Hoạt Ngay Qua Telegram (1 chạm)',
  auth_or_otp_code: 'Hoặc nhập mã OTP sau: {code}',
  auth_activated_success: '🎉 Tài khoản đã kích hoạt thành công! Chào mừng bạn.',
  auth_close: 'Đóng',

  // Download Agent
  nav_download_agent: 'Tải Agent Windows',
  download_agent_title: 'Tải Bộ Cài Đặt FamShield Agent',
  download_agent_desc: 'Cài đặt trực tiếp trên máy tính con để bắt đầu bảo vệ (Windows 10, 11 64-bit).',
  download_exe_btn: 'Tải FamShieldSetup.exe',
  download_instructions: 'Hướng dẫn cài đặt nhanh:',
  download_step1: '1. Nhấp tải bộ cài FamShieldSetup.exe về máy tính của con.',
  download_step2: '2. Chạy cài đặt và mở Bảng điều khiển cục bộ để nhập mã ghép đôi 6 số.',
  download_step3: '3. Máy tính của con sẽ được kích hoạt chế độ bảo vệ mạng WFP và DNS an toàn ngay lập tức.',
};

export type TranslationKeys = keyof typeof vi;


