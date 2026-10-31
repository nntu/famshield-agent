# Changelog

Tất cả các thay đổi quan trọng của dự án **FamShield Windows Agent** được ghi lại trong tài liệu này.

Định dạng tài liệu dựa trên [Keep a Changelog](https://keepachangelog.com/vi/1.0.0/), và dự án tuân thủ [Semantic Versioning](https://semver.org/).

> 📚 **Lưu trữ kỹ thuật chuyên sâu:**  
> File `CHANGELOG.md` này đóng vai trò **Tóm Tắt Tổng Quan (Executive / User-Facing Summary)**.  
> Chi tiết kỹ thuật, phân tích kiến trúc, lược đồ migration và danh sách tệp sửa đổi của từng phiên bản được lưu trữ độc lập tại thư mục [`docs/releases/`](docs/releases/).

---

## [Unreleased]

### Security
- **Khắc Phục Nhận Diện Nhầm Mã Độc & Tối Ưu Hóa Bộ Cài Đặt Inno Setup (Anti-Virus False Positive Remediation, `installer/FamShield.iss`, `Cargo.toml`)**:
  - **Loại bỏ cờ Stealth Mode (`CreateUninstallRegKey=yes`, `UpdateUninstallLogAppName=yes`)**: Đăng ký chính quy vào Windows Settings (`Apps & Features`) và Control Panel, triệt tiêu heuristic phát hiện "Trojan Dropper / Stealth Rootkit" của Microsoft Defender và Kaspersky. Bảo vệ chống gỡ trộm được đảm bảo qua Master PIN trong hàm `InitializeUninstall`.
  - **Loại bỏ lệnh PowerShell ngầm với `-ExecutionPolicy Bypass` và `sc.exe sdset`**: Dỡ bỏ các tiến trình con dễ bị EDR/AMSI gắn cờ là Trojan DNS Hijacking hoặc cố ý khóa quyền Service Control Manager. Giao toàn bộ việc cấu hình DNS và NRPT cho tiến trình Rust Service thực thi an toàn cấp native.
  - **Bổ sung siêu dữ liệu chuẩn Windows PE Header (`VersionInfo*`)**: Khai báo đầy đủ `VersionInfoVersion`, `VersionInfoCompany`, `VersionInfoDescription`, `VersionInfoCopyright`, `VersionInfoProductName` giúp nâng cao điểm uy tín (reputation score) của file EXE cài đặt.
  - **Nâng cấp thuật toán nén `Compression=lzma2/ultra64`**: Tăng tốc độ giải nén và giảm thiểu việc kích hoạt packer heuristic từ các engine AV.
  - **Cấu hình Profile Release Tối Ưu ([`Cargo.toml`](Cargo.toml))**: Kích hoạt `strip = true`, `lto = "thin"`, `panic = "abort"` và `codegen-units = 1` cho bản dựng release, xóa sạch debug symbols và đường dẫn nội bộ khỏi các file nhị phân (`famshield-service.exe`, `famshield-session.exe`).

### Added
- **Tái Cấu Trúc & Phân Rã Các File Frontend Khổng Lồ Thành Kiến Trúc Module Chuẩn (Frontend Component Modularization & Architecture Refactoring, `web`)**:
  - Phân rã `SecurityTab.tsx` (từ 5,091 dòng xuống 498 dòng - giảm >90%): Tách thành 11 card và modal độc lập trong [`web/src/components/security/`](web/src/components/security/).
  - Phân rã `RulesTab.tsx` (từ 2,269 dòng xuống <850 dòng - giảm >62%): Tách các subtab và modal vào [`web/src/components/rules/`](web/src/components/rules/).
  - Phân rã `SetupWizard.tsx` (từ 1,079 dòng xuống 225 dòng - giảm ~80%): Tách 5 bước hướng dẫn cài đặt vào [`web/src/components/wizard/`](web/src/components/wizard/).
  - Phân rã `ActivityTab.tsx` và `LogsTab.tsx`: Giảm 40–50% độ dài, tách component biểu đồ, bảng biểu và modal drilldown chuyên biệt.
  - Bảo toàn 100% tính năng, zero regression và rút ngắn thời gian biên dịch Vite xuống 250ms–300ms.
- **Tối Ưu Hóa Toàn Diện Frontend & Nâng Cấp Hệ Thống Bảng Dữ Liệu (`@tanstack/react-table`, `@tanstack/react-virtual`, `@tanstack/react-query`, Vite Code-Splitting, `web`)**:
  - Tích hợp bộ ba tối ưu hiệu năng TanStack (`react-table`, `react-virtual`, `react-query`): Virtual windowing 60fps cho nhật ký DNS lớn, quản lý cache server-state thông minh.
  - Tách bundle và tải lười (`React.lazy` + `Suspense`): Giảm dung lượng tải ban đầu từ 880 kB xuống 83 kB (gzip 24 kB - giảm 90%).
  - Tối ưu manual chunks trong Vite và xuất file Excel/CSV theo yêu cầu qua dynamic import `xlsx`.
- **Thu Thập Thông Tin Mạng & Địa Chỉ IP Đa Giao Diện Qua `sysinfo` (`famshield-windows`, `famshield-sync`, `web`)**:
  - Khai thác tính năng `network` của `sysinfo = "0.33"` thu thập tên interface, địa chỉ MAC, danh sách IP và lưu lượng TX/RX.
  - Trích xuất IP nội bộ thích ứng trong `PcInfoDto` và hiển thị danh sách card mạng kèm badge IP trên Web Dashboard.
- **Bộ Công Cụ Kiểm Thử Tối Ưu Tốc Độ Cao (`test.ps1`)**:
  - Hỗ trợ `.\test.ps1 -Changed` (chạy test 1-3s cho crate có code thay đổi) và `.\test.ps1 -Target <crate>`.
  - Tích hợp chạy song song đa luồng với `cargo-nextest` trên máy phát triển và CI workflow (`.github/workflows/ci.yml`).

### Changed
- **Rà Soát, Nâng Cấp & Đồng Bộ Hóa Dependencies Workspace ([`Cargo.toml`](Cargo.toml))**:
  - **Nâng cấp các crate async & networking lên phiên bản mới nhất**:
    - `tokio`: Nâng từ `1.38` lên `1.43` (tối ưu hóa runtime và asynchronous I/O).
    - `async-nats`: Nâng từ `0.50` lên `0.51` (tương thích giao thức NATS mới nhất, tăng cường khả năng tự phục hồi kết nối mTLS).
    - `arc-swap`: Nâng từ `1.7` lên `1.9` (tối ưu hóa concurrency đọc ghi không khóa cho cache phân giải DNS).
    - `thiserror`: Nâng từ `1.0` lên `2.0` (tuân thủ tiêu chuẩn macro derive lỗi hiện đại của Rust 2021/2024).
    - `sysinfo`: Khai báo chính xác phiên bản `0.40.1` với cờ `features = ["system", "network"]`.
  - **Quy hoạch & Tập trung hóa toàn bộ dependencies vào `[workspace.dependencies]`**:
    - Đưa các thư viện `windows = "0.58"`, `axum = "0.7"`, `tower = "0.5"`, `http-body-util = "0.1"`, `tray-icon = "0.26"`, `muda = "0.21"` vào cấu hình workspace gốc.
    - Cập nhật toàn bộ các crate con (`local_admin`, `service`, `session`, `windows`) sang cú pháp `{ workspace = true }`, triệt tiêu hoàn toàn nguy cơ lệch phiên bản (version drift) và xung đột kiểu dữ liệu giữa các module.
- **Tối Ưu Hóa Hồ Sơ Biên Dịch & Liên Kết Test ([`Cargo.toml`](Cargo.toml))**:
  - Cấu hình `[profile.dev] split-debuginfo = "unpacked"` tăng tốc độ biên dịch gia tăng trên Windows.
  - Cấu hình `[profile.test] debug = 1` giảm >70% dung lượng PDB và tối ưu hóa thư viện bên ngoài với `opt-level = 2`.
- **Cập Nhật Tiêu Chuẩn Kiểm Thử Phân Tầng Dành Cho AI Agents ([`AGENTS.md`](AGENTS.md))**:
  - Quy định sử dụng `test.ps1` phân tầng để tiết kiệm thời gian phản hồi và bảo tồn token ngữ cảnh.
- **Tối Ưu Hóa Hệ Thống Chỉ Mục SQLite Cục Bộ Client (Database Index Optimization & Migration V7, `famshield-storage`)**:
  - Bổ sung migration `V7__optimize_indexes.sql` đánh chỉ mục chuyên sâu cho `usage_segments`, `visual_evidence`, `query_logs` và `outbox`, loại bỏ hoàn toàn Full Table Scan.

---

## [1.0.4] - 2026-10-31

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.4.md`](docs/releases/v1.0.4.md)

### Added
- **Giám Sát Chéo 2 Chiều Giữa Session & Service & Khởi Động Lại Từ Khay Hệ Thống**: Module `ServiceWatcher` tự động phát hiện sự cố dừng dịch vụ đột ngột trong 6 giây; cho phép phụ huynh khởi động lại Windows Service an toàn ngay từ Tray Menu qua xác thực Master PIN / 2FA TOTP.
- **Ghi Vết Kiểm Toán Sự Cố & Đẩy Cảnh Báo An Ninh Lên Cloud**: Tính toán chính xác thời gian gián đoạn (downtime), phát hiện tắt máy đột ngột (`service_abrupt_shutdown`) và phát sinh cảnh báo thời gian thực khi có hành vi dò mã trên máy con.
- **Thu Thập Thông Số Máy Tính & Phần Cứng Toàn Diện**: Tích hợp thư viện chuẩn công nghiệp `sysinfo = "0.33"` thu thập tên máy tính, CPU, RAM, thời gian hoạt động (uptime) và hiển thị trực quan lên Web Dashboard (`DashboardTab.tsx`).
- **Phân Biệt Nguồn Gốc Chính Sách Chặn (Cloud Server vs. Local Máy Trạm)**: Bổ sung huy hiệu phân biệt quy tắc đẩy từ máy chủ đám mây (`☁️ MÁY CHỦ`) và quy tắc do phụ huynh tạo tại máy (`💻 TẠI CHỖ`), hỗ trợ nút kéo/đẩy chính sách tức thì qua REST API.
- **Định Danh Máy Trạm Tránh Trùng Lặp**: Tự động nhận diện Windows `MachineGuid` và gắn hậu tố độc nhất vào tên máy tính (`DESKTOP-FAMSHIELD (688F)`), triệt tiêu nguy cơ trùng tên khi triển khai nhiều thiết bị chung image.

### Changed
- **Chuẩn Hóa Nhận Diện Tên Máy Tính**: Sử dụng `sysinfo::System::host_name()` kết hợp cơ chế fallback đa tầng an toàn thay vì đọc trực tiếp biến môi trường `COMPUTERNAME`.
- **Chuẩn Hóa Quy Trình Xác Thực Mã Tổ Chức (Tenant ID Reconciliation)**: Triệt tiêu mã tenant giả lập, đồng bộ trực tiếp qua `POST /v1/device/enroll` và tuân thủ quy trình lưu trữ nguyên tử 3 bước với commit marker.

### Fixed
- **Tương Thích Clippy Rust 1.99+**: Khắc phục cảnh báo nghiêm ngặt `-D clippy::chunks-exact-to-as-chunks` trong module giải mã hex của `famshield-core`.

---

## [1.0.3] - 2026-10-24

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.3.md`](docs/releases/v1.0.3.md)

