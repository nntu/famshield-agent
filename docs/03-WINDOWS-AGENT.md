# Rust Windows Agent Specification

The endpoint consists of a privileged Windows service and an unprivileged per-session helper. The service is authoritative for policy, persistence, enforcement, and server communication. The helper supplies interactive-session signals and user notices. Both components must be native x64 binaries for the V1 target.

## Rust workspace boundaries

- `core`: policy types, domain matching, schedules, quotas, decisions, event types, visual verification triggers, and clocks expressed as traits.
- `storage`: SQLite migrations, repositories, policy cache, command journal, outbox, visual evidence cache, retention, and recovery.
- `dns`: UDP and TCP listeners, DNS parsing, filtering, SafeSearch rewriting, cache policy, upstream DoH, and WFP engine reconciliation.
- `windows`: safe wrappers for services, sessions, processes, ETW (`Microsoft-Windows-Kernel-Process`) tracing, WDAC/AppLocker management, User-Mode WFP (`fwpmu.dll`), signer inspection, Registry, DPAPI, firewall, adapters, power events, and named-pipe ACLs.
- `service`: lifecycle orchestration, health state, enforcement, configuration reconciliation, and shutdown recovery.
- `session`: foreground and idle tracking, notices, time-request UI, and DXGI Desktop Duplication capture with local privacy blur.
- `local_admin`: local guardian authentication (Argon2id + DPAPI machine secret), local rule CRUD, local reporting queries, and cloud enrollment triggers.
- `sync`: enrollment, mTLS, heartbeats, policy download, event batches, visual evidence upload, commands, and backoff (active only in Online Mode).
- `updater`: signed manifest verification, staged replacement, health confirmation, and rollback.

Public Windows wrappers must contain and document `unsafe`; other crates should consume safe interfaces. Production code must not call command-line tools where a stable Windows API is already wrapped, except for a documented compatibility fallback.

## Core domain traits and type safety (`core`)

The `core` crate defines the pure business logic and contracts via traits, guaranteeing that policy evaluation, timekeeping, and decision-making can be thoroughly unit-tested without depending on Windows APIs or filesystem I/O:

- **`Clock` Trait:** Abstracts time to allow deterministic testing of schedules, daily midnight resets, and DST transitions without manipulating system clocks:
  ```rust
  pub trait Clock: Send + Sync {
      fn now_utc(&self) -> chrono::DateTime<chrono::Utc>;
      fn monotonic_now(&self) -> std::time::Instant;
  }
  ```
- **Typestate Machine for Operational Modes:** Guarantees at compile time that an unenrolled agent cannot initiate sync uploads, and an enrolled agent cannot accidentally execute unvalidated local overrides:
  ```rust
  pub struct LocalMode;
  pub struct OnlineManaged { credential: DeviceCredential };

  pub struct AgentController<Mode> {
      storage: Arc<dyn StorageBackend>,
      mode: Mode,
  }
  ```
