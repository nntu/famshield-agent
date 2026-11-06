# Changelog

Tất cả các thay đổi quan trọng của dự án **FamShield Windows Agent** được ghi lại trong tài liệu này.

Định dạng tài liệu dựa trên [Keep a Changelog](https://keepachangelog.com/vi/1.0.0/), và dự án tuân thủ [Semantic Versioning](https://semver.org/).

> 📚 **Lưu trữ kỹ thuật chuyên sâu:**  
> File `CHANGELOG.md` này đóng vai trò **Tóm Tắt Tổng Quan (Executive / User-Facing Summary)**.  
> Chi tiết kỹ thuật, phân tích kiến trúc, lược đồ migration và danh sách tệp sửa đổi của từng phiên bản được lưu trữ độc lập tại thư mục [`docs/releases/`](docs/releases/).

---

## [Unreleased]

---

## [1.0.6] - 2026-11-06

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.6.md`](docs/releases/v1.0.6.md)

### Fixed
- **Khắc Phục Lỗi Logic Giới Hạn Quota Thời Lượng Ứng Dụng**: Sửa lỗi lệch tên trường JSON giữa Web UI (`quota_minutes`) và bộ điều khiển `controller.rs` (`max_minutes_per_day`), đảm bảo ứng dụng dùng quá hạn mức luôn bị đóng cưỡng bức và khóa màn hình bảo vệ con trẻ.
- **Khắc Phục Lỗi Đảo Ngược Danh Sách Cho Phép Khi Nhập Excel**: Chuẩn hóa việc phân tích cột `Hành Động` (`action`), khắc phục lỗi tự động gán tên miền whitelist tiếng Việt sang danh sách chặn `BLOCK` và bổ sung nhãn phân loại trong `ExcelImportModal`.
- **Triệt Tiêu Nguy Cơ Crash Do Panic & Kẹt Cờ Monitor-Only**: Thay thế toàn bộ các lời gọi `parse_from_rfc3339().unwrap()` bằng cơ chế fallback an toàn (Zero Panic Rule 4); đồng bộ chuẩn xác cờ `monitor_mode` hai chiều khi Cloud cập nhật chính sách.
- **Chuẩn Hóa Số Liệu Động & Bỏ Phân Biệt Hoa Thường Tên Tiến Trình**: Nâng cấp truy vấn thời lượng ứng dụng với `COLLATE NOCASE` và chuẩn hóa đuôi `.exe`; hiển thị số liệu thời gian sử dụng thực tế và tổng số lượt DNS/bằng chứng toàn diện trên Dashboard.

### Added
- **Loại Bỏ Triệt Để Lazy-Loading & Đạt Tốc Độ Chuyển Tab Tức Thì 0ms**: Chuyển toàn bộ các tab và modal sang nạp tĩnh trực tiếp trong bundle chính, loại bỏ `<Suspense>` và màn hình chờ tải mô-đun; tối ưu nạp dữ liệu song song trong `ActivityTab`.
- **Tách Trang Tác Giả & Mã Nguồn Sang Phân Hệ Menu Riêng Biệt**: Bổ sung tab độc lập `✨ Tác giả` (`author`) lên thanh điều hướng chính `Navbar.tsx` với màn hình `AuthorTab.tsx` chi tiết về sứ mệnh, kiến trúc bảo mật và giấy phép MIT.
- **Tái Cấu Trúc Giao Diện Bảo Mật & Ghép Nối Với Hệ Thống Submenu Phân Trang**: Phân tách 11 card phân tán của `SecurityTab` thành 4 trang chức năng chuyên biệt qua `SecurityTabNav`: Tài Khoản & Bảo Mật, Đám Mây & Ghép Nối, Mạng & Hệ Thống, Dữ Liệu & Khôi Phục.
- **Quản Lý Danh Mục Web Phân Trang Chống Treo Giao Diện**: Bảng dữ liệu phân trang hiệu năng cao `CategoryDetailModal`, hỗ trợ nạp tệp danh sách chặn `.txt`, `.hosts`, Adblock và trích xuất danh mục.

---

## [1.0.5] - 2026-10-31

> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [`docs/releases/v1.0.5.md`](docs/releases/v1.0.5.md)

### Security
- **Khắc Phục Nhận Diện Nhầm Mã Độc & Tối Ưu Hóa Bộ Cài Đặt Inno Setup**: Loại bỏ cờ Stealth Mode và lệnh PowerShell ngầm; đăng ký chính quy vào Windows Settings, giao cấu hình DNS cho Native Rust Service; bổ sung metadata PE Header và thuật toán nén `lzma2/ultra64` triệt tiêu cảnh báo false positive từ Defender/Kaspersky.

### Added
- **Tái Cấu Trúc Toàn Diện Frontend & Phân Rã Component Chuẩn**: Tách các màn hình khổng lồ (`SecurityTab` giảm >90%, `RulesTab` giảm >62%, `SetupWizard` giảm ~80%) thành các component/modal chuyên biệt; rút ngắn thời gian HMR và biên dịch Vite xuống 250ms–300ms.
- **Tối Ưu Hóa Hiệu Năng Frontend & Nâng Cấp Bảng Dữ Liệu TanStack**: Tích hợp `@tanstack/react-table`, `@tanstack/react-virtual` cuộn ảo 60fps cho nhật ký DNS, kết hợp `React.lazy` và dynamic import giảm dung lượng tải ban đầu từ 880 kB xuống 83 kB (gzip 24 kB - giảm 90%).
- **Thu Thập Thông Tin Card Mạng & Địa Chỉ IP Đa Giao Diện**: Mở rộng `sysinfo` thu thập tên interface, địa chỉ MAC, danh sách IP và lưu lượng TX/RX, hiển thị danh sách card mạng kèm badge IP trên Web Dashboard.
- **Bộ Công Cụ Kiểm Thử Tối Ưu Tốc Độ Cao (`test.ps1`)**: Cung cấp runner kiểm thử phân tầng theo module thay đổi (`.\test.ps1 -Changed`) chỉ mất 1-3s, tích hợp `cargo-nextest` chạy song song đa luồng.

### Changed
- **Nâng Cấp & Quy Hoạch Tập Trung Workspace Dependencies**: Cập nhật các thư viện lõi (`tokio 1.43`, `async-nats 0.51`, `arc-swap 1.9`, `thiserror 2.0`, `sysinfo 0.40.1`, `rusqlite 0.40.0`, `axum 0.8`), đưa `windows`, `axum`, `tower`, `tray-icon` vào `[workspace.dependencies]` triệt tiêu version drift. Bắt buộc thay thế `cargo test` bằng `cargo nextest` chống treo trên Windows.
- **Tối Ưu Hóa Hồ Sơ Biên Dịch & Chỉ Mục SQLite (Migration V7)**: Kích hoạt `split-debuginfo = "unpacked"` cho dev, giảm >70% dung lượng PDB khi test; bổ sung migration `V7__optimize_indexes.sql` loại bỏ Full Table Scan trên các bảng nhật ký và outbox.

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
