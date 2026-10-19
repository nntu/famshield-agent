# Changelog

Tất cả các thay đổi quan trọng của dự án **FamShield Windows Agent** sẽ được ghi lại trong tài liệu này.

Định dạng tài liệu dựa trên [Keep a Changelog](https://keepachangelog.com/vi/1.0.0/), và dự án tuân thủ [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
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