### Added
- **Tham Số Hóa Toàn Diện & Quản Lý Cấu Hình Hệ Thống Động**: Chuyển đổi các hằng số vận hành (chu kỳ sync, timeout DNS, giới hạn khóa PIN, thời hạn lưu log) thành cấu hình động lưu trong SQLite `agent_meta`, tùy chỉnh trực tiếp tại Web Admin không cần biên dịch lại.
- **URL Check Version Tự Động Thích Ứng Theo Chế Độ**: Tự động chuyển đổi URL kiểm tra bản mới theo chế độ độc lập (`Standalone`) hoặc theo địa chỉ máy chủ quản trị tập trung (`OnlineManaged`).
- **Trung Tâm Quản Lý Lỗi & Xem Nhật Ký Trực Quan**: Giao diện Terminal dark-theme đọc/tải log dịch vụ thời gian thực; bảng quản lý lỗi với cơ chế khử trùng lặp (dedup) và nút đẩy lô lỗi lên Cloud Server qua NATS broker.
- **Chế Độ Giám Sát Yên Lặng Toàn Diện (Silent Monitoring / Stealth Mode)**: Vẫn ghi nhận 100% nhật ký và bằng chứng vi phạm nhưng triệt tiêu hoàn toàn thông báo Toast/Balloon trên màn hình con trẻ.

