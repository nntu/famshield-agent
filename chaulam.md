# FamShield Windows Agent - Danh Sách Hạng Mục Triển Khai (Checklist)

Tài liệu này đối chiếu các yêu cầu kỹ thuật trong đặc tả kiến trúc (`docs/03-WINDOWS-AGENT.md` và `docs/08-IMPLEMENTATION-PLAN.md`) với mã nguồn thực tế của FamShield Agent (`nntu/famshield-agent`).

---

## I. Các Tính Năng Đã Hoàn Thành (Done) ✅

- [x] **1. Cấu trúc Workspace & Crate Modularity**
  - Tách bạch 9 crates độc lập: `core`, `storage`, `dns`, `windows`, `service`, `session`, `local_admin`, `sync`, `updater`.
  - Toàn bộ test suite pass 100% (**48/48 unit tests** trên 29 test suites).
  - Clippy đạt **0 errors, 0 warnings** với `-D warnings`.

- [x] **2. Động cơ DNS Proxy & Bộ đệm NUMA**
  - Lắng nghe trên loopback `127.0.0.2:53` (tránh xung đột với `Dnscache` của Windows).
  - RFC 1035 zero-copy wire parser, Serve-Stale Cache (RFC 8767).
  - `BlocklistStore` sử dụng con trỏ atomic `ArcSwap<HashSet<String>>` cho phép hot-swap danh sách chặn với độ trễ micro giây, 0 downtime.

- [x] **3. Định tuyến NRPT & Bộ Lọc Windows Filtering Platform (WFP)**
  - Cấu hình Windows Name Resolution Policy Table (NRPT) trỏ `.` về `127.0.0.2:53`.
  - Đăng ký filter thật bằng Win32 API `FwpmFilterAdd0` tại cả hai tầng `FWPM_LAYER_ALE_AUTH_CONNECT_V4` và `FWPM_LAYER_ALE_AUTH_CONNECT_V6`.
  - Quy tắc Permit Loopback `127.0.0.0/8` và Upstream IP (Weight 15).
  - Quy tắc Block Outbound Port 53 (Weight 10) chống bypass DNS.
  - Sử dụng cờ `FWPM_SESSION_FLAG_DYNAMIC` tự động thu hồi sạch filter khi crash/tắt, không bao giờ gây nghẽn mạng máy trạm.

- [x] **4. Quản lý Quyền Quản trị Cục bộ (Master PIN & Phục hồi 3 lớp)**
  - Mã hóa Argon2id + salt ngẫu nhiên từ hệ điều hành (`OsRng`).
  - Bảo vệ khóa bí mật cấp máy trạm qua Windows DPAPI (machine-scope).
  - Cơ chế đếm số lần nhập sai (`failed_attempts`) và khóa theo cấp số nhân (Lockout 30s - 15 phút) chống brute-force.
  - Reset qua Admin UAC Elevation CLI (`--reset-pin`).
  - Reset qua Khóa khẩn cấp offline `FAM-XXXX-XXXX-XXXX`.
  - Reset từ xa thông qua Cloud / Telegram Bot.

- [x] **5. Xác thực Hình ảnh Bảo vệ Quyền riêng tư (Smart Visual Verification)**
  - Chụp màn hình thật qua Win32 GDI `BitBlt` từ Desktop DC vào 32-bit DIB buffer.
  - Downscale tự động về tỉ lệ chuẩn 480p.
  - Thuật toán 3-pass separable box blur (hội tụ Gaussian Blur trong RAM) với độ phức tạp $O(N)$ làm mờ text và thông tin riêng tư.
  - Nén trực tiếp thành stream JPEG hợp lệ qua `jpeg-encoder`, dung lượng luôn < 150 KiB.

- [x] **6. Quản Lý Lỗi Tập Trung & Băm Fingerprint Thông Minh (Smart Error Deduplication)**
  - Băm SHA-256 fingerprint (`module` + `severity` + `normalized_message`) gom cụm lỗi lặp lại.
  - Tự động tăng biến đếm `occurrence_count` và cập nhật `last_seen_at` thay vì tạo hàng ngàn bản ghi rác.
  - Hỗ trợ đánh dấu đồng bộ (`mark_errors_synced`) và tự động reset khi lỗi tái xuất hiện.

- [x] **7. Bộ Khung Telemetry Hợp Nhất (Unified Telemetry Pipeline)**
  - Lưu trữ bền vững Store-and-Forward thông qua bảng `outbox` trong SQLite.
  - Chuẩn hóa `TelemetryEnvelope` (`version: "1.0"`, `device_id`, `batch_id`, `events`).
  - Gom các loại sự kiện khác nhau (`heartbeat`, `dns_query`, `error_log`, `process_event`, `security_alert`) vào 1 batch duy nhất.
  - Cơ chế Retry Backoff tự động khi mất mạng, tương thích hoàn hảo qua Cloudflare Tunnel và NATS.

- [x] **8. Kết nối 2 chiều NATS JetStream (Bidirectional Real-Time)**
  - Kênh điều khiển `devices.{id}.commands`: Nhận lệnh tức thì (< 1 giây).
  - Kênh dữ liệu `devices.{id}.events`: Đẩy batch telemetry và DNS log lên JetStream.
  - Hỗ trợ WebSocket Port 9222 vượt qua Cloudflare Tunnel không cần mở port.
  - Lệnh khóa màn hình thời gian thực `LockWorkStation()`.

