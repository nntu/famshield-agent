# Master Implementation Plan and Milestone Roadmap

ChildProtect delivery is structured into two sequential execution phases:
- **Phase 1: Local Standalone MVP (Offline-First / Zero-Cloud):** Ships a fully functional, self-contained Windows application where guardians manage rules, inspect logs, and review privacy-preserved visual verification frames locally using a Master PIN.
- **Phase 2: Online Centralized Management (Cloud Platform & SaaS):** Adds multi-device fleet enrollment, remote policy signing, push commands, and a multi-tenant web console via the Go control plane.

Development must proceed by independently verifiable milestones. Each pull request must satisfy the milestone exit criteria before advancing to the next.

---

## Phase 1: Local Standalone MVP (Offline-First)

### Milestone 0: Workspace Foundation & Architecture Bounds
- Establish Rust workspace (`agent/crates/{core, storage, dns, windows, service, session, local_admin, sync, updater}`).
- Configure Go module (`server/`) and web workspace (`web/`).
- Set up CI for Windows (MSVC) and Linux, enforcing rustfmt, clippy, cargo check, and contract validation.
- Implement compile-time architecture checks ensuring `agent/crates/core` remains strictly independent of platform APIs, filesystem, network, and GUI code.

*Exit Criteria:* Clean repository builds and passes all tests on Windows and Linux CI with zero warnings.

---

### Milestone 1: Contracts and Core Policy Engine
- Implement domain types and validation matching `contracts/policy.schema.json` in Rust and Go.
- Port domain normalization (subdomain hierarchy), rule precedence (app override > family > category), monotonic schedules, daily quotas, and temporary allowances.
- Generate golden fixtures covering DST shifts, midnight resets, conflicting rules, and malformed inputs.

*Exit Criteria:* Rust and Go engines validate the same fixture test cases with 100% identical decision outcomes.

---

### Milestone 2: Windows Service, SQLite Storage & Local Authentication
- **Service Shell (`service`):** Implement `LocalSystem` Windows service lifecycle, SCM dispatcher, structured Windows Event Log logging, and clean shutdown handler.
- **Persistence (`storage`):** Initialize SQLite in WAL mode (`PRAGMA journal_mode=WAL`, `synchronous=NORMAL`), migration runner, and table schemas (`local_rules`, `usage_segments`, `query_logs`, `visual_evidence`, `local_admin_auth`).
- **Master PIN Security:** Implement Argon2id password hashing with unique per-machine salt secured via Windows DPAPI machine scope. Implement failed-attempt lockout with exponential backoff.
- **Local IPC:** Set up message-framed Windows Named Pipe (`\\.\pipe\ChildProtectService`) protected with restrictive Windows Security Descriptors (SDDL), allowing only `LocalSystem` and verified interactive child sessions.

*Exit Criteria:* Service installs and starts cleanly; SQLite survives abrupt process termination; Master PIN authentication succeeds and rejects brute-force attempts; unprivileged users cannot write to the database or service pipe.

---

### Milestone 3: High-Performance DNS Engine & User-Mode WFP (NUMA Integration)
- **NUMA-Inspired DNS Engine (`dns`):**
  - **Loopback Binding:** Bind UDP and TCP listeners on `127.0.0.2:53` (and `[::1]:53`) to prevent address conflicts with Windows `Dnscache`.
  - **Binary Wire Parser:** Implement zero-dependency DNS packet buffer (`BytePacketBuffer`) for RFC 1035 wire encoding/decoding.
  - **Serve-Stale Cache (RFC 8767):** Cache upstream responses with background prefetching for records with <10% TTL remaining; return stale answers with TTL=1 on upstream network failure.
  - **Subdomain Blocklist Store:** Implement right-to-left label traversal (`v16.tiktok.com` matches `tiktok.com`) with sub-microsecond atomic pointer swapping (`ArcSwap<HashSet<String>>`) for zero-contention rule reloads.
  - **RAM-Buffered Log Flushing:** Buffer DNS query logs in a ring buffer and write to SQLite in batches (500 items or 5-second interval) to prevent SSD write wear.
  - **NRPT System Routing:** Register a Windows Name Resolution Policy Table (NRPT) rule under `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig` routing namespace `.` to `127.0.0.2:53`.
- **WFP Network Lockdown (`windows`):**
  - Integrate user-mode Windows Filtering Platform (`fwpmu.dll`) using `FWPM_SESSION_FLAG_DYNAMIC`.
  - Add filters blocking outbound UDP/TCP port 53 for all processes except the ChildProtect service.
  - Add filters dropping outbound connections to known public DoH (DNS-over-HTTPS) and proxy endpoints.

*Exit Criteria:* All system DNS queries route through `127.0.0.2:53`; external DNS and DoH bypasses fail; stopping or killing the service automatically releases WFP filters and restores normal network operation without bricking connectivity.

---

### Milestone 4: Process Monitoring, WDAC & Smart Visual Verification
- **Process Lifecycle & Identity:**
  - Consume Event Tracing for Windows (ETW) via `Microsoft-Windows-Kernel-Process` for 0ms process creation detection.
  - Implement Authenticode signature verification, canonical path resolution, and launcher-to-child relationship tracking.
  - Generate dynamic Windows Defender Application Control (WDAC) / AppLocker policies to prohibit blocked binaries at launch, backed by immediate user-mode termination fallback.
- **Usage Attribution:**
  - Session helper tracks active foreground window and user idle duration (`GetLastInputInfo`).
  - Calculate monotonic foreground usage segments; exclude background launchers and locked states.
