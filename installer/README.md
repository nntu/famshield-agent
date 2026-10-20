# FamShield Windows Installer & Packaging Guide

Tài liệu này giới thiệu chi tiết về kiến trúc, cơ chế bảo mật và quy trình đóng gói bộ cài đặt **FamShield Windows Agent (`FamShieldSetup.exe`)**.

---

## 1. Tổng Quan Kiến Trúc Bộ Cài Đặt (Installer Architecture)

Bộ cài đặt FamShield được xây dựng dựa trên [Inno Setup 6](https://jrsoftware.org/isinfo.php), được tối ưu hóa đặc biệt dành riêng cho môi trường **Windows 10/11 (64-bit)** với các đặc thù:
- **Chạy với đặc quyền cao nhất**: `PrivilegesRequired=admin` để đăng ký dịch vụ hệ thống và cấu hình mạng cấp kernel.
- **Mô hình 2 tiến trình độc lập**:
  - `famshield-service.exe`: Dịch vụ chạy ngầm vĩnh viễn dưới tài khoản `LocalSystem` trong **Windows Session 0**, tự động khởi động cùng hệ điều hành (`start= auto`).
  - `famshield-session.exe`: Trợ lý tương tác chạy trong **User Session (Session 1+)**, quản lý biểu tượng khay hệ thống (System Tray), thông báo vi phạm và hook bàn phím Win32.
- **Tích hợp giao diện quản trị cục bộ**: Nhúng toàn bộ bundle frontend Vite React (`web/dist`) phục vụ bởi máy chủ HTTP nội bộ `127.0.0.1:38123`.

---

## 2. Các Cơ Chế Bảo Mật & Chống Can Thiệp Cốt Lõi

### 2.1. Chế Độ Tàng Hình (Stealth Mode)
Trẻ em hoặc người dùng máy trạm thường tìm cách vào **Windows Control Panel -> Programs and Features** hoặc **Windows Settings -> Apps & Features** để bấm "Uninstall".
Bộ cài đặt FamShield giải quyết triệt để vấn đề này bằng cấu hình tàng hình:
```ini
CreateUninstallRegKey=no
UpdateUninstallLogAppName=no
```
- **Kết quả**: Ứng dụng **hoàn toàn không xuất hiện** trong danh sách gỡ phần mềm của Windows.
- Ngăn chặn hoàn toàn mọi nỗ lực gỡ trộm bằng các công cụ gỡ phần mềm thông thường của hệ điều hành.

### 2.2. Khóa Gỡ Cài Đặt Bằng Master PIN & Xác Thực 2 Lớp (2FA / OTP)
Để gỡ bỏ FamShield khỏi máy trạm, phụ huynh bắt buộc phải xác thực danh tính:
1. Mở Bảng điều khiển Web Admin (`http://127.0.0.1:38123`) $\to$ Mục **Vùng Nguy Hiểm: Gỡ Cài Đặt**.
2. Nhập chính xác **Master PIN** (xác thực Argon2id).
3. Nếu đã bật 2FA: Bắt buộc nhập thêm **Mã OTP 6 số** từ Authenticator hoặc **Mã Dự Phòng Khẩn Cấp**.

### 2.3. Quy Trình Hoàn Trả Hệ Thống Sạch Sẽ (Clean-Slate Teardown)
Khi lệnh gỡ cài đặt hợp lệ được thực thi, tiến trình uninstaller tự động khôi phục toàn bộ cấu hình máy trạm về nguyên bản 100%:
- **Dừng và giải phóng file**: Hạ tiến trình `famshield-session.exe` và dừng dịch vụ `FamShieldService`.
- **Phục hồi card mạng**: Reset toàn bộ cấu hình DNS của tất cả các adapter mạng về DHCP hoặc DNS ban đầu (`Set-DnsClientServerAddress -ResetServerAddresses`).
- **Dọn dẹp Registry**: Xóa sạch toàn bộ quy tắc NRPT DNS Policy Config và các cờ chặn DoH trên Chrome/Edge.
- **Xóa sạch bộ nhớ đệm**: Thực thi `ipconfig /flushdns` và giải phóng toàn bộ WFP dynamic session filters.

---

## 3. Quy Trình Đóng Gói Bộ Cài Đặt (Build & Packaging)

### Bước 1: Biên Dịch Giao Diện Web Admin (Frontend)
```powershell
rtk npm --prefix web run build
```
*(Kết quả sinh ra thư mục bundle tĩnh tại `web/dist/`).*

### Bước 2: Biên Dịch Các Binary Rust (Backend Release)
```powershell
rtk cargo build --release --workspace
```
*(Kết quả sinh ra `target/release/famshield-service.exe` và `target/release/famshield-session.exe`).*

### Bước 3: (Khuyến nghị Production) Ký Chữ Ký Số (Code Signing)
```powershell
signtool sign /f "path/to/certificate.pfx" /p "password" /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 "target/release/famshield-service.exe"
signtool sign /f "path/to/certificate.pfx" /p "password" /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 "target/release/famshield-session.exe"
```

### Bước 4: Đóng Gói Trình Cài Đặt Qua Inno Setup
Chạy lệnh biên dịch script `installer/FamShield.iss`:
```powershell
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer/FamShield.iss
```
File cài đặt hoàn chỉnh sẽ được tạo tại:
`installer/Output/FamShieldSetup-1.0.0.exe` (hoặc phiên bản tương ứng).

---

## 4. Cấu Trúc Thư Mục Sau Khi Cài Đặt

Khi cài đặt trên máy người dùng, FamShield sẽ cư trú tại 2 vị trí chuẩn của Windows:

```
C:\Program Files\FamShield\
├── famshield-service.exe       # Windows Service chạy ngầm (Session 0)
├── famshield-session.exe       # Trợ lý khay hệ thống (Session 1+)
├── famshield.ico               # Icon ứng dụng
└── web\dist\                   # Bundle giao diện Web Admin nội bộ

C:\ProgramData\FamShield\       # Thư mục dữ liệu SQLite & Logs
├── famshield.db                # Cơ sở dữ liệu SQLite cục bộ (WAL mode)
├── admin_port.txt              # Cổng HTTP động của Local Admin
└── evidence\                   # Ảnh chụp bằng chứng vi phạm (đã Gaussian Blur)
```
