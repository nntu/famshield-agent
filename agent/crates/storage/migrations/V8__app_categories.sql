-- Application Categories Catalog and Classification Rules
CREATE TABLE IF NOT EXISTS app_categories (
    id TEXT PRIMARY KEY NOT NULL,
    name_vi TEXT NOT NULL,
    name_en TEXT NOT NULL,
    icon TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0,
    description_vi TEXT,
    description_en TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_category_rules (
    rule_id TEXT PRIMARY KEY NOT NULL,
    category_id TEXT NOT NULL REFERENCES app_categories(id) ON DELETE CASCADE,
    pattern TEXT NOT NULL,
    match_type TEXT NOT NULL DEFAULT 'contains', -- 'contains', 'exact', 'starts_with', 'ends_with'
    app_title TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_app_category_rules_cat ON app_category_rules(category_id);
CREATE INDEX IF NOT EXISTS idx_app_category_rules_priority ON app_category_rules(priority DESC);

-- Seed Standard Categories
INSERT OR IGNORE INTO app_categories (id, name_vi, name_en, icon, is_system, description_vi, description_en, sort_order) VALUES
('gaming', 'Trò chơi (Gaming)', 'Gaming', 'Gamepad2', 0, 'Trò chơi trực tuyến, game launcher và giả lập', 'Online games, game launchers and emulators', 10),
('browsers', 'Trình duyệt (Browsers)', 'Web Browsers', 'Globe', 0, 'Trình duyệt lướt web (Chrome, Edge, Cốc Cốc, Firefox...)', 'Web browsing software (Chrome, Edge, Firefox...)', 20),
('social', 'Mạng xã hội & Chat (Social & Chat)', 'Social Media & Chat', 'MessageCircle', 0, 'Ứng dụng nhắn tin, mạng xã hội (Discord, Zalo, Telegram...)', 'Messaging and social media apps (Discord, Zalo, Telegram...)', 30),
('education_work', 'Học tập & Làm việc (Education & Work)', 'Education & Work', 'BookOpen', 0, 'Phần mềm văn phòng, họp trực tuyến, tài liệu học tập', 'Productivity, online meetings, study and office documents', 40),
('development', 'Lập trình (Development)', 'Development & Tech', 'Code2', 0, 'Công cụ lập trình, IDE, terminal, git', 'Developer tools, IDEs, terminals and git clients', 50),
('entertainment', 'Giải trí & Media (Entertainment)', 'Entertainment & Media', 'Tv', 0, 'Nghe nhạc, xem phim, phát video đa phương tiện', 'Music streaming, video playback and multimedia', 60),
('utilities', 'Tiện ích & Công cụ (Utilities)', 'Utilities & Tools', 'Sliders', 0, 'Tiện ích nén file, ghi chú, máy tính, công cụ đồ họa', 'File archiving, notes, calculators, screenshot and utilities', 70),
('system', 'Hệ thống Windows (System & Services)', 'Windows System', 'Cpu', 1, 'Các tiến trình nền hệ điều hành và dịch vụ cốt lõi của Windows', 'Windows core operating system background processes and services', 80),
('other', 'Ứng dụng Khác (Other)', 'Other Apps', 'Layers', 0, 'Các ứng dụng và tiến trình khác', 'Other miscellaneous applications and processes', 90);

-- Seed System Processes Rules (Highest Priority 100)
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-sys-svchost', 'system', 'svchost', 'contains', 'Dịch vụ Hệ thống (Service Host)', 1, 100),
('rule-sys-csrss', 'system', 'csrss', 'contains', 'Client Server Runtime Subsystem', 1, 100),
('rule-sys-smss', 'system', 'smss', 'contains', 'Session Manager Subsystem', 1, 100),
('rule-sys-services', 'system', 'services', 'contains', 'Dịch vụ Windows (Services.exe)', 1, 100),
('rule-sys-lsass', 'system', 'lsass', 'contains', 'Local Security Authority', 1, 100),
('rule-sys-wininit', 'system', 'wininit', 'contains', 'Windows Initialization', 1, 100),
('rule-sys-winlogon', 'system', 'winlogon', 'contains', 'Windows Logon Process', 1, 100),
('rule-sys-dwm', 'system', 'dwm', 'contains', 'Desktop Window Manager', 1, 100),
('rule-sys-fontdrvhost', 'system', 'fontdrvhost', 'contains', 'Font Driver Host', 1, 100),
('rule-sys-sihost', 'system', 'sihost', 'contains', 'Shell Infrastructure Host', 1, 100),
('rule-sys-taskhostw', 'system', 'taskhostw', 'contains', 'Host Process for Windows Tasks', 1, 100),
('rule-sys-runtimebroker', 'system', 'runtimebroker', 'contains', 'Runtime Broker', 1, 100),
('rule-sys-searchhost', 'system', 'searchhost', 'contains', 'Windows Search Host', 1, 100),
('rule-sys-startmenu', 'system', 'startmenuexperiencehost', 'contains', 'Start Menu Host', 1, 100),
('rule-sys-shellexp', 'system', 'shellexperiencehost', 'contains', 'Windows Shell Experience', 1, 100),
('rule-sys-conhost', 'system', 'conhost', 'contains', 'Console Window Host', 1, 100),
('rule-sys-ctfmon', 'system', 'ctfmon', 'contains', 'CTF Loader (Bộ gõ & Input)', 1, 100),
('rule-sys-spoolsv', 'system', 'spoolsv', 'contains', 'Print Spooler Service', 1, 100),
('rule-sys-searchindexer', 'system', 'searchindexer', 'contains', 'Windows Search Indexer', 1, 100),
('rule-sys-dllhost', 'system', 'dllhost', 'contains', 'COM Surrogate', 1, 100),
('rule-sys-smartscreen', 'system', 'smartscreen', 'contains', 'Windows Defender SmartScreen', 1, 100),
('rule-sys-sechealth', 'system', 'securityhealthservice', 'contains', 'Windows Security Health Service', 1, 100),
('rule-sys-msmpeng', 'system', 'msmpeng', 'contains', 'Microsoft Defender Antivirus', 1, 100),
('rule-sys-explorer', 'system', 'explorer', 'contains', 'Windows Explorer (File Browser)', 1, 90),
('rule-sys-taskmgr', 'system', 'taskmgr', 'contains', 'Task Manager', 1, 90),
('rule-sys-famshield-svc', 'system', 'famshield-service', 'contains', 'FamShield Core Service', 1, 100),
('rule-sys-famshield-ses', 'system', 'famshield-session', 'contains', 'FamShield User Session Helper', 1, 100);

-- Seed Gaming Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-game-roblox', 'gaming', 'roblox', 'contains', 'Roblox Player', 0, 80),
('rule-game-minecraft', 'gaming', 'minecraft', 'contains', 'Minecraft', 0, 80),
('rule-game-javaw', 'gaming', 'javaw', 'contains', 'Minecraft (Java Engine)', 0, 75),
('rule-game-genshin', 'gaming', 'genshin', 'contains', 'Genshin Impact', 0, 80),
('rule-game-yuanshen', 'gaming', 'yuanshen', 'contains', 'Genshin Impact (CN)', 0, 80),
('rule-game-league', 'gaming', 'league', 'contains', 'League of Legends (LoL)', 0, 80),
('rule-game-riot', 'gaming', 'riot', 'contains', 'Riot Games Client', 0, 80),
('rule-game-valorant', 'gaming', 'valorant', 'contains', 'Valorant', 0, 80),
('rule-game-steam', 'gaming', 'steam', 'contains', 'Steam Client', 0, 80),
('rule-game-epic', 'gaming', 'epicgames', 'contains', 'Epic Games Launcher', 0, 80),
('rule-game-csgo', 'gaming', 'csgo', 'contains', 'Counter-Strike', 0, 80),
('rule-game-cs2', 'gaming', 'cs2', 'contains', 'Counter-Strike 2', 0, 80),
('rule-game-dota', 'gaming', 'dota', 'contains', 'Dota 2', 0, 80),
('rule-game-ldplayer', 'gaming', 'ldplayer', 'contains', 'Giả lập LDPlayer', 0, 80),
('rule-game-dnplayer', 'gaming', 'dnplayer', 'contains', 'Giả lập LDPlayer Engine', 0, 80),
('rule-game-nox', 'gaming', 'nox', 'contains', 'Giả lập NoxPlayer', 0, 80),
('rule-game-bluestacks', 'gaming', 'bluestacks', 'contains', 'Giả lập BlueStacks', 0, 80),
('rule-game-pubg', 'gaming', 'pubg', 'contains', 'PUBG: BATTLEGROUNDS', 0, 80),
('rule-game-fifa', 'gaming', 'fifa', 'contains', 'EA Sports FC / FIFA', 0, 80),
('rule-game-gta', 'gaming', 'gta', 'contains', 'Grand Theft Auto', 0, 80);

-- Seed Browsers Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-browser-chrome', 'browsers', 'chrome', 'contains', 'Google Chrome', 0, 80),
('rule-browser-edge', 'browsers', 'msedge', 'contains', 'Microsoft Edge', 0, 80),
('rule-browser-coccoc', 'browsers', 'coccoc', 'contains', 'Cốc Cốc Browser', 0, 80),
('rule-browser-firefox', 'browsers', 'firefox', 'contains', 'Mozilla Firefox', 0, 80),
('rule-browser-brave', 'browsers', 'brave', 'contains', 'Brave Browser', 0, 80),
('rule-browser-opera', 'browsers', 'opera', 'contains', 'Opera Browser', 0, 80),
('rule-browser-vivaldi', 'browsers', 'vivaldi', 'contains', 'Vivaldi Browser', 0, 80),
('rule-browser-tor', 'browsers', 'tor', 'contains', 'Tor Browser', 0, 80);

-- Seed Social & Communication Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-social-discord', 'social', 'discord', 'contains', 'Discord', 0, 80),
('rule-social-zalo', 'social', 'zalo', 'contains', 'Zalo', 0, 80),
('rule-social-telegram', 'social', 'telegram', 'contains', 'Telegram', 0, 80),
('rule-social-messenger', 'social', 'messenger', 'contains', 'Facebook Messenger', 0, 80),
('rule-social-whatsapp', 'social', 'whatsapp', 'contains', 'WhatsApp', 0, 80),
('rule-social-skype', 'social', 'skype', 'contains', 'Skype', 0, 80),
('rule-social-slack', 'social', 'slack', 'contains', 'Slack', 0, 80),
('rule-social-viber', 'social', 'viber', 'contains', 'Viber', 0, 80);

-- Seed Education & Work Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-edu-zoom', 'education_work', 'zoom', 'contains', 'Zoom Meetings', 0, 80),
('rule-edu-teams', 'education_work', 'teams', 'contains', 'Microsoft Teams', 0, 80),
('rule-edu-word', 'education_work', 'winword', 'contains', 'Microsoft Word', 0, 80),
('rule-edu-excel', 'education_work', 'excel', 'contains', 'Microsoft Excel', 0, 80),
('rule-edu-powerpnt', 'education_work', 'powerpnt', 'contains', 'Microsoft PowerPoint', 0, 80),
('rule-edu-onenote', 'education_work', 'onenote', 'contains', 'Microsoft OneNote', 0, 80),
('rule-edu-acrobat', 'education_work', 'acrobat', 'contains', 'Adobe Acrobat Reader', 0, 80),
('rule-edu-foxit', 'education_work', 'foxit', 'contains', 'Foxit PDF Reader', 0, 80),
('rule-edu-notion', 'education_work', 'notion', 'contains', 'Notion', 0, 80);

-- Seed Development Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-dev-vscode', 'development', 'code', 'contains', 'Visual Studio Code', 0, 80),
('rule-dev-vs', 'development', 'devenv', 'contains', 'Microsoft Visual Studio', 0, 80),
('rule-dev-idea', 'development', 'idea64', 'contains', 'IntelliJ IDEA', 0, 80),
('rule-dev-pycharm', 'development', 'pycharm', 'contains', 'PyCharm', 0, 80),
('rule-dev-webstorm', 'development', 'webstorm', 'contains', 'WebStorm', 0, 80),
('rule-dev-sublime', 'development', 'sublime', 'contains', 'Sublime Text', 0, 80),
('rule-dev-git', 'development', 'git', 'contains', 'Git / Git GUI', 0, 70);

-- Seed Entertainment & Media Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-media-spotify', 'entertainment', 'spotify', 'contains', 'Spotify Music', 0, 80),
('rule-media-vlc', 'entertainment', 'vlc', 'contains', 'VLC Media Player', 0, 80),
('rule-media-potplayer', 'entertainment', 'potplayer', 'contains', 'PotPlayer', 0, 80),
('rule-media-kmplayer', 'entertainment', 'kmplayer', 'contains', 'KMPlayer', 0, 80),
('rule-media-netflix', 'entertainment', 'netflix', 'contains', 'Netflix App', 0, 80);

-- Seed Utilities Rules
INSERT OR IGNORE INTO app_category_rules (rule_id, category_id, pattern, match_type, app_title, is_system, priority) VALUES
('rule-util-winrar', 'utilities', 'winrar', 'contains', 'WinRAR Archiver', 0, 80),
('rule-util-7z', 'utilities', '7z', 'contains', '7-Zip File Manager', 0, 80),
('rule-util-notepad', 'utilities', 'notepad', 'contains', 'Notepad', 0, 80),
('rule-util-calc', 'utilities', 'calc', 'contains', 'Windows Calculator', 0, 80),
('rule-util-paint', 'utilities', 'paint', 'contains', 'Paint', 0, 80),
('rule-util-snipping', 'utilities', 'snipping', 'contains', 'Snipping Tool', 0, 80);
