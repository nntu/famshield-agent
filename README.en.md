# FamShield Windows Endpoint Agent

[🇻🇳 Tiếng Việt](README.md) | [🇬🇧 English](README.en.md) | [🤝 Contribution Guidelines](CONTRIBUTING.md)

**FamShield Agent** is the Windows endpoint protection agent for the **FamShield Parental Control & Endpoint Security** ecosystem.

The project is built 100% in **modern Rust**, emphasizing zero memory corruption, robust safety without BSOD/crashes, and a lightweight background footprint (< 15MB RAM).

> **Note:** The cloud management server (Go Control Plane, NATS JetStream, Telegram Bot & Web Console) is located at:  
> 👉 [https://github.com/nntu/famshield-server](https://github.com/nntu/famshield-server)

---

## 1. Workspace Architecture (Idiomatic Rust)

The agent consists of 9 decoupled, modular crates:

| Crate | Role & Responsibilities | Status |
| :--- | :--- | :--- |
| **`famshield-core`** | Domain models, strongly-typed semantic Newtypes, policy rules, and State Machine. | **Implemented** ✅ |
| **`famshield-service`** | Windows Service SCM Dispatcher (`windows-service`), framed Named Pipe IPC server with SDDL ACL. | **Implemented** ✅ |
| **`famshield-session`** | User-session helper: Win32 GDI BitBlt Capture + Gaussian Blur, System Tray, Toast, Native PIN dialog. | **Implemented** ✅ |
| **`famshield-dns`** | Zero-copy DNS proxy on loopback (`127.0.0.2:53`), Serve-Stale Cache, ArcSwap lock-free Blocklist hot-swap. | **Implemented** ✅ |
| **`famshield-windows`** | WFP Port 53 filter (`FWPM_SESSION_FLAG_DYNAMIC`), NRPT Registry, ETW Kernel 0ms & ToolHelp32 monitor. | **Implemented** ✅ |
| **`famshield-local-admin`** | Local Web Admin Console (Axum :48123), bilingual SPA, domain/app rule CRUD, DNS telemetry, blurred evidence, Master PIN Argon2id + DPAPI. | **Implemented** ✅ |
| **`famshield-storage`** | Local SQLite (WAL mode): Durable Outbox, Error Deduplication, Query Logs, Visual Evidence. | **Implemented** ✅ |
| **`famshield-sync`** | Freemium Engine (`StandaloneFree` vs `OnlineManaged`), Telemetry Envelope, NATS JetStream, HMAC Auth. | **Implemented** ✅ |
| **`famshield-updater`** | ECDSA P-256 digital signature verification, atomic staging binary swap, 30s watchdog rollback. | **Implemented** ✅ |

---

## 2. Key Technical Features

### 🛡️ Windows Filtering Platform (WFP) Port 53 Enforcement
- Directly interfaces with Windows Filtering Platform API (`fwpmu.dll`) at `FWPM_LAYER_ALE_AUTH_CONNECT_V4` and `FWPM_LAYER_ALE_AUTH_CONNECT_V6`.
- **High-priority Permit rule** (Weight 15, `FWP_ACTION_PERMIT`): Allows loopback `127.0.0.0/8` and configured upstream DNS resolvers.
- **Block rule** (Weight 10, `FWP_ACTION_BLOCK`): Blocks all outbound Port 53 traffic, preventing DNS bypass via external resolvers (e.g., 8.8.8.8 or 1.1.1.1).
- Uses dynamic session (`FWPM_SESSION_FLAG_DYNAMIC`): Windows automatically purges registered filters if the agent stops or terminates abnormally, preventing network lockup or BSOD.

### 📷 Smart Visual Verification (Privacy-Preserving Screen Capture)
- Captures desktop visuals on violation events using Win32 GDI `BitBlt` from Desktop DC into an in-memory 32-bit DIB buffer.
- Downscales automatically to standard 480p and applies a 3-pass separable box blur (Gaussian blur approximation in RAM, $O(N)$ complexity) to blur private text and credentials.
- Encodes directly to valid JPEG stream via `jpeg-encoder`, keeping storage size strictly below 150 KiB.

### 🔐 Multi-Tier Master PIN Security (Argon2id + DPAPI + Lockout)
- Password hashing via Argon2id with cryptographically secure random salt (`OsRng`).
- Machine-scope encryption using Windows Data Protection API (DPAPI) to protect secrets in the SQLite database.
- **Brute-force protection**: Exponential lockout timer (30s to 15m) upon repeated failed attempts.
- **3-tier recovery model**: Administrator UAC elevation CLI (`--reset-pin`), offline emergency recovery key `FAM-XXXX-XXXX-XXXX`, and remote cloud reset.

### 💻 Local Web Admin & Centralized Management Console
- Runs directly on local Axum HTTP server at `http://127.0.0.1:48123` (Session 0 Service) with a high-performance **React 19 + TypeScript + Vite** frontend.
- Automatically launches in default browser upon post-install completion (`installer/FamShield.iss`) and accessible directly from Windows System Tray menu (`famshield-session`).
- Interactive **5-step Initial Setup Wizard** on first boot: Guides parents through secure initialization, Master PIN creation, and Emergency Recovery Key issuance (`FAM-XXXX-XXXX-XXXX`).
- **Comprehensive Centralized Management Server Settings**:
  - Customizable Server URL/Endpoint, Organization/Tenant ID, Device Friendly Name, and Enrollment Code.
  - Configurable telemetry & policy synchronization frequency (15s – 15m) and toggleable Strict TLS Verification for self-signed certificates in private enterprise/school LANs.
  - Integrated **Async Network Connection Probe** (Tokio TCP probe measuring round-trip latency in ms) and **Instant Policy Synchronization** trigger.
- Integrated **Audit-Only / Monitor Mode**: Instant toggle between Active Enforcement and Audit-Only monitoring. When active, blacklisted queries and violations are logged without blocking network traffic.
- Integrated **Tamper-Evident Backup & Disaster Recovery (.famshield-bak)**: Export and restore complete system configurations and rulesets backed by SHA-256 integrity checksums, operating 100% offline.
- Integrated **Comprehensive Activity & Screen Time Timeline**: Visually tracks total screen time today, applications launched, active time windows, granular durations, and full web browsing history in a unified chronological feed.
- Integrated **Inappropriate Keystroke Keyword Monitoring**: Real-time detection of profanity, cyberbullying, self-harm, and adult keywords using an in-memory 128-char sliding buffer (`KeystrokeSlidingBuffer`), Aho-Corasick multi-pattern search, and Vietnamese/teencode normalization. Strictly follows **Privacy-by-Design** with zero keylogging to disk.
- Seamless transition between **100% Offline StandaloneFree** and **Centralized OnlineManaged** modes.
- Full bilingual support (Vietnamese & English), Master PIN protection, and secure `HttpOnly; SameSite=Strict` cookies.

### 📊 Unified Telemetry Pipeline (Store-and-Forward)
- Durable **Store-and-Forward** offline-first telemetry backed by SQLite `outbox` table.
- Standardized **Telemetry Envelope** batching diverse events (Heartbeat, DNS Log, Error Log, Process Lifecycle, Security Alert) into unified transmissions.
- Optimized for transmission through Cloudflare Tunnel and NATS JetStream with payload compression and idempotency.
- Smart SHA-256 fingerprinting for Error Log Deduplication, grouping recurring faults and preventing log flooding.

### 💎 Freemium Architecture (Free Standalone vs Paid Cloud Managed)
FamShield Agent cleanly isolates two operational modes:
- **Free Standalone Mode (`StandaloneFree`)**:
  - **100% Free & Open Source**: Fully functional offline-first, maximum privacy with zero cloud account or external connectivity required.
  - Complete endpoint protection suite: Loopback DNS proxy, WFP Port 53 block, Master PIN Argon2id + DPAPI, brute-force lockout, GDI screen capture with Gaussian blur, foreground & AFK idle tracking, self-protection, and native Win32 System Tray & Dialogs.
- **Paid Cloud Managed Mode (`OnlineManaged`)**:
  - Enrolled via a 6-digit one-time code (`EnrollmentManager`).
  - Centralized multi-device management via Web Console & Telegram Bot.
  - Instant bi-directional command dispatch via NATS JetStream: Emergency session lock (`lock_session`), real-time blocklist hot-swap (`update_blacklist`), cloud PIN reset (`reset_pin`), and installer approval/rejection (`approve_install`/`deny_install`).
  - Cryptographic HMAC-SHA256 signature verification on all remote commands with anti-replay protection (maximum timestamp drift $\le 300\text{s}$).

---

## 3. Quickstart & Installation

### Requirements:
- Windows 10 (1809+) or Windows 11 (x86_64).
- Rust 1.78+ (`cargo`), Inno Setup 6 (for packaging the Windows installer).
- Administrator privileges for service installation.

### Management via PowerShell Script (`build.ps1`):
```powershell
# 1. Compile Release binaries and install Windows Service + NRPT Policy
.\build.ps1 install

# 2. Check service status and loopback DNS resolution
.\build.ps1 status

# 3. Reset Master PIN using Administrator privilege (UAC Elevation)
.\build.ps1 reset-pin -NewPin "8888"

# 4. Clean uninstall, fully restoring Windows default network configuration
.\build.ps1 uninstall
```

### Packaging Windows Installer (Inno Setup):
Build script located at [`installer/FamShield.iss`](installer/FamShield.iss):
```powershell
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\FamShield.iss
```
Output executable: `installer\Output\FamShieldSetup-0.1.0.exe`.

---

## 4. Build Quality Gates (Testing & CI)

```powershell
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
All **60 unit tests** across 29 test suites pass 100% green (DNS wire parser, WFP filter lifecycle, GDI capture blur, Argon2id lockout, DPAPI secret migration, SQLite Outbox retry backoff, Telemetry batching, ETW Kernel tracing, Online/Offline Signed Updater ECDSA P-256, and Freemium HMAC command authentication).

---

## 5. Development & Contributing

We welcome community contributions (bug fixes, new features, performance optimizations, and documentation)!

- 📜 **Contribution Guidelines**: Please review [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a Pull Request.
- 🚦 **Pull Request Quality Standards**:
  - Code must be formatted with `cargo fmt --all -- --check`.
  - Zero clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
  - All workspace tests must pass (`cargo test --workspace`).
  - Follow [Conventional Commits](https://www.conventionalcommits.org/) for git commit messages.
- 🐛 **Issue Reporting & Feature Requests**: Submit via [GitHub Issues](https://github.com/nntu/famshield-agent/issues) using provided templates.
