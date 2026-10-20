# FamShield Agent - Quy Tắc Dành Cho AI Coding Agents (AGENTS.md)

Tài liệu này định nghĩa các nguyên tắc và chỉ thị bắt buộc dành cho **AI Coding Agents** (Gemini Antigravity, Claude Code, Cursor, GitHub Copilot, v.v.) khi tham gia đọc hiểu, sửa đổi, refactor hoặc mở rộng mã nguồn trong repository **FamShield Agent**.

---

## 1. Quy Tắc Bắt Buộc: Cập Nhật CHANGELOG.md Theo Phiên Bản & Ngày Tháng

> ⚠️ **QUY TẮC BẮT BUỘC (MANDATORY RULE):**  
> Mỗi khi thực hiện một **thay đổi lớn (Major / Significant Changes)** — bao gồm:
> - Thêm tính năng mới (New features)
> - Tái cấu trúc kiến trúc hoặc module (Architectural refactorings)
> - Nâng cấp hoặc thay đổi cơ chế bảo mật (Security patches & enhancements)
> - Thay đổi giao thức mạng, schema DB, hoặc format IPC / Telemetry
> 
> **AI Agent BẮT BUỘC phải cập nhật [CHANGELOG.md](CHANGELOG.md)**:
> 1. Ghi nhận thay đổi vào đúng mục phiên bản (`[X.Y.Z] - YYYY-MM-DD`) hoặc mục `[Unreleased]`.
> 2. Phân loại thay đổi theo các tiêu đề chuẩn của [Keep a Changelog](https://keepachangelog.com/):
>    - `### Added`: Các tính năng, crate, API, hoặc subsystem mới.
>    - `### Changed`: Các thay đổi về hành vi, thuật toán, hoặc cấu trúc hiện tại.
>    - `### Deprecated`: Các tính năng chuẩn bị bị loại bỏ trong tương lai.
>    - `### Removed`: Các tính năng hoặc file bị xóa bỏ.
>    - `### Fixed`: Các lỗi kỹ thuật, memory leak, panic, hoặc lỗ hổng bảo mật đã xử lý.
>    - `### Security`: Các nâng cấp trực tiếp liên quan đến bảo mật (mã hóa, ACL, chữ ký).
> 3. Tuyệt đối không xóa lịch sử các phiên bản cũ đã có trong CHANGELOG.

---

## 2. Quy Tắc Tiết Kiệm Token (RTK Proxy CLI)

Khi thực thi bất kỳ lệnh shell nào trên hệ thống (terminal / powershell), AI Agent **BẮT BUỘC** phải gắn tiền tố `rtk`:

```bash
rtk cargo test
rtk cargo clippy --workspace --all-targets -- -D warnings
rtk git status
rtk git diff
```

- `rtk` tự động nén và lọc output đầu ra, tiết kiệm 60–90% token ngữ cảnh.
- Không sử dụng lệnh raw trừ khi cần gỡ lỗi đặc biệt với `rtk proxy <cmd>`.

---

## 3. Tiêu Chuẩn Chất Lượng Mã Nguồn & Chiến Lược Kiểm Thử (Build Quality Gate & Tiered Testing)

Nhằm tối ưu hóa tốc độ phát triển (Developer Experience), bảo tồn tài nguyên CPU và tiết kiệm token ngữ cảnh của AI Agent, quy trình kiểm thử và chất lượng mã nguồn bắt buộc tuân theo **Chiến Lược Kiểm Thử Phân Tầng (Tiered Testing Strategy)**:

### 3.1. Trong Quá Trình Code (Fast Inner Loop - TDD)
> 🛑 **NGHIÊM CẤM:** Tuyệt đối không chạy full workspace test (`rtk cargo test --workspace`) sau mỗi thay đổi nhỏ hoặc mỗi hàm vừa viết xong. Việc này gây lãng phí 15–45s và tiêu tốn hàng nghìn token không cần thiết.

- **Chỉ chạy test nhắm mục tiêu (Targeted Test)** cho đúng crate hoặc test case đang xử lý:
  ```bash
  rtk cargo test -p <crate_name> -- <test_name>
  # Ví dụ:
  rtk cargo test -p famshield-local-admin -- totp
  rtk cargo test -p famshield-dns -- cache
  ```
  *(Thời gian phản hồi chỉ 0.5s – 1.5s, nhận diện lỗi tức thì).*
- **Kiểm tra biên dịch & cảnh báo nhẹ cục bộ:**
  ```bash
  rtk cargo check -p <crate_name>
  rtk cargo clippy -p <crate_name> -- -D warnings
  ```

### 3.2. Cổng Nghiệm Thu Cuối Cùng Trước Khi Bàn Giao (Final Handover Quality Gate)
**Chỉ chạy đúng 1 LẦN DUY NHẤT** khi toàn bộ tính năng lớn đã hoàn thành xong xuôi, trước khi báo cáo kết quả với người dùng:

1. **Format:** `rtk cargo fmt --all -- --check` (Không có sai lệch định dạng).
2. **Biên dịch:** `rtk cargo check --workspace --all-targets` (Biên dịch thành công).
3. **Clippy Linter:** `rtk cargo clippy --workspace --all-targets -- -D warnings` (Tuyệt đối **0 errors, 0 warnings**).
4. **Unit Tests:** `rtk cargo test --workspace` (Toàn bộ các test suite phải **PASS 100%**).
5. **Frontend (nếu có thay đổi UI):** `rtk pnpm --prefix web run build` (Biên dịch Vite thành công; hỗ trợ pnpm v12.x standalone).

### 3.3. Rà Soát & Quản Lý Phiên Bản Thư Viện (Dependency Versioning & Modern Standards)
- Luôn ưu tiên sử dụng phiên bản mới nhất, ổn định của các crate dependencies (ví dụ `totp-rs = "6.0"`, `hickory-proto = "0.26"`).
- Khi nâng cấp hoặc bổ sung crate mới:
  - Tận dụng tối đa các mẫu thiết kế hiện đại (Builder pattern, Stack Allocation / Zero-Heap, Zeroize bảo vệ RAM).
  - Tuyệt đối loại bỏ cảnh báo sử dụng hàm lỗi thời (`#[deprecated]`).
  - Kiểm tra tính tương thích giữa các crate trong workspace trước khi commit.

---

## 4. An Toàn Hệ Thống Windows & Ngăn Ngừa BSOD / Treo Máy

FamShield Agent chạy dưới tiến trình đặc quyền `LocalSystem` trong Windows Session 0. Agent phải luôn tuân thủ:

- **Zero BSOD & Network Lockup:** Mọi bộ lọc Windows Filtering Platform (WFP) bắt buộc phải đăng ký với cờ `FWPM_SESSION_FLAG_DYNAMIC`. Không bao giờ tạo static filter vĩnh viễn gây kẹt mạng máy trạm khi service gặp sự cố.
- **Không Panic trong Service Loop:** Không sử dụng `unwrap()` hoặc `panic!()` trong các đường dẫn chạy ngầm (`famshield-service`, `famshield-dns`, `famshield-windows`). Sử dụng enum `thiserror` với kiểu trả về `Result<T, E>`.
- **Bảo vệ Secret qua DPAPI:** Mọi mật khẩu, PIN hash, hoặc token bí mật lưu trữ cục bộ phải được mã hóa qua Windows Data Protection API (machine-scope). Tuyệt đối cấm ghi plain-text.
- **Privacy-by-Design:** Mọi ảnh chụp màn hình vi phạm phải được làm mờ (Gaussian Blur) trực tiếp trong RAM trước khi ghi xuống đĩa hoặc gửi qua mạng.
- **Bảo vệ IPC Named Pipe:** Luôn áp dụng chuỗi SDDL ACL chặt chẽ và framing độ dài tiền tố (giới hạn an toàn 256 KiB) để chống tấn công cạn kiệt bộ nhớ.

---

## 5. Bảo Toàn Kiến Trúc Freemium (Freemium Integrity)

Agent phải luôn duy trì sự tách biệt rành mạch giữa 2 chế độ:
- **`OperatingMode::StandaloneFree`**: Hoạt động 100% offline, miễn phí, độc lập, bảo vệ riêng tư tối đa, **không bao giờ phụ thuộc vào server hay mạng Internet**.
- **`OperatingMode::OnlineManaged`**: Chỉ kích hoạt khi người dùng chủ động ghép nối qua mã xác thực đám mây. Mọi lệnh điều khiển từ xa phải được xác thực bằng chữ ký mật mã HMAC-SHA256 và kiểm tra cửa sổ chống phát lại (Anti-replay window $\le 300\text{s}$).

> 🛑 **Nghiêm cấm:** Không bao giờ thêm logic khiến chế độ StandaloneFree bị phụ thuộc vào API máy chủ đám mây hoặc bị khóa tính năng bảo vệ máy trạm.

---

## 6. Đồng Bộ Hóa Tài Liệu Song Ngữ

Mỗi khi thay đổi API, tính năng hiển thị cho người dùng, hoặc quy trình cài đặt:
- Cập nhật đồng bộ cả hai tài liệu: [README.md](README.md) (Tiếng Việt) và [README.en.md](README.en.md) (Tiếng Anh).
- Đảm bảo các liên kết GitHub Markdown trỏ đúng file và đúng format `file:///...`.

---

## 7. Quy Tắc Bắt Buộc: Quản Lý Phiên Bản Telemetry & Đối Chiếu Chéo Với Server (Telemetry Versioning & Cross-Reconciliation)

> ⚠️ **QUY TẮC BẮT BUỘC (MANDATORY RULE):**  
> Bất kỳ khi nào thực hiện sửa đổi, bổ sung hoặc điều chỉnh cấu trúc dữ liệu **Telemetry** tại Client Agent:
> - Thêm/sửa trường trong [`TelemetryEnvelope`](agent/crates/sync/src/telemetry.rs) hoặc [`TelemetryEvent`](agent/crates/sync/src/telemetry.rs)
> - Thêm loại sự kiện mới vào enum [`TelemetryEventType`](agent/crates/sync/src/telemetry.rs)
> - Thay đổi payload của `HeartbeatPayload`, `dns_query`, `process_event`, `error_log`, `security_alert`, v.v.
>
> **AI Agent BẮT BUỘC phải thực hiện 4 bước sau trước khi hoàn thành công việc:**
>
> 1. **Tăng Phiên Bản Telemetry (Version Bump):**
>    - Bắt buộc tăng trường `version` trong `TelemetryEnvelope` (ví dụ từ `1` lên `2`), hoặc bump `agent_version` tương ứng trong `HeartbeatPayload`.
>    - Tuyệt đối không âm thầm thay đổi schema/format gói tin mà giữ nguyên version cũ.
>
> 2. **Cập Nhật Hợp Đồng OpenAPI Chung (`contracts/openapi.yaml`):**
>    - Cập nhật schema `EventBatch`, `DeviceEvent`, `HeartbeatRequest` tương ứng trong file hợp đồng OpenAPI chuẩn tại cả 2 repo (`famshield-agent/contracts/openapi.yaml` và `famshield-server/contracts/openapi.yaml`).
>
> 3. **Đối Chiếu Chéo Bắt Buộc Với Cloud Server (Cross-Reconciliation):**
>    - Mở và đối chiếu trực tiếp với mã nguồn Go Cloud Server tại:
>      - [`server/pkg/model/model.go`](file:///d:/projects/famshield/famshield-server/server/pkg/model/model.go)
>      - [`server/pkg/api/server.go`](file:///d:/projects/famshield/famshield-server/server/pkg/api/server.go)
>    - Đảm bảo Go Server đã hỗ trợ trường mới và luôn duy trì **tính tương thích ngược (Backward Compatibility)** cho các agent phiên bản cũ.
>
> 4. **Chạy Bộ Test Đối Chiếu (Verification Gate):**
>    - Phía Client: `rtk cargo test -p famshield-sync` (PASS 100%).
>    - Phía Server: `rtk go test -v ./pkg/api -run TestTelemetryClientReconciliation` (PASS 100%).
>    - Ghi nhận chi tiết thay đổi Telemetry Version vào `CHANGELOG.md`.

