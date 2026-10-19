# FamShield Windows Endpoint Agent

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
| **`famshield-service`** | Windows Service chạy dưới quyền `LocalSystem` trong Session 0. | **In Progress** 🔄 |
| **`famshield-session`** | Helper chạy trong user session: GDI BitBlt Capture + Gaussian Blur, System Tray, Toast, PIN dialog. | **Implemented** ✅ |
| **`famshield-dns`** | DNS Proxy NUMA loopback (`127.0.0.2:53`), Serve-Stale Cache, ArcSwap Hot-swap Blocklist. | **Implemented** ✅ |
| **`famshield-windows`** | WFP Port 53 enforcement (`FWPM_SESSION_FLAG_DYNAMIC`), NRPT Registry, ToolHelp32 process monitor. | **Implemented** ✅ |
| **`famshield-local-admin`** | Quản trị độc lập: Master PIN Argon2id + DPAPI, lockout lũy thừa, khóa cứu hộ offline. | **Implemented** ✅ |
| **`famshield-storage`** | SQLite nội bộ (chế độ WAL): Durable Outbox, Error Deduplication, Query Logs, Visual Evidence. | **Implemented** ✅ |
| **`famshield-sync`** | Unified Telemetry Envelope, SQLite Outbox pipeline, NATS JetStream 2 chiều, Cloud Enrollment. | **Implemented** ✅ |
| **`famshield-updater`** | Atomic swap nhị phân và Watchdog timer 30s tự động rollback phiên bản nếu gặp sự cố. | **Prototype** 🧪 |

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
Toàn bộ **48 unit tests** đạt trạng thái pass 100% trên 29 test suites (DNS wire parser, WFP filter lifecycle, GDI capture blur, Argon2id lockout, DPAPI secret migration, SQLite Outbox retry backoff, và Telemetry batching).