- **Smart Visual Verification:**
  - Session helper captures screen frame via DXGI Desktop Duplication API (with GDI fallback) strictly upon trigger events (blocked app, blocked domain, or child time request).
  - Downscale frame to maximum 480p and apply Gaussian blur to text and input regions in memory to obscure personal text, messages, and passwords.
  - Transmit compressed JPEG (<150 KiB) via Named Pipe to service for storage in `visual_evidence`.
  - Display transparent visual toast / system tray notice to the child; enforce 48-hour automated deletion.

*Exit Criteria:* Games and restricted apps are intercepted immediately at launch; usage tracking accurately records foreground time across sleep/wake/lock cycles; visual verification frames capture blurred layouts and purge after 48 hours.

---

### Milestone 5: Local Management Interface (Local Mode MVP)
- Build a lightweight, responsive Local Management Console (embedded via WebView2 or local loopback HTTPS at `https://127.0.0.1:5380`):
  - **Authentication:** Master PIN login with session timeout.
  - **Rule Editor:** CRUD interfaces for application rules, daily quotas, weekly schedule windows, domain blocklists, SafeSearch, and WFP enforcement toggles.
  - **Local Dashboard:** Real-time active application indicator, daily quota progress bar, 7-day and 30-day foreground usage charts.
  - **Domain Query Log:** Filterable table of DNS-observed domains with search and category filters.
  - **Visual Verification Gallery:** Thumbnails of recent trigger events with countdown to 48-hour purge.
  - **Cloud Transition Wizard:** "Link to Cloud" dialog accepting a 6-digit enrollment code.

*Exit Criteria:* A guardian can install the package on a clean Windows machine, configure rules via Master PIN, observe live protection and reporting completely offline, and experience zero external network transmission.

---

## Phase 2: Online Centralized Management (Cloud Platform)

### Milestone 6: Go Control Plane & Device Enrollment
- **Control Plane Server (`server/`):**
  - Implement tenant isolation, RBAC (Owner, Guardian, Read-Only), and PostgreSQL schema migrations.
  - Implement short-lived (15-minute) 6-digit enrollment code generation.
  - Mutual TLS (mTLS) certificate issuance: Endpoint submits CSR and receives device certificate and CA trust chain.
  - Policy signing service: Signs canonical policy payloads with ECDSA (P-256) private keys.
- **Endpoint Cloud Link (`sync`):**
  - Implement `enroll_to_cloud` procedure: Validates enrollment code, generates private key in DPAPI, establishes mTLS session, and transitions agent state to Online Mode.
  - Sync local application inventory and historical usage records to server without data loss.

*Exit Criteria:* A Local Mode device connects to cloud, completes mTLS handshake, receives signed policy, and uploads historical inventory; data is strictly segregated between tenants.

---

### Milestone 7: Bidirectional Sync, Event Ingestion & Remote Commands
- **Event Outbox Batching:**
  - Agent batches normalized events from SQLite outbox (max 1,000 events or 1 MiB) and posts to `/v1/device/events:batch`.
  - Server processes events idempotently; agent removes acknowledged records.
  - Visual verification assets upload to tenant-isolated cloud storage with an automated 48-hour lifecycle policy.
- **Remote Command Dispatch:**
  - Agent maintains authenticated long-polling (`/v1/device/commands?wait_seconds=25`).
  - Implement command handlers: `sync_policy`, `temporary_allow`, `lock_session`, `unlock_session`, `collect_diagnostics`, `update_agent`.
  - Commands execute idempotently; results post back to server.

*Exit Criteria:* Server commands execute on endpoint in <2 seconds; event batches upload reliably; offline disconnections resume without event loss or duplication.

---

### Milestone 8: Central Cloud Web Console & Multi-Device Management
- Build multi-tenant React / TypeScript web console:
  - Multi-device fleet overview, policy assignment, and compliance health status.
  - Central rule authoring with version history, diff comparison, and one-click publishing.
  - Remote request approval: Guardian reviews time-extension requests and inspects attached blurred visual verification frames.
  - Push notifications (WebPush / Email) for critical security alerts and time requests.
  - Exportable audit logs and privacy-compliant reports.

*Exit Criteria:* Guardians can manage multiple child devices from any modern desktop or mobile browser; remote approvals apply within seconds.

---

### Milestone 9: Auto-Update, Packaging & Production Hardening
- **Agent Updater (`updater`):**
  - Manifest verification: Authenticate signed release manifest, channel, and SHA-256 hash.
  - Staged atomic upgrade: New service validates health check before replacing old binary; automatic rollback on failure.
- **Installer Packaging:**
  - Create Windows MSI installer / Inno Setup package bundling the Windows Service, Session Helper, and Local UI.
  - Automated installation, service registration, NRPT rule setup, and uninstallation cleanup.
- **Compliance & Penetration Testing:**
  - Security audit verifying zero keylogger/screenshot leaks, non-bypassable WFP filtering, and compliance with COPPA, GDPR-K, and Vietnam Decree 13/2023/NĐ-CP.

*Exit Criteria:* Clean install, upgrade, rollback, cloud link, cloud unlink, and uninstall workflows execute cleanly without leaving lingering network settings or system instability.

---

## Pull Request Completion Rules

Every pull request must explicitly state:
1. Target milestone and requirement reference.
2. Threat model impact and privacy assessment.
3. Schema / contract validation impact.
4. Automated unit, integration, and platform test results.
5. Rollback and failure behavior.
