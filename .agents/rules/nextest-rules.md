# Quy Tắc Kiểm Thử: Thay Thế `cargo test` Bằng `cargo nextest`

## Quy Tắc Bắt Buộc

Khi chạy unit test hoặc integration test trong repository **FamShield Agent**, AI Agent **BẮT BUỘC** phải sử dụng `cargo nextest run` (kèm tiền tố `rtk`) thay cho `cargo test` thông thường:

```bash
# Thay thế rtk cargo test --workspace
rtk cargo nextest run --workspace

# Thay thế rtk cargo test -p <crate_name>
rtk cargo nextest run -p <crate_name>

# Thay thế rtk cargo test -p <crate_name> -- <filter>
rtk cargo nextest run -p <crate_name> -E 'test(<filter>)'

# Hoặc sử dụng runner tự động test.ps1
.\test.ps1 -Changed
.\test.ps1 -Target <crate_name>
```

## Lý Do

1. **Hiệu năng & Tốc độ vượt trội**: `nextest` chạy song song từng test case trong các tiến trình con độc lập, nhanh hơn 3x–5x so với bộ test runner mặc định của Cargo.
2. **Khắc phục triệt để treo / stall trên Windows**: Cargo test thông thường chạy các test case trong cùng tiến trình đa luồng, thường xuyên bị khóa file PDB, kẹt handle hoặc bị Windows Defender/EDR can thiệp dẫn đến treo vô tận (stall/timeout). Nextest tích hợp sẵn cơ chế giám sát timeout độc lập và cô lập lỗi hoàn hảo.
3. **Tiết kiệm Token (Tích hợp RTK)**: Đầu ra của `nextest` cực kỳ súc tích, chỉ hiển thị tóm tắt và chi tiết bài test thất bại (nếu có), giúp tiết kiệm 70–90% token cho AI Agent.

