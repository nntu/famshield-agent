# FamShield Parental Control Platform

FamShield is a modern parental-control platform for Windows supporting **Dual-Mode Operation**:
1. **Local Mode (Standalone / Zero-Cloud):** Operates completely offline on a single machine. Guardians configure rules, schedules, quotas, and view usage reports and visual verification frames locally via a password/PIN-protected interface. No cloud account or external connection is required.
2. **Online Mode (Cloud-Managed):** Enrolls the device into a Go-based control plane. Guardians distribute signed policies across multiple devices, receive batched events, issue remote commands (e.g., instant lock, allowance extensions), and monitor activity via a centralized web console.

Both modes share the identical Rust endpoint core, enforcing local policies via Windows Filtering Platform (WFP) user-mode filtering, ETW / WDAC process restrictions, and SQLite storage. Transitions between Local and Online modes are seamless without data loss.

The existing .NET Framework code remains the behavioral reference during migration. New implementation work must follow the specifications in [`docs/README.md`](docs/README.md).


## Target components

- `agent/`: Rust workspace for the Windows service, per-session helper, WFP network filter, DNS proxy, process monitor, local storage, synchronization, and updater.
- `server/`: Go API, workers, database migrations, and administrative services.
- `web/`: browser-based management interface.
- `contracts/`: machine-readable API and policy contracts shared across components.
- `docs/`: product, architecture, security, testing, migration, and delivery specifications.

## Product boundaries

FamShield reports foreground application usage and DNS-observed domains. Network enforcement uses Windows Filtering Platform (WFP) user-mode APIs to reliably prevent DNS and DoH bypass without third-party kernel driver risks. Application control combines user-mode termination with WDAC / AppLocker enforcement. Visual verification is strictly event-triggered (e.g., blocked attempt or time request) with privacy-preserving blur and transparent child notification; continuous screen recording, keystrokes, clipboard contents, messages, microphone, or covert surveillance are prohibited. Full browser URLs require a separately reviewed browser extension and are not part of V1.

## Implementation status

The core rewrite of FamShield has completed its foundation phase:
- **Rust Endpoint**: 9 crates implementing LocalSystem Service, Session Helper, NUMA DNS proxy, WFP network blocking, process tracker, DPAPI storage, and atomic auto-updater.
- **Go Control Plane**: High-throughput REST API integrated with **NATS JetStream** for low-latency event ingestion.
- **Web Console**: Modern React 19 + TypeScript + Vite 8 guardian dashboard.

## Quickstart & Deployment

- **Endpoint Agent Installation (Windows Admin)**:
  ```powershell
  # Build release binaries and install Windows Service + NRPT DNS route
  .\build.ps1 install
  ```
- **Control Plane & Web Console (Docker)**:
  ```bash
  # Starts NATS JetStream (:4222, :8222), Go API (:8080), and Web Console (:3000)
  docker compose up -d --build
  ```
- Detailed production architecture, mass deployment (GPO/Intune), Inno Setup installers, and operational runbooks are documented in [`docs/13-DEPLOYMENT-GUIDE.md`](docs/13-DEPLOYMENT-GUIDE.md).


