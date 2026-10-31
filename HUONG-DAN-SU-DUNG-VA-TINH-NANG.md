# Hướng dẫn sử dụng FamShield Agent

Tài liệu này giúp phụ huynh thiết lập FamShield, thêm quy tắc và xem dữ liệu trong Web Admin. Tên menu bên dưới dùng giao diện tiếng Việt. Một số chức năng đám mây chỉ hoạt động khi thiết bị kết nối được với máy chủ FamShield tương thích.

## Mở Web Admin

Trên máy Windows đã cài FamShield, mở trình duyệt và truy cập `http://127.0.0.1:38123`. Nếu trang không mở, xem cổng hiện dùng trong `%ProgramData%\FamShield\admin_port.txt`.

FamShield điều chỉnh DNS của Windows để áp dụng quy tắc. Nếu máy đang dùng DNS tĩnh, ghi lại cấu hình trước khi cài đặt hoặc gỡ cài đặt. Trình gỡ cài đặt đặt DNS adapter về DHCP.

## Thiết lập lần đầu

Trình thiết lập Web Admin hướng dẫn bạn:

1. Tạo và xác nhận Master PIN gồm sáu chữ số.
2. Chọn hồ sơ **Cân Bằng**, **Nghiêm Ngặt** hoặc **Cơ Bản**. Đọc phần mô tả trong trình thiết lập để biết quy tắc nào sẽ được áp dụng.
3. Chọn chế độ vận hành. `StandaloneFree` dùng cấu hình tại máy. `OnlineManaged` cần URL máy chủ và mã ghép nối hợp lệ.
4. Hoàn tất thiết lập và lưu **Khóa Khôi Phục** được hiển thị. Cất khóa ở nơi riêng tư, vì khóa này cần thiết để đặt lại PIN nếu bạn quên.

Màn hình đăng nhập hiện không có liên kết tự đặt lại PIN. Nếu quên PIN, liên hệ người quản trị Windows và chuẩn bị Khóa Khôi Phục.

## Các mục trong Web Admin

| Mục | Dùng để |
| --- | --- |
| **Tổng quan** | Xem trạng thái agent, số liệu tóm tắt và truy vấn DNS gần đây. |
| **Hoạt động Giám sát** | Xem dòng thời gian, hoạt động ứng dụng, tên miền được ghi nhận và vi phạm từ khóa. |
| **Quy tắc & Giới hạn** | Thêm tên miền vào danh sách chặn, đặt giới hạn hằng ngày cho ứng dụng và nhập/xuất quy tắc bằng bảng tính. |
| **Nhật ký DNS** | Lọc truy vấn DNS theo thời gian và kết quả; xem tổng hợp theo tên miền hoặc từng truy vấn. |
| **Đọc Logs & Lỗi** | Xem log hệ thống và các lỗi được agent ghi nhận. |
| **Bằng chứng Vi phạm** | Xem ảnh bằng chứng nếu agent đã tạo dữ liệu. Có thể chọn mức làm mờ cho ảnh xem trong giao diện. |
| **Bảo mật & Ghép nối** | Đổi Master PIN, bật xác thực hai lớp, cấu hình một số tùy chọn bảo vệ và quản lý kết nối máy chủ. |

Tên và nội dung trong các mục có thể thay đổi theo phiên bản. Bảng trống chỉ có nghĩa là hiện chưa có dữ liệu phù hợp để hiển thị.

## Thêm quy tắc chặn tên miền

1. Mở **Quy tắc & Giới hạn**.
2. Trong phần **Chặn Tên Miền**, nhập tên miền, ví dụ `example.com`. Không thêm `https://` hoặc đường dẫn trang.
3. Có thể nhập lý do để nhận biết quy tắc sau này.
4. Chọn **Thêm Chặn Tên Miền**.
5. Kiểm tra danh sách. Quy tắc tại chỗ có thể xóa từ bảng; quy tắc do máy chủ quản lý cần được thay đổi trên Cloud Console.

## Đặt giới hạn hằng ngày cho ứng dụng

1. Mở **Quy tắc & Giới hạn** và tìm phần **Giới Hạn Ứng Dụng**.
2. Nhập tên file thực thi của ứng dụng, chẳng hạn `roblox.exe`.
3. Nhập số phút cho phép mỗi ngày.
4. Chọn **Đặt Hạn Mức App** và kiểm tra tên ứng dụng cùng giới hạn trong bảng.

## Nhập hoặc xuất danh sách quy tắc

Trong **Quy tắc & Giới hạn**, bạn có thể tải file mẫu, nhập `.xlsx`, `.xls` hoặc `.csv`, và xuất các quy tắc hiện có. Khi nhập, xem số lượng tên miền và ứng dụng trong màn hình xem trước rồi mới xác nhận. Kiểm tra bảng sau khi nhập để phát hiện mục sai hoặc trùng.

## Xem hoạt động và nhật ký

### Hoạt động Giám sát

