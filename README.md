# FamShield Windows Endpoint Agent

[🇻🇳 Tiếng Việt](README.md) | [🇬🇧 English](README.en.md) | [🤝 Hướng Dẫn Đóng Góp (Contributing)](CONTRIBUTING.md)

**FamShield Agent** là thành phần bảo vệ máy trạm Windows thuộc hệ sinh thái **FamShield Parental Control & Endpoint Security**. 

Dự án được xây dựng 100% bằng **Rust hiện đại**, chú trọng an toàn bộ nhớ tuyệt đối, hiệu năng cực cao, không gây BSOD/treo máy, và chạy ngầm siêu nhẹ (< 15MB RAM).

> **Lưu ý:** Phần máy chủ đám mây (Go Control Plane, NATS JetStream, Telegram Bot & Web Console) được đặt tại:  
> 👉 [https://github.com/nntu/famshield-server](https://github.com/nntu/famshield-server)

---

## 1. Cấu Trúc Không Gian Làm Việc (Rust Workspace)

Agent bao gồm 9 crates chuyên biệt hóa theo nguyên lý Idiomatic Rust:

| Crate | Vai Trò Kỹ Thuật | Trạng Thái |
| :--- | :--- | :--- |
| **`famshield-core`** | Domain models, Newtypes ngữ nghĩa, luật lọc, và State Machine. | **Implemented** ✅ |
| **`famshield-service`** | Windows Service SCM Dispatcher (`windows-service`), Named Pipe IPC server có SDDL ACL. | **Implemented** ✅ |
| **`famshield-session`** | Helper chạy trong user session: GDI BitBlt Capture + Gaussian Blur, System Tray, Toast, PIN dialog. | **Implemented** ✅ |
| **`famshield-dns`** | DNS Proxy NUMA loopback (`127.0.0.2:53`), Serve-Stale Cache, ArcSwap Hot-swap Blocklist. | **Implemented** ✅ |
| **`famshield-windows`** | WFP Port 53 enforcement (`FWPM_SESSION_FLAG_DYNAMIC`), NRPT Registry, ETW Kernel 0ms & ToolHelp32. | **Implemented** ✅ |
| **`famshield-local-admin`** | Quản trị độc lập: Master PIN Argon2id + DPAPI, lockout lũy thừa, khóa cứu hộ offline. | **Implemented** ✅ |
| **`famshield-storage`** | SQLite nội bộ (chế độ WAL): Durable Outbox, Error Deduplication, Query Logs, Visual Evidence. | **Implemented** ✅ |
| **`famshield-sync`** | Freemium Engine (`StandaloneFree` vs `OnlineManaged`), Telemetry Envelope, NATS JetStream, HMAC Auth. | **Implemented** ✅ |
| **`famshield-updater`** | Xác thực chữ ký số ECDSA P-256, Atomic swap nhị phân và Watchdog timer 30s rollback tự động. | **Implemented** ✅ |

---

## 2. Tính Năng Kỹ Thuật Nổi Bật

### 🛡️ Windows Filtering Platform (WFP) Port 53 Enforcement
- Tích hợp trực tiếp Windows Filtering Platform API (`fwpmu.dll`) tại `FWPM_LAYER_ALE_AUTH_CONNECT_V4` và `FWPM_LAYER_ALE_AUTH_CONNECT_V6`.
- Quy tắc ưu tiên cao (Weight 15, `FWP_ACTION_PERMIT`): Cho phép toàn bộ dải IP loopback `127.0.0.0/8` và upstream DNS đã cấu hình.
- Quy tắc chặn (Weight 10, `FWP_ACTION_BLOCK`): Chặn triệt để mọi truy vấn DNS trực tiếp ra ngoài máy trạm, ép 100% lưu lượng đi qua bộ lọc FamShield.
- Sử dụng phiên làm việc động (`FWPM_SESSION_FLAG_DYNAMIC`), tự động hoàn nguyên sạch sẽ nếu dịch vụ tắt/crash, triệt tiêu nguy cơ rớt mạng hay BSOD.

### 📷 Smart Visual Verification (Bảo Vệ Quyền Riêng Tư)
- Chụp ảnh màn hình thật khi có sự kiện vi phạm bằng Win32 GDI `BitBlt` từ Desktop DC.
- Tự động downscale về độ phân giải chuẩn 480p và áp dụng thuật toán làm mờ 3-pass separable box blur (hội tụ Gauss trong RAM) với độ phức tạp $O(N)$ để xóa mờ thông tin nhạy cảm.
- Nén JPEG trực tiếp trong bộ nhớ bằng `jpeg-encoder`, dung lượng luôn khống chế dưới 150 KiB.

### 🔐 Bảo Mật Master PIN Đa Lớp (Argon2id + DPAPI + Lockout)
- Băm mật khẩu Argon2id kèm salt ngẫu nhiên từ hệ điều hành (`OsRng`).
- Khóa bí mật được mã hóa ở cấp độ máy trạm qua Windows Data Protection API (DPAPI machine-scope).
- Chống tấn công dò mật khẩu (Brute-force): Đếm số lần nhập sai liên tiếp, tự động khóa đăng nhập theo cấp số nhân (từ 30s đến 15 phút).
- Cơ chế khôi phục 3 lớp: UAC Elevation CLI (`--reset-pin`), Khóa cứu hộ offline `FAM-XXXX-XXXX-XXXX`, và điều khiển từ xa.

### 📊 Bộ Khung Telemetry Hợp Nhất (Unified Telemetry Pipeline)
- Mô hình **Store-and-Forward** offline-first thông qua bảng `outbox` trong SQLite.
- Chuẩn hóa **Telemetry Envelope** gom nhiều loại sự kiện (Heartbeat, DNS Log, Error Log, Process Lifecycle, Security Alert) vào một batch duy nhất.
- Hỗ trợ truyền tải tối ưu qua Cloudflare Tunnel và NATS JetStream, nén dữ liệu và chống trùng lặp (Idempotency).
- Băm fingerprint SHA-256 thông minh để gom cụm lỗi (Error Deduplication), tránh tràn log.

### 💎 Kiến Trúc Freemium (Free Standalone vs Paid Cloud Managed)
FamShield Agent phân tách rõ ràng giữa 2 chế độ hoạt động:
- **Chế độ Miễn Phí Cục Bộ (`StandaloneFree`)**:
  - **100% Miễn phí & Mã nguồn mở**: Hoạt động hoàn toàn độc lập, ngoại tuyến (offline-first), bảo đảm riêng tư tối đa và không bắt buộc kết nối server.
  - Tích hợp trọn vẹn sức mạnh bảo vệ máy trạm: DNS Proxy NUMA loopback, chặn bypass WFP Port 53, Master PIN Argon2id + DPAPI machine-scope, khóa lockout brute-force, chụp màn hình GDI + Gaussian blur trong RAM, theo dõi foreground & AFK idle, cơ chế tự bảo vệ và Micro-UI System Tray / PIN Native Dialog.
- **Chế độ Trả Phí Quản Lý Đám Mây (`OnlineManaged`)**:
  - Mở rộng thông qua mã ghép nối đám mây 6 chữ số (`EnrollmentManager`).
  - Quản lý tập trung qua Web Console & Telegram Bot đa nền tảng.
  - Nhận và thực thi lệnh từ xa tức thì qua NATS JetStream 2 chiều: Khóa máy khẩn cấp (`lock_session`), nạp danh sách đen tức thời (`update_blacklist`), đặt lại PIN từ xa (`reset_pin`), phê duyệt/từ chối cài phần mềm (`approve_install`/`deny_install`).
  - Xác thực chữ ký mã hóa HMAC-SHA256 trên mọi lệnh điều khiển và cơ chế chống tấn công phát lại (Anti-replay window 300 giây).

---

## 3. Hướng Dẫn Biên Dịch & Cài Đặt (Quickstart)

### Yêu cầu:
- Windows 10 (1809+) hoặc Windows 11 (x86_64).
- Rust 1.78+ (`cargo`), Inno Setup 6 (để đóng gói installer).
- Quyền Administrator khi cài đặt dịch vụ.

### Các lệnh điều khiển qua PowerShell (`build.ps1`):
```powershell
# 1. Biên dịch nhị phân Release và cài đặt Windows Service + NRPT Policy
.\build.ps1 install

# 2. Kiểm tra trạng thái hoạt động của Service và Loopback DNS
.\build.ps1 status

# 3. Đặt lại Master PIN với quyền Administrator (UAC Elevation)
.\build.ps1 reset-pin -NewPin "8888"

# 4. Gỡ cài đặt sạch sẽ, trả lại 100% cấu hình mạng Windows ban đầu
.\build.ps1 uninstall
```

### Đóng gói Bộ cài đặt Inno Setup:
File kịch bản tại [`installer/FamShield.iss`](installer/FamShield.iss):
```powershell
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\FamShield.iss
```
File cài đặt đầu ra: `installer\Output\FamShieldSetup-0.1.0.exe`.

---

## 4. Kiểm Thử Mã Nguồn (Build Gate)

```powershell
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
Toàn bộ **57 unit tests** đạt trạng thái pass 100% trên 29 test suites (DNS wire parser, WFP filter lifecycle, GDI capture blur, Argon2id lockout, DPAPI secret migration, SQLite Outbox retry backoff, Telemetry batching, ETW Kernel tracing, Signed Updater ECDSA P-256, và Freemium HMAC command authentication).

---

## 5. Dành Cho Nhà Phát Triển & Đóng Góp (Contributing)

Chúng tôi hoan nghênh mọi đóng góp từ cộng đồng (sửa lỗi, bổ sung tính năng, tối ưu hiệu năng và tài liệu)!

- 📜 **Quy định đóng góp chi tiết**: Vui lòng đọc kỹ [CONTRIBUTING.md](CONTRIBUTING.md) trước khi tạo Pull Request.
- 🚦 **Tiêu chuẩn Pull Request**:
  - Mã nguồn phải định dạng chuẩn `cargo fmt`.
  - Không có cảnh báo clippy (`cargo clippy --workspace --all-targets -- -D warnings`).
  - Toàn bộ unit tests phải pass 100% (`cargo test --workspace`).
  - Viết commit message theo chuẩn [Conventional Commits](https://www.conventionalcommits.org/).
- 🐛 **Báo cáo sự cố hoặc yêu cầu tính năng**: Sử dụng [GitHub Issues](https://github.com/nntu/famshield-agent/issues) theo mẫu sẵn có.
