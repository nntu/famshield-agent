use serde::{Deserialize, Serialize};

pub fn classify_domain(domain: &str) -> &'static str {
    let lower = domain.trim().to_ascii_lowercase();
    if lower.contains("youtube")
        || lower.contains("tiktok")
        || lower.contains("netflix")
        || lower.contains("twitch")
        || lower.contains("spotify")
        || lower.contains("bilibili")
    {
        "Giải trí & Video (Entertainment)"
    } else if lower.contains("facebook")
        || lower.contains("instagram")
        || lower.contains("discord")
        || lower.contains("twitter")
        || lower.contains("zalo")
        || lower.contains("threads")
        || lower.contains("reddit")
    {
        "Mạng xã hội (Social Media)"
    } else if lower.contains("roblox")
        || lower.contains("steam")
        || lower.contains("epicgames")
        || lower.contains("riot")
        || lower.contains("minecraft")
        || lower.contains("game")
    {
        "Trò chơi (Gaming)"
    } else if lower.contains("google")
        || lower.contains("bing")
        || lower.contains("duckduckgo")
        || lower.contains("coccoc")
    {
        "Công cụ Tìm kiếm (Search Engine)"
    } else if lower.contains("wikipedia")
        || lower.contains("coursera")
        || lower.contains("duolingo")
        || lower.contains("khanacademy")
        || lower.contains("edu")
    {
        "Học tập & Giáo dục (Education)"
    } else if lower.contains("github")
        || lower.contains("stackoverflow")
        || lower.contains("rust-lang")
        || lower.contains("developer")
    {
        "Lập trình & Kỹ thuật (Development)"
    } else {
        "Trang web Chung (Web / Other)"
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StaticAppClassification {
    pub title: &'static str,
    pub category_id: &'static str,
    pub category_name: &'static str,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppClassification {
    pub title: String,
    pub category_id: String,
    pub category_name: String,
    pub is_system: bool,
}

impl From<StaticAppClassification> for AppClassification {
    fn from(s: StaticAppClassification) -> Self {
        Self {
            title: s.title.to_string(),
            category_id: s.category_id.to_string(),
            category_name: s.category_name.to_string(),
            is_system: s.is_system,
        }
    }
}

pub fn classify_app_extended(app_name: &str) -> AppClassification {
    classify_app_extended_static(app_name).into()
}

pub fn classify_app_extended_static(app_name: &str) -> StaticAppClassification {
    let lower = app_name.trim().to_ascii_lowercase();
    let base = lower.trim_end_matches(".exe");

    // Check system processes first
    if matches!(
        base,
        "svchost"
            | "csrss"
            | "smss"
            | "services"
            | "lsass"
            | "wininit"
            | "winlogon"
            | "dwm"
            | "fontdrvhost"
            | "sihost"
            | "taskhostw"
            | "runtimebroker"
            | "searchhost"
            | "startmenuexperiencehost"
            | "shellexperiencehost"
            | "conhost"
            | "ctfmon"
            | "spoolsv"
            | "searchindexer"
            | "dllhost"
            | "smartscreen"
            | "securityhealthservice"
            | "securityhealthsystray"
            | "msmpeng"
            | "nissrv"
            | "system"
            | "system idle process"
            | "registry"
            | "memory compression"
            | "audiodg"
            | "wlanext"
            | "dashost"
            | "wudfhost"
            | "explorer"
            | "famshield-service"
            | "famshield-session"
            | "taskmgr"
    ) {
        return StaticAppClassification {
            title: if base == "explorer" {
                "Windows Explorer (File Browser)"
            } else if base == "taskmgr" {
                "Task Manager"
            } else {
                "Tiến trình Hệ thống Windows"
            },
            category_id: "system",
            category_name: "Hệ thống Windows (System & Services)",
            is_system: true,
        };
    }

    // Gaming
    if lower.contains("roblox") {
        StaticAppClassification {
            title: "Roblox Player",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("minecraft") || lower.contains("javaw") {
        StaticAppClassification {
            title: "Minecraft",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("genshin") || lower.contains("yuanshen") {
        StaticAppClassification {
            title: "Genshin Impact",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("league") || lower.contains("riot") || lower.contains("valorant") {
        StaticAppClassification {
            title: "Riot Games (LoL/Valorant)",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("steam") || lower.contains("epicgames") {
        StaticAppClassification {
            title: "Game Launcher (Steam/Epic)",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("csgo") || lower.contains("cs2") || lower.contains("dota") {
        StaticAppClassification {
            title: "Valve Game (CS/Dota)",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("ldplayer")
        || lower.contains("nox")
        || lower.contains("bluestacks")
        || lower.contains("memu")
        || lower.contains("dnplayer")
    {
        StaticAppClassification {
            title: "Giả lập Android Game",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    } else if lower.contains("game")
        || lower.contains("fifa")
        || lower.contains("gta")
        || lower.contains("pubg")
        || lower.contains("fortnite")
    {
        StaticAppClassification {
            title: "Trò chơi PC",
            category_id: "gaming",
            category_name: "Trò chơi (Gaming)",
            is_system: false,
        }
    }
    // Browsers
    else if lower.contains("chrome") {
        StaticAppClassification {
            title: "Google Chrome",
            category_id: "browsers",
            category_name: "Trình duyệt (Browser)",
            is_system: false,
        }
    } else if lower.contains("msedge") {
        StaticAppClassification {
            title: "Microsoft Edge",
            category_id: "browsers",
            category_name: "Trình duyệt (Browser)",
            is_system: false,
        }
    } else if lower.contains("coccoc") {
        StaticAppClassification {
            title: "Cốc Cốc Browser",
            category_id: "browsers",
            category_name: "Trình duyệt (Browser)",
            is_system: false,
        }
    } else if lower.contains("firefox")
        || lower.contains("brave")
        || lower.contains("opera")
        || lower.contains("vivaldi")
        || lower.contains("tor")
    {
        StaticAppClassification {
            title: "Trình duyệt Web",
            category_id: "browsers",
            category_name: "Trình duyệt (Browser)",
            is_system: false,
        }
    }
    // Social & Communication
    else if lower.contains("discord") {
        StaticAppClassification {
            title: "Discord",
            category_id: "social",
            category_name: "Mạng xã hội (Social Media)",
            is_system: false,
        }
    } else if lower.contains("zalo") {
        StaticAppClassification {
            title: "Zalo",
            category_id: "social",
            category_name: "Liên lạc (Communication)",
            is_system: false,
        }
    } else if lower.contains("telegram")
        || lower.contains("messenger")
        || lower.contains("whatsapp")
        || lower.contains("skype")
        || lower.contains("slack")
        || lower.contains("viber")
    {
        StaticAppClassification {
            title: "Nhắn tin & Liên lạc",
            category_id: "social",
            category_name: "Liên lạc (Communication)",
            is_system: false,
        }
    }
    // Entertainment & Media
    else if lower.contains("spotify")
        || lower.contains("vlc")
        || lower.contains("mediaplayer")
        || lower.contains("potplayer")
        || lower.contains("kmplayer")
        || lower.contains("netflix")
    {
        StaticAppClassification {
            title: "Nghe nhạc & Video",
            category_id: "entertainment",
            category_name: "Giải trí (Entertainment)",
            is_system: false,
        }
    }
    // Development & Tech
    else if lower.contains("code")
        || lower.contains("devenv")
        || lower.contains("idea64")
        || lower.contains("pycharm")
        || lower.contains("webstorm")
        || lower.contains("sublime")
        || lower.contains("git")
    {
        StaticAppClassification {
            title: "Công cụ Lập trình (IDE)",
            category_id: "development",
            category_name: "Lập trình (Development)",
            is_system: false,
        }
    }
    // Education & Productivity
    else if lower.contains("zoom") || lower.contains("teams") {
        StaticAppClassification {
            title: "Hội họp Trực tuyến",
            category_id: "education_work",
            category_name: "Học tập & Làm việc (Education/Work)",
            is_system: false,
        }
    } else if lower.contains("winword")
        || lower.contains("word")
        || lower.contains("excel")
        || lower.contains("powerpnt")
        || lower.contains("onenote")
    {
        StaticAppClassification {
            title: "Microsoft Office",
            category_id: "education_work",
            category_name: "Học tập & Làm việc (Education/Work)",
            is_system: false,
        }
    } else if lower.contains("acrobat")
        || lower.contains("acrord32")
        || lower.contains("foxit")
        || lower.contains("notion")
        || lower.contains("pdf")
    {
        StaticAppClassification {
            title: "Tài liệu & Ghi chú",
            category_id: "education_work",
            category_name: "Học tập & Làm việc (Education/Work)",
            is_system: false,
        }
    }
    // Utilities & Tools
    else if lower.contains("winrar")
        || lower.contains("7z")
        || lower.contains("notepad")
        || lower.contains("calc")
        || lower.contains("paint")
        || lower.contains("snipping")
    {
        StaticAppClassification {
            title: "Tiện ích & Công cụ",
            category_id: "utilities",
            category_name: "Tiện ích & Công cụ (Utilities)",
            is_system: false,
        }
    } else {
        StaticAppClassification {
            title: "Ứng dụng Windows",
            category_id: "other",
            category_name: "Ứng dụng Khác (Other)",
            is_system: false,
        }
    }
}

pub fn classify_app(app_name: &str) -> (&'static str, &'static str) {
    let c = classify_app_extended_static(app_name);
    (c.title, c.category_name)
}

