# FamShield Agent: Cẩm Nang Tính Năng & Hướng Dẫn Sử Dụng Toàn Diện

> **Phiên bản tài liệu:** 1.0.2
> **Dành cho:** Phụ huynh, Quản trị viên gia đình và Kỹ sư phát triển  
> **Áp dụng cho:** Hệ điều hành Windows 10 (bản 1809 trở lên) và Windows 11 (64-bit)

---

## MỤC LỤC

1. [Giới Thiệu Tổng Quan Về FamShield](#1-giới-thiệu-tổng-quan-về-famshield)
2. [Triết Lý Cốt Lõi: Bảo Vệ Riêng Tư & An Toàn Tuyệt Đối](#2-triết-lý-cốt-lõi-bảo-vệ-riêng-tư--an-toàn-tuyệt-đối)
3. [Tổng Hợp Chi Tiết Tất Cả Tính Năng (Feature Catalog)](#3-tổng-hợp-chi-tiết-tất-cả-tính-năng-feature-catalog)
   - 3.1. Hai Chế Độ Hoạt Động (Dual-Mode Operation)
   - 3.2. Kiểm Soát Mạng & Lọc Tên Miền DNS (DNS & WFP Engine)
   - 3.3. Giám Sát & Quản Lý Ứng Dụng Thông Minh (App & Time Control)
   - 3.4. Giám Sát & Lọc Từ Khóa Xấu An Toàn (Keystroke Keyword Filter)
   - 3.5. Bằng Chứng Trực Quan Tôn Trọng Riêng Tư (Smart Visual Verification)
   - 3.6. Cơ Chế Chống Can Thiệp & Bảo Vệ Hệ Thống (Anti-Tampering & Security)
   - 3.7. Bảng Điều Khiển Cục Bộ (Local Web Admin Console)
4. [Yêu Cầu Hệ Thống & Chuẩn Bị](#4-yêu-cầu-hệ-thống--chuẩn-bị)
5. [Hướng Dẫn Cài Đặt Từng Bước (Installation Guide)](#5-hướng-dẫn-cài-đặt-từng-bước-installation-guide)
   - 5.1. Cài đặt bằng bộ cài chuẩn (.exe)
   - 5.2. Cài đặt từ mã nguồn dành cho lập trình viên (Developer Setup)
6. [Hướng Dẫn Cấu Hình Lần Đầu (Initial Setup Wizard)](#6-hướng-dẫn-cấu-hình-lần-đầu-initial-setup-wizard)
7. [Hướng Dẫn Sử Dụng Các Tính Năng Hàng Ngày](#7-hướng-dẫn-sử-dụng-các-tính-năng-hàng-ngày)
   - 7.1. Bảng điều khiển tổng quan (Dashboard)
   - 7.2. Quản lý Quy tắc Chặn & Lọc Web (Rules Management)
   - 7.3. Cài đặt Hạn mức Thời gian chơi Game & Ứng dụng (App Limits)
   - 7.4. Thiết lập Lịch trình & Khung giờ Giới nghiêm (Schedules)
   - 7.5. Theo dõi Hoạt động thực tế (Activity Monitoring)
   - 7.6. Giám sát & Quản lý Từ khóa Xấu (Inappropriate Keyword Monitoring)
   - 7.7. Tra cứu Nhật ký truy cập & Vi phạm (Logs)
   - 7.8. Xem Bằng chứng hình ảnh vi phạm (Visual Evidence)
8. [Hướng Dẫn Kích Hoạt Xác Thực 2 Lớp (2FA / TOTP)](#8-hướng-dẫn-kích-hoạt-xác-thực-2-lớp-2fa--totp)
9. [Hướng Dẫn Kết Nối Đám Mây & Ghép Nối Thiết Bị (Cloud Pairing)](#9-hướng-dẫn-kết-nối-đám-mây--ghép-nối-thiết-bị-cloud-pairing)
10. [Hướng Dẫn Gỡ Cài Đặt Sạch Sẽ (Clean-Slate Teardown)](#10-hướng-dẫn-gỡ-cài-đặt-sạch-sẽ-clean-slate-teardown)
11. [Xử Lý Sự Cố Thường Gặp (FAQ & Troubleshooting)](#11-xử-lý-sự-cố-thường-gặp-faq--troubleshooting)
12. [Giấy phép](#12-giấy-phép)

---

## 1. Giới Thiệu Tổng Quan Về FamShield

**FamShield Agent** là giải pháp phần mềm thế hệ mới được thiết kế chuyên biệt để bảo vệ trẻ em và quản trị máy tính Windows trong môi trường gia đình. Được xây dựng hoàn toàn bằng ngôn ngữ lập trình hệ thống **Rust**, FamShield đạt hiệu năng vượt trội, độ ổn định tuyệt đối và tiêu tốn cực ít tài nguyên (chỉ $\approx 25-40\text{ MB}$ RAM).

FamShield giải quyết triệt để bài toán mà hầu hết các phần mềm phụ huynh hiện nay mắc phải: **xâm phạm quá đà quyền riêng tư của con cái** hoặc **quá dễ bị trẻ em am hiểu công nghệ qua mặt** (bằng cách đổi DNS, dùng VPN, fake proxy, đổi tên file game, tắt tiến trình qua Task Manager).

---

## 2. Triết Lý Cốt Lõi: Bảo Vệ Riêng Tư & An Toàn Tuyệt Đối

FamShield hoạt động dựa trên các nguyên tắc thiết kế đạo đức (Ethics & Privacy by Design):

- 🔒 **100% Offline-First (Không phụ thuộc Cloud):** Ở chế độ mặc định, toàn bộ dữ liệu cấu hình, lịch sử truy cập và bằng chứng vi phạm được lưu trữ cục bộ trong cơ sở dữ liệu SQLite mã hóa trên máy tính. Không có bất kỳ thông tin nào bị âm thầm gửi lên mạng.
- 🚫 **Nói Không Với Giám Sát Xâm Phạm:** 
  - **Không Phải Keylogger:** Hệ thống quét từ khóa chỉ sử dụng bộ đệm trượt ngắn trong RAM (tối đa 256 ký tự) và **tuyệt đối không bao giờ ghi chuỗi phím thô xuống ổ đĩa**.
  - **Bảo Vệ Mật Khẩu Tuyệt Đối:** Cơ chế tự động nhận diện và **bỏ qua 100% khi người dùng gõ vào ô mật khẩu** (`ES_PASSWORD` / Password Input), đảm bảo không bao giờ chạm đến mật khẩu cá nhân hay tài khoản ngân hàng.
  - **Không** thu âm micro hoặc mở webcam.
  - **Không** can thiệp bẻ khóa chứng chỉ HTTPS (TLS Interception / MITM).
  - **Không** quay màn hình liên tục hoặc livestream màn hình của trẻ.
- 🖼️ **Bằng Chứng Trực Quan Mờ Bảo Vệ (Smart Blurred Evidence):** Ảnh chụp màn hình chỉ được kích hoạt khi có sự kiện vi phạm hoặc khi trẻ chủ động xin gia hạn thời gian. Trước khi lưu vào đĩa, ảnh lập tức được hạ độ phân giải (tối đa 480p) và xử lý làm mờ chữ (Gaussian Blur) trực tiếp trong RAM nhằm che giấu hoàn toàn tin nhắn cá nhân, tài khoản, mật khẩu. Ảnh tự động tiêu hủy vĩnh viễn sau **48 giờ**.
- 📢 **Minh Bạch Trong Phiên Hoạt Động:** Khi có chụp ảnh xác minh, hệ thống gửi thông báo rõ ràng trên màn hình của trẻ, nghiêm cấm hoàn toàn hành vi chụp lén.

---

## 3. Tổng Hợp Chi Tiết Tất Cả Tính Năng (Feature Catalog)

### 3.1. Hai Chế Độ Hoạt Động (Dual-Mode Operation)

FamShield hỗ trợ chuyển đổi linh hoạt giữa 2 chế độ mà không làm mất dữ liệu:

| Đặc Điểm | Chế Độ Ngoại Tuyến (`StandaloneFree`) | Chế Độ Đám Mây (`OnlineManaged`) |
| :--- | :--- | :--- |
| **Mục đích** | Quản lý độc lập ngay trên máy tính của gia đình | Quản lý tập trung từ xa qua điện thoại / Web Console |
| **Chi phí** | **Miễn phí 100% trọn đời**, không quảng cáo | Tùy chọn nâng cao khi kết nối máy chủ |
| **Kết nối mạng** | Hoạt động hoàn toàn không cần Internet | Đồng bộ chính sách 2 chiều khi có mạng |
| **Xác thực** | Master PIN cục bộ + 2FA TOTP (Google Authenticator) | Đăng nhập tài khoản phụ huynh trên Cloud |
| **Điều khiển từ xa** | Không hỗ trợ | Khóa máy khẩn cấp, cấp giờ tạm thời qua HMAC-SHA256 |
| **Chống mất mạng** | Vẫn hoạt động bảo vệ 100% | Vẫn duy trì bộ luật cuối cùng khi mất kết nối mạng |

### 3.2. Kiểm Soát Mạng & Lọc Tên Miền DNS (DNS & WFP Engine)

- **Proxy DNS Cục Bộ Tốc Độ Cao:** Chạy dịch vụ DNS nội bộ tại `127.0.0.2:53` với kiến trúc bộ đệm phân vùng đa luồng (NUMA-aware Sharded Cache, Lock Striping) phản hồi truy vấn dưới $1\text{ ms}$.
- **Multi-Upstream DNS Racing Engine:** Tích hợp bộ giải quyết tên miền song song (Cloudflare `1.1.1.1`, Google `8.8.8.8`, Quad9 `9.9.9.9`, OpenDNS) chọn phản hồi nhanh nhất và tự động chuyển hướng khi mất mạng.
- **Bộ Lọc Phân Cấp Danh Mục (Category Filtering):**
  - Chặn danh mục web độc hại: Khiêu dâm / Người lớn (`Adult`), Cờ bạc cá độ (`Gambling`), Bạo lực / Vũ khí (`Violence`), Mạng xã hội (`SocialMedia`), Trò chơi điện tử (`Gaming`), Dịch vụ Streaming (`Media`).
  - Danh sách tên miền ngoại lệ Cho phép (`Allowlist`) và Cấm (`Blocklist`) tùy chỉnh.
- **Bắt Buộc Tìm Kiếm An Toàn (Enforced SafeSearch):** Tự động điều hướng và ép buộc chế độ SafeSearch ở cấp DNS cho: **Google**, **Bing**, **DuckDuckGo**, **YouTube** (Restricted Mode) và **Yandex**, ngăn chặn kết quả tìm kiếm hình ảnh/video khiêu dâm.
- **Chống Vượt Rào Cấp Kernel (WFP User-Mode Integration):** Sử dụng trực tiếp API Windows Filtering Platform (`fwpmu.dll`) mà không cần nạp driver bên thứ ba (`.sys`):
  - **Chặn cổng UDP/TCP 53 ra ngoài:** Ngăn chặn việc trẻ tự ý đổi DNS thủ công trong Windows sang 8.8.8.8 hay 1.1.1.1.
  - **Chặn VPN & DoH Bypasses:** Chặn các dải IP và cổng của các dịch vụ DNS-over-HTTPS phổ biến và phần mềm VPN cố tình lách bộ lọc.
  - **Cờ Dynamic Session (`FWPM_SESSION_FLAG_DYNAMIC`):** Bộ lọc tự động hủy an toàn nếu dịch vụ tắt, đảm bảo **100% không bao giờ gây lỗi màn hình xanh (BSOD)** hoặc kẹt mạng máy tính.

### 3.3. Giám Sát & Quản Lý Ứng Dụng Thông Minh (App & Time Control)

- **Đo Lường Thời Gian Thực Hoạt Động (Active Foreground Time):**
  - Khác với các phần mềm thông thường chỉ đếm thời gian tiến trình chạy ngầm, FamShield **chỉ tính thời gian khi ứng dụng thực sự được mở trên màn hình phía trước (Foreground Window), màn hình đang mở khóa và trẻ đang tương tác**.
  - Không tính giờ khi máy tính chuyển sang màn hình chờ (Screen Saver / Sleep / Idle) hoặc khi ứng dụng thu nhỏ xuống Taskbar.
- **Tách Biệt Launcher & Game:** Nhận diện và theo dõi quan hệ cha - con giữa các trình khởi chạy (như Steam, Epic Games, Riot Client, Battle.net) và tiến trình game thực sự. Launcher chạy ngầm không bị tính vào thời gian chơi game.
- **Hạn Mức Thời Gian Sử Dụng (App Limits):**
  - Đặt giới hạn số phút tối đa cho từng ứng dụng hoặc nhóm ứng dụng mỗi ngày (ví dụ: `RobloxPlayerBeta.exe` $\le 60$ phút/ngày).
  - Khi hết hạn mức: Tự động cảnh báo trước 5 phút $\to$ Chạm ngưỡng sẽ kích hoạt `TerminateProcess` ngay lập tức $\to$ Nếu trẻ cố mở lại, tiến trình bị chặn đóng trong $0.01\text{s}$ trước khi kịp hiển thị cửa sổ.
- **Thời Khóa Biểu & Khung Giờ Giới Nghiêm (Schedules):**
  - Định nghĩa khung giờ học tập và nghỉ ngơi theo từng ngày trong tuần (ví dụ: Thứ 2 – Thứ 6 chỉ được dùng máy từ 19:00 đến 21:00; Thứ 7 – Chủ Nhật mở rộng từ 08:00 đến 21:30).
  - Tùy chọn khóa toàn bộ máy hoặc chỉ khóa nhóm ứng dụng giải trí trong giờ học tập.
- **Cho Phép Tạm Thời (Temporary Allowance):**
  - Phụ huynh có thể cấp thêm nhanh 15 phút, 30 phút, 1 giờ hoặc 2 giờ cho con trực tiếp từ Web Admin mà không cần sửa cấu hình gốc.

### 3.4. Giám Sát & Lọc Từ Khóa Xấu An Toàn (Keystroke Keyword Filter)

FamShield trang bị hệ thống phát hiện từ khóa xấu trong thời gian thực nhằm bảo vệ trẻ em trước các nguy cơ bạo lực học đường, tự hại, và tiếp cận nội dung đồi trụy:

- **Hook Bàn Phím Siêu Tốc Trong Phiên Người Dùng (`famshield-session`):**
  - Sử dụng Win32 Low-Level Keyboard Hook (`WH_KEYBOARD_LL`) được tối ưu hóa tối đa, hoàn tất kiểm tra trong $< 1\mu\text{s}$, tuyệt đối không gây trễ phím hoặc ảnh hưởng đến trải nghiệm gõ văn bản/chơi game của trẻ.
  - Sử dụng cấu trúc **Bộ Đệm Trượt Trong RAM (Keystroke Sliding Buffer)** tối đa 256 ký tự. **Không bao giờ lưu file log phím thô xuống ổ cứng** để đảm bảo tính riêng tư tuyệt đối.
  - **Tự động nhận diện ô mật khẩu:** Khi con đăng nhập tài khoản (Email, Facebook, Game, Ngân hàng), hệ thống nhận diện cờ `ES_PASSWORD` và Windows UI Automation `UIA_IsPasswordPropertyId` để **tự động ngắt quét phím 100%**, không bao giờ chạm đến mật khẩu.
- **Thuật Toán Aho-Corasick Đa Mẫu Kết Hợp Chuẩn Hóa Tiếng Việt:**
  - **Khử dấu tiếng Việt tự động:** Nhận diện cả khi gõ có dấu lẫn không dấu (`đ` $\to$ `d`, `á/à/ả/ã/ạ/ă/â` $\to$ `a`, v.v.).
  - **Chống thủ thuật lách luật (Anti-Obfuscation):**
    - Tự động bóc tách các ký tự nối cố tình chèn vào giữa chữ: `d.m`, `d_m`, `v*l`, `f*ck`, `d-m-m` đều được nhận diện chính xác thành từ gốc.
    - Giải mã ký tự số thay chữ (Leetspeak): `0` $\to$ `o`, `1` $\to$ `i`, `3` $\to$ `e`, `4` $\to$ `a`, `5` $\to$ `s`, `7` $\to$ `t`.
    - Thu gọn khoảng cách ngắt quãng: `v c l` $\to$ `vcl`, `d j t` $\to$ `djt`.
- **5 Danh Mục Từ Khóa Giám Sát Cốt Lõi:**
  1. 🤬 **Chửi bậy / Thô tục (`Profanity`):** Các từ ngữ văng tục, xúc phạm phổ biến bằng cả Tiếng Việt và Tiếng Anh (`dit`, `djt`, `lon`, `cac`, `vcl`, `deo`, `buoi`, `cmn`, `dm`, `dmm`, `fuck`, `bitch`, `shit`...).
  2. 👊 **Bắt nạt mạng / Đe dọa (`Bullying`):** Ngôn từ thù ghét, đe dọa vũ lực (`may chet di`, `danh chet`, `dot nha`, `dap nat`, `tao se giet`...).
  3. 🆘 **Tự hại / Nguy hiểm (`SelfHarm`):** Dấu hiệu trầm cảm nguy hiểm, tìm kiếm cách thức tự tử (`tu tu`, `tu sat`, `nhay cau`, `cat co tay`, `uong thuoc ngu`, `muon chet`, `khong muon song`...).
  4. 🔞 **Người lớn / Khiêu dâm (`Adult`):** Từ khóa tìm kiếm nhạy cảm trên mạng xã hội, thanh địa chỉ hay ứng dụng chat (`sex`, `xxx`, `porn`, `nsfw`, `nude`, `hentai`, `jav`, `thu dam`, `gai goi`, `loan luan`, `lo clip`...).
  5. ⚙️ **Tùy chỉnh phụ huynh (`Custom`):** Cho phép phụ huynh tự thêm bất kỳ từ khóa hoặc cụm từ cụ thể nào muốn quản lý riêng.
- **4 Cấp Độ Nghiêm Trọng (Severity Levels):** `Low` (Nhẹ), `Medium` (Trung bình), `High` (Cao), `Critical` (Nghiêm trọng).
- **Phản Ứng Tức Thì Khi Phát Hiện:**
  - **Thông báo nhắc nhở văn hóa giao tiếp (Windows Toast Notification):** Hiển thị lời nhắc nhẹ nhàng trên màn hình của con: *"Vui lòng sử dụng ngôn từ lịch sự và tôn trọng người khác"*.
  - **Kích hoạt Bằng chứng mờ (Smart Visual Verification):** Chụp ảnh ngữ cảnh làm mờ nếu vi phạm ở cấp độ cao hoặc nguy hiểm.
  - **Ghi nhận sự kiện ẩn danh:** Chỉ lưu tên ứng dụng đang gõ (ví dụ: Google Chrome, Discord, Messenger), từ khóa bị bắt và đoạn ngữ cảnh đã che giấu (`***`), không làm lộ toàn bộ cuộc trò chuyện riêng tư của trẻ.

### 3.5. Bằng Chứng Trực Quan Tôn Trọng Riêng Tư (Smart Visual Verification)

- **Kích Hoạt Theo Sự Kiện (Event-Triggered):** Chỉ chụp đúng 1 khung hình trong 2 trường hợp duy nhất:
  1. Khi một quy tắc vi phạm nghiêm trọng bị kích hoạt (cố tình truy cập trang web cấm, mở game trong giờ giới nghiêm, hoặc phát hiện từ khóa nguy hiểm).
  2. Khi trẻ gửi yêu cầu xin thêm thời gian từ máy tính.
- **Thu Nhỏ & Làm Mờ Che Giấu (Gaussian Blur & Downscaling):** Ảnh chụp được hạ độ phân giải xuống chuẩn 480p và làm mờ các vùng ký tự văn bản trực tiếp trong RAM máy tính. Phụ huynh chỉ nhìn thấy bố cục giao diện tổng quan để xác định trẻ đang học hay đang chơi, không đọc được nội dung chat hay thông tin đăng nhập.
- **Tự Động Tiêu Hủy (48-Hour Retention):** Sau 48 giờ kể từ khi tạo, các file ảnh bằng chứng sẽ tự động được xóa vĩnh viễn khỏi ổ cứng máy tính.

### 3.6. Cơ Chế Chống Can Thiệp & Bảo Vệ Hệ Thống (Anti-Tampering & Security)

- **Đặc Quyền Windows Service (`LocalSystem` Session 0):** Tiến trình bảo vệ cốt lõi chạy dưới dịch vụ hệ thống `FamShieldService`, tách biệt khỏi phiên tương tác của người dùng. Trẻ em dù có tài khoản Administrator thông thường cũng không thể dùng Task Manager để tắt hoặc xóa file.
- **Chế Độ Ẩn Tàng Hình (Stealth Mode):** Ứng dụng tự động ẩn khỏi danh sách *Windows Control Panel* và *Settings > Apps & Features*. Trẻ không thể bấm gỡ cài đặt từ giao diện Windows thông thường.
- **Mã Hóa Bí Mật Bằng Windows DPAPI:** Mã PIN quản trị, mật khẩu và token mã hóa được bảo vệ thông qua *Data Protection API (DPAPI)* cấp độ máy trạm của Windows. Ngay cả khi trích xuất được cơ sở dữ liệu, không ai có thể giải mã được mật khẩu.
- **Xác Thực 2 Lớp (2FA / TOTP) Bằng Điện Thoại:** Hỗ trợ chuẩn RFC 6238 tương thích hoàn toàn với Google Authenticator, Microsoft Authenticator. Kể cả khi con nhìn trộm được mã Master PIN 6 số khi phụ huynh gõ, con vẫn không thể đăng nhập nếu không có điện thoại của bố mẹ.
- **Mã Khôi Phục Khẩn Cấp (Recovery Key):** Cung cấp chuỗi khóa mã hóa dự phòng `FMSH-XXXX-XXXX-XXXX` giúp phụ huynh lấy lại quyền quản trị khi quên Master PIN.

### 3.7. Bảng Điều Khiển Cục Bộ (Local Web Admin Console)

- Tích hợp sẵn máy chủ Web Admin siêu nhẹ tại cổng cục bộ mặc định: `http://127.0.0.1:38123`.
- Giao diện thân thiện, hiện đại, hỗ trợ Tiếng Việt và Tiếng Anh hoàn chỉnh.
- Bảo mật bằng phiên đăng nhập mã hóa, tự động khóa sau khoảng thời gian không thao tác.

---

## 4. Yêu Cầu Hệ Thống & Chuẩn Bị

| Thành Phần | Yêu Cầu Tối Thiểu | Khuyên Dùng |
| :--- | :--- | :--- |
| **Hệ điều hành** | Windows 10 (bản 1809 trở lên) 64-bit | Windows 10 / Windows 11 (64-bit mới nhất) |
| **Quyền tài khoản** | Quyền Quản trị viên (**Administrator**) khi cài | Quyền Administrator khi cài đặt |
| **Tài khoản của con** | Tài khoản Standard User trên Windows | Tài khoản Standard User (ngăn chỉnh sửa hệ thống) |
| **Bộ nhớ RAM** | $2\text{ GB}$ (Agent chỉ chiếm $\approx 30\text{ MB}$) | $4\text{ GB}$ trở lên |
| **Ổ cứng trống** | $150\text{ MB}$ tại ổ đĩa cài đặt hệ điều hành ($C:\$) | $500\text{ MB}$ trống |
| **Trình duyệt Web** | Microsoft Edge, Google Chrome, Cốc Cốc, Firefox | Edge hoặc Chrome phiên bản mới nhất |

---

## 5. Hướng Dẫn Cài Đặt Từng Bước (Installation Guide)

### 5.1. Cài đặt bằng bộ cài chuẩn (.exe)

Đây là phương thức cài đặt nhanh chóng và đơn giản nhất dành cho phụ huynh.

1. **Tải bộ cài đặt:** Tải file `FamShieldSetup-1.0.2.exe` chính thức.
2. **Chạy với quyền Administrator:** 
   - Nhấp chuột phải vào file `FamShieldSetup-1.0.2.exe`.
   - Chọn **"Run as administrator"** (Chạy với quyền quản trị viên).
   - Khi hộp thoại UAC (User Account Control) xuất hiện, nhấn **Yes**.
3. **Thực hiện cài đặt:**
   - Đọc thỏa thuận bảo vệ trẻ em và nhấn **Next**.
   - Thư mục cài đặt mặc định: `C:\Program Files\FamShield` (Khuyên dùng).
   - Nhấn **Install**. Quá trình cài đặt tự động diễn ra trong 5 - 10 giây.
4. **Kết quả cài đặt tự động:**
   - Dịch vụ `FamShieldService` được đăng ký và khởi chạy ngầm.
   - DNS mạng tự động trỏ về `127.0.0.2:53`.
   - Trình duyệt tự động mở trang thiết lập: `http://127.0.0.1:38123`.

### 5.2. Cài đặt từ mã nguồn dành cho lập trình viên (Developer Setup)

Nếu bạn là lập trình viên muốn tự biên dịch và kiểm thử:

1. Yêu cầu cài đặt: Rust toolchain (`cargo`), Node.js (v18+), và Inno Setup 6 (nếu muốn đóng gói `.exe`).
2. Mở PowerShell với quyền Administrator tại thư mục repository:
   ```powershell
   # Biên dịch frontend và toàn bộ crates backend
   .\build.ps1 build

   # Cài đặt dịch vụ trực tiếp vào Windows
   .\build.ps1 install

   # Kiểm tra trạng thái dịch vụ và cổng Web Admin
   .\build.ps1 status
   ```
3. Đóng gói bộ cài Inno Setup thành file `.exe`:
   ```powershell
   .\build.ps1 package
   ```
   File cài đặt đầu ra sẽ nằm tại: `installer\Output\FamShieldSetup-1.0.2.exe`.

---

## 6. Hướng Dẫn Cấu Hình Lần Đầu (Initial Setup Wizard)

Khi mở Web Admin lần đầu tiên tại `http://127.0.0.1:38123`, giao diện **Trình Hướng Dẫn Thiết Lập (Setup Wizard)** gồm 5 bước sẽ dẫn dắt bạn:

```
[ BƯỚC 1: CHÀO MỪNG ] 
       │
       ▼
[ BƯỚC 2: TẠO MASTER PIN ] ──────> (Nhập 6 số bí mật, vd: 839201)
       │
       ▼
[ BƯỚC 3: CHỌN MẪU BẢO VỆ ] ────> (Cân Bằng / Nghiêm Ngặt / Thoải Mái)
       │
       ▼
[ BƯỚC 4: CHỌN CHẾ ĐỘ ] ─────────> (Ngoại Tuyến Độc Lập / Đám Mây)
       │
       ▼
[ BƯỚC 5: LƯU MÃ KHÔI PHỤC ] ────> [ HOÀN TẤT & VÀO TRANG CHỦ ]
```

### Bước 1: Chào Mừng & Giới Thiệu
Xem nhanh tóm tắt các tính năng bảo vệ và cam kết quyền riêng tư của FamShield. Nhấn **"Bắt Đầu Thiết Lập"**.

### Bước 2: Thiết Lập Mã Master PIN
- Nhập **6 chữ số bí mật** của riêng bạn (ví dụ: `839201`).
- Nhập lại vào ô xác nhận.
- *Lưu ý:* Tuyệt đối không dùng ngày tháng năm sinh của con hoặc các dãy số dễ đoán (`123456`, `000000`).

### Bước 3: Chọn Mẫu Cấu Hình Bảo Vệ Định Sẵn (Preset Profile)
Hệ thống chuẩn bị sẵn 3 cấu hình phù hợp với độ tuổi:
- 🟢 **Cân Bằng (Balanced - Khuyên Dùng):**
  - Chặn triệt để web khiêu dâm, cờ bạc, bạo lực, lừa đảo.
  - Ép buộc SafeSearch trên Google, Bing, YouTube.
  - Giới hạn giờ chơi game phổ biến tối đa 60 phút/ngày.
- 🔴 **Nghiêm Ngặt (Strict):**
  - Chặn toàn bộ web độc hại và chặn thêm Mạng xã hội (TikTok, Facebook, Instagram).
  - Giới hạn giờ game dưới 45 phút/ngày và áp dụng giờ giới nghiêm buổi tối từ 21:00.
- 🟡 **Thoải Mái (Relaxed):**
  - Chỉ chặn web người lớn và cờ bạc nguy hiểm.
  - Không giới hạn giờ chơi game, chỉ ghi nhật ký để phụ huynh tiện theo dõi.

### Bước 4: Chọn Chế Độ Hoạt Động (Operating Mode)
- **Ngoại Tuyến Độc Lập (`StandaloneFree` - Mặc định):** Quản lý hoàn toàn trên máy tính này, không cần tài khoản, miễn phí 100%.
- **Quản Lý Đám Mây (`OnlineManaged`):** Nhập mã ghép nối 6 ký tự từ ứng dụng di động nếu bạn muốn quản lý từ xa.

### Bước 5: Lưu Khóa Khôi Phục Khẩn Cấp (Recovery Key)
- Màn hình hiển thị mã khôi phục dạng: `FMSH-ABCD-EFGH-1234`.
- Nhấn **"Sao Chép Mã Khôi Phục"** và cất vào nơi an toàn (ghi chú trên điện thoại hoặc sổ tay). Khóa này giúp bạn thiết lập lại PIN nếu chẳng may quên mật khẩu.
- Nhấn **"Hoàn Tất & Khởi Chạy Bảo Vệ"**.

---

## 7. Hướng Dẫn Sử Dụng Các Tính Năng Hàng Ngày

Sau khi hoàn tất cấu hình ban đầu, bạn có thể truy cập trang quản trị bất cứ lúc nào qua địa chỉ:  
👉 **`http://127.0.0.1:38123`**

### 7.1. Bảng điều khiển tổng quan (Dashboard)
- **Thẻ trạng thái bảo vệ:** Hiển thị trạng thái hoạt động của Dịch vụ (`FamShieldService: Running`), Bộ lọc WFP (`Active`), Bộ phân giải DNS nội bộ (`Resolving`), và Trợ lý phiên làm việc (`Session Agent: Connected`).
- **Thống kê trong ngày:** Tổng thời gian con đã mở máy tính hôm nay, tổng thời gian chơi game, số lượng tên miền độc hại đã chặn.
- **Thông báo nhắc nhở 2FA:** Cảnh báo nổi bật nếu tài khoản phụ huynh chưa bật xác thực 2 lớp qua điện thoại.

### 7.2. Quản lý Quy tắc Chặn & Lọc Web (Rules Management)
Truy cập tab **🛡️ Quy Tắc (Rules)** trên thanh điều hướng:
1. **Bật/Tắt Danh Mục Cấm (Category Blocking):**
   - Đánh dấu tick vào các danh mục muốn chặn: `Khiêu dâm`, `Cờ bạc`, `Mạng xã hội`, `Game online`.
   - Nhấn **"Lưu Thay Đổi"**. Thay đổi có hiệu lực ngay lập tức trong $0.1\text{s}$ ở cấp DNS.
2. **Thêm Tên Miền Cấm Cá Nhân (Custom Blocklist):**
   - Nhập tên miền muốn cấm (ví dụ: `gamevui.vn`, `roblox.com`).
   - Nhấn **"Thêm Vào Danh Sách Chặn"**.
3. **Thêm Tên Miền Ngoại Lệ (Allowlist):**
   - Nếu có trang web phục vụ việc học tập của con bị hiểu nhầm, nhập tên miền vào mục Cho phép (ví dụ: `hocmai.vn`, `violet.vn`). Danh sách này luôn được ưu tiên cao nhất.
4. **Bật Tìm Kiếm An Toàn (SafeSearch):**
   - Bật công tắc **"Bắt buộc SafeSearch cho Google, Bing, YouTube"**. Khi con tìm kiếm từ khóa nhạy cảm, công cụ tìm kiếm sẽ tự động ẩn toàn bộ ảnh/video người lớn.

### 7.3. Cài đặt Hạn mức Thời gian chơi Game & Ứng dụng (App Limits)
Cũng tại tab **🛡️ Quy Tắc**, cuộn xuống mục **"Hạn Mức Thời Gian Ứng Dụng"**:
1. Nhập tên file thực thi của game/ứng dụng (ví dụ: `RobloxPlayerBeta.exe`, `League of Legends.exe`, `GenshinImpact.exe`, `Minecraft.exe`).
2. Nhập số phút tối đa cho phép mỗi ngày (ví dụ: `60` phút).
3. Nhấn **"Thêm Hạn Mức"**.
- *Cơ chế hoạt động:* Hệ thống chỉ tích lũy thời gian khi cửa sổ game đang ở trạng thái hoạt động trên màn hình (Foreground). Khi chơi đến phút thứ 55, màn hình sẽ hiện thông báo nhắc nhở còn 5 phút. Khi chạm ngưỡng 60 phút, game sẽ bị đóng ngay lập tức.

### 7.4. Thiết lập Lịch trình & Khung giờ Giới nghiêm (Schedules)
1. Chọn tab **🛡️ Quy Tắc** $\to$ **"Khung Giờ Sử Dụng Máy Tính"**.
2. Chọn các ngày trong tuần (Thứ 2 đến Chủ Nhật).
3. Đặt khung giờ được phép sử dụng (ví dụ: `19:00 - 21:00`).
4. Chọn hành động khi ngoài khung giờ:
   - **Khóa màn hình máy tính** (Yêu cầu đăng xuất/khóa Windows).
   - **Chặn toàn bộ truy cập Internet**.
   - **Chỉ chặn các ứng dụng thuộc nhóm Giải trí & Game**.

### 7.5. Theo dõi Hoạt động thực tế (Activity Monitoring)
Truy cập tab **📊 Hoạt Động (Activity)**:
- **Biểu đồ thời gian ứng dụng:** Biểu diễn trực quan thời gian con đã dành cho từng nhóm phần mềm (Học tập, Trình duyệt, Game, Đồ họa...).
- **Danh sách tiến trình đã chạy:** Liệt kê các ứng dụng mới được phát hiện trên máy tính, thời điểm mở lần đầu và lần cuối, cùng phân loại tự động của hệ thống.

### 7.6. Giám sát & Quản lý Từ khóa Xấu (Inappropriate Keyword Monitoring)

1. **Xem Dòng Thời Gian Vi Phạm Từ Khóa (Keyword Violations Timeline):**
   - Vào tab **📊 Hoạt Động (Activity)** $\to$ Bấm chọn thẻ con **"⚠️ Từ Khóa Nhạy Cảm (Keywords)"**.
   - Màn hình hiển thị: Tổng số vi phạm phát hiện trong ngày, tên ứng dụng con đã gõ (ví dụ: `chrome.exe`, `Discord.exe`), từ khóa cụ thể bị phát hiện (ví dụ: `"chửi bậy"`, `"bắt nạt"`), thời điểm chính xác và mức độ nghiêm trọng (`Low` / `Medium` / `High` / `Critical`).
   - Ngữ cảnh xung quanh được che chắn bảo vệ quyền riêng tư (`***`), giúp phụ huynh nắm được xu hướng tâm lý của con mà không soi mói toàn bộ tin nhắn riêng tư.
2. **Cấu Hình & Quản Lý Từ Điển Từ Khóa (Keyword Rules):**
   - Vào tab **⚙️ Cài Đặt & Bảo Mật (Security)** $\to$ Cuộn đến mục **"Giám Sát Gõ Từ Khóa Bậy & Nhạy Cảm (Keystroke Filter)"**.
   - **Bật/Tắt Giám Sát Toàn Cục:** Bật hoặc tắt công tắc chính bất cứ lúc nào.
   - **Bật/Tắt Từng Từ Khóa:** Danh sách toàn bộ các từ ngữ giám sát sẵn có trong hệ thống (Tiếng Việt & Tiếng Anh). Phụ huynh có thể bấm công tắc bên cạnh từng từ để cho phép hoặc chặn giám sát từ đó.
   - **Thêm Từ Khóa Tùy Chỉnh Mới:**
     1. Bấm nút **"➕ Thêm Từ Khóa Tùy Chỉnh"**.
     2. Nhập từ hoặc cụm từ con hay dùng cần lưu ý (hệ thống tự động hỗ trợ khử dấu tiếng Việt và chống lách ký tự nối).
     3. Chọn Danh mục: `Chửi bậy`, `Bắt nạt`, `Tự hại`, `Người lớn`, hoặc `Tùy chỉnh`.
     4. Chọn Mức độ: `Thấp`, `Trung bình`, `Cao`, hoặc `Nghiêm trọng`.
     5. Bấm **"Lưu Từ Khóa"**. Quy tắc mới được đồng bộ tức thì vào bộ đệm RAM của Agent.
   - **Xóa Từ Khóa Tùy Chỉnh:** Bấm biểu tượng thùng rác bên cạnh từ khóa do phụ huynh tự tạo.

### 7.7. Tra cứu Nhật ký truy cập & Vi phạm (Logs)
Truy cập tab **📜 Nhật Ký (Logs)**:
- Xem chi tiết từng truy vấn DNS của máy tính (thời gian, tên miền truy vấn, địa chỉ IP phản hồi, kết quả: `CHO PHÉP` hoặc `CHẶN`).
- Bộ lọc thông minh: Cho phép lọc nhanh chỉ xem các sự kiện **Đã bị chặn** để phụ huynh nắm bắt con đang tò mò tìm kiếm điều gì nhằm kịp thời nhắc nhở, giáo dục.

### 7.8. Xem Bằng chứng hình ảnh vi phạm (Visual Evidence)
Truy cập tab **📷 Bằng Chứng (Evidence)**:
- Hiển thị danh sách các ảnh chụp sự kiện vi phạm gần nhất.
- Mỗi bằng chứng bao gồm: Thời gian vi phạm, sự kiện gây ra (ví dụ: *Cố tình mở Roblox ngoài giờ cho phép*, hoặc *Gõ từ khóa nghiêm trọng*), và hình ảnh thu nhỏ đã làm mờ văn bản (Gaussian Blur).
- Phụ huynh có thể nhấn vào để phóng to xem xét, hoặc bấm **Xóa ngay**. Ảnh không xóa sẽ tự động biến mất sau 48 giờ.

---

## 8. Hướng Dẫn Kích Hoạt Xác Thực 2 Lớp (2FA / TOTP)

> 💡 **Khuyên Dùng Bắt Buộc:**  
> Trẻ em có thể đứng sau lưng nhìn trộm Master PIN khi phụ huynh nhập. Kích hoạt 2FA trên điện thoại sẽ loại bỏ 100% nguy cơ này, vì mã xác thực thay đổi liên tục mỗi 30 giây trên điện thoại của bạn.

### Quy trình kích hoạt trong 3 bước:
1. **Mở hộp thoại 2FA:**
   - Trên thanh Dashboard hoặc tab **⚙️ Cài Đặt & Bảo Mật**, bấm nút: **`👉 Kích Hoạt 2FA Ngay`**.
2. **Quét mã QR bằng điện thoại:**
   - Mở ứng dụng xác thực trên điện thoại của bạn (tải miễn phí từ App Store hoặc Google Play: **Google Authenticator**, **Microsoft Authenticator**, hoặc **Authy**).
   - Chọn quét mã QR và hướng camera vào mã QR đang hiển thị trên màn hình máy tính.
3. **Lưu mã khẩn cấp & Hoàn tất:**
   - Màn hình sẽ hiển thị **8 mã dự phòng dùng một lần** (dạng `ABCD-1234`). Nhấn nút **"Sao Chép 8 Mã Dự Phòng"** và lưu lại.
   - Nhập mã 6 chữ số đang hiển thị trên ứng dụng điện thoại vào ô xác thực $\to$ Bấm **"Xác Nhận & Kích Hoạt"**.

Từ thời điểm này, mỗi khi đăng nhập vào Web Admin hoặc thực hiện thao tác nhạy cảm (như đổi luật chặn, gỡ ứng dụng), hệ thống sẽ yêu cầu cả **Master PIN** và **Mã OTP trên điện thoại**.

---

## 9. Hướng Dẫn Kết Nối Đám Mây & Ghép Nối Thiết Bị (Cloud Pairing)

Nếu sau một thời gian sử dụng chế độ Ngoại tuyến, bạn muốn nâng cấp lên chế độ **Quản lý Đám mây (OnlineManaged)** để giám sát con từ xa khi đi làm:

1. Mở ứng dụng di động hoặc Web Console của FamShield trên điện thoại/laptop của bố mẹ.
2. Chọn **"Thêm Thiết Bị Mới"** $\to$ Hệ thống sẽ cấp một **Mã Ghép Nối 6 Ký Tự** (ví dụ: `FS-8291`). Mã này có hiệu lực trong 10 phút.
3. Trên máy tính của con, mở Web Admin `http://127.0.0.1:38123` $\to$ Vào tab **⚙️ Cài Đặt & Bảo Mật**.
4. Tại mục **"Chế Độ Hoạt Động & Đám Mây"**, nhấn **"Ghép Nối Thiết Bị"**.
5. Nhập mã ghép nối 6 ký tự và xác nhận bằng Master PIN.
6. Máy tính sẽ tự động xác thực khóa bí mật qua giao thức HMAC-SHA256, chuyển sang chế độ `OnlineManaged` và bắt đầu đồng bộ báo cáo lên điện thoại của bố mẹ.

*Lưu ý:* Bất kỳ lúc nào, bạn cũng có thể nhấn **"Hủy Kết Nối (Unpair)"** để đưa máy về chế độ Ngoại Tuyến độc lập (`StandaloneFree`) ngay lập tức mà không làm mất cấu hình đã thiết lập.

---

## 10. Hướng Dẫn Gỡ Cài Đặt Sạch Sẽ (Clean-Slate Teardown)

Vì FamShield áp dụng cơ chế tự vệ tàng hình (**Stealth Mode**), ứng dụng sẽ **không xuất hiện trong Control Panel** của Windows để tránh việc trẻ em tò mò tự bấm gỡ.

Để gỡ cài đặt phần mềm một cách an toàn và sạch sẽ tuyệt đối:

### Cách 1: Gỡ từ giao diện Web Admin (Khuyên Dùng Cho Phụ Huynh)
1. Mở Web Admin tại `http://127.0.0.1:38123`.
2. Đăng nhập bằng Master PIN và mã OTP 2FA.
3. Chuyển sang tab **⚙️ Cài Đặt & Bảo Mật**.
4. Cuộn xuống cuối cùng tại mục **"Vùng Nguy Hiểm: Gỡ Cài Đặt Ứng Dụng"**.
5. Bấm nút **"Gỡ Cài Đặt FamShield Khỏi Máy Tính"**.
6. Nhập lại Master PIN để xác nhận lần cuối.
7. Trình gỡ cài đặt sẽ tự động:
   - Dừng toàn bộ dịch vụ ngầm.
   - Trả cấu hình DNS của toàn bộ card mạng về chế độ DHCP tự động ban đầu của nhà mạng.
   - Gỡ bỏ bộ lọc WFP.
   - Xóa bỏ file thực thi và dọn sạch cơ sở dữ liệu.

### Cách 2: Gỡ thủ công bằng file thực thi (Quyền Administrator)
1. Mở thư mục cài đặt mặc định: `C:\Program Files\FamShield`.
2. Tìm file `unins000.exe`.
3. Nhấp chuột phải $\to$ Chọn **"Run as administrator"**.
4. Nhập Master PIN khi được nhắc và làm theo hướng dẫn trên màn hình.

### Cách 3: Gỡ qua dòng lệnh PowerShell (Dành cho Lập trình viên)
Mở PowerShell với quyền Administrator tại thư mục repository:
```powershell
.\build.ps1 uninstall
```

---

## 11. Xử Lý Sự Cố Thường Gặp (FAQ & Troubleshooting)

### Q1: Tôi không thể mở trang `http://127.0.0.1:38123`?
- **Nguyên nhân:** Cổng `38123` có thể đang bị một phần mềm khác trên máy chiếm dụng.
- **Cách khắc phục:** FamShield tự động chọn cổng dự phòng tiếp theo. Hãy mở tệp tin sau để xem số cổng thực tế đang dùng:
  `C:\ProgramData\FamShield\admin_port.txt`  
  Ví dụ nếu file hiển thị `38124`, bạn hãy truy cập trình duyệt bằng địa chỉ `http://127.0.0.1:38124`.

### Q2: Máy tính bị mất mạng Internet sau khi cài đặt hoặc khởi động lại?
- **Nguyên nhân:** Dịch vụ DNS cục bộ `FamShieldService` chưa kịp khởi động xong.
- **Cách khắc phục:** 
  1. Nhấn tổ hợp phím `Windows + R`, gõ `services.msc` và nhấn Enter.
  2. Tìm dịch vụ `FamShieldService`, nhấn chuột phải chọn **Restart**.
  3. Nếu vẫn chưa có mạng, hãy chạy lệnh PowerShell Administrator sau để đưa DNS về DHCP tạm thời:
     ```powershell
     Get-NetAdapter | Set-DnsClientServerAddress -ResetServerAddresses
     ```

### Q3: Tôi quên mã Master PIN thì phải làm sao?
- Khi mở màn hình đăng nhập Web Admin, bấm vào liên kết **"Quên Master PIN?"**.
- Nhập chuỗi **Khóa Khôi Phục (Recovery Key)** mà bạn đã lưu ở Bước 5 khi cài đặt ban đầu (định dạng `FMSH-XXXX-XXXX-XXXX`).
- Hệ thống sẽ cho phép bạn thiết lập một mã Master PIN mới.

### Q4: Trẻ có thể đổi ngày giờ hệ thống để vượt thời gian giới hạn không?
- **Không.** FamShield sử dụng bộ đếm thời gian đơn điệu (Monotonic System Clock) được đo đếm trực tiếp từ thời gian chạy liên tục của phần cứng (Hardware Tick Counter). Trẻ đổi giờ hệ thống lùi lại trong Windows cũng hoàn toàn không ảnh hưởng đến số phút đã tính toán.

### Q5: Trẻ có thể dùng Task Manager để tắt ứng dụng không?
- **Không.** Tiến trình `FamShieldService.exe` chạy với tư cách dịch vụ hệ thống `LocalSystem` trong phân vùng bảo vệ Session 0. Ngay cả tài khoản Quản trị viên khi mở Task Manager bấm *End Task* cũng sẽ bị Windows từ chối truy cập (*Access Denied*).

---

> 🤝 **Cần thêm trợ giúp hoặc đóng góp ý kiến?**  
> Vui lòng tạo issue thảo luận tại: [GitHub Issues](https://github.com/nntu/famshield-agent/issues) hoặc tham khảo tài liệu kỹ thuật chi tiết tại thư mục [docs/](docs/README.md).

---

## 12. Giấy phép

FamShield Agent được phát hành theo giấy phép [MIT](LICENSE). Trừ khi có thông báo giấy phép riêng, giấy phép này áp dụng cho mã nguồn và tài liệu trong repository. Thư viện và tài sản của bên thứ ba vẫn tuân theo giấy phép riêng; giấy phép MIT không cấp quyền sử dụng tên hoặc logo FamShield.
