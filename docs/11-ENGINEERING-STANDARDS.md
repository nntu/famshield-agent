# Engineering Standards and Rust Architectural Philosophy

These standards govern the FamShield platform across the Rust Windows agent, Go control plane, and web interfaces. They codify Rust's core philosophy—**type safety, zero-cost abstractions, fearless concurrency, RAII resource management, and explicit error handling**—into concrete engineering rules.

---

## 1. Rust Architectural Philosophy

### 1.1 Type-Driven Design: Make Illegal States Unrepresentable
- **Parse, Don't Validate:** Do not pass raw strings or primitives through internal pipelines hoping they are valid. Parse raw input at system boundaries into validated, strongly typed domain structures.
- **Newtype Pattern:** Wrap raw primitives in semantic newtypes to prevent argument transposition and ensure invariant validation at construction:
  ```rust
  pub struct NormalizedDomain(String);
  pub struct ProcessHash([u8; 32]);
  pub struct SessionId(u32);
  pub struct MonotonicDuration(std::time::Duration);
  pub struct MasterPinHash(String);
  ```
- **Typestate Pattern for State Machines:** Model lifecycle states as distinct types so invalid operations fail at compile time:
  ```rust
  pub struct StandaloneDevice;
  pub struct EnrolledDevice { credential: DeviceCredential };

  pub struct Device<State> {
      id: DeviceId,
      state: State,
  }

  // Outbox upload is ONLY available on EnrolledDevice at compile time
  impl Device<EnrolledDevice> {
      pub async fn sync_events(&mut self) -> Result<SyncResult, SyncError> { ... }
  }
  ```

### 1.2 Explicit, Exhaustive Error Handling (No Panics)
- **Zero Panics in Production:** `unwrap()` and `expect()` are strictly forbidden in production paths. In unit tests and compile-time constants, `expect("descriptive invariant")` is acceptable.
- **Typed Error Enums:** Every crate defines an explicit, exhaustive error enum using `thiserror`. Errors must map directly to stable machine-readable reason codes:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum DnsError {
      #[error("WFP engine binding failed: {0}")]
      WfpBindFailed(u32),
      #[error("Malformed DNS packet: {0}")]
      MalformedPacket(String),
      #[error("Upstream timeout after {0:?}")]
      UpstreamTimeout(Duration),
  }
  ```
- **Conversion Discipline:** Implement `From` only for unambiguous, lossless conversions between layer boundaries. Never erase error context.

### 1.3 RAII & Panic Safety for Windows OS Resources
- **Guaranteed Cleanup via `Drop`:** Any system-wide mutation (WFP sublayers, NRPT registry entries, Named Pipe handles, impersonation tokens) must be encapsulated in an RAII guard struct implementing `Drop`:
  ```rust
  pub struct WfpSessionGuard {
      engine_handle: HANDLE,
  }

  impl Drop for WfpSessionGuard {
      fn drop(&mut self) {
          // Guaranteed cleanup even if worker threads panic
          unsafe { FwpmEngineClose0(self.engine_handle); }
      }
  }
  ```
- Use `FWPM_SESSION_FLAG_DYNAMIC` so the Windows kernel automatically tears down WFP filters if the process terminates abruptly, preventing network bricking.

### 1.4 Memory Ownership, Borrowing & Zero-Cost Abstractions
- **Zero-Allocation Wire Hot Paths:** Packet parsing (`wire.rs`, `buffer.rs`) and ETW event consumption must borrow byte slices (`&[u8]`) directly from network buffers, avoiding heap allocation (`String`, `Vec`) in high-frequency loops.
- **Lock-Free Read Operations:** Use `ArcSwap` or sharded RCU structures for data read frequently and updated rarely (e.g. domain blocklists):
  ```rust
  pub struct BlocklistStore {
      domains: arc_swap::ArcSwap<std::collections::HashSet<String>>,
  }
  // Lookup is 100% lock-free and sub-microsecond; updates execute atomic pointer swap
  ```
- **Bounded Channels & Ring Buffers:** Thread boundaries communicate exclusively via bounded channels (`tokio::sync::mpsc` or `crossbeam-channel`). Unbounded queues are prohibited.

### 1.5 Concurrency Architecture & Runtime Isolation
- **Strict Separation of I/O and CPU/OS Work:**
  - The Tokio async runtime is reserved exclusively for asynchronous network I/O (`sync` mTLS, `local_admin` HTTP server).
  - CPU-bound cryptographic operations (Argon2id, Authenticode signature verification), synchronous SQLite queries, and blocking Windows APIs must execute via `tokio::task::spawn_blocking` or dedicated OS worker threads.
- **Never Hold Locks Across Await Points:** Synchronous mutex guards (`std::sync::MutexGuard`) must never be held across `.await` points to prevent executor thread starvation and deadlocks.

---

## 2. Workspace Crate Boundaries (Dependency Inversion)

Dependencies point strictly inward toward pure domain crates:

```text
[service] ────► [windows] ────┐
   │                          ▼
   ├──────────► [dns] ────► [core] ◄──── [contracts]
   │                          ▲
   ├──────────► [session] ────┤
   │                          │
   └──────────► [storage] ────┘