### Changed
- **Chuẩn Hóa Đơn Nhất Tên Miền (Single Domain `famshield.ttcantho.io.vn`)**: Đồng bộ toàn bộ codebase về một domain duy nhất định tuyến theo Path trên cổng chuẩn HTTPS (443).
- **Chặn DNS Tàng Hình (Stealth NXDOMAIN Blocking)**: Khi chặn web, phản hồi mã lỗi `NXDOMAIN` chuẩn RFC 1035 để trình duyệt hiển thị lỗi không tìm thấy máy chủ tự nhiên, không hiện pop-up chặn lộ liễu.
- **Tái Cấu Trúc Module Hex & Hợp Nhất Hằng Số**: Gom các hàm xử lý hex zero-allocation vào `famshield-core::hex` và cập nhật đồ thị tri thức codebase-memory.

### Fixed
- **Sửa Lỗi Kết Nối NATS Broker Qua Cổng 9222**: Tự động chuẩn hóa URL NATS sang `wss://<host>/nats` khi chạy qua Reverse Proxy công khai.
- **Sửa Lỗi So Sánh Phiên Bản Cũ (v0.1.1)**: Chuẩn hóa thuật toán so sánh SemVer thay cho so sánh chuỗi thô, thanh lọc metadata phiên bản cũ tồn đọng.
- **Triệt Tiêu Nguy Cơ Spin Loop / 100% CPU**: Sửa triệt để các vòng lặp rảnh trên kênh Tokio mpsc, Named Pipe IPC, UDP socket `recv_from` và HTTP batch sync.

