# FamShield Contributing Guide & Developer Rules

[🇻🇳 Tiếng Việt](#quy-định-đóng-góp-tiếng-việt) | [🇬🇧 English](#contribution-guidelines-english)

---

<a name="quy-định-đóng-góp-tiếng-việt"></a>
## 🇻🇳 Quy Định & Hướng Dẫn Phát Triển Cho Lập Trình Viên (Tiếng Việt)

Cảm ơn bạn đã quan tâm đóng góp cho **FamShield Windows Agent**! Dự án này là phần mềm an ninh máy trạm bảo vệ trẻ em, chạy với quyền đặc quyền cao nhất (`LocalSystem`) trên hệ điều hành Windows. Vì vậy, mọi dòng code đều phải tuân thủ nghiêm ngặt các tiêu chuẩn kỹ thuật dưới đây.

### 1. Nguyên Tắc Cốt Lõi (Core Principles)

- 🛡️ **An toàn bộ nhớ tuyệt đối**: Không để xảy ra BSOD (Màn hình xanh), Memory leak hay rớt mạng máy trạm.
- ⚡ **Siêu nhẹ & Tiết kiệm tài nguyên**: Toàn bộ tiến trình chạy ngầm phải giữ mức tiêu thụ RAM < 15MB và CPU < 0.5%.
- 🔒 **Bảo vệ quyền riêng tư theo thiết kế (Privacy-by-Design)**: Không bao giờ lưu trữ mật khẩu/PIN dạng plain-text (phải dùng DPAPI + Argon2id); mọi hình ảnh chụp màn hình đều phải được làm mờ (Gaussian Blur) trong RAM trước khi ghi xuống đĩa hoặc truyền đi.
- 🔄 **Xử lý lỗi phòng thủ (Defensive Error Handling)**: Không dùng `unwrap()` hay `panic!()` trong logic dịch vụ chính (`service`, `dns`, `windows`). Tất cả lỗi đều phải được mô hình hóa qua enum `thiserror` và trả về `Result`.

---

### 2. Quy Trình Làm Việc Với Git (Git Workflow)

1. **Nhánh chính (Default Branch)**: `master`
2. **Quy tắc đặt tên nhánh**:
   - Tính năng mới: `feat/<tên-tính-năng>` (VD: `feat/wfp-ipv6-tuning`)
   - Sửa lỗi: `fix/<tên-lỗi>` (VD: `fix/dns-cache-ttl`)
   - Tài liệu: `docs/<chủ-đề>` (VD: `docs/architecture-update`)
   - Tối ưu hiệu năng: `perf/<tối-ưu>` (VD: `perf/box-blur-simd`)
   - Kiểm thử: `test/<tên-test>` (VD: `test/etw-mock-suite`)
3. **Quy chuẩn Commit Message (Conventional Commits)**:
   Mọi commit message phải tuân thủ định dạng:
   ```text
   <type>(<scope>): <mô tả ngắn bằng chữ thường, không dấu chấm cuối câu>

   [mô tả chi tiết tùy chọn về lý do thay đổi và giải pháp kỹ thuật]
   ```
   - **Type**: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.
   - **Scope**: `dns`, `windows`, `service`, `session`, `local_admin`, `storage`, `sync`, `updater`, `core`, `ci`.
   - *Ví dụ:* `feat(dns): support edns0 buffer size negotiation`

---

### 3. Build Quality Gate (Yêu Cầu Trước Khi Tạo Pull Request)

Trước khi commit và tạo Pull Request, bạn **bắt buộc** phải chạy và vượt qua toàn bộ 4 bước kiểm tra chất lượng:

```powershell
# 1. Kiểm tra định dạng code
cargo fmt --all -- --check

# 2. Biên dịch toàn bộ workspace
cargo check --workspace --all-targets

# 3. Kiểm tra cảnh báo (0 errors, 0 warnings bắt buộc)
cargo clippy --workspace --all-targets -- -D warnings

# 4. Chạy toàn bộ test suites (yêu cầu 100% pass)
cargo test --workspace
```

> **Mẹo:** Nếu sử dụng CLI/AI Assistant trong terminal, vui lòng dùng tiền tố `rtk` (e.g. `rtk cargo test`) để tối ưu tốc độ và token.

---

### 4. Quy Trình Gửi Pull Request (PR Process)

1. Fork kho mã nguồn và tạo nhánh mới từ `master`.
2. Viết mã và bổ sung unit test tương ứng cho tính năng/lỗi vừa sửa.
3. Chạy Quality Gate thành công 100%.
4. Tạo Pull Request trên GitHub hướng về nhánh `master`.
5. Điền đầy đủ thông tin vào mẫu [Pull Request Template](.github/pull_request_template.md).
6. Tất cả các job trong GitHub Actions CI phải xanh lá (Pass).
7. Được duyệt (Approve) bởi ít nhất 1 maintainer trước khi merge.

---

<a name="contribution-guidelines-english"></a>
## 🇬🇧 Developer Contribution Guidelines (English)

Thank you for contributing to **FamShield Windows Agent**! As endpoint security software running under `LocalSystem` on Windows, code quality, stability, and memory safety are paramount.

### 1. Core Engineering Principles

- 🛡️ **Zero BSOD & Crash-Proof**: Never compromise system stability. Dynamic sessions for WFP and safe Win32 wrappers are strictly enforced.
- ⚡ **Minimal Footprint**: Background runtime memory consumption must stay under 15MB RAM and CPU under 0.5%.
- 🔒 **Privacy-by-Design**: Plaintext secrets are strictly prohibited (use DPAPI + Argon2id). Screen captures must be blurred in-memory before disk/network persistence.
- 🔄 **Defensive Error Handling**: Avoid `unwrap()` and `panic!()` in core runtime paths (`service`, `dns`, `windows`). Use strongly typed `thiserror` enums with `Result`.

---

### 2. Git Workflow & Branching

1. **Base Branch**: `master`
2. **Branch Naming**:
   - Features: `feat/<feature-name>`
   - Fixes: `fix/<bug-name>`
   - Documentation: `docs/<topic>`
   - Performance: `perf/<optimization>`
   - Testing: `test/<suite-name>`
3. **Commit Conventions**:
   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   ```text
   <type>(<scope>): <concise imperative description without trailing dot>

   [Optional detailed body explaining rationale and design decisions]
   ```
   - **Types**: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.
   - **Scopes**: `dns`, `windows`, `service`, `session`, `local_admin`, `storage`, `sync`, `updater`, `core`, `ci`.

---

### 3. Local Verification Quality Gates

All contributions must cleanly pass these 4 checks locally before opening a PR:

```powershell
# 1. Format check
cargo fmt --all -- --check

# 2. Workspace compilation
cargo check --workspace --all-targets

# 3. Clippy linter with zero warnings tolerance
cargo clippy --workspace --all-targets -- -D warnings

# 4. Workspace unit tests (100% pass)
cargo test --workspace
```

---

### 4. Pull Request Review & Merge Requirements

- Open PR against `master`.
- Fill out the [.github/pull_request_template.md](.github/pull_request_template.md).
- Automated CI pipeline must pass on Windows runners.
- At least one code review approval from core maintainers is required before merging.
