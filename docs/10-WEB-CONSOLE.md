# Guardian Web Console and Local Interface Requirements

The management interface operates in two complementary form factors depending on the system's operational mode:
1. **Local Admin Interface (Local Mode):** Single-endpoint dashboard running locally on the Windows machine (served on localhost over loopback HTTPS or embedded via native WebView2), secured by the Master PIN. Allows offline rule editing, local log review, visual evidence inspection, and cloud linking.
2. **Central Cloud Web Console (Online Mode):** Multi-tenant browser application hosted on the Go control plane, allowing remote fleet management, multi-device rule authoring, push notifications, and cross-device reporting.

Both interfaces share a consistent visual language, responsive layout, and reporting semantics.

## Navigation and pages (Cloud Web Console)

- **Overview:** online and degraded devices, policy drift, active alerts, recent blocks, and current-day usage.
- **Children:** profile, assigned devices, policy, allowance requests, current application state, application and game usage, and observed domains.
- **Devices:** enrollment, agent and Windows versions, capabilities, subsystem health, discovered applications, recently running applications, policy state, outbox lag, update state, and revocation.
- **Policies:** drafts, validation, version history, assignments, schedules, application rules, web rules, telemetry, and change comparison.
- **Requests:** pending, approved, denied, expired, actor, reason, attached privacy-blurred visual verification frame, and resulting command.
- **Reports:** foreground application duration, DNS-observed domains, block events with attached visual verification frames, export, and retention controls.
- **Audit:** administrative actions and security events for authorized roles.
- **Settings:** members, roles, tenant retention, release channel, notifications, and privacy controls.

## Navigation and pages (Local Admin Interface)

- **Dashboard:** current active application, daily usage quota progress, today's top games and apps, and recent block alerts.
- **Rules & Limits:** local CRUD management for application blocklists, time schedules, category quotas, domain blocking, SafeSearch, and WFP settings.
- **Local Reports:** 7-day and 30-day interactive charts of foreground app usage, DNS domain query logs, and block incident timeline.
- **Visual Evidence:** gallery of event-triggered, privacy-blurred verification frames with remaining retention countdown (48h).
- **Security & Cloud:** Master PIN change, session timeout settings, and the "Link to Cloud" wizard.


## Core workflows

Device enrollment displays a short-lived code and its expiry, then shows the new device pending first heartbeat. Policy editing is draft-based: validate continuously, show conflicting rules, summarize impact, require a change note, publish an immutable version, and assign it separately. Temporary allowance requires target, duration, child or device scope, and guardian reason.

When evaluating time-extension requests or investigating block incidents, guardians can inspect the event-triggered visual verification frame. The console renders the locally blurred thumbnail to verify the app layout without revealing sensitive text, displaying a clear notice that all visual verification assets are permanently purged within 48 hours.


The application inventory allows a guardian to search discovered software, inspect verified publisher and product identity, see first and last seen times, review the classification source, assign a category, and create a rule for one application or all applications in a category. The live view displays running and foreground states separately and marks data stale when the device heartbeat expires.

Potentially disruptive actions—lock, revoke, strict DNS failure mode, forced update, or uninstall authorization—show affected devices and require confirmation. The UI reports command state rather than claiming success when a request is merely queued.

## Reporting semantics

Application charts show active foreground duration and explain excluded locked, idle, and background time. Game reports aggregate only applications authoritatively classified as `game`, display classification source, and avoid counting a background launcher as gameplay. Domain reports use the label “DNS-observed domains” and explain caching and background lookups. The UI must not use “browser history” unless a future approved browser extension supplies that evidence.

Times render in the viewer's selected timezone while preserving UTC in APIs. Reports identify missing intervals caused by offline queues, degraded session helpers, disabled telemetry, or retention expiry.

## Validation and errors

Forms consume server reason codes and retain unsaved drafts after recoverable failure. Policy validation links each error to the exact rule and field. Optimistic-concurrency conflicts offer reload and compare; they never overwrite another administrator's changes silently.

Permission-denied, expired session, revoked device, offline device, command expiry, unsupported capability, and partial report data have distinct states. The console displays correlation identifiers for support without revealing internal stack traces.

## Accessibility and privacy

All workflows support keyboard navigation, visible focus, semantic labels, appropriate contrast, and screen-reader status announcements. Color is not the only status indicator. Child names and activity data are hidden from browser analytics, support widgets, and third-party scripts. Authentication tokens are not stored in local storage when a secure HTTP-only session design is available.

## Frontend boundaries

Use generated or validated API types from the OpenAPI contract. Keep server state, form drafts, and presentation state separate. Centralize permission checks for presentation but treat server authorization as authoritative. Automated browser tests cover each role, critical workflow, validation failure, and responsive layout.
