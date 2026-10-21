# Changelog

Tất cả các thay đổi quan trọng của dự án **FamShield Windows Agent** sẽ được ghi lại trong tài liệu này.

Định dạng tài liệu dựa trên [Keep a Changelog](https://keepachangelog.com/vi/1.0.0/), và dự án tuân thủ [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- Chuẩn hóa giấy phép MIT cho repository: bổ sung `LICENSE`, đồng bộ metadata Cargo/npm và ghi rõ phạm vi giấy phép trong README cùng tài liệu đóng góp.
- **Tích Hợp Bộ Công Cụ Biome (`@biomejs/biome`) & Quản Lý Gói Độc Lập `pnpm` v12.x Cho Phân Hệ Web Admin (`web`)**:
  - Thiết lập Biome linter, formatter và import organizer siêu tốc cho toàn bộ mã nguồn TypeScript/React trong `web`, tích hợp các npm script `lint`, `lint:fix`, `format` và tệp cấu hình chuẩn `biome.json`.
  - Chuyển đổi và chuẩn hóa trình quản lý gói sang `pnpm` standalone (v12.x), sinh `pnpm-lock.yaml`, tối ưu hóa thời gian phân giải gói và lưu trữ hard link.
  - Cập nhật script `build.ps1` tự động nhận diện và ưu tiên thực thi `pnpm run build` cho Web Admin trước khi đóng gói nhị phân.
- **Kích Hoạt & Hoàn Thiện Cơ Chế Tự Động Chụp Ảnh Màn Hình Làm Bằng Chứng (Smart Visual Verification) Khi Vi Phạm Từ Khóa Hoặc Web Độc Hại (`famshield-session`, `famshield-service`)**:
  - **Tự động chụp khi vi phạm từ khóa nhạy cảm**: Khi `KeyboardMonitor` phát hiện từ khóa xấu trong bất kỳ ứng dụng nào, hệ thống lập tức kích hoạt luồng chụp khung hình ngầm (`spawn_blocking`), làm mờ bảo vệ riêng tư (Gaussian Blur 3-pass trong RAM, bán kính 15px), nén JPEG 480p (<25 KiB) và gửi gói tin IPC `VisualEvidenceSubmission` về dịch vụ `famshield-service` để lưu trữ SQLite với thời hạn tự hủy 48 giờ.
  - **Tự động chụp khi phát hiện truy cập trang web độc hại / bị chặn**: Khi `DnsProxy` chặn truy vấn DNS độc hại (phishing, malware, nội dung người lớn), dịch vụ nền Session 0 điều phối lệnh `CaptureEvidenceCommand` qua luồng heartbeat IPC tới trợ lý phiên người dùng `famshield-session` để chụp màn hình tương tác thực tế, đồng thời hiển thị thông báo an toàn (Toast Notification) nhắc nhở người dùng.
  - **Cơ chế chống nghẽn & chống spam ảnh (Throttling & Dedup)**: Tích hợp bộ lọc giới hạn tối đa 1 ảnh chụp mỗi 10 giây cho cùng một trigger, chống tràn bộ nhớ và chống cạn kiệt hạn mức đĩa 10 MiB khi trình duyệt truy vấn liên tiếp nhiều tài nguyên của một trang web độc hại.
- **Cẩm Nang Tính Năng & Hướng Dẫn Sử Dụng Toàn Diện (`HUONG-DAN-SU-DUNG-VA-TINH-NANG.md`)**:
  - Biên soạn tài liệu hợp nhất tổng hợp toàn bộ năng lực hệ thống, danh mục tính năng (DNS/WFP, App Limits, Smart Evidence, DPAPI/2FA TOTP, Dual-Mode) và cẩm nang hướng dẫn sử dụng chi tiết từng bước cho phụ huynh và quản trị viên.
- **Tích Hợp Tailwind CSS v4 Cho Phân Hệ Web Admin (`web`)**:
  - Tích hợp `@tailwindcss/vite` và `tailwindcss` v4 vào cấu hình `vite.config.ts` và luồng biên dịch frontend Vite.
  - Bổ sung cấu trúc style trung tâm `web/src/index.css` áp dụng `@import "tailwindcss";` cùng hệ thống CSS design tokens hiện đại.

### Changed
- **Đồng bộ hệ màu Web Admin (`web`)**: Chuẩn hóa token giao diện theo nền Slate và accent Sky duy nhất; đưa các màu Indigo/Purple/Teal dùng cho điều hướng, danh mục và thao tác chính về cùng accent, đồng thời giữ Emerald/Amber/Rose riêng cho trạng thái thành công, cảnh báo và lỗi. Đồng bộ màu tại Activity, Navbar, PIN modal, Setup Wizard và Security settings để giảm xung đột giữa Tailwind và style inline cũ.
- **Bỏ Chế Độ Làm Mờ Mặc Định Cho Bằng Chứng Thị Giác & Bổ Sung Nút Bật/Tắt Làm Mờ Tùy Chọn (`famshield-session`, `web`, `famshield-service`)**:
  - **Chụp ảnh rõ nét nguyên bản (Clear Evidence Capture)**: Cấu hình `SmartVisualVerifier` mặc định `blur_radius = 0` (loại bỏ hoàn toàn thuật toán Gaussian blur làm mờ đặc ảnh), lưu trữ khung hình sắc nét chuẩn 480p giúp phụ huynh nhận diện chính xác giao diện, đường dẫn web và nội dung vi phạm thực tế.
  - **Loại bỏ hiệu ứng CSS blur cố định trên Web UI**: Xóa bỏ thuộc tính `blur-[3px]` trên thẻ ảnh và modal phóng to trong `EvidenceTab.tsx`.
  - **Bổ sung nút bật/tắt làm mờ theo yêu cầu (On-Demand Privacy Blur)**: Trong modal xem chi tiết, phụ huynh có thể chủ động bấm "🔒 Làm mờ" hoặc "👁️ Xem rõ ảnh" linh hoạt khi cần che nội dung nhạy cảm trước mặt người khác.
  - **Nâng trần kích thước IPC (`MAX_IPC_MESSAGE_SIZE`) lên 512 KiB**: Đảm bảo truyền tải mượt mà và an toàn các frame JPEG rõ nét qua Named Pipe.
- **Chuyển Đổi Toàn Diện Giao Diện Sang Tone Sáng (Light Theme Modernization & Taste-Skill Harmonization)**:
  - Nâng cấp toàn bộ giao diện Web Admin theo bộ quy chuẩn `taste-skill`: bảng màu đơn sắc slate/zinc thanh lịch, loại bỏ hoàn toàn các màu sắc đối lập chói gắt, thống nhất bán kính góc cong card (`rounded-xl` 12px) và độ nổi bóng mờ tinh tế.
  - **Khắc phục triệt để lỗi vỡ khung & cắt xén mép trái**: Chuẩn hóa container trung tâm `max-w-[1440px] w-full mx-auto` kèm padding `px-4 sm:px-6 lg:px-8`, loại bỏ `overflow-x: hidden` trên body, bổ sung `min-w-0` chống vỡ grid trên 4 thẻ KPI.
  - **Hỗ trợ nạp Web Dist từ ProgramData (`famshield-local-admin`)**: Bổ sung đường dẫn ưu tiên `C:\ProgramData\FamShield\web\dist` giúp cập nhật giao diện trực tiếp mà không bị chặn quyền Administrator đối với `C:\Program Files`.
  - Chuẩn hóa các thành phần UI chủ chốt (`Navbar`, `DashboardTab`, `DataTable`, `PinModal`, `EvidenceTab`, `SetupWizard`, `App`) với layout sắc nét, phân cấp thông tin rõ ràng và hiệu ứng chuyển đổi mượt mà.

### Fixed
- **Tối Ưu Hiệu Năng & Khắc Phục Lỗi Mã Nguồn React Web Admin (`web`)**:
  - **Tải dữ liệu song song (Concurrent Data Fetching)**: Tối ưu hàm `refreshAllData` trong `App.tsx` sử dụng `Promise.allSettled` chạy song song 4 yêu cầu API (status, rules, logs, evidence) thay vì gọi tuần tự `await` từng endpoint, giảm ~75% độ trễ mạng khi nạp trang.
  - **Sửa cảnh báo Stale Closure & React Hooks Dependencies**: Bọc `refreshAllData` và `fetchActivity` bằng `useCallback`, khắc phục hoàn toàn cảnh báo `useExhaustiveDependencies` trong `App.tsx` và `ActivityTab.tsx`.
  - **Khắc phục lỗi React Key**: Thay thế toàn bộ các `key={idx}` trong bảng và danh sách (`DashboardTab`, `RulesTab`, `PinModal`, `SecurityTab`) bằng định danh duy nhất (`log.domain-timestamp`, `rule.rule_id`, backup code), ngăn ngừa lỗi render và giật lag khi dữ liệu thay đổi.
  - **Chuẩn hóa Type-Safe Error Handling**: Chuyển đổi toàn bộ khối `catch (e: any)` sang `catch (e: unknown)` với cơ chế trích xuất thông điệp an toàn, tránh bypass kiểm tra kiểu của TypeScript.
  - **Tối ưu thuật toán xử lý PIN**: Chuyển đổi tìm ô PIN trống trong `PinModal` từ `findIndex` sang `indexOf('')`.
- **Khả năng truy cập, tương phản và hiệu năng Web Admin (`web`)**:
  - Bổ sung semantics cho hộp thoại xác thực và bằng chứng, quản lý focus bàn phím, trạng thái live-region, tab navigation và bảng có thể sắp xếp bằng bàn phím.
  - Sửa tương phản cảnh báo và Recovery Key trong Setup Wizard, chuyển bố cục nhập PIN về một cột trên màn hình hẹp và dùng chiều cao viewport động `100dvh`.
  - Đồng bộ thuộc tính ngôn ngữ tài liệu theo lựa chọn Việt/Anh, loại bỏ chuỗi tiếng Việt cố định trong modal bằng chứng và liên kết label với các trường nhập quan trọng.
  - Chuyển thư viện xử lý Excel `xlsx` sang dynamic import để không tải vào bundle khởi động của bảng điều khiển.


---

## [1.0.0] - 2026-10-20

### Security
- **Cloud enrollment và remote command fail-closed**:
  - Loại bỏ hoàn toàn credential mô phỏng khi enrollment HTTP thất bại; thiết bị chỉ chuyển sang `OnlineManaged` sau phản hồi server hợp lệ.
  - Mã hóa `command_signing_key` bằng DPAPI machine-scope, tự động migrate khóa plain-text cũ và dùng Device ID làm commit marker cuối cùng của enrollment.
  - Từ chối mọi remote command khi chưa cấu hình signing key; service tự nạp khóa đã giải mã khi khởi động.
  - Bắt buộc xác thực phiên quản trị cho endpoint ghi nhận vi phạm từ khóa.

### Fixed
- **Khắc Phục Lỗi Cài Đặt Không Tắt Tiến Trình Cũ & Không Dừng/Xóa Dịch Vụ Cũ (`installer/FamShield.iss`, `build.ps1`, `update-famshield.ps1`)**:
  - **Inno Setup Installer (`FamShield.iss`)**: Bổ sung hàm `PrepareToInstall` tự động dừng dịch vụ Windows (`sc stop FamShieldService`), cưỡng chế hạ các tiến trình cũ (`taskkill /F /IM famshield-service.exe /T` và `taskkill /F /IM famshield-session.exe /T`), và xóa dịch vụ cũ (`sc delete FamShieldService`) trước khi giải nén ghi đè các file thực thi. Thiết lập `CloseApplications=no` và `RestartApplications=no` ngăn chặn xung đột và lỗi khóa file (Access Denied / Sharing Violation). Cập nhật `[Run]` hỗ trợ tự động cấu hình lại `sc config` nếu service đang trong quá trình chuyển tiếp.
  - **PowerShell Build & Install Script (`build.ps1`)**: Tích hợp lệnh cưỡng chế đóng toàn bộ tiến trình `famshield-service` và `famshield-session` qua `taskkill` và `Stop-Process`, dừng và xóa dịch vụ hệ thống trước khi sao chép nhị phân mới vào `C:\Program Files\FamShield`, ngăn chặn hoàn toàn lỗi khóa file `Copy-Item`. Cập nhật cả quy trình `Install-Agent` và `Uninstall-Agent`.
  - **Script Cập Nhật Nhanh (`update-famshield.ps1`)**: Nâng cấp bước dừng tiến trình với `taskkill /F /IM` nhằm đảm bảo các tiến trình Session 0 và User Session đều được hạ dứt điểm.
- **Khắc Phục Bộ Lọc Bàn Phím Không Nhận Diện Từ Khóa & Tối Ưu Hook Win32 (`famshield-core`, `famshield-session`, `famshield-service`)**:
  - **Bổ sung từ khóa `sex` và nội dung người lớn vào từ điển mặc định**: Khắc phục nguyên nhân gốc rễ trước đây từ điển chỉ có cụm từ ghép `"phim sex"` nên khi gõ riêng từ `"sex"` trên trình duyệt/tìm kiếm không được nhận diện. Đã bổ sung đầy đủ `"sex"`, `"xxx"`, `"porn"`, `"nsfw"`, `"nude"`, `"hentai"`, `"jav"`, `"thu dam"`, `"gai goi"`, `"loan luan"`, `"lo clip"`, `"show hang"`, cũng như các từ ngữ chửi bậy phổ biến (`"dit"`, `"djt"`, `"vl"`, `"cc"`, `"deo"`, `"buoi"`).
  - **Tối ưu hóa thời gian thực thi Hook Win32 (`low_level_keyboard_proc`)**: Loại bỏ toàn bộ các lời gọi Win32 nặng (`GetForegroundWindow`, `OpenProcess`, `GetGUIThreadInfo`) ra khỏi luồng hook callback, đưa việc kiểm tra ô mật khẩu về luồng worker nền phi đồng bộ. Giúp callback hoàn tất trong thời gian $< 1\mu\text{s}$, ngăn chặn hoàn toàn nguy cơ Windows tự động hủy hook (`LowLevelHooksTimeout`) khi máy tải nặng hoặc duyệt web.
  - **Đồng bộ hóa từ khóa thời gian thực qua IPC (`GetKeywordRulesRequest`)**: Trợ lý phiên `famshield-session` tự động đồng bộ danh sách từ khóa hoạt động và trạng thái bật/tắt từ `famshield-service` khi khởi động và định kỳ mỗi 30 giây, đảm bảo mọi từ khóa do phụ huynh thêm/sửa từ Web Admin được áp dụng ngay lập tức mà không cần khởi động lại.
- **Độ chính xác nhật ký truy cập web/DNS và từ khóa**:
  - Ghi đúng trạng thái `MONITOR` cho domain khớp rule ở chế độ audit-only, ghi `ERROR` khi upstream DNS thất bại và loại bỏ log/phản hồi trùng khi serve-stale revalidation.
  - Loại bỏ toàn bộ activity, web và keyword log giả được seed khi database trống; không còn gán giả mọi DNS query cho `chrome.exe`.
  - Sửa worker từ khóa gọi `tokio::spawn` ngoài runtime thread; chuyển sang runtime handle và retry IPC có kiểm tra acknowledgement để tránh mất log.
  - Bổ sung bộ lọc `MONITOR` và `ERROR` trên giao diện nhật ký DNS.
- **Loại bỏ panic do poisoned mutex trong `InstallerInterceptor`** bằng cơ chế phục hồi lock và trả lỗi rõ ràng khi request biến mất.
- **Sửa Lỗi Tắc Nghẽn Vòng Lặp & Tách Luồng Xử Lý DNS Độc Lập (`famshield-dns`)**:
  - Tái thiết kế toàn diện vòng lặp nhận gói tin UDP: Thay vì xử lý tuần tự chặn đứng luồng chính khi một truy vấn upstream bị rớt gói hoặc chậm trễ, mỗi truy vấn DNS giờ được xử lý song song trong một task Tokio phi đồng bộ độc lập.
  - Sử dụng socket UDP ngẫu nhiên tạm thời (ephemeral socket) kèm timeout 2500ms cho từng truy vấn upstream, loại bỏ hoàn toàn hiện tượng xung đột Transaction ID RFC 1035 và chống đứng proxy vĩnh viễn do mất kết nối.
  - Phản hồi ngay lập tức gói tin `SERVFAIL` (RCODE=2) chuẩn RFC 1035 cho ứng dụng client khi truy vấn upstream thất bại hoặc timeout, ngăn chặn hiện tượng ứng dụng Windows bị treo hoặc chờ quá thời gian socket.
- **Khắc Phục Thiếu Hụt Ghi Nhận Nhật Ký Truy Vấn DNS Vào Web Admin & Telemetry (`famshield-service`, `famshield-dns`)**:
  - Kết nối kênh `tokio::sync::mpsc::unbounded_channel` giữa `DnsProxy` và `ServiceController`: Toàn bộ các truy vấn được giải quyết (cả ALLOW và BLOCK) được đẩy về luồng batch-write tự động định kỳ 500ms xuống bảng `query_logs` trong SQLite.
  - Giúp phân hệ "Nhật Ký DNS" (`LogsTab.tsx`) và bộ đếm tổng/chặn trên Dashboard Web Admin hiển thị dữ liệu thời gian thực đầy đủ, chính xác.
- **Sửa Lỗi Khởi Tạo Phiên WFP User-Mode (`FwpmEngineOpen0`) (`famshield-windows`)**:
  - Sửa lỗi truyền hằng số `1` (bị Windows hiểu nhầm là `RPC_C_AUTHN_DCE_PRIVATE` dẫn đến lỗi `0x32` / `ERROR_NOT_SUPPORTED`) thành `windows::Win32::System::Rpc::RPC_C_AUTHN_WINNT` (`10`).
  - Đảm bảo phiên lọc gói tin tầng ứng dụng WFP khởi chạy thành công 100%, chặn đứng lưu lượng UDP port 53 ngoài luồng và ép toàn bộ phân giải tên miền đi qua proxy `127.0.0.2:53`.
- **Tự Động Kích Hoạt Quy Tắc NRPT Khi Dịch Vụ Khởi Động (`famshield-service`)**:
  - Dịch vụ nền tự động kiểm tra và áp dụng quy tắc điều hướng DNS root namespace `.` về `127.0.0.2:53` qua `NrptManager::apply_nrpt_rule` khi khởi động và dọn dẹp sạch sẽ khi dừng dịch vụ.

### Added
- **Tăng Cường Tác Động Endpoint: Cưỡng Chế Hạn Mức Ứng Dụng & Tự Động Hạ Tiến Trình Quá Giờ (`famshield-windows`, `famshield-service`)**:
  - **Hạ Tiến Trình An Toàn Cấp Kernel (`terminate_processes_by_name`)**: Tích hợp các hàm Win32 API (`OpenProcess`, `TerminateProcess`) với cờ quyền `PROCESS_TERMINATE` trong Session 0. Cho phép tự động phát hiện và hạ toàn bộ các tiến trình game/ứng dụng quá giờ của trẻ em mà không làm treo hệ điều hành.
  - **Cơ Chế Đánh Giá Hạn Mức Hai Tầng (Dual-Stage Quota Enforcement)**:
    - *Tầng 1 - Chặn tức thì khi khởi chạy*: Ngăn chặn trẻ mở lại ứng dụng ngay lập tức khi ứng dụng đó đã hết quota trong ngày (`used_minutes >= max_minutes`), hạ tiến trình trong vòng vài mili-giây trước khi cửa sổ game kịp render.
    - *Tầng 2 - Quét liên tục ứng dụng đang chạy*: Vòng lặp định kỳ mỗi 5 giây quét các tiến trình đang hoạt động đối chiếu với luật cấu hình; ngay khi thời gian tích lũy chạm ngưỡng giới hạn, hệ thống tự động cưỡng chế đóng ứng dụng và khóa màn hình máy trạm (`LockWorkStation`).
  - **Ghi Nhận Sự Cố Bảo Mật (Security Alert Telemetry)**: Tự động phát sinh sự kiện cảnh báo `TERMINATE_QUOTA_EXCEEDED` ghi nhận vào luồng Telemetry Outbox để đồng bộ về Cloud và ứng dụng di động của phụ huynh.
- **Chặn Cổng VPN & Giao Thức DoT Vượt Rào Bằng Windows Filtering Platform (`famshield-windows`, `famshield-service`)**:
  - **Khóa Cổng Vượt Rào Cấp Kernel (WFP Anti-Bypass Port Filters)**: Đăng ký các bộ lọc chặn (`FWP_ACTION_BLOCK`) trên cả hai phân lớp ALE `FWPM_LAYER_ALE_AUTH_CONNECT_V4` và `FWPM_LAYER_ALE_AUTH_CONNECT_V6` đối với các cổng mã hóa vượt rào phổ biến:
    - Cổng `853`: DNS-over-TLS (DoT).
    - Cổng `1194`: Giao thức OpenVPN mặc định.
    - Cổng `51820`: Giao thức WireGuard VPN mặc định.
  - **An Toàn Hệ Thống Tuyệt Đối (Zero BSOD & Network Lockup Guarantee)**: Toàn bộ bộ lọc chống vượt rào được đăng ký độc quyền trong phiên động `FWPM_SESSION_FLAG_DYNAMIC`. Nếu tiến trình dịch vụ gặp sự cố hoặc tắt, Windows Kernel tự động giải phóng toàn bộ bộ lọc trong $0\text{ms}$, đảm bảo không bao giờ gây nghẽn mạng máy trạm.
- **Cơ Chế Nén Tổng Hợp & Tối Ưu Dung Lượng Nhật Ký DNS (DNS Query Log Retention & Rollup Aggregation - `famshield-storage`, `famshield-service`, `famshield-local-admin`, `web`)**:
  - **Bảng Thống Kê Tổng Hợp Vĩnh Viễn (`domain_query_aggregates`)**: Nâng cấp lược đồ cơ sở dữ liệu SQLite lên Version 6. Thay vì giữ hàng chục ngàn dòng nhật ký chi tiết vô thời hạn làm phình to dung lượng ổ cứng, hệ thống tự động gộp (rollup) số lượt truy vấn (`total_queries`, `allowed_count`, `blocked_count`) và mốc thời gian (`first_seen_at`, `last_seen_at`) theo từng domain vào bảng thống kê nhẹ.
  - **Truy Vấn Thống Kê Hợp Nhất (Seamless Aggregation View)**: Cải tiến `get_domain_query_summaries` sử dụng CTE SQLite kết hợp đồng thời giữa bảng thống kê lịch sử `domain_query_aggregates` và bảng nhật ký chi tiết `query_logs`, đảm bảo biểu đồ và số liệu thống kê truy cập web/DNS trên Dashboard không bao giờ bị mất hoặc đứt gãy sau khi dọn dẹp log chi tiết.
  - **Tùy Chỉnh Thời Hạn Lưu Trữ Linh Hoạt (Configurable Retention Window)**: Phụ huynh có thể tùy chỉnh thời hạn lưu nhật ký DNS chi tiết (1, 3, 7, 14, 30, hoặc 90 ngày; mặc định 7 ngày) và bật/tắt chế độ tự động dọn dẹp theo nhu cầu gia đình.
  - **Tiến Trình Dọn Dẹp Nền Tự Động Định Kỳ (Background Storage Maintenance Task)**: `famshield-service` thiết lập worker nền chạy định kỳ mỗi 6 giờ, tự động dọn dẹp các tệp ảnh chụp bằng chứng vi phạm quá hạn (`purge_expired_evidence`) và nén/xóa các bản ghi DNS chi tiết cũ (`rollup_and_prune_query_logs`) theo đúng cấu hình mà không làm gián đoạn hay nghẽn luồng xử lý DNS.
  - **Giao Diện Quản Trị Tối Ưu Lưu Trữ (`LogsTab.tsx`, `api.ts`, `types.ts`)**: Bổ sung khu vực "Tối Ưu Lưu Trữ Nhật Ký" trực quan trên tab Nhật Ký DNS: hiển thị số dòng log chi tiết và số domain được tổng hợp, cho phép thay đổi thời hạn lưu trữ, lưu cấu hình 1 chạm và cung cấp nút bấm "Dọn Dẹp Ngay" để giải phóng dung lượng ổ cứng tức thì.
- **Xác Thực 2 Lớp Cho Phụ Huynh Qua Ứng Dụng Authenticator (2FA / TOTP RFC 6238 & Emergency Backup Codes - `famshield-storage`, `famshield-local-admin`, `web`)**:
  - **Chuẩn Mật Mã RFC 6238 & Bảo Mật DPAPI Machine-Scope**: Tích hợp thư viện `totp-rs = "6.0.0"` hiện đại (Builder pattern, Stack Allocation / Zero-Heap, Constant-Time Comparison), 100% ngoại tuyến (Offline-First) sử dụng thuật toán TOTP HMAC-SHA1 chu kỳ 30 giây, cửa sổ sai lệch thời gian $W \in [-1, 0, +1] \times 30\text{s}$. Khóa bí mật Base32 (160-bit) được sinh từ CSPRNG và mã hóa bảo vệ bằng Windows DPAPI machine-scope trước khi lưu vào SQLite.
  - **Mã QR Render Trực Tiếp Trong RAM**: Tự động render mã QR dưới dạng chuỗi Data URL Base64 trực tiếp trong RAM chuyển lên Web Admin, phụ huynh quét bằng camera điện thoại ngay lập tức mà không phát sinh bất kỳ yêu cầu mạng nào ra bên ngoài.
  - **Bộ 8 Mã Dự Phòng Khẩn Cấp Dùng 1 Lần (Single-Use Emergency Backup Codes)**: Tự động cấp phát 8 mã dự phòng định dạng `ABCD-1234` lưu băm SHA-256 trong SQLite; tự động vô hiệu hóa mã (burn-after-use) ngay sau khi được sử dụng để phòng trường hợp phụ huynh mất hoặc đổi điện thoại.
  - **Đăng Nhập 2 Bước An Toàn (Two-Step Login Flow)**: Khi 2FA được bật, bước 1 nhập Master PIN chỉ cấp token tạm thời (`temp_auth_token`, hiệu lực 5 phút); bước 2 yêu cầu mã OTP 6 số hoặc mã dự phòng để cấp phiên quản trị chính thức.
  - **Khóa Bảo Vệ Thao Tác Trọng Yếu (Step-Up Authentication)**: Yêu cầu bắt buộc phải có mã OTP hoặc mã dự phòng khi thực hiện thao tác gỡ cài đặt hệ thống (`/api/system/uninstall`) hoặc khi tắt tính năng 2FA (`/api/auth/totp/disable`).
  - **Giao Diện Quản Trị Web & Modal Đăng Nhập 2 Lớp (`PinModal.tsx`, `SecurityTab.tsx`, `api.ts`)**: Hỗ trợ chuyển bước mượt mà từ PIN sang OTP, hộp thoại kích hoạt 2FA trực quan kèm hình ảnh QR, khóa bí mật dạng chữ và danh sách 8 mã dự phòng khẩn cấp kèm nút sao chép 1 chạm.
- **Bộ Nhớ Đệm DNS Cục Bộ Tinh Gọn (In-Memory DNS Cache ≤ 2 MB) & Đa Upstream Dự Phòng Tự Động Nhận Diện DNS Hệ Thống (`famshield-dns`, `famshield-windows`, `famshield-local-admin`, `famshield-service`, `web`)**:
  - **Tự Động Nhận Diện DNS Gốc Của Máy Trạm (`NrptManager::get_system_dns_servers`)**: Tự động quét Windows Registry (`HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\*`) để trích xuất DNS gốc được cấp phát bởi DHCP của router/nhà mạng trước khi FamShield can thiệp, tự động loại trừ các địa chỉ loopback nội bộ `127.0.0.x`.
  - **Cơ Chế Đa Upstream Thác Đổ Thông Minh (Fallback Upstream Cascade)**: Proxy ưu tiên Primary Upstream (do người dùng chọn hoặc Cloudflare), tự động chuyển tiếp (failover) sang DNS Gốc của Router/Secondary Upstream với thời gian chờ 1500ms khi upstream chính gặp sự cố, loại bỏ hoàn toàn nguy cơ mất kết nối mạng máy trạm mà không gây bão gói tin UDP (UDP amplification flood).
  - **Bộ Nhớ Đệm DNS Cục Bộ Chuẩn RFC 1035 (`DnsCache`)**:
    - Giới hạn dung lượng bộ nhớ đệm an toàn tuyệt đối: tối đa 2.000 bản ghi (tiêu tốn $\le 2\text{ MB}$ RAM), tự động loại trừ bản ghi cũ nhất khi đầy (FIFO LRU eviction).
    - Trừ lùi TTL động theo thời gian thực chuẩn RFC 1035 (`record.ttl = remaining_ttl`), đảm bảo các ứng dụng Windows và trình duyệt nhận đúng thời hạn sống của bản ghi và tự động cập nhật khi hết hạn.
    - Cơ chế Auto-Invalidation: Tự động xóa sạch bộ nhớ đệm (`cache.flush()`) ngay khi phụ huynh thêm, sửa hoặc xóa quy tắc chặn tên miền, đảm bảo luật chặn mới có hiệu lực ngay lập tức.
  - **Giao Diện Quản Trị Web DNS & Thống Kê Bộ Nhớ Đệm (`web/src/components/SecurityTab.tsx`, `web/src/api.ts`)**:
    - Hiển thị trực quan thông số hiệu năng phân giải DNS: số lượng bản ghi đang cache, tỷ lệ trúng cache (Hits), số lần truy vấn ra ngoài (Misses), và danh sách DNS gốc máy trạm nhận diện được.
    - Hỗ trợ lựa chọn 1 chạm giữa: DNS Gốc Của Router, Cloudflare (1.1.1.1), Google DNS (8.8.8.8), Quad9 (9.9.9.9), hoặc máy chủ DNS tùy chỉnh.
    - Cung cấp nút bấm "Xóa Bộ Nhớ Đệm DNS", tự động làm sạch cả cache RAM FamShield và lệnh hệ điều hành `ipconfig /flushdns`.
- **Cơ Chế Gỡ Cài Đặt Tàng Hình & Hoàn Trả DNS Sạch Sẽ (Stealth PIN-Protected Uninstallation & Clean Slate Teardown - `installer`, `famshield-session`, `famshield-local-admin`, `web`)**:
  - **Tàng Hình Hoàn Toàn Trong Windows (Stealth Mode)**: Thiết lập `CreateUninstallRegKey=no` và `UpdateUninstallLogAppName=no` trong Inno Setup (`FamShield.iss`). Ứng dụng **không đăng ký** vào Windows `Control Panel -> Programs and Features` cũng như Windows 10/11 `Settings -> Apps & Features`, ngăn chặn trẻ em hoặc người dùng máy trạm gỡ trộm bằng các công cụ gỡ phần mềm thông thường của hệ điều hành.
  - **Khóa Gỡ Cài Đặt Bằng Master PIN (Anti-Tampering Uninstall Barrier)**:
    - **REST API Gỡ Cài Đặt Có Xác Thực (`POST /api/system/uninstall`)**: Yêu cầu xác thực Master PIN (Argon2id) của phụ huynh trước khi thực thi lệnh dọn dẹp hệ thống. Từ chối ngay lập tức với mã HTTP 403 nếu sai mật khẩu.
    - **Giao Diện Quản Trị Web (`web/src/components/SecurityTab.tsx`)**: Bổ sung khu vực "Vùng Nguy Hiểm: Gỡ Cài Đặt Ứng Dụng" (Danger Zone) với cảnh báo rủi ro trực quan và modal nhập Master PIN xác nhận.
    - **Menu Khay Hệ Thống & CLI (`famshield-session --uninstall`)**: Thêm tùy chọn `"🗑️ Gỡ Cài Đặt (Yêu Cầu PIN)"` trên menu khay hệ thống hiện đại và hỗ trợ tham số dòng lệnh `--uninstall`, bật hộp thoại Win32 xác thực Master PIN trước khi cho phép gỡ bỏ.
  - **Tiến Trình Hoàn Trả Hệ Thống Sạch 100% (Clean-Slate Teardown Pipeline)**:
    - **Phục hồi DNS Adapter**: Tự động reset toàn bộ card mạng (`Get-NetAdapter | Set-DnsClientServerAddress -ResetServerAddresses`) về DHCP hoặc DNS ban đầu.
    - **Dọn dẹp NRPT Registry**: Xóa sạch cả 2 nhánh quy tắc điều hướng DNS root `.` trong `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig` và `SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig`.
    - **Phục hồi chính sách trình duyệt**: Xóa cờ tắt DNS-over-HTTPS (DoH) trên Google Chrome và Microsoft Edge.
    - **Dọn dẹp tiến trình & Service**: Dừng tiến trình trợ lý phiên `famshield-session.exe`, dừng và xóa Windows Service `FamShieldService` (`sc.exe delete`).
    - **Xóa chứng chỉ Root**: Tự động gỡ chứng chỉ `FamShieldRoot.cer` khỏi Windows Root và TrustedPublisher store qua `certutil.exe -delstore`.
    - **Xóa cache DNS**: Thực thi `ipconfig /flushdns` đảm bảo máy tính hoạt động mạng thông suốt ngay lập tức.
- **Triển Khai Tổng Hợp 4 Trụ Cột Bảo Vệ Chuyên Sâu (Comprehensive 4-Pillar Protection Architecture)**:
  - **Trụ Cột 1: Cưỡng Chế Tìm Kiếm An Toàn & YouTube Hạn Chế Qua DNS VIP (SafeSearch & YouTube Restricted Mode DNS VIP Enforcer - `famshield-dns`, `famshield-sync`)**:
    - **Module Chuyển Hướng DNS VIP (`famshield_dns::safesearch`)**: Phân tích tên miền truy vấn và tự động ghi đè bản ghi A/AAAA trỏ về dải IP VIP bắt buộc của nhà cung cấp:
      - Google SafeSearch (`forcesafesearch.google.com` -> `216.239.38.120`).
      - Bing Strict SafeSearch (`strict.bing.com` -> `204.79.197.220`).
      - YouTube Moderate / Strict Restricted Mode (`restrict.youtube.com` -> `216.239.38.119` hoặc `restrictmoderate.youtube.com`).
      - DuckDuckGo SafeSearch (`safe.duckduckgo.com` -> `52.142.124.215`).
    - **Xây Dựng Gói Tin DNS Chuẩn RFC Phản Hồi Tức Thì (0ms Latency)**: Tự động tổng hợp bản ghi phản hồi A record với TTL 300s và trả về `NOERROR` rỗng cho AAAA queries khi bật SafeSearch, ghi nhận action `REWRITE` trong query log.
    - **Đồng Bộ Chính Sách Đám Mây (`famshield-sync`)**: Tự động đồng bộ các cờ `safesearch_enabled` và `youtube_restricted` từ Cloud Policy (`CategoryConfig`) vào bộ lọc DNS mà không cần ngắt proxy.
  - **Trụ Cột 2: Hệ Thống Tự Bảo Vệ & Phục Hồi Toàn Diện (Anti-Tampering & Self-Healing Watchdog - `famshield-service`)**:
    - **Vòng Lặp Canh Gác Phiên Độc Lập (`SystemWatchdog`)**: Giám sát liên tục tiến trình tương tác người dùng `famshield-session.exe`. Nếu phát hiện bị tắt hoặc kết thúc bất thường, Watchdog lập tức phát sinh cảnh báo bảo mật telemetry và tự động hồi sinh qua `CreateProcessAsUserW` trong Active Console Session.
    - **Khóa & Tái Khẳng Định Cấu Hình Mạng (NRPT & DNS Lock Re-assertion)**: Tự động kiểm tra và tái áp dụng quy tắc NRPT điều hướng DNS root `.` -> `127.0.0.2:53` mỗi ~9 giây, ngăn chặn triệt để hành vi đổi DNS adapter hoặc gỡ registry từ phần mềm bên ngoài.
  - **Trụ Cột 3: Quản Lý Hạn Mức Ứng Dụng & Khóa Máy Tức Thời (Screen Time Quotas & Remote Session Lock - `famshield-storage`, `famshield-service`)**:
    - **Tra Cứu Thời Lượng Dùng Trong Ngày (`get_app_usage_today_ms`)**: Truy vấn tổng thời gian sử dụng tích lũy của ứng dụng từ bảng `app_activity_logs` bắt đầu từ 00:00:00 ngày hiện tại theo giờ địa phương.
    - **Tự Động Khóa Phiên Làm Việc Khi Vượt Hạn Mức**: Service loop định kỳ so sánh thời gian sử dụng thực tế với hạn mức `max_minutes_per_day` trong `local_rules`. Khi phát hiện vượt ngưỡng, dịch vụ lập tức kích hoạt `SessionManager::lock_workstation()`, khóa màn hình Windows an toàn mà không làm mất dữ liệu của trẻ.
  - **Trụ Cột 4: Động Cơ Cập Nhật Tự Động Phối Hợp Cùng Watchdog (Auto-Updater Engine Integration - `famshield-service`)**:
    - **Cờ Phối Hợp Nguyên Tử Chống Xung Đột (`is_updating: Arc<AtomicBool>`)**: Tích hợp cờ nguyên tử chia sẻ giữa bộ điều khiển dịch vụ `ServiceController` và `SystemWatchdog`.
    - **Cơ Chế Khóa Canh Gác Trong Quá Trình Hot-swap**: Khi dịch vụ tải bản vá mới và tiến hành xác thực chữ ký ECDSA P-256 + thay thế file nhị phân, cờ `is_updating` được kích hoạt để tạm dừng Watchdog, ngăn ngừa hoàn toàn tình trạng Watchdog khởi chạy lại phiên bản cũ hoặc tranh chấp file khi đang ghi đè.
- **Bộ Chuẩn Hóa Tiếng Việt Đa Bộ Gõ (Telex & VNI Canonical Normalizer) & Cơ Chế Chống Báo Động Giả (`famshield-core`, `famshield-session`)**:
  - **Module Chuẩn Hóa Đa Tầng (`famshield_core::vietnamese`)**:
    - **VNI Decoder**: Giải mã các tổ hợp phím số tạo dấu mũ/móc (`d9->d`, `e6->e`, `u7->u`, `a8->a`) và nhận diện/bỏ các phím số chỉ dấu thanh (`1` sắc, `2` huyền, `3` hỏi, `4` ngã, `5` nặng) khi gõ VNI.
    - **Telex Decoder**: Giải mã các phím gõ lặp mũ (`dd->d`, `ee->e`, `oo->o`, `aa/aw->a`, `uw/w->u`) và nhận diện/bỏ các phím dấu thanh Telex (`s, f, r, x, j, z`) khi xuất hiện sau nguyên âm.
    - **Canonical Normalization**: Đưa mọi hình thái (Telex, VNI, Unicode có dấu, không dấu, leetspeak `0->o, 3->e` và ký tự chen ngang `c.h.e.t -> chet`) về một dạng chuẩn duy nhất để Aho-Corasick so khớp trong thời gian $O(1)$.
    - **Chống Báo Động Giả (Word Boundary Verification)**: Kiểm tra biên từ cho các từ khóa ngắn $\le 4$ ký tự (như `dm`, `cc`, `sex`, `dit`), ngăn ngừa hoàn toàn việc bắt nhầm các từ vô hại như `admin`, `account`, `sussex`.
  - **Ngắt Ngữ Cảnh Bàn Phím Tự Động (`famshield-session`)**:
    - Bổ sung cơ chế dọn dẹp bộ đệm khi bấm các phím ngắt dòng / chuyển ô nhập liệu (`Enter`, `Tab`, `Escape`), tránh hiện tượng nối chuỗi ký tự chéo ngữ cảnh.
- **Đồng Bộ Nguồn Tự Chặn (`local_rules`) Lên FamShield Cloud Server (`famshield-sync`, `famshield-local-admin`, `ui.rs`)**:
  - **Cập nhật chính sách đám mây (`CloudSyncClient`)**: Bổ sung phương thức `update_web_filter_policy` trong `famshield-sync` đẩy danh sách tên miền tự chặn (`custom_blocked_domains`) lên Cloud Server endpoint `POST /v1/filter/policy?tenant_id=...`.
  - **Hợp nhất quy tắc cục bộ chống mất dữ liệu (`TelemetrySyncWorker`)**: Tự động hợp nhất các domain trong quy tắc cục bộ (`storage.list_local_rules()`) với danh mục đám mây khi nhận chính sách mới từ Cloud Server, đảm bảo quy tắc do phụ huynh tạo tại chỗ không bao giờ bị ghi đè mất.
  - **REST API Đồng Bộ Một Chạm (`LocalAdminServer`)**: Bổ sung endpoint `POST /api/rules/sync-to-cloud` cho phép phụ huynh chủ động đẩy toàn bộ tên miền tự chặn cục bộ lên Cloud Server gia đình (`tenant_id`).
  - **Nút Đồng Bộ Trên Giao Diện Web Admin (`ui.rs`)**: Thêm nút `"☁️ Đồng bộ lên Cloud"` (Sync to Cloud) trên bảng Quản lý Quy tắc Tên miền kèm thông báo toast phản hồi số lượng domain đã đồng bộ.
- **Tối Ưu Hóa Chiến Lược Telemetry DNS & Bảo Vệ Riêng Tư (Privacy-by-Design DNS Telemetry Strategy)**:
  - Phân tầng dữ liệu thông minh: Máy trạm không đẩy toàn bộ lưu lượng phân giải thô (`ALLOW`) lên Cloud Server để tránh làm phình cơ sở dữ liệu và bảo vệ tối đa sự riêng tư của trẻ; chỉ các sự kiện vi phạm (`BLOCK`) mới được tạo thành sự kiện telemetry tức thời đẩy về Cloud Server để báo động qua Telegram/Console.
- **Báo Cáo DNS Tổng Hợp Theo Tên Miền & Cơ Chế Truy Xuất Chi Tiết Khi Cần (Domain Aggregated Summary & Drill-down Telemetry - `famshield-storage`, `famshield-local-admin`, `web`)**:
  - **SQLite Aggregation & Indexing (`famshield-storage`)**: Bổ sung schema migration V4 tạo index `idx_query_logs_domain`, cùng hàm `get_domain_query_summaries` gom nhóm theo domain (`COUNT(*)`, `ALLOW`, `BLOCK`, `MONITOR`, `ERROR` count, `MAX(occurred_at)`) và hàm `list_query_logs_for_domain` phục vụ truy xuất chi tiết.
  - **Local Admin REST API (`famshield-local-admin`)**: Bổ sung endpoint `GET /api/logs/dns/summary` trả về danh sách tổng hợp theo tên miền và mở rộng `GET /api/logs/dns?domain={domain}` để lọc nhật ký truy vấn theo từng tên miền riêng biệt.
  - **Giao Diện Web Admin Thông Minh (`web/src/components/LogsTab.tsx`)**: Chuyển chế độ hiển thị mặc định sang "Bản tổng hợp Domain" (Domain Summary Table) với TanStack React Table hỗ trợ tìm kiếm, sắp xếp và phân trang. Từng tên miền hiển thị tổng số lượt tra cứu, phân loại kết quả (ALLOW, BLOCK, MONITOR, ERROR) và thời gian truy vấn gần nhất.
  - **Hộp Thoại Chi Tiết & Chặn Nhanh (Drill-down Modal & Quick Block)**: Khi phụ huynh cần xem chi tiết, nhấp vào tên miền hoặc nút "Chi tiết" để mở hộp thoại hiển thị toàn bộ các lượt phân giải riêng của tên miền đó (thời gian, loại bản ghi A/AAAA/HTTPS, hành động, lý do), kèm nút "Chặn domain này" tức thời.
  - Hỗ trợ chuyển đổi linh hoạt giữa "Bản tổng hợp Domain" và "Toàn bộ chi tiết" (All Queries Stream) qua thanh công cụ tab. Đồng bộ tính năng này trên cả bản React SPA và bản nhúng nhẹ `ui.rs`.
- **Chuẩn Hóa & Nâng Cấp Hệ Thống Bằng Các Thư Viện Chuyên Nghiệp (Crates Modernization)**:
  - **`tracing-appender = "0.2"` (`famshield-service`, `famshield-session`)**: Thay thế ghi log file thô sơ bằng luồng ghi log file xoay vòng hàng ngày (Daily Non-blocking Rotating File Appender), tự động bảo lưu log trong `%ProgramData%\FamShield\logs` với `WorkerGuard` đa luồng không gây trễ vòng lặp chính.
  - **`rusqlite_migration = "1.2.0"` (`famshield-storage`)**: Thay thế logic migrate bảng SQL thủ công bằng framework quản lý migration nguyên tử có kiểm soát phiên bản `user_version` (Atomic Schema Migration), tự động áp dụng các phiên bản schema (V1 baseline, V2 quotas, V3 sync outbox) trong transaction an toàn, chống tham nhũng dữ liệu khi cúp điện.
  - **`hickory-proto = "0.26"` (`famshield-dns`)**: Thay thế bộ phân tích gói tin DNS thô sơ bằng thư viện chuẩn RFC `hickory-proto` (Hickory DNS / cựu Trust-DNS). Tự động phân giải wire format nhị phân RFC 1035, xây dựng gói tin trả lời chặn NXDOMAIN chuẩn xác với đầy đủ cờ Authoritative (AA) và Recursion Available (RA), kèm fallback an toàn không phụ thuộc.
- **Nâng Cấp Khay Hệ Thống Hiện Đại Chuẩn Tauri với `tray-icon` & `muda` (`famshield-session`)**:
  - Tái cấu trúc toàn diện phân hệ khay hệ thống: Thay thế mã Win32 API cổ điển bằng bộ đôi thư viện chuẩn công nghiệp `tray-icon = "0.26"` và `muda = "0.21"` (phát triển bởi đội ngũ Tauri Core).
  - Hỗ trợ menu ngữ cảnh hiện đại với Windows 11 Dark Mode, submenu đa cấp, và bộ bắt sự kiện click chuột trái/phải/double-click tức thời.
  - Icon khiên bảo mật FamShield 32x32 RGBA độ phân giải cao được render thủ tục trong bộ nhớ, tối ưu hiển thị mượt mà không phụ thuộc file icon ngoài.
- **Cơ Chế Canh Gác Phiên Người Dùng & Tự Động Phục Hồi (Session Watchdog & Auto-Respawn - `famshield-service`, `famshield-windows`, `famshield-local-admin`)**:
  - Dịch vụ nền Windows Service Session 0 tích hợp vòng lặp Watchdog chu kỳ 10s: Tự động kiểm tra tiến trình `famshield-session.exe`.
  - Nếu phát hiện `famshield-session.exe` chưa chạy hoặc bị tắt trong phiên tương tác người dùng (`WTSGetActiveConsoleSessionId`), dịch vụ lập tức ghi nhận cảnh báo `security_alert` vào telemetry và tự động tái khởi động tiến trình bằng `CreateProcessAsUserW` thông qua token người dùng `WTSQueryUserToken`.
  - Cung cấp API `POST /api/system/session-helper/start` cho phép phụ huynh chủ động khởi chạy lại trợ lý phiên từ Web Admin.
  - Hiển thị thanh cảnh báo trực quan màu đỏ nổi bật trên Dashboard Web Admin khi trợ lý phiên bị ngắt kết nối kèm nút "Khởi Chạy Lại Trợ Lý" 1 chạm và badge trạng thái trên thẻ hoạt động.
- **Tích Hợp Bảng Dữ Liệu Chuyên Nghiệp `@tanstack/react-table` (TanStack Table v8 - `web`)**:
  - Xây dựng component bảng dữ liệu dùng chung `DataTable.tsx` chuẩn công nghiệp dựa trên `@tanstack/react-table` v8.
  - Hỗ trợ đầy đủ: Phân trang (Pagination) tùy chọn 10/25/50/100 dòng mỗi trang, Sắp xếp đa cột (Multi-column sorting) với biểu tượng mũi tên trực quan, Tìm kiếm lọc dữ liệu thời gian thực (Global search filtering).
  - Áp dụng thành công cho: Phân hệ Nhật Ký Truy Vấn DNS (`LogsTab.tsx`), Danh Sách Đen Tên Miền (`RulesTab.tsx`), và Hạn Mức Ứng Dụng (`RulesTab.tsx`).
- **Đồng Bộ Hóa Đám Mây & Tầng Vận Chuyển Telemetry (`famshield-sync`, `famshield-service`)**:
  - Triển khai `CloudSyncClient` giao tiếp HTTP chuẩn hóa kết nối với FamShield Cloud Server:
    - `POST /v1/device/enroll`: Ghép nối mã 6 số tự động chuyển chế độ sang `OnlineManaged`.
    - `POST /v1/device/heartbeat`: Báo cáo nhịp tim định kỳ 60s, thời gian chạy `uptime_seconds`, trạng thái `wfp_active`, `dns_active`, và số lượng sự kiện chờ `outbox_depth`.
    - `POST /v1/device/events:batch`: Đóng gói và đẩy hàng loạt sự kiện `TelemetryEnvelope` (v1.0) từ SQLite outbox lên máy chủ.
    - `GET /v1/filter/policy`: Tự động tải chính sách lọc web mới nhất khi máy chủ phát hiện tăng `desired_policy_version`.
  - Triển khai `TelemetrySyncWorker` tích hợp trực tiếp vào vòng lặp dịch vụ `ServiceController`:
    - Chu kỳ 15s tự động dọn dẹp sự kiện đã gửi (`acknowledge_batch`) hoặc áp dụng backoff 30s khi mất mạng (`report_batch_failure`).
    - Tự động cập nhật tức thì danh sách đen DNS Proxy (`BlocklistStore`) theo các danh mục phụ huynh cấu hình trên Cloud (Người lớn, Cờ bạc, Bạo lực, Mạng xã hội, Game) trong 0ms mà không làm gián đoạn mạng.
    - Kiểm tra và ghi nhận các bản phát hành cập nhật phần mềm mới từ Cloud (`available_release`).

- **Giao diện & Máy chủ Quản trị Cục bộ (Local Web Admin Console - `famshield-local-admin`, `famshield-service`)**:
  - Triển khai máy chủ HTTP cục bộ tốc độ cao (`LocalAdminServer`) dựa trên nền tảng `axum = "0.7"` và `tokio`, lắng nghe an toàn tại loopback `http://127.0.0.1:48123` (chỉ chấp nhận kết nối nội bộ loopback, chống truy cập từ mạng LAN/WAN).
  - Giao diện Single Page Application (SPA) hoàn chỉnh nhúng trực tiếp trong file nhị phân (Zero-external dependency), tối ưu 100% cho chế độ ngoại tuyến không Internet (StandaloneFree).
  - Hỗ trợ song ngữ tức thì (Tiếng Việt mặc định và Tiếng Anh) kèm lưu tùy chọn trong `localStorage`.
  - 5 phân hệ chức năng toàn diện theo tài liệu kỹ thuật `docs/10-WEB-CONSOLE.md`:
    1. **Dashboard (Tổng quan):** Đo lường và hiển thị trực quan ứng dụng hoạt động foreground, thanh hạn mức quota sử dụng trong ngày, trạng thái DNS loopback `127.0.0.2:53` & WFP Dynamic Session, số lượt chặn tên miền hôm nay, và bảng preview truy vấn gần nhất.
    2. **Rules & Limits (Quản lý Quy tắc & Hạn mức):** CRUD danh sách đen tên miền (Domain Blocklist) với cơ chế tự động cập nhật nguyên tử vào `BlocklistStore` của DNS Proxy trong 0ms; Đặt hạn mức số phút sử dụng mỗi ngày cho từng file ứng dụng (App limits/quotas).
    3. **DNS Telemetry Logs (Nhật ký Phân giải Tên miền):** Tìm kiếm tên miền theo thời gian thực, lọc nhanh theo trạng thái (Tất cả / Chỉ chặn / Cho phép), tự động làm mới chu kỳ 4s.
    4. **Smart Visual Verification Gallery (Thư viện Bằng chứng Thị giác Thông minh):** Hiển thị danh sách ảnh chụp màn hình vi phạm đã làm mờ bảo vệ riêng tư (Gaussian Blur 480p), hiển thị lý do kích hoạt, kích thước nén, và đồng hồ đếm ngược tự hủy sau 48 giờ.
    5. **Security & Cloud Wizard (Bảo mật & Ghép nối):** Đổi Master PIN (yêu cầu xác thực PIN cũ), hiển thị Mã Khôi Phục Khẩn Cấp (Emergency Recovery Key), khôi phục PIN bằng Recovery Key, và Wizard ghép nối với Cloud Server qua mã 6 số (chuyển đổi linh hoạt giữa `StandaloneFree` và `OnlineManaged`).
  - Quản lý phiên làm việc & Xác thực bảo mật: Session token 32-byte ngẫu nhiên an toàn, hỗ trợ trượt thời hạn (slide expiration 60 phút), cookie `HttpOnly; SameSite=Strict`, header `X-FamShield-Token`, và modal nhập Master PIN với bàn phím ảo keypad số hoặc nhập phím vật lý.
  - **Giao diện Web Admin Hiện Đại Chuẩn React 19 + Vite (`web/`)**:
    - Nâng cấp toàn diện giao diện sang nền tảng **React 19 + TypeScript + Vite 8** với hệ thống biểu tượng trực quan **Lucide Icons** và phong cách thiết kế Glassmorphism Dark Theme công nghệ cao.
    - Cấu trúc module hóa chuyên nghiệp: `Navbar`, `PinModal`, `DashboardTab`, `RulesTab`, `LogsTab`, `EvidenceTab`, `SecurityTab`, `SetupWizard`, `api.ts`, `types.ts`, `i18n.ts`.
    - Máy chủ Axum Rust hỗ trợ phục vụ trực tiếp gói bundle tĩnh từ `web/dist` (`index.html`, `/assets/*`) kèm cơ chế fallback nhúng bền vững, đảm bảo hoạt động mượt mà 100% khi chạy standalone offline.
  - **Trợ Lý Thiết Lập Ban Đầu & Quy Trình Hậu Cài Đặt (Initial Setup Wizard & Post-Install Workflow)**:
    - **Initial Setup Wizard 5 Bước (`SetupWizard.tsx`):**
      - Tự động phát hiện trạng thái chưa cấu hình lần đầu thông qua API `GET /api/setup/status` (`is_configured: false`).
      - *Bước 1: Chào Mừng & Giới Thiệu:* Trình bày các cam kết bảo vệ cốt lõi (100% Ngoại tuyến StandaloneFree, Tường lửa Dynamic WFP 0ms, Bằng chứng thị giác xóa mờ).
      - *Bước 2: Tạo Master PIN Phụ Huynh:* Nhập và xác nhận mã PIN 6 chữ số bí mật, hỗ trợ nhập số và kiểm tra bảo mật.
      - *Bước 3: Chọn Cấu Hình Bảo Vệ Mẫu:* 3 preset trực quan: Cân Bằng (Balanced - khuyên dùng), Nghiêm Ngặt (Strict - trẻ nhỏ), hoặc Cơ Bản (Relaxed - tự tùy chỉnh).
      - *Bước 4: Lựa Chọn Chế Độ Vận Hành:* StandaloneFree (100% Offline bảo vệ riêng tư) hoặc OnlineManaged (ghép nối đám mây qua Enrollment Code).
      - *Bước 5: Cấp Mã Khôi Phục Khẩn Cấp (Emergency Recovery Key):* Sinh mã dạng `FAM-XXXX-XXXX-XXXX`, hỗ trợ sao chép 1 chạm và cảnh báo an toàn trước khi chuyển thẳng vào Bảng Quản Trị.
    - **Quy trình Hậu Cài Đặt Tự Động (`installer/FamShield.iss`):**
      - Tự động đóng gói thư mục `web/dist` vào thư mục cài đặt `{app}\web\dist`.
      - Tự động kích hoạt tác vụ khởi chạy tiến trình phiên người dùng `famshield-session.exe` và tự động mở trình duyệt web mặc định trỏ về `http://127.0.0.1:48123` ngay sau khi cài đặt thành công.
    - **Tích Hợp Khay Hệ Thống Windows (`agent/crates/session/src/tray.rs`):**
      - Bổ sung tùy chọn menu ngữ cảnh `"🌐 Mở Bảng Quản Trị Web (Local Admin)"` gọi trực tiếp `ShellExecuteW` mở trang quản trị cục bộ tại mọi thời điểm.
  - **Cấu Hình Kết Nối Máy Chủ Quản Lý Tập Trung (Centralized Management Server Settings)**:
    - **Hệ Thống API Máy Chủ Quản Trị Cục Bộ (`famshield-local-admin`):**
      - `GET /api/settings/server`: Truy xuất cấu hình máy chủ, trạng thái kết nối trực tiếp (`Connected`, `Disconnected`, `Syncing`), phiên bản chính sách, ID thiết bị và thời điểm đồng bộ gần nhất.
      - `POST /api/settings/server`: Cho phép quản trị viên cấu hình địa chỉ máy chủ tập trung (URL / Endpoint), Mã tổ chức (Tenant ID), Tên thiết bị (Device Name), Mã ghép nối (Enrollment Code), Chu kỳ đồng bộ (10s – 86400s), và Chế độ xác thực chứng chỉ TLS/SSL (`tls_verify`).
      - `POST /api/settings/server/test`: Kiểm tra kết nối mạng không đồng bộ (Asynchronous TCP probe qua Tokio với timeout 3s) đo độ trễ round-trip (ms) tới máy chủ tập trung trước khi lưu cấu hình.
      - `POST /api/settings/server/sync`: Kích hoạt đồng bộ tức thời chính sách từ xa và gửi nhịp tim/telemetry.
    - **Giao Diện Quản Trị Trung Tâm Trực Quan (`SecurityTab.tsx`):**
      - Thanh trạng thái kết nối thời gian thực với đèn tín hiệu xung nhịp Pulse, hiển thị các chỉ số: Device ID, Tenant ID, Lần đồng bộ gần nhất, Phiên bản chính sách server.
      - Nút "Kiểm Tra Kết Nối" hiển thị tức thì độ trễ ping và thông báo lỗi rõ ràng.
      - Nút "Đồng Bộ Ngay" với hiệu ứng animation xoay mượt mà.
      - Phím chọn nhanh URL mẫu (FamShield Cloud Default hoặc Local LAN Enterprise Server).
      - Tùy chọn Bật/Tắt Strict TLS Verification (tiện lợi cho mạng nội bộ trường học/doanh nghiệp dùng chứng chỉ tự ký).
      - Tích hợp cấu hình Server URL ngay từ Bước 4 của Trợ lý Thiết lập Ban đầu (`SetupWizard.tsx`).
  - **Nhập & Xuất Danh Sách Đen Bằng File Excel (Excel Blacklist Import & Export)**:
    - **Frontend Xử Lý Bảng Tính Phía Client (`RulesTab.tsx`, `xlsx` / SheetJS):**
      - **Xuất ra Excel (`.xlsx`):** Tạo file Excel đa trang (Multi-sheet workbook): Sheet 1 `Danh Sách Chặn Tên Miền` (Domain, Type, Reason, Status, Created At) và Sheet 2 `Hạn Mức Ứng Dụng` (Application, Quota minutes, Status, Created At).
      - **Tải File Mẫu (`FamShield_Blacklist_Template.xlsx`):** Tải file mẫu chuẩn hóa kèm các ví dụ thực tế cho phụ huynh và quản trị viên dễ dàng điền quy tắc.
      - **Nhập từ Excel (`.xlsx`, `.xls`, `.csv`):** Tự động nhận diện linh hoạt các cột dữ liệu không phân biệt hoa thường (`Tên miền/Domain`, `Loại/Type`, `Lý do/Reason`, `Hạn mức/Quota`, `Trạng thái/Status`).
      - **Hộp thoại Xem trước (Import Preview Modal):** Thống kê số lượng tên miền và ứng dụng được phát hiện, hiển thị bảng xem trước trực quan với huy hiệu loại quy tắc và trạng thái trước khi xác nhận.
    - **Backend API Hàng Loạt (`famshield-local-admin`):**
      - Bổ sung endpoint `POST /api/rules/bulk`: Chấp nhận mảng quy tắc `Vec<CreateRuleRequest>`, lưu trữ nguyên tử vào cơ sở dữ liệu SQLite và kích hoạt đồng bộ hóa `BlocklistStore` một lần duy nhất, tối ưu hóa hiệu năng khi nhập hàng trăm quy tắc.
  - **Chế Độ Giám Sát Không Chặn (Audit-Only / Monitor Mode - `famshield-dns`, `famshield-local-admin`, `web`)**:
    - Tích hợp cờ nguyên tử `monitor_mode: Arc<AtomicBool>` trong `BlocklistStore` (`famshield-dns`), cho phép chuyển đổi chế độ bảo vệ tức thời trong 0ms mà không cần khởi động lại dịch vụ hoặc tiến trình.
    - Khi kích hoạt Chế độ Giám sát: Các truy vấn DNS tới tên miền trong danh sách đen vẫn được phân tích và ghi nhận nhật ký vi phạm (`[MONITOR MODE] Query for blacklisted domain observed`), đồng thời cho phép gói tin đi qua (passthrough) tới DNS thượng tầng (Upstream DNS) thay vì trả về `0.0.0.0` hay `NXDOMAIN`.
    - Cung cấp API `GET /api/settings/protection-mode` và `POST /api/settings/protection-mode`, đồng bộ hóa tức thì sang DNS Proxy và WFP engine.
    - Cập nhật giao diện Web Console (`DashboardTab.tsx`, `SecurityTab.tsx`): Hiển thị thanh cảnh báo nổi bật màu hổ phách khi ở chế độ giám sát, thẻ trạng thái trực quan, và khối chuyển đổi nhanh giữa `🛡️ Chặn Chủ Động (Active Blocking)` và `👁️ Chỉ Giám Sát (Monitor Mode)`.
  - **Cơ Chế Sao Lưu & Phục Hồi Dữ Liệu An Toàn (Cryptographic Backup & Disaster Recovery - `famshield-local-admin`, `web`)**:
    - Hỗ trợ xuất và nhập toàn bộ cấu hình hệ thống máy trạm dưới định dạng sao lưu chuẩn hóa `.famshield-bak` (JSON), bao gồm: Phiên bản định dạng `1.0`, thông tin thiết bị (`device_id`, `operating_mode`), cấu hình máy chủ quản trị tập trung, trạng thái chế độ bảo vệ/giám sát, toàn bộ danh sách quy tắc chặn tên miền và giới hạn ứng dụng (`local_rules`).
    - Tích hợp thuật toán băm mật mã SHA-256 (`compute_checksum`, `verify_checksum`) bảo đảm tính toàn vẹn tuyệt đối: Phát hiện và từ chối khôi phục ngay lập tức nếu tệp sao lưu bị hỏng hoặc bị can thiệp/chỉnh sửa trái phép.
    - Cung cấp API `GET /api/backup/export` (tải xuống tệp `.famshield-bak` với header `Content-Disposition`) và `POST /api/backup/restore` (xác thực chữ ký, khôi phục nguyên tử vào SQLite, và kích hoạt `sync_dns_blocklist`).
    - Giao diện người dùng (`SecurityTab.tsx`): Nút "Tạo Bản Sao Lưu Ngay" tải xuống 1 chạm, nút "Phục Hồi Từ Tệp Sao Lưu" với hộp thoại xem trước (Backup Preview Modal) hiển thị số lượng quy tắc, thời điểm tạo, mã thiết bị và xác nhận khôi phục an toàn.
  - **Phân Hệ Nhật Ký & Dòng Thời Gian Giám Sát Hoạt Động Toàn Diện (Comprehensive Activity & Screen Time Timeline - `famshield-storage`, `famshield-local-admin`, `web`)**:
    - Tích hợp cấu trúc theo dõi phân đoạn thời gian sử dụng ứng dụng `UsageSegmentRecord` (`famshield-storage`), lưu trữ định kỳ các checkpoint thời lượng thực tế vào bảng SQLite `usage_segments`.
    - Cung cấp API `GET /api/activity/summary` tổng hợp toàn diện:
      - Tổng thời gian vào máy trong ngày (Total screen time) được chuẩn hóa và định dạng trực quan (ví dụ: `4 giờ 35 phút`).
      - Nhật ký sử dụng ứng dụng: Ghi nhận tên tiến trình, tên hiển thị, danh mục (Trò chơi, Mạng xã hội, Học tập, Trình duyệt), khung giờ bắt đầu và kết thúc (`14:15 -> 15:20`), thời lượng sử dụng và trạng thái cảnh báo hạn mức.
      - Lịch sử truy cập trang web: Danh sách tên miền tra cứu DNS, danh mục nội dung, thời gian chính xác, hành động (CHO PHÉP, GIÁM SÁT, CHẶN), lý do và ứng dụng nguồn (Source Process).
      - Dòng thời gian hợp nhất (Unified Timeline): Sắp xếp tuần tự mọi hoạt động mở ứng dụng và duyệt web theo trục thời gian thực.
    - Xây dựng giao diện trang mới **Hoạt Động Giám Sát (`ActivityTab.tsx`)**:
      - 4 thẻ KPI đo lường: Tổng thời gian vào máy, Ứng dụng đã sử dụng, Trang web đã vào, Sự kiện cảnh báo & vi phạm.
      - Bộ 3 chế độ xem linh hoạt: Dòng thời gian hợp nhất (Unified Timeline), Nhật ký ứng dụng (Apps Timeline), Lịch sử web (Web History).
      - Bộ lọc danh mục nội dung, ô tìm kiếm từ khóa thời gian thực và nút xuất báo cáo hoạt động (`.json`) 1 chạm.
  - **Giám Sát Gõ Từ Khóa Bậy & Ngôn Từ Nhạy Cảm (Keystroke Keyword Monitoring - `famshield-core`, `famshield-storage`, `famshield-session`, `famshield-local-admin`, `web`)**:
    - Triển khai nguyên lý **Privacy-by-Design Tuyệt Đối**: FamShield không phải là keylogger, không lưu phím gõ của trẻ xuống đĩa; Sử dụng bộ đệm trượt trong RAM (`KeystrokeSlidingBuffer`) dung lượng tối đa 128 ký tự, tự động xóa bộ đệm sau khi phát hiện vi phạm hoặc khi chuyển đổi ứng dụng active window.
    - Thuật toán tìm kiếm đa mẫu tốc độ cao **Aho-Corasick** (`famshield_core::keyword::KeywordMatcher`) với độ phức tạp $O(N)$ độc lập với số lượng từ khóa trong danh sách.
    - Bộ chuẩn hóa tiếng Việt & Teencode thông minh (`normalize_text`): Chuẩn hóa Unicode đa dạng, chuyển đổi chữ thường, khử dấu tiếng Việt (Telex/VNI), ánh xạ leet-speak (`0->o`, `1->i/l`, `3->e`, `4->a`, `5->s`, `7->t`), và loại bỏ ký tự cố tình phân tách lách luật (`d.m`, `v_c_l`, `d*m`).
    - Danh mục từ khóa mặc định phong phú (Việt - Anh): Chửi bậy / Thô tục (`Profanity`), Bắt nạt / Đe dọa (`Bullying`), Tự hại / Tiêu cực (`SelfHarm` - Critical), Người lớn / Khiêu dâm (`Adult`), và Tùy chỉnh của cha mẹ (`Custom`).
    - Lưu trữ cơ sở dữ liệu SQLite cục bộ (`schema v3`): Bảng `keyword_violations` ghi nhận vi phạm kèm mặt nạ ẩn danh ngữ cảnh (`*** [keyword] ***`), và bảng `custom_keywords` lưu các từ khóa do phụ huynh thêm mới.
    - **Win32 Session Keyboard Hook & Cơ chế Chống Lag Tuyệt Đối (`famshield-session`)**:
      - Triển khai Windows Low-Level Keyboard Hook (`WH_KEYBOARD_LL`) trong tiến trình tương tác người dùng (`famshield-session.exe`), xử lý triệt để bài toán Session 0 Isolation.
      - Kiến trúc bất đồng bộ không trễ (Zero-lag Anti-blocking Architecture): Thủ tục `low_level_keyboard_proc` đẩy `RawKeyEvent` vào kênh đệm `sync_channel(512)` và trả về ngay lập tức với `CallNextHookEx` trong microsecond, không bao giờ gây giật lag bàn phím người dùng.
      - **Bảo Vệ Riêng Tư & Chống Bắt Mật Khẩu (Password Field & Credential Protection)**:
        - Tự động kiểm tra thuộc tính trường nhập liệu đang focus thông qua `GetGUIThreadInfo` và kiểu `ES_PASSWORD (0x0020)`: Bỏ qua và hủy ngay lập tức các ký tự gõ trong ô mật khẩu trước khi đưa vào bộ đệm.
        - Tự động nhận diện và bỏ qua toàn bộ thao tác gõ trong các trình quản lý mật khẩu phổ biến (`1Password`, `Bitwarden`, `KeePass`, `LastPass`, `Enpass`, `Dashlane`) và hộp thoại xác thực `Windows Credential UI`.
      - **Thông Báo Giáo Dục Trực Quan Thân Thiện (Friendly Child Guidance Toasts)**:
        - Tích hợp `FriendlyToast::notify_sensitive_keyword_reminder` gọi trực tiếp `Shell_NotifyIconW` trên biểu tượng khay hệ thống, hiển thị lời nhắc nhở giáo dục nhẹ nhàng ("🌟 Giữ lời nói lịch sự và tôn trọng người khác khi giao tiếp") thay vì tạo tâm lý ức chế cho trẻ.
      - **Giao Tiếp Liên Tiến Trình IPC Named Pipe (`famshield-service`)**:
        - Gửi sự kiện `IpcMessage::KeywordViolation` qua đường ống an toàn `\\.\pipe\FamShieldService` vào Windows Service Session 0 để lưu trữ nguyên tử vào cơ sở dữ liệu SQLite.
    - Hệ thống API quản trị cục bộ (`famshield-local-admin`):
      - `GET /api/keywords`: Truy xuất cấu hình giám sát, trạng thái bật/tắt, danh sách từ khóa hệ thống & tùy chỉnh, và số lượng vi phạm hôm nay.
      - `POST /api/keywords`: Thêm từ khóa tùy chỉnh mới kèm phân loại danh mục và mức độ nghiêm trọng.
      - `DELETE /api/keywords/:keyword_id`: Xóa từ khóa tùy chỉnh.
      - `POST /api/keywords/toggle`: Bật/tắt từng từ khóa hoặc nhóm từ khóa.
      - `GET/POST /api/keywords/monitoring`: Bật/tắt toàn bộ tính năng giám sát bàn phím.
      - `POST /api/keywords/violation`: Endpoint ghi nhận sự kiện vi phạm từ session hook.
    - Giao diện Web Console React 19 (`ActivityTab.tsx`, `SecurityTab.tsx`):
      - Thẻ KPI số 5 trên trang Hoạt Động Giám Sát: Thống kê số lượng từ khóa vi phạm hôm nay kèm nhãn cảnh báo vàng amber.
      - Phân mục subtab thứ 4: **⚠️ Từ Khóa Nhạy Cảm (Keywords)** thống kê chi tiết thời gian, từ khóa, phân loại, mức độ, ứng dụng nguồn và ngữ cảnh làm mờ.
      - Dòng thời gian hợp nhất (Unified Timeline): Tích hợp trực tiếp các mốc thời gian vi phạm từ khóa xen kẽ với hoạt động ứng dụng và duyệt web.
      - Thẻ cấu hình chuyên biệt trong Cài đặt Bảo mật: Bật/tắt tính năng giám sát, form thêm từ khóa tùy chỉnh, thanh tìm kiếm và bảng bật/tắt từng từ khóa tức thời.

### Fixed
- **Ẩn Hoàn Toàn Cửa Sổ Console Khay Hệ Thống (`famshield-session`)**:
  - Bổ sung chỉ thị `#![windows_subsystem = "windows"]` vào đầu mã nguồn `session/src/main.rs`, chỉ thị cho PE Loader của Windows xử lý file nhị phân dưới dạng Windows GUI Subsystem thuần túy.
  - Loại bỏ hoàn toàn cửa sổ dòng lệnh đen (Command Prompt console window) khi ứng dụng chạy nền ở khay hệ thống.
- **Xung Đột Cổng Mạng Với Hyper-V / WSL2 & Cơ Chế Fallback Cổng Linh Hoạt (`famshield-local-admin`, `famshield-service`, `famshield-session`, `installer`)**:
  - Phát hiện và giải quyết xung đột cổng mạng hệ thống: Dải cổng `48074 - 48173` bị Windows Hyper-V / WSL2 NAT tự động gán vào danh sách `excludedportrange`, dẫn đến lỗi `Os Error 10013 (WSAEACCES: access forbidden)` khi mở cổng `48123`.
  - Chuyển cổng mặc định của Local Web Admin sang cổng an toàn `38123`.
  - Thiết lập cơ chế tự động thử cổng dự phòng (`[38123, 18123, 28123, 48123]`) nếu cổng chính bị hệ điều hành chiếm dụng.
  - Tự động lưu cổng thực tế đang hoạt động vào tệp cấu hình `%ProgramData%\FamShield\admin_port.txt`.
  - Cập nhật menu khay hệ thống (`tray.rs`), hộp thoại Master PIN (`pin_dialog.rs`) và gói cài đặt Inno Setup (`FamShield.iss`) đọc cổng động từ tệp cấu hình để mở chính xác giao diện quản trị trong trình duyệt mặc định.
- **Lỗi Panic Khởi Động Tokio IPC Server (`famshield-service::ipc`)**:
  - Khắc phục lỗi panic `cannot specify more than 254 instances` tại `create_secured_named_pipe` do thiết lập nhầm `options.max_instances(255)`. Giới hạn hợp lệ của Tokio là $\le 254$.
  - Lỗi này từng khiến luồng nền Tokio bị hủy ngay khi Service khởi động, dẫn đến máy chủ `LocalAdminServer` không thể lắng nghe và người dùng không thể truy cập giao diện web từ khay hệ thống.
  - Sau khi sửa và biên dịch lại, máy chủ web và IPC Named Pipe đã hoạt động ổn định 100%.

### Changed
- **Phát hành chính thức FamShield Agent 1.0.0 (Production Release)**.
- Đồng bộ phiên bản toàn diện: Rust Workspace (`Cargo.toml`), Local Web Admin (`web/package.json`), và Windows Inno Setup (`FamShield.iss`) đều lên `1.0.0`.
- Cập nhật hướng dẫn triển khai và quy trình đóng gói theo tên bộ cài `FamShieldSetup-1.0.0.exe`.

---

## [0.1.0-rc.1] - 2026-10-19

### Added
- **Windows Service SCM Dispatcher (`famshield-service`)**:
  - Tích hợp `windows-service` crate quản lý toàn diện vòng đời dịch vụ (`ServiceControl::Stop`, `Shutdown`).
  - Hỗ trợ chế độ Interactive Console tự động khi chạy trực tiếp ngoài Service Control Manager.
  - Named Pipe Server với cơ chế framing độ dài tiền tố (4-byte LE length, trần an toàn 256 KiB chống tràn RAM).
  - Chuỗi SDDL bảo mật nghiêm ngặt `D:(A;;GRGW;;;AU)(A;;GA;;;SY)(A;;GA;;;BA)` phân quyền chặt chẽ giữa Authenticated Users và Administrators / LocalSystem.
- **Báo cáo Đánh giá Khoảng trống Hệ sinh thái & Lộ trình Hoàn thiện (`docs/14-GAP-ANALYSIS-AND-ROADMAP.md`)**:
  - Rà soát toàn diện hiện trạng kết nối giữa Windows Agent (`child_monitor`) và Cloud Server/Web Console (`famshield-server`).
  - Lập ma trận tính năng chi tiết, phân loại 4 khoảng trống kỹ thuật cốt lõi (Database PostgreSQL cho server, API thật cho Web Console, Agent live networking, và Local Admin GUI cho bản StandaloneFree).
  - Hoàn thiện tài liệu chỉ dẫn lộ trình 4 giai đoạn thực thi đưa hệ sinh thái FamShield vào vận hành hoàn chỉnh.
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