- **Semantic Newtypes:** Enforce invariant validation at creation (Parse, Don't Validate) instead of runtime string parsing:
  - `NormalizedDomain`: Punycode-decoded, lowercased, trailing-dot-stripped FQDN.
  - `ExecutableIdentity`: Canonicalized path, Authenticode signer thumbprint, and optional SHA-256.
  - `MonotonicUsage`: Non-negative, un-skewable duration segment.


## Service lifecycle and mode resolution

At startup the service resolves its operating mode:
- **Local Mode (Default):** When no valid cloud credential exists, the service boots with local administrative authority. It activates `local_admin` listeners, executes rules from local SQLite, logs events locally, and disables external sync loops.
- **Online Mode:** When a device credential and tenant enrollment exist, the service launches the background `sync` loop, prioritizes cloud-assigned signed policies, and pushes outbox batches over mTLS.

Startup sequence:
1. Open the Windows Event Log and a rolling diagnostic log.
2. Open SQLite, run backward-compatible migrations, and verify schema state.
3. Load device identity, operating mode, Master PIN credentials, and active policy.
4. Restore incomplete update or network transactions.
5. Start local IPC, session tracking, DNS, process monitoring, and mode-specific endpoints (`local_admin` or `sync`).
6. Reconcile desired WFP filters, DNS proxy, browser GPO, and application rules.


Shutdown must stop accepting commands, flush usage segments, commit the outbox, stop listeners, and leave network state consistent with the requested stop mode. Upgrade stop preserves protection until the replacement is healthy; uninstall restores only settings owned by ChildProtect.

## Session and usage tracking

Track Windows logon session identity, child assignment, active console state, lock state, idle duration, foreground process, and monotonic start time. Count usage only when all required conditions are true. Close the current segment when any condition changes, every five minutes as a checkpoint, and before sleep, logoff, service stop, or policy replacement.

Do not use wall-clock subtraction to calculate duration. Use a monotonic duration and attach UTC start/end timestamps for reporting. Clamp or reject impossible durations after crash recovery. Multiple processes matching the same rule contribute to one foreground rule at a time.

Maintain three separate views:

- **Running:** user-session processes currently alive, excluding protected system and service processes not belonging to the child session.
- **Foreground:** the executable that owns the active foreground window in the active child session.
- **Usage:** committed monotonic foreground duration attributed to an application and applicable category.

Process start and stop transitions update the local active-process table immediately. Coalesce lifecycle telemetry to avoid uploading high-frequency process churn. A running snapshot is for near-real-time status and short retention; usage segments are the authoritative reporting source.

## Smart visual verification

To cross-check disputed telemetry (such as disguised applications or multi-window split screens) without compromising privacy, the endpoint supports strictly event-triggered visual verification frames:

- **Triggers:** A frame is captured ONLY when a block event occurs (blocked app execution or blocked domain access), when suspicious usage classification is flagged, or when the child explicitly submits a time-extension request. Continuous screen streaming or periodic background captures are prohibited.
- **Session 0 Isolation Handling:** Because the service runs in Session 0 without desktop access, the capture is executed by the unprivileged `session` helper residing in the child's interactive desktop session using the DXGI Desktop Duplication API (with GDI BitBlt fallback).
- **Privacy Transformation:** The helper immediately processes the frame in memory before serialization: downscales resolution to maximum 480p (thumbnail quality) and applies Gaussian blur over text and input areas to ensure passwords, private messages, and personal reading material cannot be reconstructed, while UI structure and application context remain identifiable.
- **Child Transparency:** Capturing a frame triggers an instant, unobtrusive visual indicator (e.g. system tray animation or toast notice) to ensure the child is aware of the verification.
- **Quota & Retention:** Each frame is encoded as compressed JPEG (< 150 KB). The service enforces a hard quota of max 50 captures per day per child and a local visual cache ceiling of 10 MiB to prevent SQLite bloat. Frames expire after 48 hours and are permanently deleted.

## Game and application discovery

On first observation of an executable identity, store first seen, last seen, canonical executable identity, verified signer, product metadata, installed-application identity when available, and classification status. Do not upload command lines, window titles, installation user paths, or file contents.

Classification precedence is:

1. Guardian override from the assigned policy.
2. Exact entry from a signed application catalog.
3. Verified signer and product mapping from the signed catalog.
4. `unknown`.

Supported categories are `game`, `browser`, `education`, `communication`, `media`, `productivity`, `development`, `system`, and `unknown`. Heuristics such as executable name, GPU usage, fullscreen mode, or installation directory may produce a local suggestion but must not become an authoritative category without catalog or guardian confirmation.

Track parent process and launcher relationships for identity resolution, but attribute active time to the executable owning the foreground window. A launcher remaining in the notification area must not continue game time. If a game starts a renderer or anti-cheat child process, a signed catalog may define those identities as one application family so usage is not split or double-counted.

## Application identity and enforcement

Normalize executable names case-insensitively. Canonicalize paths without following untrusted user-controlled reparse points during privileged operations. Collect signer identity from verified Authenticode data, not version metadata. Calculate SHA-256 only when a policy requires it and cache by stable file identity plus size and modification time.

Rule precedence is deterministic:

1. Emergency system allowlist.
2. Explicit temporary allowance.
3. Exact hash rule.
4. Verified signer and product rule.
5. Canonical path rule.
6. Executable-name rule.
7. Application-family rule.
8. Category rule, including all games.
9. Default action.

The decision engine returns action, matched rule, reason code, and enforcement tier. The service leverages Event Tracing for Windows (ETW) via `Microsoft-Windows-Kernel-Process` provider to detect process creation with near-zero latency, avoiding polling gaps. Enforcement operates through a hybrid approach:
- Native Windows Defender Application Control (WDAC) / AppLocker rules are generated dynamically for high-confidence policy blocks, preventing processes from executing at launch time.
- Immediate user-mode termination serves as an active fallback. The service inspects and handles the process tree and records access-denied, exited, protected-process, and identity-failure outcomes. This eliminates the need for third-party kernel drivers (`.sys`), avoiding driver signing expenses, anti-cheat conflicts, and kernel crash risks.

## DNS filtering, NUMA-inspired engine, and WFP network enforcement

To eliminate port conflicts with Windows' built-in `Dnscache` service (which binds `127.0.0.1:53`), the DNS proxy binds to `127.0.0.2:53` and `[::1]:53` over UDP and TCP. 

The DNS engine incorporates the proven high-performance patterns from the Numa architecture:
- **Zero-Dependency Wire Protocol:** Custom binary packet parser and encoder (`BytePacketBuffer`) operating directly on byte slices without heavy external DNS library overhead.
- **NRPT System Routing:** Configures the Windows Name Resolution Policy Table (NRPT) under `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig` to route all queries (namespace `.`) to `127.0.0.2:53`. This avoids mutating physical adapter IP configurations, preventing network disruption across Wi-Fi reconnects.
- **Serve-Stale Cache (RFC 8767):** Caches DNS responses with background prefetching for domains near TTL expiration (<10% remaining), and serves stale records during transient network outages so user applications do not hang.
- **Subdomain Tree Matching & Atomic Swap:** Domain rules employ right-to-left label normalization (e.g. `v16.tiktok.com` matches `tiktok.com`). Rule reloads build an in-memory `HashSet` asynchronously and execute a sub-microsecond atomic pointer swap without query lock contention.
- **RAM-Buffered SQLite Logging:** Incoming DNS observations are held in a RAM ring buffer and flushed in batches (up to 500 records or every 5 seconds) to SQLite, dramatically reducing SSD disk wear.

To enforce network boundaries and prevent circumvention without third-party kernel drivers:
- **User-Mode WFP Engine (`fwpmu.dll`):** Registers a dynamic WFP sublayer with `FWPM_SESSION_FLAG_DYNAMIC`. If the service terminates unexpectedly, Windows automatically releases the filters, eliminating network brick risk.
- **Port 53 Lockdown:** Drops all outbound UDP and TCP port 53 traffic originating from processes other than ChildProtect.
- **DoH / VPN Endpoint Blocking:** Blocks connections to known public DoH (DNS-over-HTTPS) and proxy endpoints as configured by policy.



## Local IPC and storage

Use a message-framed named pipe with a versioned protocol, maximum message size, request identifier, and explicit message allowlist. Authenticate the connecting Windows session and validate that it belongs to the claimed child. Never deserialize arbitrary types or accept filesystem paths or commands from the helper.

SQLite runs in WAL mode with foreign keys, a busy timeout, explicit transactions, integrity checks after unclean shutdown, and bounded retention. Secrets and private keys are protected with DPAPI machine scope and ACLs limited to `SYSTEM` and Administrators.

## Local administration and reporting (Local Mode)

When operating in Local Mode, the Windows service manages local configuration and reporting via the `local_admin` component:
- **Authentication:** Guardians establish a Master PIN or password during initial setup. The credential is salted with an endpoint-unique key and stored using Argon2id with DPAPI machine protection. Failed authentication attempts are rate-limited with exponential backoff.
- **Rule CRUD API:** Exposes a secure localhost REST or named-pipe API allowing local guardian interfaces to create, update, or remove application and domain rules, change schedules, and adjust quotas. All modifications are validated against `policy.schema.json` before committing to SQLite.
- **Local Reporting Engine:** Generates analytical summaries directly from SQLite tables:
  - Daily and weekly active foreground application breakdown.
  - Aggregated DNS domain counts and blocked domain alerts.
  - Inspection of event-triggered visual verification frames (rendered with privacy blur and 48-hour auto-expiration countdown).
- **Cloud Transition Trigger:** Provides an `enroll_to_cloud` procedure. When the guardian enters an enrollment code from the central cloud console, the agent generates device keys, establishes mTLS with the control plane, uploads local inventory, and switches to Online Mode without losing existing monitoring history.

## Health model

Each subsystem reports `healthy`, `degraded`, or `failed`, plus a stable reason code and last transition. Empty exception handlers are prohibited. Recoverable failures use bounded retry; permanent failures remain visible in local diagnostics, heartbeat data, and the web console.