---

## [1.0.2] - 2026-10-22

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.2.md`](docs/releases/v1.0.2.md)

### Added
- **Quản Trị CSDL Với Refinery & Pre-Migration Snapshot**: Tích hợp `refinery = "0.10"` quản lý migration nguyên tử nhúng sẵn; tự động snapshot CSDL qua SQLite Online Backup API trước khi migrate, tự động rollback trong $\le 20\text{ms}$ nếu gặp sự cố.
- **Kênh Điều Khiển Thời Gian Thực Hai Chiều NATS**: Tích hợp `async-nats = "0.50"` qua WebSocket WSS trên cổng 443, thực thi lệnh khẩn cấp từ máy chủ (khóa máy, đổi PIN, nạp blacklist) dưới 50ms.
- **Nâng Cấp Giao Thức Telemetry v1.1**: Bổ sung gói tin thời lượng sử dụng màn hình ứng dụng (`app_usage`), chỉ số độ trễ DNS và tối ưu hóa HTTP Keep-Alive connection pool.
- **Đặt Lại Báo Cáo & Số Liệu Hoạt Động Về 0**: API và Modal xác thực Master PIN / 2FA cho phép làm sạch nhật ký DNS, ứng dụng hoặc bằng chứng hình ảnh an toàn.
- **Báo Cáo Gom Nhóm Thời Lượng Ứng Dụng**: Tự động gom nhóm thời gian sử dụng theo tiến trình, biểu đồ thanh ngang xếp hạng Top ứng dụng chiếm nhiều thời gian nhất.
- **Bộ Cân Bằng Tải DNS Tự Học NUMA**: Kiến trúc Lock-Free chống false sharing, tự học độ trễ thực tế qua giải thuật EWMA và tự động chọn máy chủ DNS tối ưu nhất.

### Security
- **Củng Cố Quy Trình Gỡ Cài Đặt**: Chặn đứng việc trẻ tự chạy `unins000.exe` bằng cách ép xác thực Master PIN / 2FA qua hộp thoại Win32 native; phục hồi 100% cấu hình mạng DNS adapter và NRPT Registry.
- **Tùy Chọn Mức Độ Làm Mờ Bằng Chứng**: Cho phép chuyển đổi linh hoạt giữa 3 cấp độ: Tắt mờ (rõ nét), Giảm mờ nhẹ (4px) và Mờ bảo vệ Gaussian (15px).
- **Bộ Lọc Thời Gian & Dòng Thời Gian Truy Cập DNS 24h**: Biểu đồ cột phân bố 24 khung giờ trong ngày và dòng thời gian truy cập chi tiết của từng tên miền.

### Changed
- **Đồng Bộ Hóa Toàn Diện Tài Liệu Kỹ Thuật**: Cập nhật song ngữ `README.md` & `README.en.md` và toàn bộ các tài liệu kiến trúc trong `docs/`.

---

## [1.0.1] - 2026-10-21

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.1.md`](docs/releases/v1.0.1.md)

