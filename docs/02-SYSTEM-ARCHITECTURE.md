# System Architecture and Trust Boundaries

FamShield separates local enforcement from centralized control. The Windows agent owns real-time decisions and offline continuity. The server owns identity, policy authoring, fleet state, commands, reporting, and audit. No inbound Internet listener runs on an endpoint.

## Component model

```text
[MODE 2: ONLINE CLOUD]                              [MODE 1: LOCAL STANDALONE]
Guardian browser / Mobile                            Local Guardian (Endpoint)
       |                                                        |
       | HTTPS / Auth                                           | Master PIN / 2FA TOTP
       v                                                        v
Go Control Plane + NATS Broker                     Local Web Admin (127.0.0.1:38123)
       |                                                        |
       | Outbound Dual-Channel:                                 | secured local IPC
       | 1. REST/mTLS (Telemetry Batch v1.1 & Heartbeat)        |
       | 2. NATS WebSocket wss://:443 (Cloudflare Tunnel)       |
       v                                                        v
  =====================================================================
                    Rust Windows Service (LocalSystem)
                      |            |                   |
             SQLite Database   DNS Proxy           WFP Engine
             (Refinery V1-V6)  (Multi-Upstream,    (fwpmu.dll Dynamic)
             + Online Backup   Dynamic Balancer)   Process enforcement
                      |
                secured named pipe
                      |
               Rust session helper (Child Session)
                      |
                DXGI capture & local blur filter (5 levels)
  =====================================================================

### Windows service

The service runs under `LocalSystem`. It is the authoritative engine for both Local Mode and Online Mode. It verifies policy, owns SQLite, DNS listeners, Windows Filtering Platform (WFP) user-mode engine (`fwpmu.dll`) for packet-level DNS/DoH interception, ETW process tracing, WDAC / AppLocker rule orchestration, process enforcement, synchronization, updates, and recovery. In Local Mode, it exposes a local secured interface authenticated by a salted Master PIN/password to allow rule editing, log inspection, and local reporting without touching the network.

### Session helper

One helper runs in each eligible interactive child session (bypassing Windows Session 0 graphical isolation). It detects foreground application, idle, lock, logoff, sleep, and resume; captures event-triggered visual verification frames via DXGI Desktop Duplication API with immediate local downscaling and Gaussian text blur; displays notices; and submits time requests. It has no server credential and communicates only through a named pipe restricted to the service and the expected user session.

### Local admin console (Local Mode)

A responsive, keyboard-accessible local web administration interface running on `http://127.0.0.1:38123` (with dynamic fallback ports and CSRF protection), secured by Master PIN (Argon2id + DPAPI machine secret) and optional Two-Factor Authentication (2FA TOTP via Google/Microsoft Authenticator with emergency backup codes). It enables the local guardian to:
- Create, modify, and delete application and domain rules, schedules, and quotas.
- Query local SQLite logs to generate real-time usage charts, DNS activity, and block histories.
- Configure visual verification capture settings and blur intensity (5 levels: None, Low, Medium, Heavy, Maximum).
- Perform safe report log deletion without affecting rules or credentials.
- Initiate enrollment into Online Mode when centralized cloud management is desired.

### Control plane (Online Mode)

The Go API authenticates guardians and devices, enforces tenant boundaries, persists immutable policy versions, accepts idempotent event batches, and queues bounded commands. Background workers build aggregates, expire commands, evaluate alerts, and maintain retention. When a device is unenrolled, the control plane is completely bypassed.


## Principal data flows

### Policy synchronization

1. An administrator publishes an immutable policy version.
2. The server assigns that version to a child or device.
3. The agent heartbeat reports its current version.
4. The server returns a policy reference; the agent downloads and verifies the signed envelope.
5. The agent commits the policy to SQLite before activating it.
6. The agent reports success or a structured rejection reason.

### Event and visual verification upload

1. Local components append normalized events (and bounded, blurred visual verification frames) to the SQLite outbox in a transaction.
2. Visual frames are strictly event-triggered (app blocked, domain blocked, time extension request) and capped by a local disk quota (max 10 MiB) to avoid outbox bloat.
3. The sync loop reads a bounded batch and uploads it with a stable batch identifier.
4. The server inserts events idempotently using tenant and event identifiers. Visual frames are stored with a strict 48-hour TTL.
5. The server acknowledges accepted and rejected identifiers.
6. The agent removes only acknowledged events and quarantines permanently invalid records.

### Remote commands

FamShield utilizes a dual-channel remote command architecture:
1. **Primary Real-Time Push Channel:** When enrolled in Online Mode, the agent connects via **NATS over WebSocket (`wss://`) on standard port 443** (fully compatible with Cloudflare Tunnel and enterprise egress firewalls). It subscribes to `devices.{device_id}.commands` for sub-second command delivery and returns acknowledgments to `devices.{device_id}.command_results`.
2. **Secondary Heartbeat Reconciliation Channel:** Agents periodically poll `POST /v1/device/heartbeat` over HTTPS with exponential backoff and jitter as a fail-safe fallback.

Both channels are strictly **outbound** from the endpoint; the client agent never opens listening ports on local routers or endpoint firewalls. Commands contain a type, validated payload, issuance time, expiry, sequence, HMAC-SHA256 signature, anti-replay window (<= 300s), and audit reason. The agent persists a command to SQLite before execution and returns `accepted`, `succeeded`, `failed`, or `expired`. Duplicate delivery idempotently returns the stored result without repeating side effects.

## Availability and failure policy

- Server unavailable: retain policy, queue telemetry, and retry with exponential backoff plus jitter.
- Device credential rejected: stop upload and commands, retain local enforcement, and surface enrollment failure.
- Invalid or rollback policy: reject it and keep the last verified version.
- DNS upstream unavailable: follow policy `dns_failure_mode`; default to allow resolution with a health alert, while strict mode blocks.
- Session helper unavailable: continue network and process rules, report degraded usage accuracy, and restart the helper.
- SQLite unavailable: enter degraded enforcement using the last in-memory policy, stop destructive maintenance, and raise a critical health event.

## Repository target layout

```text
agent/crates/{core,storage,dns,windows,service,session,local_admin,sync,updater}
server/{cmd,internal,migrations}
web/{src,public}
contracts/{openapi.yaml,policy.schema.json}
docs/
```

Dependencies point inward: platform adapters depend on pure domain crates; `core` must not depend on Windows, HTTP, SQLite, or UI code.
