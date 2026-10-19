### Vấn đề nghiêm trọng

1. Nhiều subsystem production vẫn là mô phỏng:

- wfp.rs (agent/crates/windows/src/wfp.rs:21) chỉ mở WFP engine session, chưa đăng ký filter chặn DNS/DoH.
- nrpt.rs (agent/crates/windows/src/nrpt.rs:13) chỉ log rồi trả Ok(()), không sửa registry.
- capture.rs (agent/crates/session/src/capture.rs:15) tạo byte JPEG giả, chưa có DXGI/GDI hay blur.
- enrollment.rs (agent/crates/sync/src/enrollment.rs:34) trả device ID và certificate giả.
- ipc.rs (agent/crates/service/src/ipc.rs:1) mới định nghĩa message; chưa có Named Pipe server, framing hoặc ACL/SDDL.
- controller.rs (agent/crates/service/src/controller.rs:79) polling process mỗi 5 giây, chưa phải ETW realtime như tài liệu.
- main.rs (agent/crates/service/src/main.rs:7) là console Tokio chờ Ctrl+C, chưa thấy Windows SCM dispatcher/service lifecycle hoàn chỉnh.

2. Bảo mật chưa đạt threat model:

- Xác thực PIN dùng Argon2 đúng hướng, nhưng authenticate (agent/crates/local_admin/src/service.rs:65) chưa có failed-attempt counter, lockout hoặc exponential backoff.
- Chưa có DPAPI machine-scope bảo vệ secret.
- Remote reset_pin nhận PIN mới trực tiếp trong command payload.
- Updater tại staged.rs (agent/crates/updater/src/staged.rs:27) mới kiểm SHA-256; chưa xác minh chữ ký release manifest.
- Chưa có lớp xác thực/ủy quyền IPC để ngăn user thường gửi message đặc quyền.

3. Build gate chưa xanh:

- cargo test --workspace: 34 passed, 1 failed.
- Test lỗi: process_monitor.rs (agent/crates/windows/src/process_monitor.rs:169). Test giả định hai lần enumerate process liên tiếp không có process nào thay đổi, nên phụ thuộc trạng thái hệ điều hành và dễ flaky.
- cargo clippy --workspace --all-targets -- -D warnings: thất bại với 2 lỗi assign_op_pattern tại traits.rs (agent/crates/core/src/traits.rs:40).
- Chưa thấy cấu hình CI trong repository.

4. Tài liệu chưa phản ánh đúng trạng thái:

README tuyên bố các khả năng như WFP, NRPT, visual verification, mTLS sync và “không gây BSOD”, trong khi một số thành phần vẫn là mock/stub. Nên gắn trạng thái Implemented / Prototype / Planned cho từng subsystem để tránh hiểu nhầm.

## Ưu tiên đề xuất

1. Đưa CI về xanh: sửa Clippy và thay process-monitor test bằng nguồn process giả/injected snapshot.
2. Hoàn thiện Windows Service + Named Pipe ACL trước khi mở bất kỳ command đặc quyền nào.
3. Implement NRPT thật và WFP filter transaction; thêm test rollback để tránh mất mạng.
4. Thêm PIN lockout, DPAPI, command authentication và signed update manifest.
5. Thay capture/enrollment mock bằng implementation thật hoặc tắt tính năng khỏi release build.
6. Giảm complexity của evaluate_process, DNS listener và NATS dispatcher.
7. Đồng bộ README/roadmap với trạng thái thực tế.

Kết luận: nền móng module và domain khá tốt cho một MVP kỹ thuật, nhưng hiện tại chỉ nên dùng cho phát triển/thử nghiệm nội bộ, chưa nên cài lên máy người dùng thật hoặc quảng bá là sản phẩm parental-control/security hoàn chỉnh.
  Worked for 2m 0s • 8:27 PM

tạo task thuc hiện hoàn thiện

Mình sẽ tạo một task tổng để đưa FamShield từ prototype lên bản hoàn thiện, với tiêu chí nghiệm thu theo từng milestone và ưu tiên khóa các rủi ro bảo mật/build trước.
Đã tạo task tổng và đặt trạng thái active.

Thứ tự thực hiện:

1. Làm xanh cargo test, Clippy và thiết lập CI.
2. Hoàn thiện Windows Service SCM và Named Pipe IPC có ACL.
3. Implement NRPT và WFP filter thật, kèm rollback an toàn.
4. Hoàn thiện bảo mật PIN: lockout, backoff, DPAPI.
5. Thay process polling bằng ETW.
6. Implement capture/blur hình ảnh thật.
7. Hoàn thiện mTLS enrollment và xác thực remote command.
8. Bổ sung chữ ký số cho updater.
9. Viết integration, security và rollback tests.
10. Đồng bộ README/roadmap với trạng thái thực tế.
