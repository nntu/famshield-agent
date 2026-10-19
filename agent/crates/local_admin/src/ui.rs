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
        .header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 16px; margin-bottom: 24px; flex-wrap: wrap; gap: 12px; }
        .header-actions { display: flex; align-items: center; gap: 12px; }
        .badge { background: rgba(56, 189, 248, 0.2); color: var(--accent); padding: 6px 14px; border-radius: 9999px; font-size: 0.85rem; font-weight: 600; }
        .lang-btn-group { display: inline-flex; background: #0f172a; border-radius: 8px; padding: 2px; border: 1px solid var(--border); }
        .lang-btn { background: transparent; border: none; color: var(--text-muted); padding: 4px 10px; border-radius: 6px; cursor: pointer; font-size: 0.8rem; font-weight: 500; transition: all 0.15s ease; }
        .lang-btn.active { background: var(--accent); color: #0f172a; font-weight: 700; }
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
            <h1 data-i18n="title">FamShield Quản trị Cục bộ</h1>
            <p class="stat-sub" data-i18n="subtitle">Chế độ Độc lập (Zero-Cloud Standalone) • Cổng bảo vệ: 127.0.0.2:53</p>
        </div>
        <div class="header-actions">
            <div class="lang-btn-group">
                <button id="btn-lang-vi" class="lang-btn active" onclick="setLanguage('vi')">🇻🇳 Tiếng Việt</button>
                <button id="btn-lang-en" class="lang-btn" onclick="setLanguage('en')">🇺🇸 English</button>
            </div>
            <div class="badge" data-i18n="auth_success">Master PIN: Xác thực thành công</div>
        </div>
    </div>

    <div class="grid">
        <div class="card">
            <h2 data-i18n="card_app_title">Ứng dụng Foreground Hoạt động</h2>
            <div class="stat-value" id="active-app">Roblox</div>
            <div class="stat-sub" data-i18n="card_app_usage">Thời gian sử dụng hôm nay: 27 / 60 phút</div>
            <div class="quota-bar"><div class="quota-fill"></div></div>
        </div>

        <div class="card">
            <h2 data-i18n="card_dns_title">Bộ lọc DNS & WFP</h2>
            <div class="stat-value">127.0.0.2:53</div>
            <div class="stat-sub"><span data-i18n="wfp_session">User-Mode WFP Session:</span> <strong style="color: var(--accent-green)" data-i18n="status_protecting">Đang bảo vệ</strong></div>
            <div class="stat-sub" style="margin-top: 4px;" data-i18n="blocked_today">Tên miền đã chặn hôm nay: 18 lượt</div>
        </div>

        <div class="card" style="grid-column: 1 / -1;">
            <h2 data-i18n="dns_logs_title">Nhật ký Truy vấn Tên miền (DNS Query Logs)</h2>
            <table>
                <thead>
                    <tr>
                        <th data-i18n="th_time">Thời gian</th>
                        <th data-i18n="th_domain">Tên miền</th>
                        <th data-i18n="th_type">Loại</th>
                        <th data-i18n="th_action">Hành động</th>
                        <th data-i18n="th_reason">Lý do</th>
                    </tr>
                </thead>
                <tbody id="dns-logs">
                    <tr>
                        <td>17:05:12</td>
                        <td>roblox.com</td>
                        <td>A</td>
                        <td><span class="tag-allow" data-i18n="tag_allow">CHO PHÉP</span></td>
                        <td>default_action_allow</td>
                    </tr>
                    <tr>
                        <td>17:02:40</td>
                        <td>chat.example-social.invalid</td>
                        <td>A</td>
                        <td><span class="tag-block" data-i18n="tag_block">CHẶN</span></td>
                        <td>explicit_rule_block</td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="card" style="grid-column: 1 / -1;">
            <h2 data-i18n="evidence_title">Bằng chứng Trực quan Thông minh (Smart Visual Verification)</h2>
            <p class="stat-sub" data-i18n="evidence_desc">Ảnh tự động làm mờ cục bộ, chỉ chụp khi có vi phạm hoặc xin thêm giờ. Tự hủy sau 48h.</p>
            <div class="evidence-grid">
                <div class="evidence-card">
                    <div class="evidence-thumb" data-i18n="evidence_blurred">Ảnh 480p Đã làm mờ</div>
                    <div class="stat-sub" style="margin-top: 4px;" data-i18n="evidence_trigger_sample">Chặn Game đêm</div>
                    <div class="countdown" data-i18n="evidence_expires">Tự hủy: 47h 12m</div>
                </div>
            </div>
        </div>
    </div>

    <script>
        const translations = {
            vi: {
                title: "FamShield Quản trị Cục bộ",
                subtitle: "Chế độ Độc lập (Zero-Cloud Standalone) • Cổng bảo vệ: 127.0.0.2:53",
                auth_success: "Master PIN: Xác thực thành công",
                card_app_title: "Ứng dụng Foreground Hoạt động",
                card_app_usage: "Thời gian sử dụng hôm nay: 27 / 60 phút",
                card_dns_title: "Bộ lọc DNS & WFP",
                wfp_session: "User-Mode WFP Session:",
                status_protecting: "Đang bảo vệ",
                blocked_today: "Tên miền đã chặn hôm nay: 18 lượt",
                dns_logs_title: "Nhật ký Truy vấn Tên miền (DNS Query Logs)",
                th_time: "Thời gian",
                th_domain: "Tên miền",
                th_type: "Loại",
                th_action: "Hành động",
                th_reason: "Lý do",
                tag_allow: "CHO PHÉP",
                tag_block: "CHẶN",
                evidence_title: "Bằng chứng Trực quan Thông minh (Smart Visual Verification)",
                evidence_desc: "Ảnh tự động làm mờ cục bộ, chỉ chụp khi có vi phạm hoặc xin thêm giờ. Tự hủy sau 48h.",
                evidence_blurred: "Ảnh 480p Đã làm mờ",
                evidence_trigger_sample: "Chặn Game đêm",
                evidence_expires: "Tự hủy: 47h 12m"
            },
            en: {
                title: "FamShield Local Administration",
                subtitle: "Zero-Cloud Standalone Mode • Protected Loopback: 127.0.0.2:53",
                auth_success: "Master PIN: Authenticated",
                card_app_title: "Active Foreground Application",
                card_app_usage: "Used today: 27 / 60 minutes",
                card_dns_title: "DNS & WFP Engine",
                wfp_session: "User-Mode WFP Session:",
                status_protecting: "Protected",
                blocked_today: "Domains blocked today: 18 queries",
                dns_logs_title: "DNS Query Telemetry Logs",
                th_time: "Time",
                th_domain: "Domain",
                th_type: "Type",
                th_action: "Action",
                th_reason: "Reason",
                tag_allow: "ALLOW",
                tag_block: "BLOCK",
                evidence_title: "Smart Visual Verification",
                evidence_desc: "On-device blurred captures triggered on violation or time extension requests. Auto-purged after 48h.",
                evidence_blurred: "480p Blurred Image",
                evidence_trigger_sample: "Late night game block",
                evidence_expires: "Expires in: 47h 12m"
            }
        };

        function setLanguage(lang) {
            const selected = translations[lang] ? lang : 'vi';
            localStorage.setItem('famshield_local_lang', selected);
            document.documentElement.lang = selected;

            document.querySelectorAll('[data-i18n]').forEach(el => {
                const key = el.getAttribute('data-i18n');
                if (translations[selected] && translations[selected][key]) {
                    el.innerText = translations[selected][key];
                }
            });

            document.getElementById('btn-lang-vi').classList.toggle('active', selected === 'vi');
            document.getElementById('btn-lang-en').classList.toggle('active', selected === 'en');
        }

        // Initialize: Default is Vietnamese ('vi')
        const currentLang = localStorage.getItem('famshield_local_lang') || 'vi';
        setLanguage(currentLang);
    </script>
</body>
</html>
"#;