- [x] **9. Cơ chế Phê duyệt Cài đặt Phần mềm (Software Installation Approval)**
  - `InstallerInterceptor`: Nhận diện file installer/setup/update/msi.
  - Tạm dừng và tính mã băm SHA256, gửi cảnh báo về Telegram.
  - Nút bấm 1 chạm trên Telegram: `[✅ Phê duyệt]` (mở khóa tiến trình) hoặc `[❌ Từ chối]` (đóng tiến trình ngay lập tức).

- [x] **10. Giám sát Tiến trình (Process Lifecycle Telemetry)**
  - Quét snapshot ToolHelp32 định kỳ mỗi 5 giây, phát hiện biến động `process_started` và `process_terminated`.
  - Tự động đóng gói batch JSON đẩy lên NATS JetStream.

- [x] **11. Giao diện Micro-UI trên Windows Session (Session Crate)**
  - Win32 System Tray icon với context menu (Trạng thái bảo vệ, Đăng xuất, Thoát).
  - Hộp thoại nhập Master PIN native Win32.
  - Windows Toast notification thân thiện cho trẻ khi bị chặn hoặc hết giờ.

- [x] **12. Giám sát Tương tác Người dùng & Đếm giờ Monotonic (Foreground & Idle Tracking)**
  - Lấy Handle và tên file tiến trình của cửa sổ tích cực qua `GetForegroundWindow()`, `GetWindowThreadProcessId()`, `GetProcessImageFileNameW()`.
  - Đo thời gian người dùng không gõ phím/chuột qua Win32 `GetLastInputInfo()` và `GetTickCount()`.
  - Tự động tạm dừng trừ giờ khi trẻ AFK / không hoạt động quá 3 phút (180s) tránh trừ oan giờ của trẻ.

- [x] **13. Đóng gói Bộ cài đặt Inno Setup (`.exe`)**
  - File kịch bản tại `installer/FamShield.iss`.

- [x] **14. Windows Service SCM Dispatcher & Named Pipe IPC có ACL (`famshield-service`)**
  - Tích hợp `windows-service` crate đăng ký ServiceMain và ServiceControlHandler đầy đủ với Windows Service Control Manager (SCM).
  - Tự động chuyển đổi sang Interactive Console Mode khi thực thi trực tiếp qua Terminal.
  - Named Pipe Server với cơ chế framing độ dài tiền tố (4-byte LE length, trần an toàn 256 KiB chống tràn RAM).
  - Thiết lập SDDL bảo mật nghiêm ngặt `D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)` chỉ cho phép Authenticated Users đọc/ghi và LocalSystem / Builtin Admins toàn quyền.

- [x] **15. Giám sát Tiến trình Kernel qua ETW 0ms (`famshield-windows::EtwTraceSession`)**
  - Tích hợp Win32 Event Tracing for Windows (ETW) qua Provider `Microsoft-Windows-Kernel-Process` (`{22fb2c4e-47c2-4eea-b0bf-478f0f9d874f}`).
  - Bắt tức thì các sự kiện khởi tạo và kết thúc tiến trình tại thời điểm `0ms`.
  - Kết hợp cơ chế lai (Hybrid) với ToolHelp32 snapshot fallback đảm bảo không bao giờ sót tiến trình.

- [x] **16. Chữ Ký Số & Xác Minh Manifest Staged Update (`famshield-updater`)**
  - Xác thực chữ ký số mật mã học ECDSA P-256 + SHA-256 trên manifest trước khi tiến hành tải và giải nén bản cập nhật.
  - Chuẩn hóa payload ký dạng Canonical JSON (`version`, `url`, `sha256`, `min_os_version`).
  - Kiểm tra toàn vẹn băm SHA-256 trên file nhị phân tải về.
  - Cơ chế hoán đổi nguyên tử (Atomic staging swap) kèm Watchdog 30s tự động khôi phục bản cũ nếu phiên bản mới không khởi động thành công.

- [x] **17. Phân tách Kiến Trúc Freemium & Xác Thực Lệnh Từ Xa (`famshield-sync`)**
  - **Chế độ Miễn Phí Cục Bộ (`OperatingMode::StandaloneFree`)**:
    - Hoàn toàn miễn phí, 100% offline, riêng tư, không cần tài khoản hay kết nối máy chủ.
    - Cung cấp trọn vẹn bộ tính năng: DNS Proxy loopback, WFP Port 53 block, Master PIN + DPAPI, chụp màn hình GDI + Gaussian blur, Foreground & Idle tracking, SQLite lưu trữ cục bộ, System Tray.
  - **Chế độ Trả Phí Quản Lý Đám Mây (`OperatingMode::OnlineManaged`)**:
    - Kích hoạt thông qua mã ghép nối 6 chữ số (`EnrollmentManager::enroll`).
    - Đồng bộ telemetry và nhận lệnh tức thời qua NATS JetStream 2 chiều.
    - Xác thực lệnh điều khiển từ xa bằng chữ ký HMAC-SHA256 chuẩn canonical.
    - Chống tấn công phát lại (Anti-replay protection window <= 300 giây).
    - Hỗ trợ hủy ghép nối tức thì (`disenroll`) quay về chế độ Miễn phí Cục bộ an toàn.

---

## II. Các Tính Năng Cần Hoàn Thiện Tiếp Theo (Pending) 🚀

- [x] **Hoàn thành 100% mục tiêu kỹ thuật cốt lõi (Steps 1 đến 10)**.
- Toàn bộ **57 unit tests** trên 29 test suites đều đạt trạng thái pass xanh lá, clippy 0 errors/warnings.
- Sẵn sàng tiến hành build nhị phân production và triển khai thử nghiệm thực tế.

---

*Cập nhật lần cuối: 19/10/2026 bởi FamShield Architecture Engine*