```

- **`agent/crates/core`:** Pure business logic. Must be `#![no_std]` capable or depend only on the Rust standard library. Strictly forbidden from importing Win32 APIs, SQLite, HTTP, or UI libraries.
- **`agent/crates/windows`:** Isolated platform layer. Encapsulates all Win32 FFI and `unsafe` blocks behind idiomatic, safe Rust traits.
- **`agent/crates/dns`:** Zero-dependency wire parser, Serve-Stale cache, and blocklist matching.
- **`agent/crates/storage`:** SQLite migrations, outbox management, and local encryption.
- **`agent/crates/session`:** Desktop helper, foreground monitor, DXGI screen capture, and privacy blur filter.
- **`agent/crates/local_admin`:** Localhost management API and Master PIN authentication.
- **`agent/crates/sync`:** Cloud mTLS communication and telemetry dispatch (Online Mode).
- **`agent/crates/updater`:** Staged atomic updater and rollback engine.

---

## 3. Go Server Engineering Standards

- **Domain-Driven Packages:** Structure code by business domain (`tenant`, `device`, `policy`, `event`, `command`) rather than technical roles (`controllers`, `models`).
- **Context & Cancellation:** Every function crossing I/O boundaries accepts `context.Context`. Unbounded database queries or HTTP calls without timeouts are banned.
- **Strict Tenant Scoping:** Every SQL statement and cache key must include `tenant_id`. Handlers extract tenant identity exclusively from authenticated JWT / mTLS session context, never from client request bodies.
- **No Orphan Goroutines:** Every background worker is supervised with a lifecycle group, cancellation context, and panic recovery handler.

---

## 4. Frontend & Interface Standards

- **Strict Type Safety:** TypeScript types are generated or validated directly against `contracts/openapi.yaml` and `contracts/policy.schema.json`.
- **Zero Raw HTML Injection:** Never render dynamic content using `dangerouslySetInnerHTML`.
- **Auditable Security:** Authentication tokens in Web Console are stored in `HttpOnly`, `SameSite=Strict`, `Secure` cookies. Local Mode uses short-lived session tokens in memory.

---

## 5. Definition of Done (DoD)

A pull request is ready for merge only when:
1. **Compilation & Lints:** `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` pass on Windows and Linux CI.
2. **Contract Conformance:** Any data model change updates both `contracts/` and markdown documentation in the same commit.
3. **Deterministic Testing:** Time-sensitive, scheduling, and DST logic are verified with mock clocks implementing `core::traits::Clock`.
4. **Safety Justification:** Every single `unsafe` block in the codebase contains a `// SAFETY:` comment explaining the invariant upheld.
5. **Memory & Resource Leaks:** System resources (sockets, handles, filters, memory) are demonstrated to clean up completely upon shutdown or error.
