# System Architecture and Trust Boundaries

FamShield separates local enforcement from centralized control. The Windows agent owns real-time decisions and offline continuity. The server owns identity, policy authoring, fleet state, commands, reporting, and audit. No inbound Internet listener runs on an endpoint.

## Component model

```text
[MODE 2: ONLINE CLOUD]                       [MODE 1: LOCAL STANDALONE]
Guardian browser / Mobile                     Local Guardian (Endpoint)
       |                                                 |
       | HTTPS / Auth                                    | Master PIN / Password
       v                                                 v
Go Control Plane (SaaS)                      Local Admin Console (UI)
       |                                                 |
       | outbound mTLS (when enrolled)                   | secured local IPC
       v                                                 v
  ==============================================================
                    Rust Windows Service (LocalSystem)
                      |       |             |
                SQLite DB   DNS proxy    WFP Engine (fwpmu.dll)
                      |       |             |
                      |   ETW / WDAC     Process enforcement
                      |
                secured named pipe
                      |
              Rust session helper (Child Session)
                      |
               DXGI capture & local blur filter
  ==============================================================
```

### Windows service

The service runs under `LocalSystem`. It is the authoritative engine for both Local Mode and Online Mode. It verifies policy, owns SQLite, DNS listeners, Windows Filtering Platform (WFP) user-mode engine (`fwpmu.dll`) for packet-level DNS/DoH interception, ETW process tracing, WDAC / AppLocker rule orchestration, process enforcement, synchronization, updates, and recovery. In Local Mode, it exposes a local secured interface authenticated by a salted Master PIN/password to allow rule editing, log inspection, and local reporting without touching the network.

### Session helper

One helper runs in each eligible interactive child session (bypassing Windows Session 0 graphical isolation). It detects foreground application, idle, lock, logoff, sleep, and resume; captures event-triggered visual verification frames via DXGI Desktop Duplication API with immediate local downscaling and Gaussian text blur; displays notices; and submits time requests. It has no server credential and communicates only through a named pipe restricted to the service and the expected user session.

### Local admin console (Local Mode)

A lightweight local administration interface (Windows native UI or localhost web server bound to `127.0.0.1` with self-signed TLS and CSRF protection) accessible only with the Master PIN. It enables the local guardian to:
- Create, modify, and delete application and domain rules, schedules, and quotas.
- Query local SQLite logs to generate real-time usage charts and block histories.
- Inspect privacy-blurred visual verification frames.
- Initiate enrollment into Online Mode when centralized management is desired.

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

Agents long-poll over outbound HTTPS. Commands contain a type, validated payload, issuance time, expiry, sequence, and audit reason. The agent persists a command before execution and returns `accepted`, `succeeded`, `failed`, or `expired`. Duplicate delivery must return the stored result without repeating side effects.


## Availability and failure policy

- Server unavailable: retain policy, queue telemetry, and retry with exponential backoff plus jitter.
- Device credential rejected: stop upload and commands, retain local enforcement, and surface enrollment failure.
- Invalid or rollback policy: reject it and keep the last verified version.
- DNS upstream unavailable: follow policy `dns_failure_mode`; default to allow resolution with a health alert, while strict mode blocks.
- Session helper unavailable: continue network and process rules, report degraded usage accuracy, and restart the helper.
- SQLite unavailable: enter degraded enforcement using the last in-memory policy, stop destructive maintenance, and raise a critical health event.

## Repository target layout

```text
agent/crates/{core,storage,dns,windows,service,session,sync,updater}
server/{cmd,internal,migrations}
web/{src,public}
contracts/{openapi.yaml,policy.schema.json}
docs/
```

Dependencies point inward: platform adapters depend on pure domain crates; `core` must not depend on Windows, HTTP, SQLite, or UI code.
