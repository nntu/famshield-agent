# Hướng Dẫn Cài Đặt & Cấu Hình Ban Đầu FamShield (Parent & Setup Guide)

Tài liệu này cung cấp hướng dẫn từng bước chi tiết dành cho phụ huynh và người quản trị khi tải về, cài đặt và thực hiện các thiết lập bảo vệ ban đầu trên máy tính Windows.

---

## 1. Giới Thiệu Chung Về FamShield Agent

**FamShield** là giải pháp bảo vệ không gian mạng và quản lý thời gian sử dụng máy tính toàn diện dành cho gia đình, hoạt động trực tiếp trên hệ điều hành Windows:
- **Hoạt động độc lập & Bảo vệ riêng tư (100% Offline-First)**: Không yêu cầu bắt buộc phải có máy chủ đám mây. Dữ liệu lịch sử duyệt web và từ khóa được bảo vệ hoàn toàn trên máy tính của bạn trong cơ sở dữ liệu SQLite cục bộ mã hóa.
- **Lọc mạng & Chống vượt rào cấp Kernel**: Tự động điều hướng và lọc mọi tên miền truy cập qua proxy nội bộ `127.0.0.2:53` kết hợp bộ lọc Windows Filtering Platform (WFP), chặn đứng các thủ thuật đổi DNS, VPN hay DoH.
- **Bảo vệ tàng hình (Stealth Mode)**: Không hiển thị trong Windows Control Panel hay Settings để ngăn trẻ tự ý gỡ cài đặt.

---

## 2. Yêu Cầu Hệ Thống

- **Hệ điều hành**: Windows 10 (bản 1809 trở lên) hoặc Windows 11 (64-bit).
- **Quyền hạn**: Quyền Quản trị viên (**Administrator**) trên máy tính khi chạy file cài đặt.
- **Dung lượng trống**: Tối thiểu $150\text{ MB}$ ổ đĩa $C:\$.
- **Bộ nhớ RAM**: Tối thiểu $2\text{ GB}$ (tiến trình FamShield chỉ tiêu tốn $\approx 25-40\text{ MB}$ RAM).

---

## 3. Quy Trình Cài Đặt (Step-by-Step Installation)

### Bước 1: Khởi Chạy Bộ Cài Đặt
1. Tải về gói cài đặt chính thức: `FamShieldSetup.exe`.
2. Nhấp chuột phải vào file `FamShieldSetup.exe` $\to$ Chọn **"Run as administrator"** (Chạy với quyền quản trị).
3. Nếu màn hình Windows UAC (User Account Control) xuất hiện, nhấn **Yes** để đồng ý cấp quyền.

### Bước 2: Tiếp Nhận & Chọn Đường Dẫn Cài Đặt
1. Đọc và đồng ý với các điều khoản bảo vệ trẻ em và quyền riêng tư.
2. Thư mục cài đặt mặc định: `C:\Program Files\FamShield` (Khuyên dùng để được bảo vệ bằng quyền hạn của hệ điều hành).
3. Nhấn **Next** $\to$ **Install** để tiến trình tự động diễn ra.

### Bước 3: Hoàn Tất Cài Đặt Tự Động
Trong vài giây, trình cài đặt sẽ tự động hoàn tất các tác vụ ngầm:
- Đăng ký và kích hoạt dịch vụ hệ thống `FamShieldService` chạy ngầm.
- Thiết lập quy tắc điều hướng DNS nội bộ (`127.0.0.2:53`) trên card mạng.
- Kích hoạt trợ lý khay hệ thống `famshield-session.exe`.
- Tự động mở trình duyệt mặc định trỏ về trang thiết lập ban đầu: `http://127.0.0.1:38123`.

---

## 4. Hướng Dẫn Cấu Hình Ban Đầu (Initial Setup Wizard)

Khi mở trang quản trị lần đầu tiên, giao diện **Trình Hướng Dẫn Thiết Lập (Setup Wizard)** sẽ xuất hiện:

