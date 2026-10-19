# FamShield Agent - Theo Dõi Vấn Đề Kỹ Thuật & Kế Hoạch Hoàn Thiện

Tài liệu này theo dõi các vấn đề kỹ thuật cốt lõi cần giải quyết để đưa FamShield Windows Agent từ prototype lên bản hoàn thiện phục vụ môi trường thực tế.

---

## I. Trạng Thái Các Vấn Đề Kỹ Thuật

### 1. Build Gate & CI (Đã hoàn thành 100% ✅)
- [x] **cargo test --workspace**: Đạt **48/48 unit tests pass** (29 test suites).
- [x] **cargo clippy --workspace --all-targets -- -D warnings**: Sạch sẽ **0 errors, 0 warnings**.
- [x] **Flaky test process_monitor.rs**: Đã thay thế bằng injected process snapshots, bảo đảm test độc lập và ổn định 100%.
- [x] **CI/CD Workflow**: Đã thiết lập `.github/workflows/ci.yml` tự động kiểm tra build, clippy và test trên Windows runner.

### 2. Bảo Mật & Threat Model (Đã giải quyết các lỗi then chốt ✅)
- [x] **Xác thực Master PIN & Lockout**: Đã triển khai bộ đếm số lần sai (`failed_attempts`), khóa theo cấp số nhân (Lockout từ 30s đến 15 phút) trong `famshield-local-admin`. Ngay cả khi nhập đúng PIN trong thời gian lockout cũng bị từ chối truy cập.
- [x] **Bảo vệ Secret cấp máy trạm**: Đã tích hợp Windows Data Protection API (DPAPI machine-scope) mã hóa toàn bộ hash PIN và khóa khôi phục trước khi lưu vào SQLite.
- [x] **Quản lý Lỗi Tập Trung & Băm Fingerprint Thông Minh**: Đã triển khai bảng `error_logs` với băm SHA-256 (`module` + `severity` + `normalized_message`) gom cụm lỗi lặp lại, tránh tràn DB và hỗ trợ đồng bộ server.
- [x] **Bộ Khung Telemetry Hợp Nhất**: Đã thiết lập bảng `outbox` SQLite theo mô hình Store-and-Forward và chuẩn hóa `TelemetryEnvelope` (`famshield-sync::telemetry`), tối ưu truyền tải qua Cloudflare Tunnel và NATS.
- [x] **Ủy quyền IPC Named Pipe**: Đã thiết lập định dạng framing (độ dài 4-byte LE, trần 256 KiB) và chuỗi SDDL ACL bảo vệ `D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)` chỉ cho phép tài khoản Authenticated Users, Administrators, SYSTEM truy cập.
- [x] **Xác thực Remote Command & Chống Replay**: Đã triển khai chữ ký mã hóa HMAC-SHA256 canonical payload kèm cửa sổ chống phát lại (replay protection <= 300 giây).

### 3. Subsystem Đang Triển Khai Thật (Thực Tế vs Mô Phỏng)
- [x] **wfp.rs (Đã hoàn thành ✅)**: Đã triển khai đăng ký filter thật bằng Win32 API `FwpmFilterAdd0` tại cả hai tầng `FWPM_LAYER_ALE_AUTH_CONNECT_V4` và `FWPM_LAYER_ALE_AUTH_CONNECT_V6`. Permit loopback `127.0.0.0/8:53` và upstream DNS (Weight 15), Block mọi outbound Port 53 khác (Weight 10). Sử dụng `FWPM_SESSION_FLAG_DYNAMIC` tự động thu hồi filter khi tắt/crash, chống nghẽn mạng.
- [x] **capture.rs (Đã hoàn thành ✅)**: Đã thay thế mock bằng Win32 GDI `BitBlt` chụp màn hình thật, downscale 480p, thuật toán 3-pass separable box blur (Gaussian) trong RAM bảo vệ quyền riêng tư, và nén JPEG < 150 KiB.
- [x] **nrpt.rs (Đã hoàn thành ✅)**: Đã ghi cấu hình trực tiếp vào Windows Registry `HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig`.
- [x] **etw.rs (ETW Kernel Process Monitor - Đã hoàn thành ✅)**: Đã triển khai `EtwTraceSession` lắng nghe provider `Microsoft-Windows-Kernel-Process` ({22fb2c4e-47c2-4eea-b0bf-478f0f9d874f}), thu nhận sự kiện tiến trình tức thì `0ms`, kết hợp cơ chế fallback ToolHelp32.
- [x] **main.rs (Windows Service SCM Dispatcher - Đã hoàn thành ✅)**: Đã triển khai ServiceMain và ServiceControlHandler (`windows-service` crate) hỗ trợ đầy đủ ServiceControl::Stop, Shutdown, và tự động chuyển sang chế độ console tương tác nếu chạy ngoài Service Control Manager.
- [x] **updater / staged.rs (Chữ ký số Manifest - Đã hoàn thành ✅)**: Đã triển khai xác minh chữ ký số ECDSA P-256 + SHA-256 trước khi hoán đổi nhị phân và cơ chế rollback tự động.
- [x] **enrollment.rs (Freemium & Cloud Enrollment - Đã hoàn thành ✅)**: Đã triển khai phân tách rõ ràng chế độ `OperatingMode::StandaloneFree` (100% miễn phí, offline cục bộ, bảo vệ quyền riêng tư) và `OperatingMode::OnlineManaged` (trả phí, quản lý tập trung từ xa qua mTLS & NATS).

---

## II. Kế Hoạch Ưu Tiên Các Bước Tiếp Theo

1. [x] **Bước 1**: Đưa CI/Test/Clippy về xanh 100%. (Đã xong)
2. [x] **Bước 2**: Bảo mật PIN: Lockout brute-force, exponential backoff, DPAPI machine-scope. (Đã xong)
3. [x] **Bước 3**: WFP filter Port 53 thật kèm rollback an toàn dynamic session. (Đã xong)
4. [x] **Bước 4**: Chụp màn hình Win32 GDI BitBlt thật + Gaussian blur trong RAM + nén JPEG. (Đã xong)
5. [x] **Bước 5**: Quản lý lỗi tập trung & Băm fingerprint SHA-256 thông minh. (Đã xong)
6. [x] **Bước 6**: Thiết lập Bộ khung Telemetry Hợp nhất (Unified Telemetry Envelope + SQLite Outbox). (Đã xong)
7. [x] **Bước 7**: Hoàn thiện Windows Service SCM Dispatcher và Named Pipe IPC có ACL. (Đã xong)
8. [x] **Bước 8**: Bổ sung lắng nghe sự kiện tiến trình qua ETW realtime 0ms. (Đã xong)
9. [x] **Bước 9**: Bổ sung chữ ký số và xác minh manifest cho Updater. (Đã xong)
10. [x] **Bước 10**: Phân tách kiến trúc Freemium: Chế độ Miễn phí Cục bộ (StandaloneFree) vs Trả phí Đám mây (OnlineManaged) & Xác thực chữ ký remote command. (Đã xong)

---

*Cập nhật lần cuối: 19/10/2026 bởi FamShield Architecture Engine*
