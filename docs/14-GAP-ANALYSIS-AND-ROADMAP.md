# FamShield Ecosystem Gap Analysis and Implementation Roadmap

Tài liệu này ghi nhận kết quả rà soát, kiểm kê và đánh giá toàn diện hiện trạng kỹ thuật giữa hai repository:
1. **Windows Agent (Rust):** [`child_monitor`](https://github.com/nntu/famshield-agent)
2. **Cloud Server & Web Console (Go/React):** [`famshield-server`](https://github.com/nntu/famshield-server)

Đồng thời xác định các khoảng trống kỹ thuật (Gaps) còn thiếu và thiết lập lộ trình thực thi hoàn thiện sản phẩm.

---

## 1. Hiện Trạng Kiến Trúc Tổng Thể

Hệ sinh thái FamShield được phân tách thành 2 dự án độc lập:

```mermaid
flowchart TD
    subgraph AgentHost["Windows Endpoint (famshield-agent)"]
        SVC["famshield-service.exe (LocalSystem)"]
        SESS["famshield-session.exe (Interactive User)"]
        DNS["DNS Proxy :53 & WFP Guard"]
        STORAGE["SQLite (WAL Mode) + DPAPI Machine"]
        UPDATER["Dual-mode Updater (Online / Offline)"]
        IPC["Named Pipe (\\\\.\\pipe\\FamShieldService)"]

        SVC <--> IPC <--> SESS
        SVC --> DNS
        SVC --> STORAGE
        SVC --> UPDATER
    end

    subgraph CloudServer["Cloud Platform (famshield-server)"]
        REST["Go REST API (/v1) :8080"]
        MEM["In-Memory Store (store.go)"]
        TG["Telegram Bot Service (Activation & Alerts)"]
        NATS["NATS JetStream :4222"]
        WEB["React 19 Web Console (Nginx :3000)"]

        REST <--> MEM
        REST <--> NATS
        REST <--> TG
        WEB <--> REST
    end

    SVC -.->|"1. Device Enroll (/v1/device/enroll)"| REST
    SVC -.->|"2. Heartbeat & Batch Telemetry (/v1/device/events:batch)"| REST
    NATS -.->|"3. Instant Remote Commands (devices.{id}.commands)"| SVC
    REST -.->|"4. OTA Releases (/v1/releases/latest)"| UPDATER
```

---

## 2. Ma Trận Đánh Giá Tính Năng (Feature Matrix & Gap Assessment)

| Thành Phần | Hạng Mục Kỹ Thuật | Trạng Thái Hiện Tại | Đánh Giá Kỹ Thuật & Khoảng Trống (Gap) | Mức Độ Ưu Tiên |
| :--- | :--- | :---: | :--- | :---: |
| **Server** | **Lưu trữ dữ liệu (Database)** | ⚠️ **Tạm thời** | Sử dụng `MemoryStore` trong RAM. Restart container sẽ mất dữ liệu thiết bị, tài khoản và logs. Cần chuyển sang **PostgreSQL**. | **P0** (Bắt buộc) |
| **Server** | **Xác thực mTLS & CA** | ⚠️ **Mock chuỗi** | Endpoint `/v1/device/enroll` trả về chuỗi PEM tĩnh. Cần cấp chứng chỉ x509 thật từ CA nội bộ. | **P1** |
| **Server** | **Quản lý & Ký số Policy** | ❌ **Chưa có** | Chưa có API tạo draft policy, validate schema `policy.schema.json`, publish version, và ký số payload bằng ECDSA P-256. | **P1** |
| **Server** | **Lưu trữ ảnh bằng chứng (Evidence)** | ❌ **Chưa có** | Chưa có handler `POST /v1/device/evidence/{evidence_id}` nhận JPEG binary, lưu Object Storage (MinIO/S3) và TTL 48h. | **P2** |
| **Server** | **Telegram Bot kích hoạt & cảnh báo** | ✅ **Hoàn thành** | Đã triển khai kích hoạt 1-chạm qua deep-link (`/start <token>`), OTP 6 số, và đẩy tin nhắn cảnh báo tức thì. | Hoàn tất |
| **Server** | **NATS JetStream Queue** | ✅ **Hoàn thành** | Đã cấu hình NATS JetStream container, publish sự kiện telemetry và lệnh điều khiển từ xa. | Hoàn tất |
| **Web UI** | **Đăng ký & Tải bộ cài** | ✅ **Hoàn thành** | Modal đăng ký kích hoạt Telegram và download file `FamShieldSetup.exe` trực tiếp. | Hoàn tất |
| **Web UI** | **Đa ngôn ngữ (i18n)** | ✅ **Hoàn thành** | Hỗ trợ song ngữ Tiếng Việt (mặc định) và Tiếng Anh. | Hoàn tất |
| **Web UI** | **Kết nối API thật** | ⚠️ **Dữ liệu Mock** | Các danh sách thiết bị, trẻ em và sự kiện trong `App.tsx` đang dùng `useState` tĩnh, chưa gọi API từ Server. | **P0** |
| **Web UI** | **Quản lý Luật (Rule Management)** | ❌ **Chưa có** | Chưa có giao diện thêm/sửa/xóa quy tắc chặn app, chặn web, lịch sử dụng cho từng trẻ em. | **P1** |
| **Web UI** | **Xem ảnh bằng chứng vi phạm** | ❌ **Chưa có** | Chưa có gallery xem ảnh chụp màn hình vi phạm đã làm mờ và nút duyệt gia hạn giờ chơi. | **P2** |
| **Agent** | **Dịch vụ Windows Service (SCM)** | ✅ **Hoàn thành** | Service Control Manager Dispatcher chuẩn, Interactive Console mode, Named Pipe SDDL. | Hoàn tất |
| **Agent** | **DNS Proxy & WFP Firewall** | ✅ **Hoàn thành** | Bind loopback `127.0.0.2:53`, Dynamic WFP session chống rò rỉ và BSOD, chặn port 53 ngoại vi. | Hoàn tất |
| **Agent** | **Giám sát Kernel ETW** | ✅ **Hoàn thành** | Bắt sự kiện tạo/hủy tiến trình 0ms qua ETW, ToolHelp32 fallback, phát hiện vi phạm. | Hoàn tất |
| **Agent** | **Bảo mật Master PIN & DPAPI** | ✅ **Hoàn thành** | Argon2id + salt ngẫu nhiên, mã hóa DPAPI machine-scope, lockout chống brute-force. | Hoàn tất |
| **Agent** | **Chụp màn hình mờ bảo vệ riêng tư** | ✅ **Hoàn thành** | GDI BitBlt, Gaussian Blur trong RAM, nén JPEG < 150 KiB, tự hủy sau 48h. | Hoàn tất |
| **Agent** | **Dual-mode Auto-Updater** | ✅ **Hoàn thành** | Cập nhật Online (xác thực ECDSA manifest) & Ngoại tuyến (USB hot-folder), rollback an toàn. | Hoàn tất |
| **Agent** | **Bộ cài đặt Inno Setup 64-bit** | ✅ **Hoàn thành** | Native 64-bit (`C:\Program Files\FamShield`), ký số Authenticode SHA-256. | Hoàn tất |
| **Agent** | **HTTP & NATS Client kết nối mạng** | ⚠️ **Sẵn sàng khung** | Đã có `NatsDispatcher`, HMAC signature, anti-replay, nhưng chưa gắn kết nối TCP `async-nats` vào loop chính. | **P1** |
| **Agent** | **Giao diện Local Admin Web UI** | ❌ **Chưa có** | Chưa có giao diện Web cục bộ (WebView2 / HTTPS localhost:5380) cho chế độ 100% Offline Free. | **P2** |

---

## 3. Chi Tiết 4 Khoảng Trống Kỹ Thuật Cốt Lõi (Core Gaps)

### 3.1. Khoảng trống 1: Cơ sở dữ liệu Persistent cho Cloud Server (P0)
- **Hiện trạng:** `famshield-server/server/pkg/store/store.go` đang dùng `MemoryStore`.
- **Rủi ro:** Khi container khởi động lại hoặc gặp sự cố, toàn bộ dữ liệu tài khoản phụ huynh đã kích hoạt qua Telegram, thiết bị đã ghép nối và lịch sử vi phạm đều bị mất sạch.
- **Yêu cầu giải quyết:**
  - Thêm container `postgres:16-alpine` vào `docker-compose.yml`.
  - Triển khai `PostgresStore` thay thế `MemoryStore` với các bảng: `users`, `tenants`, `devices`, `policies`, `device_events`, `visual_evidences`, `commands`.
  - Giữ `MemoryStore` làm cache phục vụ xử lý luồng sự kiện NATS tốc độ cao.

### 3.2. Khoảng trống 2: Kết Nối API Thật Giữa Web Console và Server (P0)
- **Hiện trạng:** `web/src/App.tsx` sử dụng mock state:
  ```typescript
  const [devices] = useState<Device[]>([ ... ]); // Mock tĩnh
  ```
- **Rủi ro:** Người dùng đăng ký thành công qua Telegram nhưng khi truy cập Web Console không thể xem thiết bị thật hay điều khiển máy tính.
- **Yêu cầu giải quyết:**
  - Xây dựng API Client (`axios` / `fetch`) kết nối về backend `/v1/`.
  - Màn hình Đăng nhập (Login Form) bằng Email & Password, lưu phiên đăng nhập an toàn.
  - Tải danh sách thiết bị động từ `/v1/admin/devices` và sinh mã ghép nối 6 chữ số từ `/v1/admin/enrollment-codes`.
  - Giao diện gửi lệnh tức thì: Khóa máy ngay (`lock_session`), cập nhật danh sách đen (`update_blacklist`), reset PIN.

### 3.3. Khoảng trống 3: Tích Hợp Kết Nối Mạng Thực Tế Giữa Agent và Server (P1)
- **Hiện trạng:**
  - Crate `famshield-sync` đã hoàn thành bộ định dạng dữ liệu, xác thực chữ ký HMAC-SHA256, và cơ chế anti-replay drift $\le 300\text{s}$.
  - Tuy nhiên, `enroll_device` đang trả về dữ liệu mock, chưa thực hiện gọi HTTP POST thật lên Server.
  - Service Controller chưa khởi chạy background loop định kỳ gửi batch sự kiện từ outbox SQLite lên Cloud.
- **Yêu cầu giải quyết:**
  - Tích hợp HTTP Client (`reqwest`) trong `famshield-sync`:
    - `POST /v1/device/enroll`: Ghép nối mã 6 số với máy chủ.
    - `POST /v1/device/heartbeat`: Báo cáo trạng thái nhịp tim và phiên bản policy mỗi 60s.
    - `POST /v1/device/events:batch`: Định kỳ đẩy batch sự kiện vi phạm và log lỗi từ outbox SQLite.
  - Tích hợp NATS JetStream client (`async-nats`) lắng nghe topic `devices.{device_id}.commands` để thực thi lệnh khóa máy trong $< 1\text{s}$.

### 3.4. Khoảng trống 4: Giao Diện Quản Trị Cục Bộ Cho Bản Miễn Phí (Local Admin UI - Milestone 5) (P2)
- **Hiện trạng:**
  - Chế độ `OperatingMode::StandaloneFree` chạy 100% offline, miễn phí, không phụ thuộc vào máy chủ đám mây.
  - Hiện tại máy trạm chỉ có thể tương tác qua CLI `famshield-service.exe --reset-pin` hoặc thao tác trực tiếp với cơ sở dữ liệu SQLite.
- **Yêu cầu giải quyết:**
  - Triển khai Local Web Server tối giản (sử dụng `axum` hoặc `tiny_http`) chạy trong session helper hoặc service, lắng nghe tại `https://127.0.0.1:5380`.
  - Giao diện Web nhẹ nhàng yêu cầu nhập Master PIN để đăng nhập.
  - Cung cấp trang thêm/sửa/xóa tên miền cần chặn, xem báo cáo thời gian dùng máy trong 7 ngày, và nút "Ghép nối Cloud" bằng mã 6 số.

---

## 4. Lộ Trình Triển Khai Hoàn Thiện (Implementation Roadmap)

```mermaid
flowchart LR
    subgraph GiaiĐoạn1["Giai đoạn 1: Server & Database"]
        G1_1["Tích hợp PostgreSQL"]
        G1_2["Chuyển đổi PostgresStore"]
        G1_3["REST API Quản lý Thiết Bị"]
    end

    subgraph GiaiĐoạn2["Giai đoạn 2: Web Console Live API"]
        G2_1["Form Đăng Nhập JWT"]
        G2_2["Fetch Danh Sách Thiết Bị Thật"]
        G2_3["Bảng Điều Khiển & Gửi Lệnh"]
    end

    subgraph GiaiĐoạn3["Giai đoạn 3: Agent Live Networking"]
        G3_1["HTTP Enrollment & Heartbeat"]
        G3_2["Outbox Batch Telemetry Uploader"]
        G3_3["NATS Listener Nhận Lệnh Tức Thì"]
    end

    subgraph GiaiĐoạn4["Giai đoạn 4: Local Admin GUI"]
        G4_1["Localhost Web Server :5380"]
        G4_2["Đăng Nhập Master PIN Cục Bộ"]
        G4_3["Cấu Hình Luật Ngoại Tuyến"]
    end

    GiaiĐoạn1 --> GiaiĐoạn2
    GiaiĐoạn1 --> GiaiĐoạn3
    GiaiĐoạn3 --> GiaiĐoạn4
```

### Kế hoạch hành động theo thứ tự ưu tiên:

1. **Bước 1 (PostgreSQL & Persistent Server Storage):**
   - Viết cấu hình `postgres` trong `famshield-server/docker-compose.yml`.
   - Triển khai `PostgresStore` đảm bảo toàn bộ dữ liệu người dùng, thiết bị, và sự kiện vi phạm được lưu trữ bền vững.
2. **Bước 2 (Web Console Live Data & Control Panel):**
   - Bổ sung trang Đăng nhập (Login) trên Web Console.
   - Kết nối `App.tsx` vào API `/v1/admin/devices` để hiển thị đúng các máy trạm đang online/offline.
   - Thêm nút kích hoạt lệnh tức thì: "Khóa máy ngay" và "Đổi PIN máy trạm".
3. **Bước 3 (Agent Live Sync & Telemetry):**
   - Cập nhật `agent/crates/sync` với `reqwest` thực hiện ghép nối mã 6 chữ số và gửi nhịp tim định kỳ.
   - Kết nối NATS JetStream thực tế để nhận lệnh khóa máy từ xa $< 1\text{s}$.
4. **Bước 4 (Local Admin Web UI cho Bản Miễn Phí):**
   - Xây dựng giao diện web cục bộ phục vụ người dùng ngoại tuyến không có nhu cầu kết nối đám mây.