### Added
- **Hạ Tầng Tự Động Cập Nhật Thông Minh Zero-Cost**: Triển khai Cloudflare Worker định tuyến `/version.json` và phân phối bộ cài qua GitHub Releases; cơ chế Watchdog A/B Rollback 60s tự động phục hồi bản cũ nếu bản mới bị lỗi.
- **Hiện Đại Hóa Frontend Với Biome & pnpm v12.x**: Tích hợp bộ linter/formatter Biome siêu tốc và chuẩn hóa gói thư viện với `pnpm`.
- **Tự Động Chụp Bằng Chứng Thị Giác (Smart Visual Verification)**: Chụp màn hình ngầm khi phát hiện gõ từ khóa nhạy cảm hoặc truy cập web cấm, mã hóa nén JPEG và lưu trữ tự hủy sau 48 giờ.
- **Tích Hợp Tailwind CSS v4**: Nâng cấp hệ thống thiết kế Web Admin với Tailwind CSS v4 hiện đại.

### Changed
- **Hiện Đại Hóa Giao Diện Sang Tone Sáng (Light Theme Modernization)**: Bảng màu slate/zinc thanh lịch theo chuẩn `taste-skill`, loại bỏ lỗi co giật màn hình và cắt xén layout.
- **Chụp Bằng Chứng Rõ Nét Mặc Định**: Mặc định lưu ảnh rõ nét và bổ sung nút bấm làm mờ riêng tư theo yêu cầu trên Web UI.

### Fixed
- **Tối Ưu Hiệu Năng & Khắc Phục Lỗi React**: Nạp dữ liệu song song với `Promise.allSettled`, sửa cảnh báo hooks, chuẩn hóa xử lý lỗi Type-Safe.

### Security
- Chuẩn hóa chính sách không tự ý phân phối root certificate tự ký trên môi trường phân phối chính thức.

---

