# Hướng dẫn sử dụng FamShield Agent

Tài liệu này hướng dẫn cài đặt và tìm các khu vực chính trong Web Admin của FamShield Agent. Tính năng phụ thuộc vào phiên bản agent và trạng thái các dịch vụ Windows. Tài liệu kỹ thuật có thể mô tả yêu cầu hoặc kế hoạch; xem [README](README.md) và [changelog](CHANGELOG.md) để biết thông tin hiện hành.

## Cài đặt và thiết lập lần đầu

1. Tải bộ cài mới nhất từ [GitHub Releases](https://github.com/nntu/famshield-agent/releases) và chạy bằng quyền Administrator.
2. Mở Web Admin tại `http://127.0.0.1:38123`. Nếu cổng mặc định không hoạt động, xem `%ProgramData%\FamShield\admin_port.txt`.
3. Trong trình thiết lập, tạo và xác nhận Master PIN gồm sáu chữ số.
4. Chọn một hồ sơ cấu hình: **Cân Bằng**, **Nghiêm Ngặt**, hoặc **Cơ Bản**. Đọc mô tả hiển thị trong trình thiết lập trước khi chọn.
5. Chọn chế độ vận hành. `StandaloneFree` dùng cấu hình tại máy; `OnlineManaged` cần URL máy chủ và mã ghép nối hợp lệ.
6. Lưu **Khóa Khôi Phục** được cấp sau khi hoàn tất. Khóa này dùng để đặt lại PIN nếu bạn quên.

> FamShield điều chỉnh DNS của Windows để áp dụng quy tắc. Ghi lại cấu hình DNS tĩnh trước khi cài đặt. Khi gỡ cài đặt, DNS adapter được đặt lại về DHCP.

## Các khu vực trong Web Admin

Thanh điều hướng hiện có các khu vực sau:

- **Dashboard**: trạng thái và thông tin tổng quan mà agent cung cấp.
- **Activity**: dữ liệu hoạt động được agent ghi nhận.
- **Rules**: quản lý các quy tắc cục bộ được hỗ trợ trong phiên bản đang chạy.
- **Logs** và **System Logs**: nhật ký sự kiện và vận hành có sẵn trên thiết bị.
- **Evidence**: xem bằng chứng nếu phiên bản và cấu hình hiện tại tạo dữ liệu này.
- **Security**: thiết lập bảo mật và quản trị cục bộ.

Tên mục có thể được bản địa hóa theo ngôn ngữ đang chọn. Nếu một bảng chưa có dữ liệu, điều đó không xác nhận rằng tính năng tương ứng đã hoạt động; hãy xem trạng thái agent và log hệ thống.

## Quản lý PIN và recovery key

Giữ Master PIN và recovery key riêng tư. Không lưu recovery key trong tài liệu hoặc nơi trẻ có thể truy cập. Nếu quên PIN, dùng chức năng khôi phục trên Web Admin và recovery key đã lưu.

## Đổi DNS về DHCP khi gỡ cài đặt

Chạy `unins000.exe` trong thư mục cài đặt bằng quyền Administrator. Với bản cài từ mã nguồn, mở PowerShell Administrator tại repository và chạy:

```powershell
.\build.ps1 uninstall
```

Sau khi gỡ, khôi phục DNS tĩnh đã ghi lại nếu thiết bị trước đó không dùng DHCP. Hướng dẫn cài đặt chi tiết nằm trong [Installation and Initial Setup Guide](docs/17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md).

## Tài liệu tham khảo

- [Chỉ mục tài liệu kỹ thuật](docs/README.md)
- [Hợp đồng API](contracts/openapi.yaml)
- [Schema chính sách](contracts/policy.schema.json)
- [Nhật ký thay đổi](CHANGELOG.md)