Chọn thẻ phù hợp để xem dòng thời gian, hoạt động ứng dụng, hoạt động web theo tên miền hoặc vi phạm từ khóa. Dùng ô tìm kiếm và bộ lọc danh mục để thu hẹp kết quả. Nút làm mới tải lại dữ liệu; có thể bật tự làm mới hoặc xuất báo cáo hoạt động.

Hoạt động web ở đây dựa trên truy vấn DNS. Nhật ký này cho biết tên miền được tra cứu, không phải URL đầy đủ, trang con hoặc nội dung đã xem.

### Nhật ký DNS

1. Chọn **Nhật ký DNS**.
2. Chọn chế độ tổng hợp theo tên miền hoặc danh sách từng truy vấn.
3. Chọn khoảng thời gian: hôm nay, 24 giờ qua, 7 ngày qua, toàn bộ dữ liệu hoặc khoảng tùy chọn.
4. Lọc theo tất cả, bị chặn, được cho phép, được giám sát hoặc lỗi.
5. Dùng **Xuất Nhật Ký DNS (CSV)** nếu cần lưu bản báo cáo.

### Log hệ thống và lỗi

Trong **Đọc Logs & Lỗi**, chọn **Hệ thống** để xem log vận hành hoặc **Lỗi** để xem lỗi agent ghi nhận. Khi cần hỗ trợ, cung cấp thời điểm xảy ra lỗi và nội dung log liên quan.

## Xem bằng chứng và điều chỉnh mức làm mờ

Mở **Bằng chứng Vi phạm**. Nếu có ảnh, chọn mức làm mờ phù hợp trước khi mở ảnh để xem chi tiết. Mức làm mờ điều chỉnh cách ảnh hiển thị trong giao diện; nó không thay thế quy tắc lưu trữ ảnh. Mỗi mục hiển thị thời điểm và thời gian còn lại trước khi hết hạn.

Nếu danh sách trống, agent chưa có bằng chứng để hiển thị hoặc dữ liệu đã hết hạn.

## Bảo vệ tài khoản quản trị

### Đổi Master PIN

Trong **Bảo mật & Ghép nối**, tìm phần **Đổi Mã Master PIN**. Nhập PIN hiện tại, PIN mới và xác nhận PIN mới, rồi chọn **Lưu Mã PIN Mới**. Nếu giao diện tạo Khóa Khôi Phục mới, hãy lưu lại khóa đó.

### Bật xác thực hai lớp

1. Trong **Bảo mật & Ghép nối**, tìm mục xác thực Authenticator và chọn **Kích Hoạt 2FA Authenticator**.
2. Dùng ứng dụng xác thực trên điện thoại để quét mã QR.
3. Nhập mã đang hiển thị để xác nhận.
4. Lưu các mã dự phòng dùng một lần ở nơi an toàn. Mã này giúp bạn đăng nhập khi không dùng được ứng dụng xác thực.

Khi 2FA được bật, một số thao tác quản trị yêu cầu mã OTP hoặc mã dự phòng cùng Master PIN.

### Giám sát từ khóa

Trong **Bảo mật & Ghép nối**, mục **Giám Sát Gõ Từ Khóa Bậy & Nhạy Cảm** cho phép bật hoặc tạm dừng tính năng này, quản lý từ khóa theo dõi và thêm từ khóa tùy chỉnh. Khi thêm, nhập cụm từ, chọn danh mục và mức độ. Kết quả xuất hiện trong **Hoạt động Giám sát** nếu có phát hiện.

Tính năng này quét nội dung gõ trong bộ đệm bộ nhớ để tìm từ khóa đã cấu hình; nó không nhằm lưu lại toàn bộ lịch sử bàn phím.

## Đặt lại báo cáo

Nút **Đặt Lại Báo Cáo** trong phần hoạt động hoặc nhật ký cho phép chọn phạm vi dữ liệu cần xóa: DNS, hoạt động ứng dụng, bằng chứng, từ khóa hoặc toàn bộ báo cáo. Đọc kỹ phạm vi trước khi xác nhận. Thao tác yêu cầu Master PIN và mã OTP nếu 2FA đang bật; dữ liệu báo cáo đã xóa không thể khôi phục từ giao diện này.

## Kết nối máy chủ đám mây và Quản lý từ xa qua Web

FamShield hỗ trợ hai cấp độ giao diện web:
1. **Web Admin Cục Bộ (Local):** Chạy trực tiếp trên máy con tại `http://127.0.0.1:38123`, truy cập bằng Master PIN để cấu hình tại chỗ khi không có mạng.
2. **Cloud Web Console (Đám mây):** Giao diện web quản trị tập trung chạy trên máy chủ FamShield (ví dụ `http://localhost:3000` hoặc tên miền gia đình), cho phép phụ huynh quản lý con từ xa trên điện thoại hoặc máy tính cá nhân.

### Ghép nối Agent vào Cloud Web Console

Để chuyển agent từ chế độ `StandaloneFree` (cục bộ) sang `OnlineManaged` (được đám mây quản lý):

