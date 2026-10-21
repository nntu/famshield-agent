# FamShield Version Checker (Cloudflare Worker)

Worker kiểm tra phiên bản và điều phối cập nhật siêu nhẹ, **hoàn toàn miễn phí (Zero-Cost $0)** cho FamShield Agent chạy trên nền tảng Cloudflare Edge với domain duy nhất: **`famshield.ttcantho.io.vn`** (không tạo thêm bất kỳ subdomain nào khác).

---

## 1. Nguyên Tắc Hoạt Động (Single Domain & Zero-Cost Model)

- **Không sinh thêm Subdomain:** Tận dụng đúng subdomain duy nhất `famshield.ttcantho.io.vn`.
- **Định tuyến theo đường dẫn (Path-based Routing):**
  - Worker chỉ chặn và xử lý các đường dẫn:
    - `/version.json`
    - `/v1/release/*`
    - `/download/*`
  - Các đường dẫn API khác (như `/v1/device/*`, `/v1/tenant/*`, web dashboard) sẽ tự động được chuyển tiếp (`pass-through`) tới Server chính của bạn khi server chạy.
- **Lưu trữ nhị phân (.exe):** Đặt trực tiếp trên **GitHub Releases** (Băng thông và dung lượng lưu trữ miễn phí 100%).
- **Lưu trữ & Kiểm tra phiên bản (Check Ver):** Worker chạy trên gói Cloudflare Free Tier (hỗ trợ tới 100.000 requests/ngày).

---

## 2. Các API Cung Cấp Trên `famshield.ttcantho.io.vn`

| Endpoint | Method | Chức năng | Mô tả |
| :--- | :---: | :--- | :--- |
| `https://famshield.ttcantho.io.vn/version.json` | GET | Kiểm tra phiên bản mới nhất | Trả về JSON chứa số version, changelog, SHA-256 và link tải GitHub |
| `https://famshield.ttcantho.io.vn/v1/release/latest` | GET | Alias của `/version.json` | Chuẩn hóa API REST |
| `https://famshield.ttcantho.io.vn/download/latest` | GET | Tải nhanh bản mới nhất | Tự động redirect 302 sang file `.exe` trên GitHub Releases |
| `https://famshield.ttcantho.io.vn/health` | GET | Kiểm tra tình trạng Worker | Trả về status `ok` |
| `https://famshield.ttcantho.io.vn/v1/admin/release` | POST | Đăng bản phát hành mới | Cập nhật version JSON vào KV (cần header `X-Admin-Key`) |

---

## 3. Hướng Dẫn Triển Khai (Deployment)

### Bước 1: Đăng nhập Cloudflare bằng Wrangler
```bash
npx wrangler login
```

### Bước 2: Tạo KV Namespace Có Định Danh FamShield Rõ Ràng
Vì tài khoản Cloudflare của bạn chạy chung nhiều ứng dụng, hãy đặt tên có tiền tố `FAMSHIELD_` để dễ quản lý trên Cloudflare Dashboard (lưu ý dùng dấu cách, không dùng dấu hai chấm):
```bash
npx wrangler kv namespace create FAMSHIELD_VERSION_STORE
```
Lệnh trên sẽ hiển thị đoạn mã cấu hình chứa `id`. Bạn copy `id` đó và dán vào file [`wrangler.jsonc`](./wrangler.jsonc):
```jsonc
  "kv_namespaces": [
    {
      "binding": "FAMSHIELD_VERSION_STORE",
      "id": "dán_id_vừa_tạo_vào_đây"
    }
  ]
```

### Bước 3: Chạy script triển khai
```powershell
./deploy.ps1
```
Hoặc dùng lệnh:
```bash
npx wrangler deploy
```

---

## 4. Cách Đăng Phiên Bản Mới (Publishing a New Release)

Khi bạn ra mắt phiên bản mới (ví dụ `v1.0.1`):
1. Đẩy file `FamShield-Setup-v1.0.1.exe` lên mục **Releases** của GitHub repo `phamngocnghe/famshield`.
2. Gửi request cập nhật lên Worker:

```bash
curl -X POST https://famshield.ttcantho.io.vn/v1/admin/release \
  -H "Content-Type: application/json" \
  -H "X-Admin-Key: famshield-default-admin-secret" \
  -d '{
    "version": "1.0.1",
    "release_date": "2026-10-21",
    "changelog": [
      "Bổ sung chụp màn hình sắc nét khi truy cập web xấu",
      "Sửa lỗi kill tiến trình cũ khi cài đặt"
    ],
    "installer_url": "https://github.com/phamngocnghe/famshield/releases/download/v1.0.1/FamShield-Setup-v1.0.1.exe",
    "service_binary_url": "https://github.com/phamngocnghe/famshield/releases/download/v1.0.1/famshield-service.exe",
    "service_sha256": "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a"
  }'
```
