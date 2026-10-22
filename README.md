<p align="center">
  <img src="assets/logo.png" alt="FamShield" width="132">
</p>

<h1 align="center">FamShield Agent</h1>

<p align="center">
  Tác tử bảo vệ và quản lý máy tính Windows, được xây dựng bằng Rust.
</p>

<p align="center">
  <a href="README.en.md">English</a> ·
  <a href="CONTRIBUTING.md">Đóng góp</a> ·
  <a href="docs/README.md">Tài liệu kỹ thuật</a> ·
  <a href="https://github.com/nntu/famshield-agent/issues">Báo lỗi / Đề xuất</a>
</p>

FamShield Agent là phần mềm chạy trên máy Windows, cung cấp các công cụ bảo vệ và quản trị dành cho gia đình. Agent hỗ trợ chế độ cục bộ, không cần kết nối dịch vụ đám mây; phần máy chủ quản lý tập trung sẽ được triển khai sau.

## Chọn cách sử dụng

| Chế độ | Phù hợp khi | Kết nối đám mây |
| --- | --- | --- |
| **Cục bộ (`StandaloneFree`)** | Muốn cấu hình và quản lý trực tiếp trên thiết bị. | Không bắt buộc |
| **Quản lý tập trung (`OnlineManaged`)** | Quản lý thiết bị tập trung khi máy chủ được triển khai. | Sẽ triển khai sau |

Cả hai chế độ đều sử dụng agent Windows. Chế độ quản lý tập trung phụ thuộc vào dịch vụ máy chủ sẽ được triển khai trong giai đoạn sau.

## Năng lực chính

- **Kiểm soát DNS và mạng hiệu năng cao:** Proxy DNS cục bộ tích hợp bộ định tuyến DNS đa luồng (Multi-Upstream) với cơ chế tự học RTT (Dynamic Learning DNS Balancer), bộ nhớ đệm NUMA-aware lock-striped cache và Windows Filtering Platform (WFP) chế độ dynamic session ngăn chặn rò rỉ DNS/DoH.
- **Quản lý toàn diện trên thiết bị (Local Web Admin):** Giao diện quản trị web cục bộ chạy trên `127.0.0.1:38123` (hoặc cổng dự phòng), hỗ trợ bảo mật 2 lớp 2FA TOTP (Google/Microsoft Authenticator & mã khẩn cấp), quản lý danh sách quy tắc ứng dụng/tên miền, biểu đồ thống kê thời gian thực, tùy chỉnh độ mờ ảnh chụp màn hình (5 cấp độ) và tính năng xóa an toàn nhật ký báo cáo.
- **Bảo vệ danh tính & bí mật:** Xác thực Master PIN bằng thuật toán Argon2id kết hợp mã hóa bảo vệ Windows DPAPI (machine-scope); cơ sở dữ liệu SQLite cục bộ được quản lý bằng bộ công cụ di chuyển chuyên nghiệp **Refinery 0.10** cùng cơ chế tự động sao lưu trực tiếp (SQLite Online Backup API) trước khi nâng cấp schema.
- **Quản lý ứng dụng và thời gian thực:** Giám sát thời gian sử dụng ứng dụng tiền cảnh (foreground active time), tổng hợp định kỳ và đẩy vào hàng đợi telemetry (`app_usage`).
- **Riêng tư có chủ đích (Privacy-by-Design):** Ảnh xác minh vi phạm chỉ chụp theo sự kiện (chặn web/app, xin thêm giờ), xử lý làm mờ Gaussian Blur trực tiếp trong RAM trước khi lưu vào đĩa hoặc truyền đi, tự hủy sau 48 giờ. Bộ đệm phát hiện từ khóa xử lý an toàn trong RAM, không ghi log bàn phím thô.
- **Đồng bộ đám mây & điều khiển thời gian thực hai kênh:** Chế độ `OnlineManaged` hỗ trợ giao thức Telemetry Protocol v1.1 gửi hàng loạt tối ưu thích ứng (Adaptive Batching), HTTP Connection Pool với TCP keepalive, và kênh nhận lệnh điều khiển từ xa tức thì qua **NATS WebSocket (`wss://`) trên cổng chuẩn 443** (tương thích 100% Cloudflare Tunnel và Firewall doanh nghiệp, kết nối hoàn toàn Outbound không cần mở cổng NAT router).

## Kiến trúc

Workspace Rust được chia thành các crate theo trách nhiệm:

