pub const LOCAL_DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FamShield - Bảng điều khiển Quản trị Cục bộ</title>
    <style>
        :root {
            --bg-primary: #0f172a;
            --bg-card: #1e293b;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --accent: #38bdf8;
            --accent-green: #4ade80;
            --accent-red: #f87171;
            --border: #334155;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        body { background: var(--bg-primary); color: var(--text-main); padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 16px; margin-bottom: 24px; }
        .badge { background: rgba(56, 189, 248, 0.2); color: var(--accent); padding: 4px 12px; border-radius: 9999px; font-size: 0.85rem; font-weight: 600; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; }
        .card { background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; padding: 20px; }
        .card h2 { font-size: 1.15rem; margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }
        .stat-value { font-size: 2rem; font-weight: 700; color: var(--accent); margin: 8px 0; }
        .stat-sub { font-size: 0.85rem; color: var(--text-muted); }
        .quota-bar { height: 8px; background: #334155; border-radius: 4px; overflow: hidden; margin-top: 8px; }
        .quota-fill { height: 100%; width: 45%; background: var(--accent-green); border-radius: 4px; }
        table { width: 100%; border-collapse: collapse; margin-top: 12px; font-size: 0.9rem; }
        th, td { text-align: left; padding: 10px; border-bottom: 1px solid var(--border); }
        th { color: var(--text-muted); font-weight: 500; }
        .tag-block { color: var(--accent-red); font-weight: 600; }
        .tag-allow { color: var(--accent-green); font-weight: 600; }
        .evidence-grid { display: flex; gap: 12px; flex-wrap: wrap; margin-top: 12px; }
        .evidence-card { border: 1px solid var(--border); border-radius: 8px; padding: 8px; width: 160px; text-align: center; }
        .evidence-thumb { width: 100%; height: 90px; background: #0f172a; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 0.75rem; color: var(--text-muted); filter: blur(2px); }
        .countdown { font-size: 0.75rem; color: var(--accent); margin-top: 4px; }
    </style>
</head>
<body>
    <div class="header">
        <div>
            <h1>FamShield Quản trị Cục bộ</h1>
            <p class="stat-sub">Chế độ Độc lập (Zero-Cloud Standalone) • Cổng bảo vệ: 127.0.0.2:53</p>
        </div>
        <div class="badge">Master PIN: Xác thực thành công</div>
    </div>

    <div class="grid">
        <div class="card">
            <h2>Ứng dụng Foreground Hoạt động</h2>
            <div class="stat-value" id="active-app">Roblox</div>
            <div class="stat-sub">Thời gian sử dụng hôm nay: 27 / 60 phút</div>
            <div class="quota-bar"><div class="quota-fill"></div></div>
        </div>

        <div class="card">
            <h2>Bộ lọc DNS & WFP</h2>
            <div class="stat-value">127.0.0.2:53</div>
            <div class="stat-sub">User-Mode WFP Session: <strong style="color: var(--accent-green)">Đang bảo vệ</strong></div>
            <div class="stat-sub" style="margin-top: 4px;">Tên miền đã chặn hôm nay: 18 lượt</div>
        </div>

        <div class="card" style="grid-column: 1 / -1;">
            <h2>Nhật ký Truy vấn Tên miền (DNS Query Logs)</h2>
            <table>
                <thead>
                    <tr>
                        <th>Thời gian</th>
                        <th>Tên miền</th>
                        <th>Loại</th>
                        <th>Hành động</th>
                        <th>Lý do</th>
                    </tr>
                </thead>
                <tbody id="dns-logs">
                    <tr>
                        <td>17:05:12</td>
                        <td>roblox.com</td>
                        <td>A</td>
                        <td><span class="tag-allow">CHO PHÉP</span></td>
                        <td>default_action_allow</td>
                    </tr>
                    <tr>
                        <td>17:02:40</td>
                        <td>chat.example-social.invalid</td>
                        <td>A</td>
                        <td><span class="tag-block">CHẶN</span></td>
                        <td>explicit_rule_block</td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="card" style="grid-column: 1 / -1;">
            <h2>Bằng chứng Trực quan Thông minh (Smart Visual Verification)</h2>
            <p class="stat-sub">Ảnh tự động làm mờ cục bộ, chỉ chụp khi có vi phạm hoặc xin thêm giờ. Tự hủy sau 48h.</p>
            <div class="evidence-grid">
                <div class="evidence-card">
                    <div class="evidence-thumb">Ảnh 480p Đã làm mờ</div>
                    <div class="stat-sub" style="margin-top: 4px;">Chặn Game đêm</div>
                    <div class="countdown">Tự hủy: 47h 12m</div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
"#;