1. **Lấy mã ghép đôi trên Web:** Phụ huynh đăng nhập vào Cloud Web Console, mở mục **Thiết bị bảo vệ**, chọn **Ghép đôi thiết bị mới** để nhận **Mã ghép đôi 6 chữ số** (có hiệu lực 15 phút).
2. **Nhập thông tin trên máy con:** Trong Web Admin cục bộ (`http://127.0.0.1:38123`), vào **Bảo mật & Ghép nối** > **Chuyển chế độ vận hành** (hoặc thực hiện ngay từ bước Thiết lập lần đầu):
   - Chọn chế độ **OnlineManaged**.
   - Nhập **Địa chỉ máy chủ đám mây (Server URL)** (ví dụ `https://famshield.yourdomain.com`).
   - Nhập **Mã ghép đôi** đã nhận từ Web Console.
3. **Xác nhận kết nối:** Nhấn **Kết nối & Đồng bộ**. Khi thành công, agent sẽ nhận Mã tổ chức (`tenant_id`), chứng chỉ bảo mật và tự động tải phiên bản chính sách lọc web mới nhất từ máy chủ.

### Các tính năng quản trị qua Web Đám mây

Khi đã ghép nối thành công, phụ huynh có thể điều khiển và theo dõi máy con hoàn toàn từ xa qua Cloud Web Console:

| Tính năng trên Web Console | Tác vụ trên Agent Windows |
| --- | --- |
| **Phân loại & Lọc Web Đám mây** | Áp dụng tức thì danh mục hệ thống (khiêu dâm, bạo lực, game, mạng xã hội) và danh mục tùy chỉnh xuống bộ lọc WFP và DNS của máy con. |
| **Danh sách Chặn/Cho phép (Blacklist/Whitelist)** | Đẩy quy tắc tên miền toàn diện; quy tắc từ đám mây có độ ưu tiên cao và được đồng bộ tự động. |
| **Khóa / Mở khóa máy từ xa** | Phụ huynh có thể tạm thời khóa máy tính của con ngay trên trình duyệt điện thoại khi đến giờ đi ngủ hoặc học bài. |
| **Đồng bộ chính sách tức thì** | Nút **Đồng bộ tức thì (Trigger Policy Sync)** trên web buộc agent nạp lại quy tắc ngay mà không cần chờ chu kỳ kiểm tra. |
| **Theo dõi Bằng chứng Vi phạm (48h)** | Agent chụp ảnh màn hình vi phạm, tự động làm mờ tại chỗ (Local Blur 480p) và truyền an toàn lên web; ảnh tự hủy sau 48 giờ để bảo vệ quyền riêng tư theo chuẩn COPPA. |
| **Cảnh báo tức thì qua Telegram** | Nhận thông báo qua điện thoại khi máy con gặp sự cố mạng, có hành vi vi phạm nghiêm trọng hoặc khi trẻ xin thêm giờ. |
| **Hồ sơ Trẻ & Hạn ngạch hàng ngày** | Gán máy con vào từng hồ sơ con để theo dõi tiến độ thời gian sử dụng hôm nay (phút đã dùng / hạn mức cho phép). |

Chi tiết toàn bộ các thao tác trên Bảng điều khiển Đám mây được ghi trong [Hướng dẫn sử dụng FamShield Cloud Web Console](../famshield-server/HUONG-DAN-SU-DUNG-VA-TINH-NANG.md).

Nếu không có thông tin máy chủ do tổ chức hoặc gia đình quản lý cung cấp, hãy tiếp tục dùng chế độ `StandaloneFree` (cục bộ).


## Gỡ cài đặt

Mở **Bảo mật & Ghép nối**, tìm vùng gỡ cài đặt và làm theo bước xác nhận bằng Master PIN; nhập OTP nếu được yêu cầu. Có thể gỡ bằng `unins000.exe` trong thư mục cài đặt hoặc bằng `.\build.ps1 uninstall` từ PowerShell Administrator khi cài từ mã nguồn.

Khi gỡ, FamShield đặt DNS adapter về DHCP. Nếu máy trước đó dùng DNS tĩnh, khôi phục cấu hình đã ghi lại sau khi gỡ.

## Nếu Web Admin không mở

1. Kiểm tra `%ProgramData%\FamShield\admin_port.txt` và thử cổng được ghi trong file.
2. Mở **Services** của Windows và kiểm tra dịch vụ `FamShieldService` có đang chạy không.
3. Nếu vừa cài hoặc cập nhật, khởi động lại dịch vụ rồi tải lại trang.
4. Nếu vẫn gặp lỗi, xem **Đọc Logs & Lỗi** hoặc liên hệ người quản trị máy.

Hướng dẫn cài đặt nằm trong [Hướng dẫn cài đặt và thiết lập lần đầu](docs/17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md). Các tài liệu kỹ thuật và hợp đồng API được liệt kê trong [chỉ mục tài liệu](docs/README.md); trạng thái thay đổi theo phiên bản được ghi trong [changelog](CHANGELOG.md).
