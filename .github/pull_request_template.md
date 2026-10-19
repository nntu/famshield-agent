## 📌 Mô Tả Thay Đổi / Description of Changes

<!-- Tóm tắt ngắn gọn mục tiêu, nguyên nhân và giải pháp kỹ thuật của PR -->
<!-- Provide a concise summary of the goal, motivation, and technical approach of this PR -->

---

## 🎯 Loại Thay Đổi / Type of Change

- [ ] 🐛 Sửa lỗi (Bug fix không làm ảnh hưởng tính năng hiện có)
- [ ] ✨ Tính năng mới (New feature không gây breaking change)
- [ ] ⚡ Tối ưu hiệu năng (Performance optimization)
- [ ] ♻️ Tái cấu trúc mã nguồn (Refactoring without behavioral change)
- [ ] 📝 Cập nhật tài liệu (Documentation update)
- [ ] 🧪 Bổ sung/Cải thiện kiểm thử (Tests update)
- [ ] ⚠️ Breaking Change (Thay đổi có thể ảnh hưởng cấu hình hoặc API hiện hành)

---

## 📦 Crates Ảnh Hưởng / Affected Crates

- [ ] `famshield-core`
- [ ] `famshield-service`
- [ ] `famshield-session`
- [ ] `famshield-dns`
- [ ] `famshield-windows`
- [ ] `famshield-local-admin`
- [ ] `famshield-storage`
- [ ] `famshield-sync`
- [ ] `famshield-updater`
- [ ] `installer` / `.github`

---

## ✅ Bảng Kiểm Tra Chất Lượng / Pre-Merge Checklist

- [ ] Đã chạy `cargo fmt --all -- --check` và mã nguồn được căn chỉnh đúng chuẩn.
- [ ] Đã chạy `cargo check --workspace --all-targets` không phát sinh lỗi biên dịch.
- [ ] Đã chạy `cargo clippy --workspace --all-targets -- -D warnings` đạt **0 errors, 0 warnings**.
- [ ] Đã chạy `cargo test --workspace` và toàn bộ unit tests đều **PASS 100%**.
- [ ] Đã bổ sung unit tests cho tính năng mới hoặc kịch bản sửa lỗi.
- [ ] Không chứa plain-text secret, thông tin cá nhân hoặc API token nhạy cảm.
- [ ] Commit messages tuân thủ chuẩn [Conventional Commits](https://www.conventionalcommits.org/).