| Crate | Trách nhiệm |
| --- | --- |
| `famshield-core` | Kiểu dữ liệu miền nghiệp vụ, chính sách và trạng thái |
| `famshield-service` | Windows Service, điều phối và IPC |
| `famshield-session` | Thành phần chạy trong phiên người dùng, thông báo và giao diện hệ thống |
| `famshield-dns` | Proxy DNS đa luồng, dynamic RTT learning balancer, NUMA-aware cache và xử lý chính sách tên miền |
| `famshield-windows` | Tích hợp Windows, WFP dynamic session, NRPT và theo dõi tiến trình |
| `famshield-local-admin` | Web Admin cục bộ, bảo mật 2FA TOTP và API quản trị trên thiết bị |
| `famshield-storage` | Cơ sở dữ liệu SQLite, quản lý migration qua Refinery 0.10, snapshot backup guard và hàng đợi outbox |
| `famshield-sync` | Telemetry v1.1, kết nối NATS WebSocket qua Cloudflare Tunnel, HTTP connection pool và xác thực lệnh từ xa |
| `famshield-updater` | Kiểm tra chữ ký và cập nhật agent |

## Bắt đầu trên Windows

### Yêu cầu hệ thống

- Windows 10 (1809 trở lên) hoặc Windows 11, kiến trúc x64.
- Quyền Administrator để cài đặt dịch vụ và cấu hình DNS.

### Cài đặt bằng bộ cài

1. Tải `FamShieldSetup-1.0.2.exe` từ [GitHub Releases](https://github.com/nntu/famshield-agent/releases). Nếu chưa có bản phát hành, xem mục [Tự đóng gói bộ cài](#tự-đóng-gói-bộ-cài) bên dưới.
2. Nhấp phải vào file cài đặt, chọn **Run as administrator**, rồi xác nhận yêu cầu UAC.
3. Làm theo hướng dẫn trên màn hình. Trình cài đặt đăng ký Windows Service, cấu hình DNS cho agent và khởi chạy giao diện thiết lập ban đầu.
4. Tạo Master PIN và hoàn tất cấu hình trong Web Admin.

Web Admin thường mở tại [http://127.0.0.1:38123](http://127.0.0.1:38123). Nếu cổng này không khả dụng, agent sẽ thử các cổng dự phòng; cổng đang dùng được ghi tại `%ProgramData%\FamShield\admin_port.txt`.

> **Lưu ý về mạng:** FamShield thay đổi cấu hình DNS của Windows để áp dụng chính sách. Khi gỡ cài đặt, bộ gỡ cài đặt đặt DNS adapter về DHCP; nếu thiết bị đang dùng DNS tĩnh, hãy ghi lại cấu hình trước khi cài hoặc gỡ.

### Gỡ cài đặt

Chạy `unins000.exe` trong thư mục cài đặt, thường là `C:\Program Files\FamShield`, với quyền Administrator và làm theo xác nhận trên màn hình. Nếu cài từ mã nguồn, có thể gỡ bằng `.\build.ps1 uninstall` trong PowerShell Administrator.

### Cài đặt từ mã nguồn

Cài Rust toolchain, mở PowerShell Administrator tại thư mục repository, sau đó chạy:

```powershell
.\build.ps1 install
.\build.ps1 status
```

### Tự đóng gói bộ cài

Cài Rust, Node.js và Inno Setup 6, sau đó chạy trong PowerShell tại thư mục repository:

```powershell
.\build.ps1 package
```

Bộ cài được tạo tại `installer\Output\FamShieldSetup-1.0.2.exe`.

## Phát triển và kiểm thử

Một số lệnh thường dùng trong repository:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Tài liệu

- [Cẩm nang tính năng & Hướng dẫn sử dụng toàn diện](HUONG-DAN-SU-DUNG-VA-TINH-NANG.md)
- [Hướng dẫn cài đặt & cấu hình ban đầu dành cho phụ huynh](docs/17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md)
- [Giới thiệu kiến trúc bộ cài đặt Inno Setup](installer/README.md)
- [Tài liệu kiến trúc và đặc tả](docs/README.md)
- [Hướng dẫn đóng góp](CONTRIBUTING.md)
- [Nhật ký thay đổi](CHANGELOG.md)
- [Hợp đồng OpenAPI](contracts/openapi.yaml)
- [JSON Schema chính sách](contracts/policy.schema.json)

## Giấy phép

FamShield Agent được phát hành theo giấy phép [MIT](LICENSE). Trừ khi có thông báo giấy phép riêng, giấy phép này áp dụng cho mã nguồn và tài liệu trong repository. Các thư viện và tài sản của bên thứ ba vẫn tuân theo giấy phép riêng của chúng. Việc cấp phép không đồng nghĩa với việc cấp quyền sử dụng nhãn hiệu hoặc logo FamShield.

## Đóng góp

Chào mừng các báo cáo lỗi, đề xuất cải tiến và đóng góp mã nguồn. Vui lòng đọc [CONTRIBUTING.md](CONTRIBUTING.md) trước khi mở Pull Request và tạo issue tại [GitHub Issues](https://github.com/nntu/famshield-agent/issues) nếu cần trao đổi.