```
[ BƯỚC 1: CHÀO MỪNG ] 
       │
       ▼
[ BƯỚC 2: THIẾT LẬP MASTER PIN ] (Nhập mã 6 chữ số bí mật)
       │
       ▼
[ BƯỚC 3: CHỌN MẪU CẤU HÌNH LỌC ] (Cân bằng / Nghiêm ngặt / Thoải mái)
       │
       ▼
[ BƯỚC 4: CHỌN CHẾ ĐỘ HOẠT ĐỘNG ] (Ngoại Tuyến Miễn Phí / Đám Mây)
       │
       ▼
[ BƯỚC 5: LƯU MÃ KHÔI PHỤC ] ──> [ HOÀN TẤT & VÀO BẢNG ĐIỀU KHIỂN ]
```

### Bước 4.1: Đặt Mã Master PIN Quản Trị
- **Mục đích**: Master PIN là chiếc chìa khóa duy nhất để bạn truy cập bảng điều khiển, sửa đổi luật chặn hoặc gỡ phần mềm.
- Nhập **6 chữ số bí mật** của bạn (ví dụ: `198520`).
- Nhập lại một lần nữa để xác nhận.
> ⚠️ **Lưu ý:** Không đặt ngày sinh của con hoặc các dãy số dễ đoán (`123456`, `000000`).

### Bước 4.2: Chọn Mẫu Cấu Hình Lọc Định Sẵn (Preset Profile)
Hệ thống cung cấp sẵn 3 cấp độ bảo vệ để bạn chọn chỉ với 1 chạm:
1. **Cân Bằng (Balanced - Khuyên dùng)**: Chặn toàn bộ web khiêu dâm, cờ bạc, bạo lực, ép SafeSearch và giới hạn thời gian chơi game phổ biến.
2. **Nghiêm Ngặt (Strict)**: Chặn bổ sung mạng xã hội (TikTok, Facebook, Instagram) và giới hạn giờ chơi game dưới 45 phút/ngày.
3. **Thoải Mái (Relaxed)**: Chỉ cảnh báo và ghi nhật ký hoạt động mà không chặn đứng hoàn toàn.

### Bước 4.3: Chọn Chế Độ Hoạt Động (Operating Mode)
- **Ngoại Tuyến Độc Lập (StandaloneFree - Mặc định)**:
  - Hoạt động 100% miễn phí, vĩnh viễn, không cần kết nối mạng đến bất kỳ máy chủ nào.
  - Thích hợp cho gia đình chỉ muốn quản lý trực tiếp trên máy tính.
- **Quản Lý Đám Mây (OnlineManaged)**:
  - Chọn chế độ này nếu bạn muốn nhận cảnh báo và điều khiển máy tính từ xa qua ứng dụng di động của FamShield.
  - Nhập mã ghép nối 6 ký tự được cấp trên ứng dụng điện thoại.

### Bước 4.4: Lưu Trữ Khóa Khôi Phục Khẩn Cấp (Recovery Key)
- Hệ thống sẽ hiển thị một chuỗi **Khóa Khôi Phục** định dạng `FMSH-XXXX-XXXX-XXXX`.
- Bấm **"Sao Chép Mã Khôi Phục"** và cất vào nơi an toàn (ghi vào sổ tay hoặc ứng dụng ghi chú cá nhân). Khóa này sẽ giúp bạn mở lại máy nếu chẳng may quên mã Master PIN.
- Nhấn **"Hoàn Tất Cấu Hình & Vào Bảng Điều Khiển"**.

---

## 5. Kích Hoạt Xác Thực 2 Lớp (2FA / OTP) Bằng Điện Thoại

> 🛡️ **KHUYÊN DÙNG BẮT BUỘC:**  
> Trẻ em ngày nay rất tinh mắt và có thể nhìn trộm Master PIN khi bạn gõ phím. Bật 2FA sẽ triệt tiêu hoàn toàn nguy cơ này bằng mã OTP 6 số chỉ có trên điện thoại của bạn.

