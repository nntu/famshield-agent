# FamShield Windows Endpoint Agent

**FamShield Agent** là thành phần bảo vệ máy trạm Windows thuộc hệ sinh thái **FamShield Parental Control & Endpoint Security**. 

Dự án được xây dựng 100% bằng **Rust hiện đại**, chú trọng an toàn bộ nhớ tuyệt đối, hiệu năng cực cao, không gây BSOD/treo máy, và chạy ngầm siêu nhẹ (< 15MB RAM).

> **Lưu ý:** Phần máy chủ đám mây (Go Control Plane, NATS JetStream, Telegram Bot & Web Console) đã được tách thành repository độc lập tại:  
> 👉 [https://github.com/nntu/famshield-server](https://github.com/nntu/famshield-server)

---

## 1. Cấu Trúc Không Gian Làm Việc (Rust Workspace)

Agent bao gồm 9 crates chuyên biệt hóa theo nguyên lý Idiomatic Rust:

| Crate | Vai Trò Kỹ Thuật |
| :--- | :--- |
| **`famshield-core`** | Domain models, kiểu dữ liệu ngữ nghĩa (Newtypes), luật lọc, và State Machine. |
| **`famshield-service`** | Windows Service chạy dưới quyền `LocalSystem` trong Session 0. |
| **`famshield-session`** | Helper chạy trong phiên người dùng (Session 1+): Khay hệ thống (System Tray), Hộp thoại Master PIN, và Toast Notification thân thiện cho trẻ. |
| **`famshield-dns`** | DNS Proxy NUMA loopback (`127.0.0.2:53`) với bộ nhớ đệm Lock-free & Blocklist Bit-Vector. |
| **`famshield-windows`** | Bộ lọc mạng Windows Filtering Platform (WFP) user-mode (`FWPM_SESSION_FLAG_DYNAMIC`), điều hướng NRPT, và ETW Process Monitor. |
| **`famshield-local-admin`** | Quản trị độc lập không cần Cloud, xác thực Master PIN (Argon2id + Salt) và cơ chế khôi phục qua Khóa cứu hộ / UAC. |
| **`famshield-storage`** | CSDL SQLite nội bộ (chế độ WAL), lưu trữ outbox sự kiện và cache policy. |
| **`famshield-sync`** | Đồng bộ hóa mTLS và trao đổi dữ liệu với Cloud Control Plane. |
| **`famshield-updater`** | Atomic swap nhị phân và Watchdog timer 30s tự động rollback phiên bản nếu gặp sự cố. |

---

## 2. Hướng Dẫn Biên Dịch & Cài Đặt (Quickstart)

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

## 3. Kiểm Thử Mã Nguồn

```powershell
cargo check --workspace
cargo test --workspace
```
Toàn bộ 25 unit tests kiểm thử an toàn DNS, WFP, Argon2id PIN reset, và Session Helper.
