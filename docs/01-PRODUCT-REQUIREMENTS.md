# Product Requirements for ChildProtect Parental Control Platform

ChildProtect helps a verified guardian manage Windows usage for children across one or more devices. The system operates in two distinct operational modes:
1. **Local Mode (Standalone / Zero-Cloud):** Single-device management directly on the Windows endpoint. The guardian manages rules, views logs, and reviews visual verification frames locally using a Master PIN/Password. No cloud infrastructure, internet connectivity, or subscription is required.
2. **Online Mode (Centralized Cloud Management):** Multi-device fleet management connected to a Go control plane and Web Console. Guardians publish signed policies, queue remote commands, receive batched telemetry, and inspect cross-device reports.

The system must support seamless transition between Local and Online modes without data loss. Offline continuity is preserved in both modes.

## Actors and permissions

- **Local guardian (Local Mode):** Authenticates directly on the Windows endpoint using a salted Master PIN/password stored via DPAPI; authors local rules, manages schedules, inspects local logs, views local visual evidence, and initiates cloud linking.
- **Organization owner (Online Mode):** Manages billing scope, administrators, retention, and device enrollment.
- **Guardian administrator (Online Mode):** Manages child profiles, policies, devices, temporary exceptions, and reports from the central web console.
- **Read-only guardian (Online Mode):** Views status and reports without changing policy.
- **Child user:** Receives notices, views remaining time, and may submit a time request.
- **Device agent:** Enforces assigned or local policy, maintains SQLite storage, and uploads attributable events when enrolled online.

In Online Mode, every server-side operation is tenant-scoped. A guardian may access only tenants and child profiles granted by role-based access control.

## Operating modes and transitions

### Local Mode (Standalone)
- **Local Rule Management:** Guardians create, update, and delete rules directly on the endpoint (application blocklists, time schedules, quotas, web domain filtering, SafeSearch, WFP network restrictions).
- **Local Logging & Telemetry:** Monotonic foreground usage, process discovery, domain observations, and event-triggered visual verification frames are stored in SQLite.
- **Local Dashboard & Reporting:** A password-protected local console (localhost HTTPS or native desktop UI) provides daily/weekly charts, active process lists, recent blocks, and visual evidence review.
- **Privacy & Sovereignty:** 100% of telemetry and visual evidence remains on the device; zero data is transmitted over the internet.

### Online Mode (Cloud-Managed)
- **Enrollment:** An unmanaged or Local Mode device is linked to a cloud tenant using a short-lived 6-digit enrollment code.
- **Policy Precedence:** When enrolled, the cloud-assigned signed policy takes precedence over local draft rules. Local inventory and historical logs migrate seamlessly into the cloud account.
- **Remote Commands:** Real-time commands (immediate lock, temporary allowance, diagnostic collection, agent update).
- **Offline Resilience:** If disconnected from the server, the agent continues enforcing the last verified cloud policy as if operating locally.
- **Detachment:** A device can be unlinked from the cloud, immediately reverting to standalone Local Mode while retaining active protection.

## V1 functional requirements

### Device lifecycle


- Enroll a device using a one-time, short-lived code.
- Bind the installation to a stable device identity and unique credential.
- Report agent version, Windows version, capabilities, active sessions, policy version, health, and last contact.
- Revoke, rename, reassign, or retire a device without deleting its audit history.
- Continue enforcing the last verified policy during server or network outages.

### Application monitoring and control

