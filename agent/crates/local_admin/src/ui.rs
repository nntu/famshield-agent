pub const LOCAL_DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FamShield - Bảng điều khiển Quản trị Cục bộ</title>
    <style>
        :root {
            --bg-primary: #0b1120;
            --bg-card: #1e293b;
            --bg-card-hover: #243248;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --accent: #38bdf8;
            --accent-hover: #0ea5e9;
            --accent-green: #34d399;
            --accent-red: #f87171;
            --accent-orange: #fbbf24;
            --border: #334155;
            --radius: 10px;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif; }
        body { background: var(--bg-primary); color: var(--text-main); min-height: 100vh; display: flex; flex-direction: column; }
        
        /* Top Navigation Header */
        header { background: #0f172a; border-bottom: 1px solid var(--border); padding: 14px 28px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px; position: sticky; top: 0; z-index: 50; }
        .logo-group { display: flex; align-items: center; gap: 12px; }
        .logo-icon { width: 34px; height: 34px; background: linear-gradient(135deg, #0284c7, #38bdf8); border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 1.25rem; font-weight: bold; color: #fff; box-shadow: 0 4px 12px rgba(56, 189, 248, 0.3); }
        .logo-title { font-size: 1.2rem; font-weight: 700; letter-spacing: -0.02em; }
        .mode-pill { background: rgba(56, 189, 248, 0.15); color: var(--accent); padding: 3px 10px; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; border: 1px solid rgba(56, 189, 248, 0.3); }
        
        .header-controls { display: flex; align-items: center; gap: 14px; }
        .lang-switch { display: inline-flex; background: #1e293b; border-radius: 8px; padding: 2px; border: 1px solid var(--border); }
        .lang-btn { background: transparent; border: none; color: var(--text-muted); padding: 5px 10px; border-radius: 6px; cursor: pointer; font-size: 0.8rem; font-weight: 500; transition: all 0.15s; }
        .lang-btn.active { background: var(--accent); color: #0b1120; font-weight: 700; }
        
        .auth-status-badge { display: flex; align-items: center; gap: 8px; background: rgba(52, 211, 153, 0.12); color: var(--accent-green); border: 1px solid rgba(52, 211, 153, 0.25); padding: 6px 12px; border-radius: 8px; font-size: 0.82rem; font-weight: 600; }
        .auth-dot { width: 8px; height: 8px; background: var(--accent-green); border-radius: 50%; box-shadow: 0 0 8px var(--accent-green); }
        .btn-logout { background: #334155; border: none; color: #cbd5e1; padding: 6px 12px; border-radius: 8px; cursor: pointer; font-size: 0.82rem; font-weight: 600; transition: all 0.15s; }
        .btn-logout:hover { background: #475569; color: #fff; }

        /* Navigation Tabs Bar */
        .tabs-nav { background: #0f172a; border-bottom: 1px solid var(--border); padding: 0 28px; display: flex; gap: 4px; overflow-x: auto; }
        .tab-btn { background: transparent; border: none; color: var(--text-muted); padding: 14px 18px; font-size: 0.92rem; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 8px; border-bottom: 3px solid transparent; transition: all 0.15s; white-space: nowrap; }
        .tab-btn:hover { color: #f1f5f9; }
        .tab-btn.active { color: var(--accent); border-bottom-color: var(--accent); }

        /* Main Container */
        main { flex: 1; padding: 28px; max-width: 1400px; width: 100%; margin: 0 auto; }
        .tab-content { display: none; }
        .tab-content.active { display: block; animation: fadeIn 0.2s ease-in-out; }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }

        /* Stat Grid */
        .stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; margin-bottom: 24px; }
        .card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); padding: 22px; position: relative; }
        .card-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 10px; }
        .card-title { font-size: 0.95rem; font-weight: 600; color: var(--text-muted); display: flex; align-items: center; gap: 8px; }
        .card-value { font-size: 2.1rem; font-weight: 700; color: var(--text-main); margin-bottom: 6px; }
        .card-sub { font-size: 0.85rem; color: var(--text-muted); }
        .quota-bar-wrapper { margin-top: 14px; }
        .quota-bar { height: 8px; background: #334155; border-radius: 4px; overflow: hidden; }
        .quota-fill { height: 100%; width: 45%; background: var(--accent-green); border-radius: 4px; transition: width 0.4s; }

        /* Tables & Lists */
        .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
        .section-title { font-size: 1.25rem; font-weight: 700; display: flex; align-items: center; gap: 8px; }
        table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
        th, td { text-align: left; padding: 12px 14px; border-bottom: 1px solid var(--border); }
        th { color: var(--text-muted); font-weight: 600; background: rgba(15, 23, 42, 0.4); text-transform: uppercase; font-size: 0.78rem; letter-spacing: 0.04em; }
        tr:hover { background: rgba(51, 65, 85, 0.25); }
        
        .tag-allow { background: rgba(52, 211, 153, 0.15); color: var(--accent-green); padding: 3px 10px; border-radius: 6px; font-weight: 700; font-size: 0.78rem; }
        .tag-block { background: rgba(248, 113, 113, 0.15); color: var(--accent-red); padding: 3px 10px; border-radius: 6px; font-weight: 700; font-size: 0.78rem; }
        .tag-gray { background: #334155; color: #94a3b8; padding: 3px 8px; border-radius: 6px; font-size: 0.78rem; }

        /* Forms & Buttons */
        .form-row { display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; }
        input[type="text"], input[type="password"], input[type="number"], select {
            background: #0f172a; border: 1px solid var(--border); color: #fff; padding: 10px 14px; border-radius: 8px; font-size: 0.9rem; outline: none; transition: border-color 0.15s;
        }
        input:focus { border-color: var(--accent); }
        .btn-primary { background: var(--accent); color: #0b1120; border: none; padding: 10px 18px; border-radius: 8px; font-weight: 700; font-size: 0.9rem; cursor: pointer; display: inline-flex; align-items: center; gap: 8px; transition: background 0.15s; }
        .btn-primary:hover { background: var(--accent-hover); }
        .btn-danger { background: rgba(248, 113, 113, 0.15); color: var(--accent-red); border: 1px solid rgba(248, 113, 113, 0.3); padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; font-weight: 600; cursor: pointer; transition: all 0.15s; }
        .btn-danger:hover { background: var(--accent-red); color: #0b1120; }
        .btn-secondary { background: #334155; color: #f8fafc; border: none; padding: 8px 14px; border-radius: 8px; font-size: 0.85rem; font-weight: 600; cursor: pointer; }
        .btn-secondary:hover { background: #475569; }

        /* Search & Filters */
        .filter-group { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
        .filter-btn { background: #1e293b; border: 1px solid var(--border); color: var(--text-muted); padding: 6px 12px; border-radius: 6px; cursor: pointer; font-size: 0.82rem; font-weight: 600; }
        .filter-btn.active { background: var(--accent); color: #0b1120; border-color: var(--accent); }

        /* Evidence Cards */
        .evidence-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 18px; margin-top: 18px; }
        .evidence-card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); overflow: hidden; display: flex; flex-direction: column; transition: transform 0.15s, border-color 0.15s; }
        .evidence-card:hover { transform: translateY(-2px); border-color: #475569; }
        .evidence-img-box { width: 100%; height: 160px; background: #0f172a; position: relative; overflow: hidden; display: flex; align-items: center; justify-content: center; }
        .evidence-img { width: 100%; height: 100%; object-fit: cover; filter: blur(3px); transition: filter 0.3s; }
        .evidence-img-badge { position: absolute; top: 10px; right: 10px; background: rgba(15, 23, 42, 0.85); backdrop-filter: blur(4px); color: var(--accent-orange); padding: 4px 8px; border-radius: 6px; font-size: 0.72rem; font-weight: 700; border: 1px solid rgba(251, 191, 36, 0.3); }
        .evidence-info { padding: 14px; flex: 1; display: flex; flex-direction: column; justify-content: space-between; }
        .evidence-trigger { font-weight: 700; font-size: 0.95rem; margin-bottom: 4px; color: #f1f5f9; }
        .evidence-meta { font-size: 0.8rem; color: var(--text-muted); margin-bottom: 10px; }
        .evidence-countdown { font-size: 0.78rem; color: var(--accent); font-weight: 600; display: flex; align-items: center; gap: 4px; }

        /* Security Cards */
        .security-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(380px, 1fr)); gap: 24px; }
        .security-card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); padding: 24px; }
        .recovery-key-box { background: #0f172a; border: 1px dashed var(--accent); padding: 14px; border-radius: 8px; font-family: monospace; font-size: 1.1rem; color: var(--accent-green); text-align: center; margin: 14px 0; letter-spacing: 0.1em; user-select: all; }

        /* PIN Login Modal */
        .modal-overlay { position: fixed; inset: 0; background: rgba(11, 17, 32, 0.85); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 100; padding: 20px; }
        .pin-modal { background: #1e293b; border: 1px solid var(--border); border-radius: 16px; padding: 32px; width: 100%; max-width: 420px; text-align: center; box-shadow: 0 20px 40px rgba(0,0,0,0.5); }
        .pin-inputs { display: flex; justify-content: center; gap: 10px; margin: 24px 0; }
        .pin-digit { width: 44px; height: 52px; background: #0f172a; border: 2px solid var(--border); border-radius: 10px; font-size: 1.6rem; color: #fff; text-align: center; outline: none; }
        .pin-digit:focus { border-color: var(--accent); box-shadow: 0 0 12px rgba(56, 189, 248, 0.3); }
        .keypad { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-top: 14px; }
        .keypad-btn { background: #0f172a; border: 1px solid var(--border); color: #f8fafc; font-size: 1.3rem; font-weight: 600; padding: 14px; border-radius: 10px; cursor: pointer; transition: background 0.15s; }
        .keypad-btn:hover { background: #334155; }
        .keypad-btn:active { background: var(--accent); color: #0b1120; }
        .modal-error { color: var(--accent-red); font-size: 0.85rem; margin-top: 12px; min-height: 20px; }

        /* Toast Notifications */
        .toast-box { position: fixed; bottom: 24px; right: 24px; z-index: 200; display: flex; flex-direction: column; gap: 8px; }
        .toast { background: #1e293b; border: 1px solid var(--border); color: #fff; padding: 12px 18px; border-radius: 8px; font-size: 0.88rem; box-shadow: 0 8px 24px rgba(0,0,0,0.4); display: flex; align-items: center; gap: 10px; animation: slideIn 0.25s ease-out; }
        @keyframes slideIn { from { transform: translateX(100%); } to { transform: translateX(0); } }

        /* Privacy Notice Banner */
        .privacy-banner { background: rgba(56, 189, 248, 0.08); border: 1px solid rgba(56, 189, 248, 0.25); border-radius: var(--radius); padding: 16px 20px; margin-bottom: 20px; display: flex; align-items: center; gap: 14px; }
        .privacy-icon { font-size: 1.8rem; }
    </style>
</head>
<body>

    <!-- Header -->
    <header>
        <div class="logo-group">
            <div class="logo-icon">🛡️</div>
            <div>
                <div class="logo-title">FamShield</div>
                <div style="font-size: 0.75rem; color: var(--text-muted);" data-i18n="header_sub">Bảo Vệ Gia Đình Cục Bộ</div>
            </div>
            <span class="mode-pill" id="badge-mode">StandaloneFree</span>
        </div>

        <div class="header-controls">
            <div class="lang-switch">
                <button id="btn-lang-vi" class="lang-btn active" onclick="setLanguage('vi')">🇻🇳 VN</button>
                <button id="btn-lang-en" class="lang-btn" onclick="setLanguage('en')">🇺🇸 EN</button>
            </div>
            <div class="auth-status-badge">
                <div class="auth-dot"></div>
                <span id="auth-status-text" data-i18n="status_authenticated">Đã xác thực</span>
            </div>
            <button class="btn-logout" onclick="logout()" data-i18n="btn_logout">Đăng xuất</button>
        </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="tabs-nav">
        <button class="tab-btn active" onclick="switchTab('dashboard')" data-i18n="nav_dashboard">📊 Tổng quan</button>
        <button class="tab-btn" onclick="switchTab('rules')" data-i18n="nav_rules">🛡️ Quy tắc & Giới hạn</button>
        <button class="tab-btn" onclick="switchTab('logs')" data-i18n="nav_logs">📋 Nhật ký DNS</button>
        <button class="tab-btn" onclick="switchTab('evidence')" data-i18n="nav_evidence">🖼️ Bằng chứng Vi phạm</button>
        <button class="tab-btn" onclick="switchTab('security')" data-i18n="nav_security">⚙️ Bảo mật & Ghép nối</button>
    </nav>

    <!-- Main Content -->
    <main>
        <!-- 1. DASHBOARD TAB -->
        <section id="tab-dashboard" class="tab-content active">
            <div class="stat-grid">
                <div class="card">
                    <div class="card-header">
                        <div class="card-title" data-i18n="card_app_title">Ứng dụng Hoạt động</div>
                        <span class="tag-allow" id="app-tag" data-i18n="tag_active">Hoạt động</span>
                    </div>
                    <div class="card-value" id="dash-active-app">Roblox.exe</div>
                    <div class="card-sub" id="dash-quota-text" data-i18n="dash_quota_text">Đã dùng: 27 / 60 phút hôm nay</div>
                    <div class="quota-bar-wrapper">
                        <div class="quota-bar"><div class="quota-fill" id="dash-quota-bar" style="width: 45%;"></div></div>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <div class="card-title" data-i18n="card_dns_title">Bộ lọc DNS Loopback</div>
                        <span class="tag-allow">127.0.0.2:53</span>
                    </div>
                    <div class="card-value" style="color: var(--accent-green);" data-i18n="status_protecting">Đang bảo vệ</div>
                    <div class="card-sub" id="dash-dns-sub" data-i18n="dash_dns_sub">Dynamic WFP Session • Không rò rỉ Port 53</div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <div class="card-title" data-i18n="card_blocked_today">Tên miền Đã chặn Hôm nay</div>
                        <span class="tag-block" id="dash-blocked-badge">CHẶN</span>
                    </div>
                    <div class="card-value" id="dash-blocked-count">0</div>
                    <div class="card-sub" id="dash-total-queries">Tổng số 0 truy vấn DNS ghi nhận</div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <div class="card-title" data-i18n="card_rules_title">Quy tắc Áp dụng</div>
                        <span class="tag-gray" id="dash-rules-count">0 luật</span>
                    </div>
                    <div class="card-value" id="dash-rules-num">0</div>
                    <div class="card-sub" data-i18n="dash_rules_sub">Chặn tên miền, SafeSearch, giới hạn app</div>
                </div>
            </div>

            <!-- Mini Recent Logs Feed -->
            <div class="card">
                <div class="section-header">
                    <div class="section-title" data-i18n="recent_logs_title">Truy vấn DNS Gần nhất</div>
                    <button class="btn-secondary" onclick="switchTab('logs')" data-i18n="btn_view_all_logs">Xem tất cả nhật ký &rarr;</button>
                </div>
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
                    <tbody id="dash-mini-logs">
                        <tr><td colspan="5" style="text-align:center; color:var(--text-muted);" data-i18n="loading">Đang tải dữ liệu...</td></tr>
                    </tbody>
                </table>
            </div>
        </section>

        <!-- 2. RULES & LIMITS TAB -->
        <section id="tab-rules" class="tab-content">
            <div class="card" style="margin-bottom: 24px;">
                <div class="section-header">
                    <div class="section-title" data-i18n="title_domain_block">🌐 Chặn Tên Miền (Domain Blocklist)</div>
                </div>
                <p class="card-sub" style="margin-bottom: 16px;" data-i18n="desc_domain_block">
                    Tên miền bị chặn sẽ được trả về 0.0.0.0 ngay tại cổng DNS loopback 127.0.0.2:53 trong 0ms. Tự động áp dụng cho mọi tên miền phụ (*.domain.com).
                </p>
                <div class="form-row">
                    <input type="text" id="rule-domain-input" placeholder="ví dụ: tiktok.com, roblox.com..." style="flex: 2; min-width: 220px;">
                    <input type="text" id="rule-domain-reason" placeholder="Lý do chặn (tùy chọn)" style="flex: 2; min-width: 200px;">
                    <button class="btn-primary" onclick="addDomainRule()" data-i18n="btn_add_domain">➕ Thêm Tên Miền Chặn</button>
                </div>

                <table>
                    <thead>
                        <tr>
                            <th data-i18n="th_rule_target">Tên miền / Mục tiêu</th>
                            <th data-i18n="th_rule_type">Loại</th>
                            <th data-i18n="th_date_added">Ngày thêm</th>
                            <th data-i18n="th_status">Trạng thái</th>
                            <th data-i18n="th_action">Thao tác</th>
                        </tr>
                    </thead>
                    <tbody id="rules-domain-table">
                        <tr><td colspan="5" style="text-align:center; color:var(--text-muted);" data-i18n="loading">Đang tải danh sách...</td></tr>
                    </tbody>
                </table>
            </div>

            <div class="card">
                <div class="section-header">
                    <div class="section-title" data-i18n="title_app_limits">📱 Giới Hạn Ứng Dụng (App Limits & Quotas)</div>
                </div>
                <p class="card-sub" style="margin-bottom: 16px;" data-i18n="desc_app_limits">
                    Theo dõi thời gian sử dụng foreground qua ETW Kernel Monitor. Khóa hoặc cảnh báo khi vượt quá hạn mức mỗi ngày.
                </p>
                <div class="form-row">
                    <input type="text" id="rule-app-name" placeholder="Tên file: roblox.exe, genshin.exe..." style="flex: 2; min-width: 220px;">
                    <input type="number" id="rule-app-quota" placeholder="Phút/ngày (ví dụ 60)" style="flex: 1; min-width: 140px;" min="1" max="1440">
                    <button class="btn-primary" onclick="addAppRule()" data-i18n="btn_add_app">➕ Đặt Hạn Mức App</button>
                </div>

                <table>
                    <thead>
                        <tr>
                            <th data-i18n="th_app_name">Ứng dụng</th>
                            <th data-i18n="th_daily_limit">Hạn mức / ngày</th>
                            <th data-i18n="th_status">Trạng thái</th>
                            <th data-i18n="th_action">Thao tác</th>
                        </tr>
                    </thead>
                    <tbody id="rules-app-table">
                        <tr><td colspan="4" style="text-align:center; color:var(--text-muted);" data-i18n="loading">Đang tải danh sách...</td></tr>
                    </tbody>
                </table>
            </div>
        </section>

        <!-- 3. DNS LOGS TAB -->
        <section id="tab-logs" class="tab-content">
            <div class="card">
                <div class="section-header">
                    <div>
                        <div class="section-title" data-i18n="dns_logs_title">Nhật Ký Truy Vấn Tên Miền (DNS Telemetry Logs)</div>
                        <p class="card-sub" data-i18n="dns_logs_desc">Ghi nhận mọi yêu cầu phân giải tên miền từ các ứng dụng trên thiết bị</p>
                    </div>
                    <div class="filter-group">
                        <input type="text" id="logs-search" placeholder="🔍 Tìm tên miền..." oninput="filterLogs()" style="width: 200px;">
                        <button class="filter-btn active" id="btn-filter-all" onclick="setLogFilter('all')" data-i18n="filter_all">Tất cả</button>
                        <button class="filter-btn" id="btn-filter-block" onclick="setLogFilter('block')" data-i18n="filter_block">Chỉ chặn</button>
                        <button class="filter-btn" id="btn-filter-allow" onclick="setLogFilter('allow')" data-i18n="filter_allow">Cho phép</button>
                        <button class="btn-secondary" onclick="fetchLogs()" data-i18n="btn_refresh">🔄 Làm mới</button>
                    </div>
                </div>

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
                    <tbody id="logs-table-body">
                        <tr><td colspan="5" style="text-align:center; color:var(--text-muted);" data-i18n="loading">Đang tải nhật ký...</td></tr>
                    </tbody>
                </table>
            </div>
        </section>

        <!-- 4. VISUAL EVIDENCE TAB -->
        <section id="tab-evidence" class="tab-content">
            <div class="privacy-banner">
                <div class="privacy-icon">🛡️</div>
                <div>
                    <strong style="color: var(--accent);" data-i18n="privacy_title">Quyền Riêng Tư Là Tối Thượng (Privacy-by-Design)</strong>
                    <p style="font-size: 0.85rem; color: var(--text-muted); margin-top: 2px;" data-i18n="privacy_desc">
                        Toàn bộ ảnh chụp sự kiện vi phạm được làm mờ Gaussian trực tiếp trong RAM trước khi lưu xuống đĩa. Không bao giờ lưu ảnh sắc nét. Tự động xóa vĩnh viễn sau 48 giờ.
                    </p>
                </div>
            </div>

            <div class="section-header">
                <div class="section-title" data-i18n="evidence_title">Thư Viện Bằng Chứng Trực Quan (Smart Visual Verification)</div>
                <button class="btn-secondary" onclick="fetchEvidence()" data-i18n="btn_refresh">🔄 Làm mới</button>
            </div>

            <div class="evidence-grid" id="evidence-container">
                <div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 40px;" data-i18n="loading">
                    Đang tải danh sách bằng chứng...
                </div>
            </div>
        </section>

        <!-- 5. SECURITY & CLOUD TAB -->
        <section id="tab-security" class="tab-content">
            <div class="security-grid">
                <!-- Change Master PIN -->
                <div class="security-card">
                    <h2 class="section-title" style="margin-bottom: 12px;" data-i18n="title_change_pin">🔐 Đổi Mã Master PIN</h2>
                    <p class="card-sub" style="margin-bottom: 20px;" data-i18n="desc_change_pin">Mã Master PIN dùng để truy cập vào bảng quản trị và phê duyệt trên máy trạm.</p>
                    
                    <div style="margin-bottom: 14px;">
                        <label style="display:block; font-size:0.85rem; margin-bottom:6px; color:var(--text-muted);" data-i18n="lbl_curr_pin">Mã PIN Hiện tại (6 số):</label>
                        <input type="password" id="pin-current" maxlength="8" style="width:100%;">
                    </div>
                    <div style="margin-bottom: 14px;">
                        <label style="display:block; font-size:0.85rem; margin-bottom:6px; color:var(--text-muted);" data-i18n="lbl_new_pin">Mã PIN Mới (6 số):</label>
                        <input type="password" id="pin-new" maxlength="8" style="width:100%;">
                    </div>
                    <div style="margin-bottom: 18px;">
                        <label style="display:block; font-size:0.85rem; margin-bottom:6px; color:var(--text-muted);" data-i18n="lbl_confirm_pin">Xác nhận Mã PIN Mới:</label>
                        <input type="password" id="pin-confirm" maxlength="8" style="width:100%;">
                    </div>
                    <button class="btn-primary" style="width:100%; justify-content:center;" onclick="changeMasterPin()" data-i18n="btn_save_pin">Lưu Mã PIN Mới</button>
                    
                    <div id="pin-success-box" style="display:none; margin-top:16px;">
                        <div style="font-size:0.85rem; color:var(--accent-green); margin-bottom:4px;" data-i18n="pin_change_success">Đổi PIN thành công! Hãy lưu lại Mã Khôi Phục:</div>
                        <div class="recovery-key-box" id="returned-recovery-key"></div>
                    </div>
                </div>

                <!-- Cloud Linking Wizard -->
                <div class="security-card">
                    <h2 class="section-title" style="margin-bottom: 12px;" data-i18n="title_cloud_link">☁️ Ghép Nối Cloud Server (Freemium)</h2>
                    <p class="card-sub" style="margin-bottom: 16px;" data-i18n="desc_cloud_link">
                        Nâng cấp từ chế độ Ngoại tuyến (StandaloneFree) lên Quản lý Từ xa (OnlineManaged) qua Telegram Bot hoặc Cloud Web Console.
                    </p>

                    <div style="background:#0f172a; border-radius:8px; padding:14px; margin-bottom:16px; border:1px solid var(--border);">
                        <div style="font-size:0.85rem; color:var(--text-muted);" data-i18n="lbl_curr_mode">Chế độ hiện tại:</div>
                        <div style="font-size:1.1rem; font-weight:700; color:var(--accent); margin:4px 0;" id="sec-curr-mode">StandaloneFree (100% Ngoại tuyến Miễn phí)</div>
                        <div style="font-size:0.78rem; color:var(--text-muted);" id="sec-device-id">ID thiết bị: dev-local-001</div>
                    </div>

                    <div style="margin-bottom: 16px;">
                        <label style="display:block; font-size:0.85rem; margin-bottom:6px; color:var(--text-muted);" data-i18n="lbl_enroll_code">Mã Ghép Nối 6 Chữ Số:</label>
                        <input type="text" id="cloud-enroll-code" maxlength="6" placeholder="123456" style="width:100%; font-size:1.2rem; letter-spacing:0.2em; text-align:center;">
                    </div>

                    <div style="display:flex; gap:10px;">
                        <button class="btn-primary" style="flex:1; justify-content:center;" onclick="enrollCloud()" data-i18n="btn_link_cloud">Ghép Nối Cloud</button>
                        <button class="btn-secondary" onclick="disenrollCloud()" data-i18n="btn_unlink_cloud">Hủy Ghép Nối</button>
                    </div>
                </div>
            </div>
        </section>
    </main>

    <!-- Master PIN Login Modal -->
    <div class="modal-overlay" id="login-modal" style="display:none;">
        <div class="pin-modal">
            <div style="font-size: 2.2rem; margin-bottom: 10px;">🛡️</div>
            <h2 style="font-size: 1.3rem; margin-bottom: 6px;" data-i18n="modal_pin_title">Nhập Master PIN Phụ Huynh</h2>
            <p style="font-size: 0.85rem; color: var(--text-muted);" data-i18n="modal_pin_sub">Vui lòng nhập 6 chữ số để mở khóa bảng điều khiển</p>

            <div class="pin-inputs">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d0" oninput="pinInput(0)" onkeydown="pinKeydown(event, 0)">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d1" oninput="pinInput(1)" onkeydown="pinKeydown(event, 1)">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d2" oninput="pinInput(2)" onkeydown="pinKeydown(event, 2)">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d3" oninput="pinInput(3)" onkeydown="pinKeydown(event, 3)">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d4" oninput="pinInput(4)" onkeydown="pinKeydown(event, 4)">
                <input type="password" class="pin-digit" maxlength="1" id="pin-d5" oninput="pinInput(5)" onkeydown="pinKeydown(event, 5)">
            </div>

            <div class="keypad">
                <button class="keypad-btn" onclick="pressKey('1')">1</button>
                <button class="keypad-btn" onclick="pressKey('2')">2</button>
                <button class="keypad-btn" onclick="pressKey('3')">3</button>
                <button class="keypad-btn" onclick="pressKey('4')">4</button>
                <button class="keypad-btn" onclick="pressKey('5')">5</button>
                <button class="keypad-btn" onclick="pressKey('6')">6</button>
                <button class="keypad-btn" onclick="pressKey('7')">7</button>
                <button class="keypad-btn" onclick="pressKey('8')">8</button>
                <button class="keypad-btn" onclick="pressKey('9')">9</button>
                <button class="keypad-btn" onclick="clearPin()" style="font-size:0.9rem;">C</button>
                <button class="keypad-btn" onclick="pressKey('0')">0</button>
                <button class="keypad-btn" onclick="backspacePin()" style="font-size:0.9rem;">⌫</button>
            </div>

            <div class="modal-error" id="login-error-msg"></div>
        </div>
    </div>

    <!-- Toast Notification Container -->
    <div class="toast-box" id="toast-box"></div>

    <script>
        const i18n = {
            vi: {
                header_sub: "Bảo Vệ Gia Đình Cục Bộ",
                status_authenticated: "Đã xác thực",
                btn_logout: "Đăng xuất",
                nav_dashboard: "📊 Tổng quan",
                nav_rules: "🛡️ Quy tắc & Giới hạn",
                nav_logs: "📋 Nhật ký DNS",
                nav_evidence: "🖼️ Bằng chứng Vi phạm",
                nav_security: "⚙️ Bảo mật & Ghép nối",
                card_app_title: "Ứng dụng Hoạt động",
                tag_active: "Hoạt động",
                dash_quota_text: "Đã dùng: {used} / {total} phút hôm nay",
                card_dns_title: "Bộ lọc DNS Loopback",
                status_protecting: "Đang bảo vệ",
                dash_dns_sub: "Dynamic WFP Session • Không rò rỉ Port 53",
                card_blocked_today: "Tên miền Đã chặn Hôm nay",
                dash_rules_title: "Quy tắc Áp dụng",
                dash_rules_sub: "Chặn tên miền, SafeSearch, giới hạn app",
                recent_logs_title: "Truy vấn DNS Gần nhất",
                btn_view_all_logs: "Xem tất cả nhật ký &rarr;",
                th_time: "Thời gian",
                th_domain: "Tên miền",
                th_type: "Loại",
                th_action: "Hành động",
                th_reason: "Lý do",
                loading: "Đang tải dữ liệu...",
                title_domain_block: "🌐 Chặn Tên Miền (Domain Blocklist)",
                desc_domain_block: "Tên miền bị chặn sẽ được trả về 0.0.0.0 ngay tại cổng DNS loopback 127.0.0.2:53 trong 0ms. Tự động áp dụng cho mọi tên miền phụ (*.domain.com).",
                btn_add_domain: "➕ Thêm Tên Miền Chặn",
                th_rule_target: "Tên miền / Mục tiêu",
                th_rule_type: "Loại",
                th_date_added: "Ngày thêm",
                th_status: "Trạng thái",
                title_app_limits: "📱 Giới Hạn Ứng Dụng (App Limits & Quotas)",
                desc_app_limits: "Theo dõi thời gian sử dụng foreground qua ETW Kernel Monitor. Khóa hoặc cảnh báo khi vượt quá hạn mức mỗi ngày.",
                btn_add_app: "➕ Đặt Hạn Mức App",
                th_app_name: "Ứng dụng",
                th_daily_limit: "Hạn mức / ngày",
                dns_logs_title: "Nhật Ký Truy Vấn Tên Miền (DNS Telemetry Logs)",
                dns_logs_desc: "Ghi nhận mọi yêu cầu phân giải tên miền từ các ứng dụng trên thiết bị",
                filter_all: "Tất cả",
                filter_block: "Chỉ chặn",
                filter_allow: "Cho phép",
                btn_refresh: "🔄 Làm mới",
                privacy_title: "Quyền Riêng Tư Là Tối Thượng (Privacy-by-Design)",
                privacy_desc: "Toàn bộ ảnh chụp sự kiện vi phạm được làm mờ Gaussian trực tiếp trong RAM trước khi lưu xuống đĩa. Không bao giờ lưu ảnh sắc nét. Tự động xóa vĩnh viễn sau 48 giờ.",
                evidence_title: "Thư Viện Bằng Chứng Trực Quan (Smart Visual Verification)",
                title_change_pin: "🔐 Đổi Mã Master PIN",
                desc_change_pin: "Mã Master PIN dùng để truy cập vào bảng quản trị và phê duyệt trên máy trạm.",
                lbl_curr_pin: "Mã PIN Hiện tại (6 số):",
                lbl_new_pin: "Mã PIN Mới (6 số):",
                lbl_confirm_pin: "Xác nhận Mã PIN Mới:",
                btn_save_pin: "Lưu Mã PIN Mới",
                pin_change_success: "Đổi PIN thành công! Hãy lưu lại Mã Khôi Phục:",
                title_cloud_link: "☁️ Ghép Nối Cloud Server (Freemium)",
                desc_cloud_link: "Nâng cấp từ chế độ Ngoại tuyến (StandaloneFree) lên Quản lý Từ xa (OnlineManaged) qua Telegram Bot hoặc Cloud Web Console.",
                lbl_curr_mode: "Chế độ hiện tại:",
                lbl_enroll_code: "Mã Ghép Nối 6 Chữ Số:",
                btn_link_cloud: "Ghép Nối Cloud",
                btn_unlink_cloud: "Hủy Ghép Nối",
                modal_pin_title: "Nhập Master PIN Phụ Huynh",
                modal_pin_sub: "Vui lòng nhập 6 chữ số để mở khóa bảng điều khiển"
            },
            en: {
                header_sub: "Local Guardian Protection",
                status_authenticated: "Authenticated",
                btn_logout: "Logout",
                nav_dashboard: "📊 Dashboard",
                nav_rules: "🛡️ Rules & Limits",
                nav_logs: "📋 DNS Logs",
                nav_evidence: "🖼️ Visual Evidence",
                nav_security: "⚙️ Security & Cloud",
                card_app_title: "Active Foreground App",
                tag_active: "Active",
                dash_quota_text: "Used: {used} / {total} minutes today",
                card_dns_title: "Loopback DNS Filter",
                status_protecting: "Protected",
                dash_dns_sub: "Dynamic WFP Session • Zero Port 53 Leakage",
                card_blocked_today: "Domains Blocked Today",
                dash_rules_title: "Active Rules",
                dash_rules_sub: "Domain blocks, SafeSearch, app quotas",
                recent_logs_title: "Recent DNS Queries",
                btn_view_all_logs: "View All Logs &rarr;",
                th_time: "Time",
                th_domain: "Domain",
                th_type: "Type",
                th_action: "Action",
                th_reason: "Reason",
                loading: "Loading data...",
                title_domain_block: "🌐 Domain Blocklist",
                desc_domain_block: "Blocked domains resolve to 0.0.0.0 on loopback 127.0.0.2:53 in 0ms. Automatically applies to all subdomains (*.domain.com).",
                btn_add_domain: "➕ Block Domain",
                th_rule_target: "Target / Domain",
                th_rule_type: "Type",
                th_date_added: "Date Added",
                th_status: "Status",
                title_app_limits: "📱 Application Limits & Quotas",
                desc_app_limits: "Monitors active foreground duration via Kernel ETW. Blocks or alerts upon quota expiration.",
                btn_add_app: "➕ Set App Limit",
                th_app_name: "Application",
                th_daily_limit: "Daily Limit",
                dns_logs_title: "DNS Telemetry Logs",
                dns_logs_desc: "Comprehensive log of domain resolution queries across all device applications",
                filter_all: "All",
                filter_block: "Blocked Only",
                filter_allow: "Allowed Only",
                btn_refresh: "🔄 Refresh",
                privacy_title: "Privacy-by-Design Paramount",
                privacy_desc: "All violation screenshots are Gaussian blurred in RAM before persisting to disk. Raw captures are never stored. Automatically purged after 48h.",
                evidence_title: "Smart Visual Verification Gallery",
                title_change_pin: "🔐 Change Master PIN",
                desc_change_pin: "Master PIN authorizes access to local management and on-device overrides.",
                lbl_curr_pin: "Current PIN (6 digits):",
                lbl_new_pin: "New PIN (6 digits):",
                lbl_confirm_pin: "Confirm New PIN:",
                btn_save_pin: "Save New PIN",
                pin_change_success: "PIN successfully updated! Please save your Recovery Key:",
                title_cloud_link: "☁️ Cloud Link Wizard (Freemium)",
                desc_cloud_link: "Transition seamlessly between 100% Offline Standalone and Online Managed fleet controls.",
                lbl_curr_mode: "Operational Mode:",
                lbl_enroll_code: "6-digit Enrollment Code:",
                btn_link_cloud: "Link to Cloud",
                btn_unlink_cloud: "Unlink from Cloud",
                modal_pin_title: "Guardian Master PIN",
                modal_pin_sub: "Enter your 6-digit Master PIN to unlock console"
            }
        };

        let currentLang = localStorage.getItem('famshield_lang') || 'vi';
        let sessionToken = localStorage.getItem('famshield_token') || '';
        let allLogs = [];
        let logFilter = 'all';

        function setLanguage(lang) {
            currentLang = lang;
            localStorage.setItem('famshield_lang', lang);
            document.documentElement.lang = lang;
            document.getElementById('btn-lang-vi').classList.toggle('active', lang === 'vi');
            document.getElementById('btn-lang-en').classList.toggle('active', lang === 'en');

            document.querySelectorAll('[data-i18n]').forEach(el => {
                const key = el.getAttribute('data-i18n');
                if (i18n[lang] && i18n[lang][key]) {
                    el.innerHTML = i18n[lang][key];
                }
            });
        }

        function showToast(msg, type = 'info') {
            const box = document.getElementById('toast-box');
            const toast = document.createElement('div');
            toast.className = 'toast';
            toast.style.borderColor = type === 'error' ? 'var(--accent-red)' : type === 'success' ? 'var(--accent-green)' : 'var(--border)';
            toast.innerHTML = (type === 'error' ? '⚠️ ' : type === 'success' ? '✅ ' : 'ℹ️ ') + msg;
            box.appendChild(toast);
            setTimeout(() => { toast.remove(); }, 3500);
        }

        // Authentication & Session
        async function checkSession() {
            // Check URL query parameters for token (e.g. from tray menu: ?token=xxx)
            const urlParams = new URLSearchParams(window.location.search);
            const urlToken = urlParams.get('token') || urlParams.get('auth_token');
            if (urlToken) {
                sessionToken = urlToken;
                localStorage.setItem('famshield_token', urlToken);
                window.history.replaceState({}, document.title, window.location.pathname);
            }

            if (!sessionToken) {
                document.getElementById('login-modal').style.display = 'flex';
                focusPinDigit(0);
                return;
            }

            try {
                const res = await fetch('/api/auth/verify', {
                    headers: { 'X-FamShield-Token': sessionToken }
                });
                const data = await res.json();
                if (data.authenticated) {
                    document.getElementById('login-modal').style.display = 'none';
                    initDashboard();
                } else {
                    document.getElementById('login-modal').style.display = 'flex';
                    focusPinDigit(0);
                }
            } catch (e) {
                document.getElementById('login-modal').style.display = 'flex';
            }
        }

        async function logout() {
            try {
                await fetch('/api/auth/logout', {
                    method: 'POST',
                    headers: { 'X-FamShield-Token': sessionToken }
                });
            } catch (e) {}
            sessionToken = '';
            localStorage.removeItem('famshield_token');
            document.getElementById('login-modal').style.display = 'flex';
            clearPin();
        }

        // PIN Keypad & Input Handling
        let pinBuffer = ['', '', '', '', '', ''];
        function pressKey(num) {
            for (let i = 0; i < 6; i++) {
                if (pinBuffer[i] === '') {
                    pinBuffer[i] = num;
                    document.getElementById(`pin-d${i}`).value = '•';
                    if (i < 5) focusPinDigit(i + 1);
                    if (i === 5) submitPin();
                    break;
                }
            }
        }

        function clearPin() {
            pinBuffer = ['', '', '', '', '', ''];
            for (let i = 0; i < 6; i++) {
                document.getElementById(`pin-d${i}`).value = '';
            }
            document.getElementById('login-error-msg').innerText = '';
            focusPinDigit(0);
        }

        function backspacePin() {
            for (let i = 5; i >= 0; i--) {
                if (pinBuffer[i] !== '') {
                    pinBuffer[i] = '';
                    document.getElementById(`pin-d${i}`).value = '';
                    focusPinDigit(i);
                    break;
                }
            }
        }

        function focusPinDigit(idx) {
            const el = document.getElementById(`pin-d${idx}`);
            if (el) el.focus();
        }

        function pinInput(idx) {
            const el = document.getElementById(`pin-d${idx}`);
            const val = el.value.replace(/\D/g, '');
            if (val) {
                pinBuffer[idx] = val.slice(-1);
                el.value = '•';
                if (idx < 5) focusPinDigit(idx + 1);
                else submitPin();
            } else {
                pinBuffer[idx] = '';
            }
        }

        function pinKeydown(e, idx) {
            if (e.key === 'Backspace' && !pinBuffer[idx] && idx > 0) {
                focusPinDigit(idx - 1);
            } else if (e.key === 'Enter') {
                submitPin();
            }
        }

        async function submitPin() {
            const pin = pinBuffer.join('');
            if (pin.length < 6) return;

            const errEl = document.getElementById('login-error-msg');
            errEl.innerText = currentLang === 'vi' ? 'Đang xác thực...' : 'Verifying...';

            try {
                const res = await fetch('/api/auth/login', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ pin })
                });
                const data = await res.json();
                if (data.success && data.token) {
                    sessionToken = data.token;
                    localStorage.setItem('famshield_token', sessionToken);
                    document.getElementById('login-modal').style.display = 'none';
                    showToast(currentLang === 'vi' ? 'Xác thực Master PIN thành công!' : 'Authenticated successfully!', 'success');
                    initDashboard();
                } else {
                    errEl.innerText = data.error || (currentLang === 'vi' ? 'Mã PIN không đúng' : 'Invalid PIN');
                    clearPin();
                }
            } catch (e) {
                errEl.innerText = currentLang === 'vi' ? 'Lỗi kết nối máy chủ' : 'Server connection failed';
                clearPin();
            }
        }

        // Tabs
        function switchTab(tabId) {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

            const targetBtn = Array.from(document.querySelectorAll('.tab-btn')).find(b => b.getAttribute('onclick').includes(tabId));
            if (targetBtn) targetBtn.classList.add('active');

            const content = document.getElementById(`tab-${tabId}`);
            if (content) content.classList.add('active');

            if (tabId === 'rules') fetchRules();
            else if (tabId === 'logs') fetchLogs();
            else if (tabId === 'evidence') fetchEvidence();
            else if (tabId === 'dashboard') fetchStatus();
        }

        // API Calls with Session Token
        async function api(path, opts = {}) {
            opts.headers = opts.headers || {};
            opts.headers['X-FamShield-Token'] = sessionToken;
            const res = await fetch(path, opts);
            if (res.status === 401) {
                logout();
                throw new Error('Unauthorized');
            }
            return res;
        }

        // Data Loaders
        async function fetchStatus() {
            try {
                const res = await api('/api/status');
                const data = await res.json();
                document.getElementById('dash-active-app').innerText = data.active_foreground_app;
                document.getElementById('dash-blocked-count').innerText = data.blocked_queries_today;
                document.getElementById('dash-total-queries').innerText = `${currentLang === 'vi' ? 'Tổng số' : 'Total'} ${data.total_queries_today} ${currentLang === 'vi' ? 'truy vấn DNS ghi nhận' : 'queries recorded'}`;
                document.getElementById('dash-rules-num').innerText = data.rules_count;
                document.getElementById('dash-rules-count').innerText = `${data.rules_count} ${currentLang === 'vi' ? 'luật' : 'rules'}`;
                
                const pct = Math.min(100, Math.round((data.used_today_minutes / (data.daily_quota_minutes || 60)) * 100));
                document.getElementById('dash-quota-bar').style.width = `${pct}%`;
                document.getElementById('dash-quota-text').innerText = `${currentLang === 'vi' ? 'Đã dùng:' : 'Used:'} ${data.used_today_minutes} / ${data.daily_quota_minutes} ${currentLang === 'vi' ? 'phút hôm nay' : 'minutes today'}`;

                document.getElementById('badge-mode').innerText = data.operating_mode;
                document.getElementById('sec-curr-mode').innerText = data.operating_mode === 'StandaloneFree' ? 'StandaloneFree (100% Ngoại tuyến Miễn phí)' : 'OnlineManaged (Cloud Pro Tier)';
                document.getElementById('sec-device-id').innerText = `${currentLang === 'vi' ? 'ID thiết bị:' : 'Device ID:'} ${data.device_id}`;

                // Fetch mini logs
                const logsRes = await api('/api/logs/dns?limit=5');
                const logs = await logsRes.json();
                renderMiniLogs(logs);
            } catch (e) {}
        }

        function renderMiniLogs(logs) {
            const tbody = document.getElementById('dash-mini-logs');
            if (!logs || logs.length === 0) {
                tbody.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted);">${currentLang === 'vi' ? 'Chưa có dữ liệu truy vấn' : 'No query logs available'}</td></tr>`;
                return;
            }
            tbody.innerHTML = logs.map(l => {
                const isBlock = l.action === 'BLOCK';
                const timeStr = new Date(l.occurred_at).toLocaleTimeString();
                return `<tr>
                    <td>${timeStr}</td>
                    <td><strong>${l.domain}</strong></td>
                    <td><span class="tag-gray">${l.qtype}</span></td>
                    <td><span class="${isBlock ? 'tag-block' : 'tag-allow'}">${l.action}</span></td>
                    <td style="color:var(--text-muted); font-size:0.8rem;">${l.reason}</td>
                </tr>`;
            }).join('');
        }

        async function fetchRules() {
            try {
                const res = await api('/api/rules');
                const rules = await res.json();
                
                const domainRules = rules.filter(r => r.rule_type === 'domain');
                const appRules = rules.filter(r => r.rule_type === 'application');

                const domTable = document.getElementById('rules-domain-table');
                if (domainRules.length === 0) {
                    domTable.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted);">${currentLang === 'vi' ? 'Chưa có tên miền nào bị chặn' : 'No blocked domains configured'}</td></tr>`;
                } else {
                    domTable.innerHTML = domainRules.map(r => {
                        const dateStr = new Date(r.created_at).toLocaleDateString();
                        return `<tr>
                            <td><strong>${r.rule_id}</strong></td>
                            <td><span class="tag-gray">${r.rule_type}</span></td>
                            <td>${dateStr}</td>
                            <td><span class="${r.enabled ? 'tag-allow' : 'tag-gray'}">${r.enabled ? (currentLang === 'vi' ? 'Kích hoạt' : 'Enabled') : (currentLang === 'vi' ? 'Tắt' : 'Disabled')}</span></td>
                            <td><button class="btn-danger" onclick="deleteRule('${r.rule_id}')">${currentLang === 'vi' ? 'Xóa' : 'Delete'}</button></td>
                        </tr>`;
                    }).join('');
                }

                const appTable = document.getElementById('rules-app-table');
                if (appRules.length === 0) {
                    appTable.innerHTML = `<tr><td colspan="4" style="text-align:center; color:var(--text-muted);">${currentLang === 'vi' ? 'Chưa có giới hạn ứng dụng nào' : 'No application quotas configured'}</td></tr>`;
                } else {
                    appTable.innerHTML = appRules.map(r => {
                        let quota = '60 phút';
                        try {
                            const p = JSON.parse(r.payload);
                            if (p.max_minutes_per_day) quota = `${p.max_minutes_per_day} ${currentLang === 'vi' ? 'phút' : 'minutes'}`;
                        } catch (e) {}
                        return `<tr>
                            <td><strong>${r.rule_id}</strong></td>
                            <td>${quota}</td>
                            <td><span class="${r.enabled ? 'tag-allow' : 'tag-gray'}">${r.enabled ? (currentLang === 'vi' ? 'Áp dụng' : 'Active') : (currentLang === 'vi' ? 'Tắt' : 'Disabled')}</span></td>
                            <td><button class="btn-danger" onclick="deleteRule('${r.rule_id}')">${currentLang === 'vi' ? 'Xóa' : 'Delete'}</button></td>
                        </tr>`;
                    }).join('');
                }
            } catch (e) {}
        }

        async function addDomainRule() {
            const input = document.getElementById('rule-domain-input');
            const reasonInput = document.getElementById('rule-domain-reason');
            const domain = input.value.trim().toLowerCase();
            if (!domain) {
                showToast(currentLang === 'vi' ? 'Vui lòng nhập tên miền cần chặn' : 'Please enter domain to block', 'error');
                return;
            }

            const payload = JSON.stringify({
                action: 'BLOCK',
                reason: reasonInput.value.trim() || 'local_guardian_rule'
            });

            try {
                const res = await api('/api/rules', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        rule_id: domain,
                        rule_type: 'domain',
                        payload_json: payload,
                        enabled: true
                    })
                });
                if (res.ok) {
                    showToast(currentLang === 'vi' ? `Đã thêm chặn tên miền: ${domain}` : `Added domain block: ${domain}`, 'success');
                    input.value = '';
                    reasonInput.value = '';
                    fetchRules();
                    fetchStatus();
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi thêm quy tắc' : 'Failed to create rule', 'error');
            }
        }

        async function addAppRule() {
            const appInput = document.getElementById('rule-app-name');
            const quotaInput = document.getElementById('rule-app-quota');
            const app = appInput.value.trim().toLowerCase();
            const quota = parseInt(quotaInput.value.trim(), 10) || 60;
            if (!app) {
                showToast(currentLang === 'vi' ? 'Vui lòng nhập tên ứng dụng' : 'Please enter app name', 'error');
                return;
            }

            const payload = JSON.stringify({
                max_minutes_per_day: quota,
                action: 'LIMIT'
            });

            try {
                const res = await api('/api/rules', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        rule_id: app,
                        rule_type: 'application',
                        payload_json: payload,
                        enabled: true
                    })
                });
                if (res.ok) {
                    showToast(currentLang === 'vi' ? `Đã đặt giới hạn cho: ${app}` : `Set quota limit for: ${app}`, 'success');
                    appInput.value = '';
                    quotaInput.value = '';
                    fetchRules();
                    fetchStatus();
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi tạo giới hạn app' : 'Failed to set app limit', 'error');
            }
        }

        async function deleteRule(ruleId) {
            if (!confirm(currentLang === 'vi' ? `Bạn có chắc muốn xóa quy tắc '${ruleId}'?` : `Delete rule '${ruleId}'?`)) return;
            try {
                const res = await api(`/api/rules/${encodeURIComponent(ruleId)}`, { method: 'DELETE' });
                if (res.ok) {
                    showToast(currentLang === 'vi' ? 'Đã xóa quy tắc thành công' : 'Rule deleted', 'success');
                    fetchRules();
                    fetchStatus();
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi xóa quy tắc' : 'Failed to delete rule', 'error');
            }
        }

        async function fetchLogs() {
            try {
                const res = await api('/api/logs/dns?limit=150');
                allLogs = await res.json();
                renderLogs();
            } catch (e) {}
        }

        function setLogFilter(filter) {
            logFilter = filter;
            document.getElementById('btn-filter-all').classList.toggle('active', filter === 'all');
            document.getElementById('btn-filter-block').classList.toggle('active', filter === 'block');
            document.getElementById('btn-filter-allow').classList.toggle('active', filter === 'allow');
            renderLogs();
        }

        function filterLogs() {
            renderLogs();
        }

        function renderLogs() {
            const query = (document.getElementById('logs-search').value || '').trim().toLowerCase();
            const tbody = document.getElementById('logs-table-body');

            let filtered = allLogs.filter(l => {
                if (logFilter === 'block' && l.action !== 'BLOCK') return false;
                if (logFilter === 'allow' && l.action !== 'ALLOW') return false;
                if (query && !l.domain.toLowerCase().includes(query)) return false;
                return true;
            });

            if (filtered.length === 0) {
                tbody.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted);">${currentLang === 'vi' ? 'Không tìm thấy nhật ký phù hợp' : 'No matching logs found'}</td></tr>`;
                return;
            }

            tbody.innerHTML = filtered.map(l => {
                const isBlock = l.action === 'BLOCK';
                const timeStr = new Date(l.occurred_at).toLocaleTimeString();
                return `<tr>
                    <td>${timeStr}</td>
                    <td><strong>${l.domain}</strong></td>
                    <td><span class="tag-gray">${l.qtype}</span></td>
                    <td><span class="${isBlock ? 'tag-block' : 'tag-allow'}">${l.action}</span></td>
                    <td style="color:var(--text-muted); font-size:0.82rem;">${l.reason}</td>
                </tr>`;
            }).join('');
        }

        async function fetchEvidence() {
            const container = document.getElementById('evidence-container');
            try {
                const res = await api('/api/evidence?limit=50');
                const list = await res.json();
                if (list.length === 0) {
                    container.innerHTML = `<div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 40px;">
                        ${currentLang === 'vi' ? 'Không có bằng chứng vi phạm nào được ghi nhận gần đây' : 'No visual evidence captures recorded'}
                    </div>`;
                    return;
                }

                container.innerHTML = list.map(e => {
                    const timeStr = new Date(e.occurred_at).toLocaleString();
                    const expHours = Math.max(0, Math.round((new Date(e.expires_at) - new Date()) / (1000 * 60 * 60)));
                    return `<div class="evidence-card">
                        <div class="evidence-img-box">
                            <img class="evidence-img" src="${e.thumbnail_url}" alt="Evidence Frame">
                            <div class="evidence-img-badge">🔒 Blurred 480p</div>
                        </div>
                        <div class="evidence-info">
                            <div>
                                <div class="evidence-trigger">⚠️ ${e.trigger}</div>
                                <div class="evidence-meta">${timeStr} • ${(e.size_bytes / 1024).toFixed(1)} KB</div>
                            </div>
                            <div class="evidence-countdown">
                                ⏳ ${currentLang === 'vi' ? `Tự hủy sau: ${expHours}h` : `Purged in: ${expHours}h`}
                            </div>
                        </div>
                    </div>`;
                }).join('');
            } catch (e) {
                container.innerHTML = `<div style="grid-column: 1 / -1; text-align: center; color: var(--accent-red); padding: 20px;">Lỗi tải bằng chứng</div>`;
            }
        }

        async function changeMasterPin() {
            const curr = document.getElementById('pin-current').value.trim();
            const pNew = document.getElementById('pin-new').value.trim();
            const pConf = document.getElementById('pin-confirm').value.trim();

            if (pNew.length < 6 || !/^\d+$/.test(pNew)) {
                showToast(currentLang === 'vi' ? 'Mã PIN mới phải gồm ít nhất 6 chữ số' : 'New PIN must be at least 6 digits', 'error');
                return;
            }
            if (pNew !== pConf) {
                showToast(currentLang === 'vi' ? 'Mã PIN xác nhận không khớp' : 'PIN confirmation does not match', 'error');
                return;
            }

            try {
                const res = await api('/api/settings/pin', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ current_pin: curr, new_pin: pNew })
                });
                const data = await res.json();
                if (res.ok && data.success) {
                    showToast(currentLang === 'vi' ? 'Cập nhật Master PIN thành công!' : 'Master PIN updated successfully!', 'success');
                    document.getElementById('pin-current').value = '';
                    document.getElementById('pin-new').value = '';
                    document.getElementById('pin-confirm').value = '';
                    if (data.recovery_key) {
                        document.getElementById('returned-recovery-key').innerText = data.recovery_key;
                        document.getElementById('pin-success-box').style.display = 'block';
                    }
                } else {
                    showToast(data.error || 'Lỗi đổi PIN', 'error');
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi kết nối máy chủ' : 'Server connection failed', 'error');
            }
        }

        async function enrollCloud() {
            const codeInput = document.getElementById('cloud-enroll-code');
            const code = codeInput.value.trim();
            if (code.length !== 6 || !/^\d+$/.test(code)) {
                showToast(currentLang === 'vi' ? 'Vui lòng nhập mã ghép nối 6 chữ số' : 'Enter 6-digit code', 'error');
                return;
            }

            try {
                const res = await api('/api/cloud/enroll', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ enrollment_code: code })
                });
                const data = await res.json();
                if (res.ok && data.success) {
                    showToast(currentLang === 'vi' ? 'Đã ghép nối với Cloud Server thành công!' : 'Linked to Cloud successfully!', 'success');
                    codeInput.value = '';
                    fetchStatus();
                } else {
                    showToast(data.error || 'Lỗi ghép nối', 'error');
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi kết nối' : 'Connection error', 'error');
            }
        }

        async function disenrollCloud() {
            if (!confirm(currentLang === 'vi' ? 'Quay lại chế độ 100% Ngoại tuyến StandaloneFree?' : 'Revert to 100% Offline StandaloneFree?')) return;
            try {
                const res = await api('/api/cloud/disenroll', { method: 'POST' });
                if (res.ok) {
                    showToast(currentLang === 'vi' ? 'Đã trở về chế độ StandaloneFree' : 'Reverted to StandaloneFree', 'success');
                    fetchStatus();
                }
            } catch (e) {
                showToast(currentLang === 'vi' ? 'Lỗi thao tác' : 'Action failed', 'error');
            }
        }

        function initDashboard() {
            fetchStatus();
            // Auto refresh status & logs every 4 seconds
            setInterval(() => {
                const activeTab = document.querySelector('.tab-content.active');
                if (activeTab && activeTab.id === 'tab-dashboard') {
                    fetchStatus();
                } else if (activeTab && activeTab.id === 'tab-logs') {
                    fetchLogs();
                }
            }, 4000);
        }

        // Boot
        window.addEventListener('DOMContentLoaded', () => {
            setLanguage(currentLang);
            checkSession();
        });
    </script>
</body>
</html>
"#;
