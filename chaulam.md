# FamShield Windows Agent - Danh Sách Hạng Mục Triển Khai (Checklist)

Tài liệu này đối chiếu các yêu cầu kỹ thuật trong đặc tả kiến trúc (`docs/03-WINDOWS-AGENT.md` và `docs/08-IMPLEMENTATION-PLAN.md`) với mã nguồn thực tế của FamShield Agent (`nntu/famshield-agent`).

---

## I. Các Tính Năng Đã Hoàn Thành (Done) ✅

- [x] **1. Cấu trúc Workspace & Crate Modularity**
  - Tách bạch 9 crates độc lập: `core`, `storage`, `dns`, `windows`, `service`, `session`, `local_admin`, `sync`, `updater`.
  - Toàn bộ test suite pass 100% (35/35 unit tests).
- [x] **2. Động cơ DNS Proxy & Bộ đệm NUMA**
  - Lắng nghe trên loopback `127.0.0.2:53` (tránh xung đột với `Dnscache` của Windows).
  - RFC 1035 zero-copy wire parser, Serve-Stale Cache (RFC 8767).
  - `BlocklistStore` sử dụng con trỏ atomic `ArcSwap<HashSet<String>>` cho phép hot-swap danh sách chặn với độ trễ micro giây, 0 downtime.
- [x] **3. Định tuyến NRPT & Bảo vệ User-Mode WFP**
  - Cấu hình Windows Name Resolution Policy Table (NRPT) trỏ `.` về `127.0.0.2:53`.
  - Tích hợp WFP `fwpmu.dll` với cờ `FWPM_SESSION_FLAG_DYNAMIC` tự động giải phóng khi dịch vụ tắt/crash, không gây nghẽn mạng máy trạm.
- [x] **4. Quản lý Quyền Quản trị Cục bộ (Master PIN & Phục hồi 3 lớp)**
  - Mã hóa Argon2id + salt ngẫu nhiên từ hệ điều hành.
  - Reset qua Admin UAC Elevation CLI (`--reset-pin`).
  - Reset qua Khóa khẩn cấp offline `FAM-XXXX-XXXX-XXXX`.
  - Reset từ xa thông qua Cloud / Telegram Bot.
- [x] **5. Kết nối 2 chiều NATS JetStream (Bidirectional Real-Time)**
  - Kênh điều khiển `devices.{id}.commands`: Nhận lệnh tức thì (< 1 giây).
  - Kênh dữ liệu `devices.{id}.events`: Đẩy batch telemetry và DNS log lên JetStream.
  - Hỗ trợ WebSocket Port 9222 vượt qua Cloudflare Tunnel không cần mở port.
  - Lệnh khóa màn hình thời gian thực `LockWorkStation()`.
- [x] **6. Cơ chế Phê duyệt Cài đặt Phần mềm (Software Installation Approval)**
  - `InstallerInterceptor`: Nhận diện file installer/setup/update/msi.
  - Tạm dừng và tính mã băm SHA256, gửi cảnh báo về Telegram.
  - Nút bấm 1 chạm trên Telegram: `[✅ Phê duyệt]` (mở khóa tiến trình) hoặc `[❌ Từ chối]` (đóng tiến trình ngay lập tức).
- [x] **7. Giám sát Tiến trình (Process Lifecycle Telemetry)**
  - Quét snapshot ToolHelp32 định kỳ mỗi 5 giây, phát hiện biến động `process_started` và `process_terminated`.
  - Tự động đóng gói batch JSON đẩy lên NATS JetStream.
- [x] **8. Giao diện Micro-UI trên Windows Session (Session Crate)**
  - Win32 System Tray icon với context menu (Trạng thái bảo vệ, Đăng xuất, Thoát).
  - Hộp thoại nhập Master PIN native Win32.
  - Windows Toast notification thân thiện cho trẻ khi bị chặn hoặc hết giờ.
- [x] **9. Đóng gói Bộ cài đặt Inno Setup (`.exe`)**
- [x] **10. Giám sát Tương tác Người dùng & Đếm giờ Monotonic (Foreground & Idle Tracking)**
  - Lấy Handle và tên file tiến trình của cửa sổ tích cực qua `GetForegroundWindow()`, `GetWindowThreadProcessId()`, `GetProcessImageFileNameW()`.
  - Đo thời gian người dùng không gõ phím/chuột qua Win32 `GetLastInputInfo()` và `GetTickCount()`.
  - Tự động tạm dừng trừ giờ khi trẻ AFK / không hoạt động quá 3 phút (180s) tránh trừ oan giờ của trẻ.

---

## II. Các Tính Năng Chưa Làm / Cần Hoàn Thiện Tiếp Theo (Pending) ⚠️

### 1. Xác thực Hình ảnh Bảo vệ Quyền riêng tư (Smart Visual Verification)
- [ ] Tích hợp GDI `BitBlt` / DXGI Desktop Duplication API chụp màn hình máy trạm trong `famshield-session`.
- [ ] Chỉ chụp khi có sự kiện kích hoạt (Trigger-only: trẻ truy cập web cấm, cố bật app bị chặn, hoặc gửi yêu cầu xin thêm giờ). Tuyệt đối không quay lén định kỳ.
- [ ] Áp dụng thuật toán làm mờ Gaussian Blur trực tiếp trong bộ nhớ RAM lên các vùng text/nhập liệu để bảo vệ mật khẩu, tin nhắn cá nhân.
- [ ] Nén ảnh JPEG nhỏ gọn (< 150KB), lưu trữ cục bộ với chính sách tự động xóa vĩnh viễn sau 48 giờ (48-Hour Retention Purge).

### 2. Nâng cấp Giám sát Tiến trình qua ETW (Kernel Process Tracing 0ms)
- [ ] Bổ sung module lắng nghe Event Tracing for Windows (ETW) qua Provider `Microsoft-Windows-Kernel-Process`.
- [ ] Bắt ngay lập tức sự kiện tạo tiến trình tại thời điểm `0ms` (thay thế hoặc bổ trợ cho chu kỳ thăm dò snapshot 5 giây).

---

*Cập nhật lần cuối: 19/10/2026 bởi FamShield Architecture Engine*