- Identify applications by normalized executable name, canonical path, verified signer when available, and optional SHA-256.
- Discover applications executed in an eligible child session and maintain an inventory containing first seen, last seen, verified publisher, product identity, category, and classification source.
- Distinguish process presence, foreground use, and accumulated usage. A running background process does not count as active use.
- Classify software into `game`, `browser`, `education`, `communication`, `media`, `productivity`, `development`, `system`, or `unknown` using a signed catalog or guardian override; filename-only guesses are not authoritative.
- Track launcher-to-child-process relationships so a launcher running in the background is not reported as gameplay when the actual game is closed.
- Measure usage only while the child session is active, unlocked, not idle, and the application owns the foreground window.
- Support always-block, schedule, daily quota, temporary allowance, and warning-before-expiry rules.
- Support rules targeting one application or a category such as all games, with an application-specific rule taking precedence over its category rule.
- Record start, stop, accumulated duration, enforcement result, and failure reason without collecting window text by default.
- Integrate Event Tracing for Windows (ETW) process creation monitoring and native Windows Defender Application Control (WDAC) / AppLocker policy generation to complement user-mode termination and eliminate execution latency without third-party kernel drivers.
- Expose enforcement capability and fallback states on the device record.

### Web filtering and reporting

- Operate a local IPv4 and IPv6 DNS resolver and apply allowlists, blocklists, categories, schedules, SafeSearch mappings, and known VPN or DoH endpoint rules.
- Leverage Windows Filtering Platform (WFP) user-mode APIs (`fwpmu.dll`) to enforce network filtering rules, block direct external DNS (outbound UDP/TCP 53) bypasses, and block unauthorized DoH/VPN endpoints without requiring a third-party kernel-mode driver (`.sys`).
- Record normalized domains queried for address resolution, aggregated by child, device, domain, and time window.
- Clearly label reports as DNS-observed domains. V1 does not claim URL, page, tab, or attention accuracy.

### Smart visual verification (privacy-preserving)

- Support event-triggered visual verification frames to cross-check disputed usage or verify time requests (e.g., distinguishing educational apps from entertainment running simultaneously).
- Strictly event-driven: visual frames are triggered ONLY upon specific events (application blocked, domain blocked, or child time-extension request). Continuous screen streaming or periodic background captures are prohibited.
- Privacy-preserving transformation: capture frames are locally downscaled (max 480p) and processed with Gaussian text blur to obscure readable chat text, passwords, and private correspondence while retaining recognizable UI layout for guardian verification.
- Session transparency: an explicit visual notification (e.g., system tray icon animation or toast notice) notifies the child when a verification frame is taken. Covert capture is strictly prohibited.
- Ephemeral retention: verification frames have a maximum retention of 48 hours on both endpoint outbox and server storage, after which they are permanently purged.

### Central administration

- Create reusable versioned policies and assign them to children or devices.
- Show online state, policy compliance, agent health, recent blocks, recently running applications, foreground application time, game time, domain summaries, and event-triggered visual verification frames.
- Queue bounded commands: synchronize, apply policy, temporary allow, lock, unlock, collect diagnostics, and update agent.
- Record the actor, reason, target, request, result, and timestamp for every administrative action.
- Export reports without exposing secrets or raw device credentials.

## Reliability requirements

- Policy evaluation and core DNS handling must not depend on server availability.
- Events must enter a durable local outbox before upload and may be retried without duplication.
- Restarts, sleep, clock changes, network changes, and Fast User Switching must not corrupt usage totals.
- Failure to initialize a protection component must produce a health event and visible administrative alert.
- The service must restore or preserve a safe network configuration during upgrade, rollback, and uninstall.

## Product limits and non-goals

V1 does not include third-party kernel drivers (`.sys`), arbitrary command execution, message inspection, TLS interception, webcam or microphone collection, continuous screen recording/streaming, keylogging, clipboard capture, location tracking, or hidden/covert installation. Visual verification is limited exclusively to event-triggered, blurred, transparent frames as specified above. A future browser extension may collect active-tab URLs only after a separate privacy and threat review.

## Success criteria

V1 is acceptable when a guardian can enroll a Windows device, assign policy, see discovered and recently running applications, observe accurate foreground game and application duration, block an application or the game category, block configured domains and DoH bypasses via WFP, inspect event-triggered verification frames, issue auditable temporary exceptions, tolerate at least 24 hours offline, and recover queued events without loss or duplication after reconnection.

