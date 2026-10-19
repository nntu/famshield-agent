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
- **Xác thực Chữ ký số Bản Cập nhật & Động cơ Cập nhật Đa Chế độ (`famshield-updater`)**:
  - Triển khai `OnlineUpdateClient`: Tự động kiểm tra phiên bản mới từ máy chủ đám mây (`manifest_url`), so sánh semver (`UpToDate`, `UpdateAvailable`, `Incompatible`), tải và xác thực chữ ký ECDSA P-256 + SHA-256.
  - Triển khai `OfflineUpdateEngine`: Hỗ trợ cập nhật ngoại tuyến 100% không cần Internet (qua USB flash drive hoặc hot-folder), xác thực chữ ký manifest và SHA-256 nhị phân trước khi hoán đổi.
  - Cơ chế hoán đổi nguyên tử (atomic staging swap) kèm Watchdog timer 30s tự động khôi phục bản cũ nếu phiên bản mới khởi động thất bại.
- **Bộ Nhận Diện Logo & Tích Hợp Icon Windows PE (`assets/`, `service`, `session`, `installer`)**:
  - Thiết kế logo nhận diện thương hiệu FamShield: Biểu tượng khiên bảo mật 3D công nghệ cao với dải màu cyan/blue phát sáng và biểu tượng gia đình bảo vệ tối giản.
  - Tạo bộ icon đa độ phân giải chuẩn Windows (`.ico`: 256x256, 128x128, 64x64, 48x48, 32x32, 16x16).
  - Tích hợp nhúng trực tiếp tài nguyên icon vào cấu trúc PE của các file nhị phân `famshield-service.exe` và `famshield-session.exe` thông qua `windres`.
  - Cấu hình icon trình cài đặt (`SetupIconFile`), shortcut Desktop và Start Menu trong kịch bản Inno Setup `installer/FamShield.iss`.
- **Tự động Ký số Authenticode & Tích hợp Bộ Cài Đặt (`build.ps1`, `installer/FamShield.iss`)**:
  - Hỗ trợ tự động tạo và tin cậy chứng chỉ Authenticode Code Signing cục bộ.
  - Ký số SHA-256 kèm RFC 3161 timestamping cho toàn bộ nhị phân `famshield-service.exe`, `famshield-session.exe`, và `FamShieldSetup.exe`.
  - Tự động tích hợp nạp chứng chỉ Root CA vào Windows Store trong Inno Setup.
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
- Hợp nhất và chuẩn hóa toàn bộ dependencies về `[workspace.dependencies]` tại file gốc `Cargo.toml`, loại bỏ hoàn toàn hiện tượng phân mảnh phiên bản (version drift) và tối ưu hóa thời gian biên dịch cho mục tiêu phát triển lâu dài.
- Rà soát toàn diện 17 thư viện bên thứ ba: 100% các gói đều thuộc phiên bản mới nhất, ổn định và được hỗ trợ tích cực bởi cộng đồng Rust (Tokio 1.38+, Windows 0.58, Rusqlite 0.31, RustCrypto 0.10/0.13, Thiserror 1.0, Tracing 0.1, Serde 1.0).
- Nâng cấp bộ test suite lên **60 unit tests** trên 29 test suites, đạt trạng thái pass 100%.
- Thay thế các timer quét tiến trình flaky trong kiểm thử bằng injected snapshot.
- Định dạng toàn bộ mã nguồn theo chuẩn `cargo fmt`.

### Fixed
- **Trình Cài Đặt Inno Setup Chuẩn Native 64-Bit (`installer/FamShield.iss`)**:
  - Bổ sung chỉ thị `ArchitecturesAllowed=x64compatible` và `ArchitecturesInstallIn64BitMode=x64compatible`.
  - Khắc phục triệt để lỗi cài nhầm vào thư mục `C:\Program Files (x86)\FamShield` do Inno Setup mặc định ở chế độ 32-bit khi thiếu chỉ thị kiến trúc.
  - Vô hiệu hóa cơ chế chuyển hướng WOW64 File & Registry Redirection: Đảm bảo `{autopf}` trỏ chính xác về `C:\Program Files\FamShield`, `{sys}` trỏ về native `System32`, và ghi trực tiếp vào Registry 64-bit (đặc biệt quan trọng với chính sách NRPT DNS Policy).
  - Thêm thuộc tính `RunOnceId` cho các bước gỡ dịch vụ trong `[UninstallRun]` nhằm loại bỏ cảnh báo biên dịch.

### Removed
- Dọn dẹp và hợp nhất nội dung từ hai tài liệu tạm `chaulam.md` và `chualam.md` vào `CHANGELOG.md`.