## [1.0.0] - 2026-10-20

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.0.md`](docs/releases/v1.0.0.md)

### Added
- **Cưỡng Chế Hạn Mức Sử Dụng Ứng Dụng**: Tự động quét và hạ tiến trình game/ứng dụng quá giờ ở cấp độ Win32 API kernel Session 0, tự động khóa màn hình máy trạm khi chạm ngưỡng.
- **Chặn Cổng VPN & Giao Thức DoT Vượt Rào Bằng WFP**: Đăng ký bộ lọc chặn cấp kernel đối với các cổng 853 (DoT), 1194 (OpenVPN) và 51820 (WireGuard) với cờ `FWPM_SESSION_FLAG_DYNAMIC` chống BSOD.
- **Nén Tổng Hợp Nhật Ký DNS (Rollup Aggregation)**: Tự động gom nhóm số lượt truy vấn theo tên miền vào bảng `domain_query_aggregates`, hỗ trợ tùy chỉnh thời hạn lưu trữ (1 đến 90 ngày) và tác vụ dọn dẹp nền định kỳ.
- **Xác Thực 2 Lớp Cho Phụ Huynh (2FA / TOTP RFC 6238)**: Hỗ trợ Google/Microsoft Authenticator 100% offline, render mã QR trực tiếp trong RAM, kèm 8 mã dự phòng khẩn cấp một lần.
- **Bộ Nhớ Đệm DNS Cục Bộ Tinh Gọn (≤ 2 MB) & Đa Upstream Thác Đổ**: Cache 2.000 bản ghi DNS với thuật toán trừ lùi TTL động, tự động failover sang DNS gốc máy trạm khi upstream chính gặp sự cố.
- **Triển Khai 4 Trụ Cột Bảo Vệ**: Cưỡng chế SafeSearch & YouTube Restricted qua DNS VIP rewrite; Canh gác tự phục hồi Session Watchdog; Khóa máy theo hạn mức; Tự động cập nhật phối hợp Watchdog.
- **Bộ Chuẩn Hóa Tiếng Việt Đa Bộ Gõ (Telex & VNI)**: Nhận diện ngôn từ nhạy cảm đa biến thể qua thuật toán Aho-Corasick $O(N)$ với cơ chế bảo vệ ô mật khẩu và chống báo động giả.
- **Giao Diện Quản Trị Cục Bộ Đầy Đủ (Local Web Admin React 19)**: Máy chủ Axum nhúng tĩnh phục vụ SPA React 19 + TypeScript, hỗ trợ Dashboard KPI, quản lý luật, xem bằng chứng và thiết lập bảo mật.

### Security
- **Bảo Vệ Khóa Mật Mã Bằng DPAPI**: Toàn bộ Master PIN hash (Argon2id), TOTP secret và signing key máy trạm đều được mã hóa bằng DPAPI machine-scope.
- **Fail-Closed Enrollment**: Chỉ chuyển sang `OnlineManaged` khi nhận phản hồi xác thực server hợp lệ, từ chối mọi remote command chưa ký số.

### Fixed
- **Xử Lý Lỗi Bộ Cài Đặt Inno Setup**: Tự động hạ tiến trình cũ và dừng/xóa dịch vụ cũ trước khi sao chép ghi đè file.
- **Sửa Lỗi Tắc Nghẽn Phân Giải DNS**: Tách mỗi truy vấn DNS UDP upstream thành tác vụ Tokio độc lập với timeout 2500ms và socket tạm thời.
- **Sửa Lỗi Cổng Mạng Mặc Định**: Tránh dải cổng Hyper-V/WSL2 chiếm dụng bằng cơ chế dò cổng tự động (`38123`, `18123`, `28123`).

### Changed
- **Phát hành chính thức phiên bản sản xuất FamShield Agent 1.0.0**.

---

## [0.1.0-rc.1] - 2026-10-19

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v0.1.0-rc.1.md`](docs/releases/v0.1.0-rc.1.md)

### Added
- **Windows Service SCM Dispatcher & Named Pipe Server**: Quản lý vòng đời dịch vụ Windows Session 0, kết nối liên tiến trình qua Named Pipe với SDDL ACL nghiêm ngặt.
- **Giám Sát Tiến Trình Kernel Cấp Thời Gian Thực Qua ETW**: Bắt sự kiện tạo và tắt tiến trình trong $0\text{ms}$ qua `Microsoft-Windows-Kernel-Process`.
- **Động Cơ Cập Nhật Đa Chế Độ & Chữ Ký ECDSA P-256**: Hỗ trợ kiểm tra bản cập nhật đám mây và cập nhật ngoại tuyến qua USB.
- **Bộ Nhận Diện Thương Hiệu & Logo PE**: Nhúng icon khiên bảo mật đa độ phân giải vào PE Header của các file thực thi.
- **Kiến Trúc Freemium (StandaloneFree vs. OnlineManaged)**: Hỗ trợ hoạt động hoàn toàn offline bảo vệ riêng tư hoặc ghép nối đám mây qua mã 6 số.
- **Hàng Đợi Ngoại Tuyến SQLite Outbox**: Lưu trữ bền vững dữ liệu telemetry và gửi lên server theo cơ chế Store-and-Forward kèm Exponential Backoff.
- **Bộ Lọc WFP Port 53**: Chặn đứng lưu lượng DNS vượt rào ngoài luồng bằng Windows Filtering Platform phiên động.
- **Chụp Màn Hình Làm Mờ Trong RAM**: Chụp khung hình Desktop DC, làm mờ Gaussian Blur trực tiếp trong bộ nhớ bảo vệ riêng tư và nén JPEG < 150 KiB.
- **Khóa Master PIN Argon2id & DPAPI**: Chống dò mật khẩu theo cấp số nhân và lưu trữ bí mật an toàn.

### Changed
- Hợp nhất dependencies về `[workspace.dependencies]` và rà soát toàn diện 17 thư viện bên thứ ba.
- Hoàn thiện 60 unit tests pass 100%.

### Fixed
- Cấu hình Inno Setup chuẩn Native 64-bit, triệt tiêu lỗi cài nhầm vào `Program Files (x86)` và vô hiệu hóa WOW64 redirection.
