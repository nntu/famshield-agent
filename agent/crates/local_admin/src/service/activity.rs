use super::{normalize_time_bound, LocalAdminService};
use crate::service::classifier::classify_domain;
use crate::service::types::{
    ActivitySummaryResponse, AppActivityItem, AppCategoryMetadata, AppCategoryUsageBreakdown,
    AppUsageSummary, HourlyUsagePoint, KeywordViolationDto, LocalAdminError, TimeAggregatedReport,
    UnifiedTimelineItem, WebActivityItem,
};
use chrono::Utc;
use std::collections::HashMap;

impl LocalAdminService {
    pub fn get_activity_summary(
        &self,
        active_app: &str,
    ) -> Result<ActivitySummaryResponse, LocalAdminError> {
        let now = Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();

        // 1. Grouped Application Usage Report
        let grouped_apps = self
            .get_grouped_app_usage(active_app, None, None)
            .unwrap_or_default();

        // 2. Raw App Activities Segments
        let raw_segments = self.storage.list_usage_segments(100).unwrap_or_default();
        let mut app_activities = Vec::new();
        let mut total_duration_minutes: u32 = 0;

        if !raw_segments.is_empty() {
            for (idx, seg) in raw_segments.iter().enumerate() {
                let duration_mins = (seg.duration_ms / 60_000).max(1) as u32;
                total_duration_minutes += duration_mins;
                let classification = self.classify_app(&seg.rule_id);
                let end_time = seg.start_utc + chrono::Duration::milliseconds(seg.duration_ms);
                let duration_formatted = if duration_mins >= 60 {
                    format!("{}h {:02}m", duration_mins / 60, duration_mins % 60)
                } else {
                    format!("{} phút", duration_mins)
                };

                app_activities.push(AppActivityItem {
                    id: format!("app-seg-{}", idx + 1),
                    app_name: seg.rule_id.clone(),
                    app_title: classification.title,
                    category: classification.category_name,
                    category_id: classification.category_id,
                    is_system: classification.is_system,
                    start_time: seg.start_utc,
                    end_time,
                    duration_minutes: duration_mins,
                    duration_formatted,
                    status: if duration_mins > 60 {
                        "QuotaWarning".to_string()
                    } else {
                        "Normal".to_string()
                    },
                });
            }
        }

        // If raw_segments is empty but grouped_apps has time, use grouped_apps sum
        if total_duration_minutes == 0 && !grouped_apps.is_empty() {
            total_duration_minutes = grouped_apps.iter().map(|a| a.total_duration_minutes).sum();
        }

        // 2. Web Activities
        let raw_logs = self.storage.list_recent_query_logs(200).unwrap_or_default();
        let mut web_activities = Vec::new();
        let mut blocked_count = 0;
        let mut monitored_count = 0;

        if !raw_logs.is_empty() {
            for (idx, log) in raw_logs.iter().enumerate() {
                let cat = classify_domain(&log.domain);
                if log.action == "BLOCK" {
                    blocked_count += 1;
                } else if log.action == "MONITOR" || self.is_monitor_mode() {
                    monitored_count += 1;
                }
                let app = if let Some(ref r) = log.rule_id {
                    if !r.is_empty() && (r.ends_with(".exe") || !r.contains('.')) {
                        r.clone()
                    } else if !active_app.is_empty() && active_app != "unknown" {
                        active_app.to_string()
                    } else {
                        "Trình duyệt Web".to_string()
                    }
                } else if !active_app.is_empty() && active_app != "unknown" {
                    active_app.to_string()
                } else {
                    "Trình duyệt Web".to_string()
                };

                web_activities.push(WebActivityItem {
                    id: idx + 1,
                    domain: log.domain.clone(),
                    category: cat.to_string(),
                    action: log.action.clone(),
                    reason: log.reason.clone(),
                    occurred_at: log.occurred_at,
                    source_app: app,
                });
            }
        }

        // 3. Unified Timeline
        let mut unified_timeline = Vec::new();
        for app in &app_activities {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-{}", app.id),
                item_type: "app".to_string(),
                title: app.app_title.clone(),
                subtitle: format!("{} • Sử dụng {}", app.category, app.duration_formatted),
                category: app.category.clone(),
                action: if app.status == "Active" {
                    "ACTIVE".to_string()
                } else {
                    "ALLOW".to_string()
                },
                timestamp: app.start_time,
                duration_minutes: app.duration_minutes,
            });
        }

        for web in &web_activities {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-web-{}", web.id),
                item_type: "web".to_string(),
                title: web.domain.clone(),
                subtitle: format!("{} • Qua {}", web.category, web.source_app),
                category: web.category.clone(),
                action: web.action.clone(),
                timestamp: web.occurred_at,
                duration_minutes: 0,
            });
        }

        // 3b. Keyword Violations
        let raw_violations = self.storage.list_keyword_violations(50).unwrap_or_default();
        let keyword_violations_list: Vec<KeywordViolationDto> = raw_violations
            .into_iter()
            .map(|v| KeywordViolationDto {
                id: v.id,
                timestamp: v.timestamp_utc,
                app_name: v.app_name,
                keyword_matched: v.keyword_matched,
                category: v.category,
                severity: v.severity,
                snippet_masked: v.snippet_masked,
            })
            .collect();

        for kv in &keyword_violations_list {
            unified_timeline.push(UnifiedTimelineItem {
                id: format!("tl-kw-{}", kv.id),
                item_type: "keyword".to_string(),
                title: format!("Từ khóa nhạy cảm: \"{}\"", kv.keyword_matched),
                subtitle: format!("{} • Ứng dụng {}", kv.category, kv.app_name),
                category: kv.category.clone(),
                action: "VIOLATION".to_string(),
                timestamp: kv.timestamp,
                duration_minutes: 0,
            });
        }

        // Sort unified timeline descending by timestamp
        unified_timeline.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        let hours = total_duration_minutes / 60;
        let mins = total_duration_minutes % 60;
        let total_screen_time_formatted = if hours > 0 {
            format!("{} giờ {:02} phút", hours, mins)
        } else {
            format!("{} phút", mins)
        };

        let keyword_violations_count = keyword_violations_list.len();

        Ok(ActivitySummaryResponse {
            date: date_str,
            total_screen_time_minutes: total_duration_minutes,
            total_screen_time_formatted,
            active_apps_count: if grouped_apps.is_empty() {
                app_activities.len()
            } else {
                grouped_apps.len()
            },
            total_web_visits_count: web_activities.len(),
            blocked_events_count: blocked_count,
            monitored_events_count: monitored_count,
            keyword_violations_count,
            app_activities,
            grouped_apps,
            web_activities,
            keyword_violations: keyword_violations_list,
            unified_timeline,
        })
    }

    /// Retrieve application usage aggregated and grouped by application name,
    /// calculating the total usage duration for each individual app.
    pub fn get_grouped_app_usage(
        &self,
        active_app: &str,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<AppUsageSummary>, LocalAdminError> {
        self.get_grouped_app_usage_filtered(active_app, start_time, end_time, false)
    }

    /// Retrieve application usage grouped with optional system process filtering and prioritization.
    pub fn get_grouped_app_usage_filtered(
        &self,
        active_app: &str,
        start_time: Option<&str>,
        end_time: Option<&str>,
        exclude_system: bool,
    ) -> Result<Vec<AppUsageSummary>, LocalAdminError> {
        let normalized_start = normalize_time_bound(start_time);
        let normalized_end = normalize_time_bound(end_time);
        let db_groups = self
            .storage
            .get_app_usage_grouped(normalized_start.as_deref(), normalized_end.as_deref())
            .unwrap_or_default();

        let total_screen_time_ms: i64 = db_groups.iter().map(|g| g.total_duration_ms).sum();

        let mut list = Vec::new();
        for g in db_groups {
            let duration_mins = (g.total_duration_ms / 60_000).max(1) as u32;
            let classification = self.classify_app(&g.app_name);
            if exclude_system && classification.is_system {
                continue;
            }
            let duration_formatted = if duration_mins >= 60 {
                format!("{}h {:02}m", duration_mins / 60, duration_mins % 60)
            } else {
                format!("{} phút", duration_mins)
            };
            let is_active_now =
                !active_app.is_empty() && g.app_name.eq_ignore_ascii_case(active_app);
            let percentage = if total_screen_time_ms > 0 {
                ((g.total_duration_ms as f64 / total_screen_time_ms as f64) * 100.0) as f32
            } else {
                0.0
            };

            let status = if is_active_now {
                "Active".to_string()
            } else if duration_mins > 60 {
                "QuotaWarning".to_string()
            } else {
                "Normal".to_string()
            };

            list.push(AppUsageSummary {
                app_name: g.app_name,
                app_title: classification.title,
                category: classification.category_name,
                category_id: classification.category_id,
                is_system: classification.is_system,
                total_duration_minutes: duration_mins,
                total_duration_formatted: duration_formatted,
                percentage,
                session_count: g.session_count,
                first_seen_at: g.first_seen_utc,
                last_seen_at: g.last_seen_utc,
                is_active_now,
                status,
            });
        }

        // If list is empty but we have an active_app, show it
        if list.is_empty() && !active_app.is_empty() && active_app != "unknown" {
            let classification = self.classify_app(active_app);
            if !exclude_system || !classification.is_system {
                list.push(AppUsageSummary {
                    app_name: active_app.to_string(),
                    app_title: classification.title,
                    category: classification.category_name,
                    category_id: classification.category_id,
                    is_system: classification.is_system,
                    total_duration_minutes: 1,
                    total_duration_formatted: "1 phút".to_string(),
                    percentage: 100.0,
                    session_count: 1,
                    first_seen_at: Utc::now(),
                    last_seen_at: Utc::now(),
                    is_active_now: true,
                    status: "Active".to_string(),
                });
            }
        }

        // Prioritize non-system apps first, then sort by duration descending
        list.sort_by(|a, b| {
            match (a.is_system, b.is_system) {
                (false, true) => std::cmp::Ordering::Less, // User apps prioritized at top
                (true, false) => std::cmp::Ordering::Greater,
                _ => b.total_duration_minutes.cmp(&a.total_duration_minutes),
            }
        });

        Ok(list)
    }

    /// Returns application categories catalog with real-time aggregated usage statistics.
    pub fn get_app_categories(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Vec<AppCategoryMetadata> {
        let apps = self
            .get_grouped_app_usage("", start_time, end_time)
            .unwrap_or_default();
        self.build_app_categories(&apps)
    }

    fn build_app_categories(&self, apps: &[AppUsageSummary]) -> Vec<AppCategoryMetadata> {
        let total_screen_mins: u32 = apps.iter().map(|a| a.total_duration_minutes).sum();

        let db_categories = self.storage.list_app_categories().unwrap_or_default();
        let catalog: Vec<(String, String, String, String, bool, String)> = if !db_categories
            .is_empty()
        {
            db_categories
                .into_iter()
                .map(|c| {
                    let desc = c.description_vi.or(c.description_en).unwrap_or_default();
                    (c.id, c.name_vi, c.name_en, c.icon, c.is_system, desc)
                })
                .collect()
        } else {
            vec![
                (
                    "gaming".to_string(),
                    "Trò chơi (Gaming)".to_string(),
                    "Gaming".to_string(),
                    "Gamepad2".to_string(),
                    false,
                    "Trò chơi trực tuyến, game launcher và giả lập".to_string(),
                ),
                (
                    "browsers".to_string(),
                    "Trình duyệt (Browsers)".to_string(),
                    "Web Browsers".to_string(),
                    "Globe".to_string(),
                    false,
                    "Trình duyệt lướt web (Chrome, Edge, Cốc Cốc, Firefox...)".to_string(),
                ),
                (
                    "social".to_string(),
                    "Mạng xã hội & Chat (Social & Chat)".to_string(),
                    "Social Media & Chat".to_string(),
                    "MessageCircle".to_string(),
                    false,
                    "Ứng dụng nhắn tin, mạng xã hội (Discord, Zalo, Telegram...)".to_string(),
                ),
                (
                    "education_work".to_string(),
                    "Học tập & Làm việc (Education & Work)".to_string(),
                    "Education & Work".to_string(),
                    "BookOpen".to_string(),
                    false,
                    "Phần mềm văn phòng, họp trực tuyến, tài liệu học tập".to_string(),
                ),
                (
                    "development".to_string(),
                    "Lập trình (Development)".to_string(),
                    "Development & Tech".to_string(),
                    "Code2".to_string(),
                    false,
                    "Công cụ lập trình, IDE, terminal, git".to_string(),
                ),
                (
                    "entertainment".to_string(),
                    "Giải trí & Media (Entertainment)".to_string(),
                    "Entertainment & Media".to_string(),
                    "Tv".to_string(),
                    false,
                    "Nghe nhạc, xem phim, phát video đa phương tiện".to_string(),
                ),
                (
                    "utilities".to_string(),
                    "Tiện ích & Công cụ (Utilities)".to_string(),
                    "Utilities & Tools".to_string(),
                    "Sliders".to_string(),
                    false,
                    "Tiện ích nén file, ghi chú, máy tính, công cụ đồ họa".to_string(),
                ),
                (
                    "system".to_string(),
                    "Hệ thống Windows (System & Services)".to_string(),
                    "Windows System".to_string(),
                    "Cpu".to_string(),
                    true,
                    "Các tiến trình nền hệ điều hành và dịch vụ cốt lõi của Windows".to_string(),
                ),
                (
                    "other".to_string(),
                    "Ứng dụng Khác (Other)".to_string(),
                    "Other Apps".to_string(),
                    "Layers".to_string(),
                    false,
                    "Các ứng dụng và tiến trình khác".to_string(),
                ),
            ]
        };

        let mut category_stats: HashMap<&str, (usize, u32)> = HashMap::new();
        for app in apps {
            let entry = category_stats
                .entry(app.category_id.as_str())
                .or_insert((0, 0));
            entry.0 += 1;
            entry.1 += app.total_duration_minutes;
        }

        catalog
            .into_iter()
            .map(|(id, name_vi, name_en, icon, is_system, desc)| {
                let (count, mins) = category_stats.get(id.as_str()).copied().unwrap_or((0, 0));
                let pct = if total_screen_mins > 0 {
                    ((mins as f64 / total_screen_mins as f64) * 100.0) as f32
                } else {
                    0.0
                };
                AppCategoryMetadata {
                    id,
                    name_vi,
                    name_en,
                    icon,
                    is_system,
                    description: desc,
                    app_count: count,
                    total_duration_minutes: mins,
                    percentage: pct,
                }
            })
            .collect()
    }

    /// Generate comprehensive time-aggregated report across specified period
    pub fn get_time_aggregated_report(
        &self,
        period: &str,
        start_time: Option<&str>,
        end_time: Option<&str>,
        exclude_system: bool,
    ) -> Result<TimeAggregatedReport, LocalAdminError> {
        let (calc_start, calc_end): (Option<String>, Option<String>) = match period {
            "today" => {
                let today_prefix = Utc::now().format("%Y-%m-%d").to_string();
                (
                    Some(format!("{}T00:00:00Z", today_prefix)),
                    Some(format!("{}T23:59:59Z", today_prefix)),
                )
            }
            "7d" => {
                let past = Utc::now() - chrono::Duration::days(7);
                (Some(past.to_rfc3339()), Some(Utc::now().to_rfc3339()))
            }
            "30d" => {
                let past = Utc::now() - chrono::Duration::days(30);
                (Some(past.to_rfc3339()), Some(Utc::now().to_rfc3339()))
            }
            "custom" => (
                start_time.map(|s| s.to_string()),
                end_time.map(|s| s.to_string()),
            ),
            _ => (None, None), // "all"
        };

        let start_ref = calc_start.as_deref().or(start_time);
        let end_ref = calc_end.as_deref().or(end_time);
        let normalized_start = normalize_time_bound(start_ref);
        let normalized_end = normalize_time_bound(end_ref);
        let start_ref = normalized_start.as_deref();
        let end_ref = normalized_end.as_deref();

        let apps = self.get_grouped_app_usage_filtered("", start_ref, end_ref, exclude_system)?;

        let mut total_active_mins = 0;
        let mut total_system_mins = 0;
        let mut user_apps_count = 0;
        let mut system_apps_count = 0;

        for app in &apps {
            if app.is_system {
                total_system_mins += app.total_duration_minutes;
                system_apps_count += 1;
            } else {
                total_active_mins += app.total_duration_minutes;
                user_apps_count += 1;
            }
        }

        let formatted_total = if total_active_mins >= 60 {
            format!("{}h {:02}m", total_active_mins / 60, total_active_mins % 60)
        } else {
            format!("{} phút", total_active_mins)
        };

        // Category breakdown
        let cat_meta = self.build_app_categories(&apps);
        let category_breakdown = cat_meta
            .into_iter()
            .map(|c| AppCategoryUsageBreakdown {
                category_id: c.id,
                category_name: c.name_vi,
                total_minutes: c.total_duration_minutes,
                percentage: c.percentage,
                app_count: c.app_count,
                icon: c.icon,
                is_system: c.is_system,
            })
            .collect();

        // Hourly usage distribution
        let hourly_apps = self
            .storage
            .get_hourly_app_usage_grouped(start_ref, end_ref)
            .unwrap_or_default();

        let mut hourly_distribution = Vec::with_capacity(24);
        for h in 0..24 {
            hourly_distribution.push(HourlyUsagePoint {
                hour: h as u8,
                label: format!("{:02}:00", h),
                user_minutes: 0,
                system_minutes: 0,
            });
        }

        let mut classification_cache = HashMap::new();
        for row in hourly_apps {
            let h = row.hour as usize;
            if h < 24 {
                let duration_mins = row.duration_minutes.max(1) as u32;
                let is_sys = *classification_cache
                    .entry(row.app_name.clone())
                    .or_insert_with(|| self.classify_app(&row.app_name).is_system);
                if is_sys {
                    hourly_distribution[h].system_minutes += duration_mins;
                } else {
                    hourly_distribution[h].user_minutes += duration_mins;
                }
            }
        }

        let top_user_apps = apps
            .iter()
            .filter(|a| !a.is_system)
            .take(5)
            .cloned()
            .collect();

        Ok(TimeAggregatedReport {
            period: period.to_string(),
            start_date: start_ref.unwrap_or("").to_string(),
            end_date: end_ref.unwrap_or("").to_string(),
            total_active_minutes: total_active_mins,
            total_system_minutes: total_system_mins,
            total_screen_time_formatted: formatted_total,
            user_apps_count,
            system_apps_count,
            category_breakdown,
            hourly_distribution,
            top_user_apps,
            all_apps: apps,
        })
    }

    /// Sync summarized application usage events to Cloud Server via Outbox telemetry pipeline
    pub fn sync_apps_to_server(&self) -> Result<serde_json::Value, LocalAdminError> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let apps = self.get_grouped_app_usage("", None, None)?;
        let mut queued_count = 0;

        for (idx, app) in apps.iter().enumerate() {
            let classification = self.classify_app(&app.app_name);
            let category_opt = Some(classification.category_id);

            let payload = serde_json::json!({
                "app_name": app.app_name,
                "duration_ms": (app.total_duration_minutes as i64) * 60_000,
                "session_count": app.session_count,
                "start_time": app.first_seen_at.to_rfc3339(),
                "end_time": app.last_seen_at.to_rfc3339(),
                "category": category_opt,
            });

            let clean_app = app
                .app_name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '.' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect::<String>();
            let event_id = format!(
                "app-sync-{}-{}",
                clean_app,
                now.timestamp_micros() + idx as i64
            );

            let payload_str = serde_json::to_string(&payload).unwrap_or_default();
            if self
                .storage
                .enqueue_outbox(&event_id, "app_usage", now, &payload_str)
                .is_ok()
            {
                queued_count += 1;
            }
        }

        self.storage
            .set_meta("last_app_usage_synced_at", &now_str)?;

        Ok(serde_json::json!({
            "success": true,
            "synced_at": now_str,
            "apps_synced": queued_count,
            "total_apps": apps.len(),
            "message": format!("Đã xếp hàng đồng bộ {} ứng dụng lên máy chủ đám mây", queued_count)
        }))
    }

    // Keyboard & Keyword Monitoring APIs
}
