# Changelog

Tất cả các thay đổi quan trọng của dự án **FamShield Windows Agent** sẽ được ghi lại trong tài liệu này.

Định dạng tài liệu dựa trên [Keep a Changelog](https://keepachangelog.com/vi/1.0.0/), và dự án tuân thủ [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

---

## [0.1.0-rc.1] - 2026-10-19

### Added
- **Windows Service SCM Dispatcher (`famshield-service`)**:
  - Tích hợp `windows-service` crate quản lý toàn diện vòng đời dịch vụ (`ServiceControl::Stop`, `Shutdown`).
  - Hỗ trợ chế độ Interactive Console tự động khi chạy trực tiếp ngoài Service Control Manager.
  - Named Pipe Server với cơ chế framing độ dài tiền tố (4-byte LE length, trần an toàn 256 KiB chống tràn RAM).
  - Chuỗi SDDL bảo mật nghiêm ngặt `D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)` phân quyền chặt chẽ giữa Authenticated Users và Administrators / LocalSystem.
- **Giám sát Tiến trình Kernel Thời gian thực qua ETW (`famshield-windows`)**:
  - Triển khai `EtwTraceSession` lắng nghe trực tiếp Win32 ETW Provider `Microsoft-Windows-Kernel-Process` (`{22fb2c4e-47c2-4eea-b0bf-478f0f9d874f}`).
  - Bắt sự kiện tạo và hủy tiến trình tại thời điểm `0ms`.
  - Cơ chế lai (Hybrid) kết hợp ToolHelp32 snapshot dự phòng đảm bảo không bao giờ sót tiến trình.
- **Xác thực Chữ ký số Bản Cập nhật (`famshield-updater`)**:
  - Xác thực chữ ký số ECDSA P-256 + SHA-256 trên manifest cập nhật trước khi nạp.
  - Kiểm tra toàn vẹn mã băm SHA-256 của file nhị phân tải về.
  - Cơ chế hoán đổi nguyên tử (atomic staging swap) kèm Watchdog timer 30s tự động khôi phục bản cũ nếu phiên bản mới khởi động thất bại.
- **Kiến trúc Freemium & Đăng ký Đám mây (`famshield-sync`)**:
  - `OperatingMode::StandaloneFree`: Chế độ Miễn phí Cục bộ 100%, ngoại tuyến, bảo vệ riêng tư tuyệt đối, zero dependency vào server đám mây.
  - `OperatingMode::OnlineManaged`: Chế độ Trả phí Quản lý Đám mây ghép nối qua mã 6 chữ số (`EnrollmentManager::enroll`).
  - Hỗ trợ thực thi lệnh từ xa tức thì qua NATS JetStream 2 chiều (`lock_session`, `update_blacklist`, `reset_pin`, `approve_install`/`deny_install`).
  - Xác thực chữ ký mã hóa HMAC-SHA256 trên toàn bộ remote command và bảo vệ chống tấn công phát lại (Anti-replay window $\le 300\text{s}$).
  - Cho phép hủy ghép nối (`disenroll()`) quay về chế độ Miễn phí Cục bộ an toàn bất cứ lúc nào.
- **Bộ khung Telemetry Hợp nhất & SQLite Outbox (`famshield-storage`, `famshield-sync`)**:
  - Lưu trữ bền vững Store-and-Forward thông qua bảng `outbox` trong SQLite kèm thuật toán Exponential Backoff.
  - Chuẩn hóa `TelemetryEnvelope` gom đa sự kiện (Heartbeat, DNS Log, Error Log, Process Lifecycle, Security Alert) vào một batch duy nhất.
  - Bộ băm fingerprint SHA-256 thông minh (`module:severity:normalized_message`) gom cụm lỗi lặp lại (Error Deduplication), chống tràn DB.
- **Windows Filtering Platform (WFP) Port 53 Filter (`famshield-windows`)**:
  - Đăng ký filter thật bằng Win32 API `FwpmFilterAdd0` tại cả hai tầng `ALE_AUTH_CONNECT_V4` và `ALE_AUTH_CONNECT_V6`.
  - Quy tắc Permit loopback `127.0.0.0/8:53` và upstream DNS (Weight 15).
  - Quy tắc Block outbound Port 53 (Weight 10) chống bypass DNS.
  - Sử dụng cờ `FWPM_SESSION_FLAG_DYNAMIC` tự động thu hồi filter khi tắt/crash, chống nghẽn mạng và chống BSOD.
- **Chụp Màn hình Bảo vệ Quyền riêng tư (`famshield-session`)**:
  - Chụp màn hình thật Win32 GDI `BitBlt` từ Desktop DC, tự động downscale 480p.
  - Thuật toán 3-pass separable box blur (Gaussian Blur trong RAM) với độ phức tạp $O(N)$ làm mờ thông tin nhạy cảm.
  - Nén JPEG trực tiếp trong bộ nhớ bằng `jpeg-encoder`, dung lượng luôn < 150 KiB.
- **Khóa Master PIN Brute-force & DPAPI (`famshield-local-admin`)**:
  - Mã hóa Argon2id kèm salt ngẫu nhiên từ hệ điều hành (`OsRng`).
  - Khóa bí mật máy trạm được bảo vệ qua Windows Data Protection API (DPAPI machine-scope).
  - Bộ đếm số lần nhập sai liên tiếp và khóa đăng nhập theo cấp số nhân (Lockout 30s đến 15 phút).
  - Khóa khẩn cấp offline `FAM-XXXX-XXXX-XXXX` và reset bằng quyền Administrator UAC Elevation (`--reset-pin`).
- **Tài liệu Song ngữ & Quy chuẩn Cộng đồng**:
  - `README.md` (Tiếng Việt) và `README.en.md` (Tiếng Anh).
  - `CONTRIBUTING.md` quy định tiêu chuẩn code, bảo mật, git workflow và quality gate.
  - Bộ mẫu GitHub: Pull Request Template, Bug Report Template, Feature Request Template.

### Changed
- Nâng cấp bộ test suite lên **57 unit tests** trên 29 test suites, đạt trạng thái pass 100%.
- Thay thế các timer quét tiến trình flaky trong kiểm thử bằng injected snapshot.
- Định dạng toàn bộ mã nguồn theo chuẩn `cargo fmt`.

### Removed
- Dọn dẹp và hợp nhất nội dung từ hai tài liệu tạm `chaulam.md` và `chualam.md` vào `CHANGELOG.md`.