Ngay tại màn hình Dashboard hoặc đầu tab **⚙️ Cài Đặt & Bảo Mật**, bạn sẽ thấy thông báo:
> 🔐 **Bảo Vệ Tài Khoản Phụ Huynh: Chưa Kích Hoạt Xác Thực 2 Lớp (2FA / OTP) [KHUYÊN DÙNG]**  
> ➡️ Bấm nút: **`👉 Kích Hoạt 2FA Ngay`**

### Quy trình kích hoạt trong 1 phút:
1. Mở ứng dụng xác thực trên điện thoại của bạn (chọn một trong các ứng dụng: **Google Authenticator**, **Microsoft Authenticator**, hoặc tính năng Password trên iPhone).
2. Quét hình ảnh **Mã QR** hiển thị trên màn hình máy tính.
3. **Lưu trữ 8 mã dự phòng khẩn cấp**: Hộp thoại cung cấp sẵn 8 mã dùng 1 lần (dạng `ABCD-1234`). Hãy bấm nút **"Sao Chép 8 Mã"** để phòng trường hợp điện thoại hết pin hoặc bị hỏng.
4. Nhập 6 chữ số đang hiển thị trên điện thoại vào ô xác nhận và bấm **`Kích Hoạt 2FA`**.

Từ thời điểm này, mọi thao tác đăng nhập bảng điều khiển hay gỡ cài đặt đều yêu cầu nhập mã OTP bảo vệ.

---

## 6. Thiết Lập Giới Hạn Giờ Chơi Game & Ứng Dụng (App Limit)

Để cài đặt định mức thời gian chơi game cho con:
1. Vào tab **🛡️ Quy Tắc** $\to$ Kéo xuống mục **"Hạn Mức Thời Gian Sử Dụng Ứng Dụng"**.
2. Chọn hoặc nhập tên file thực thi của game (Ví dụ: `RobloxPlayerBeta.exe`, `League of Legends.exe`, `GenshinImpact.exe`).
3. Đặt số phút tối đa mỗi ngày (Ví dụ: `60` phút).
4. Bấm **"Thêm Quy Tắc"**.

*Cơ chế tự động cưỡng chế:*
- Khi con chơi chạm ngưỡng 60 phút, hệ thống sẽ tự động **hạ tiến trình game (`TerminateProcess`)** và khóa màn hình máy tính lại. Nếu con cố tình mở lại game, game sẽ bị đóng ngay lập tức trong 0.01 giây trước khi kịp hiển thị.

---

## 7. Hướng Dẫn Gỡ Cài Đặt Ứng Dụng Đúng Cách (Clean-Slate Teardown)

Vì FamShield kích hoạt chế độ bảo vệ tàng hình (Stealth Mode), ứng dụng **không có trong Windows Control Panel**. Để gỡ bỏ ứng dụng khi không còn nhu cầu sử dụng:

1. Mở Bảng điều khiển Web Admin tại `http://127.0.0.1:38123`.
2. Đăng nhập bằng Master PIN và mã OTP 2FA.
3. Vào tab **⚙️ Cài Đặt & Bảo Mật** $\to$ Bấm vào nút chuyển nhanh **`[ 🗑️ Gỡ Cài Đặt ]`** ở thanh trên cùng.
4. Kéo xuống mục **"Vùng Nguy Hiểm: Gỡ Cài Đặt Ứng Dụng"**.
5. Nhập lại Master PIN để xác nhận.
6. Hệ thống sẽ tự động:
   - Dừng toàn bộ dịch vụ ngầm.
   - Hoàn trả 100% cài đặt mạng DNS về DHCP nguyên bản của nhà mạng.
   - Dọn dẹp sạch sẽ toàn bộ cấu hình Registry và file tạm thời.
