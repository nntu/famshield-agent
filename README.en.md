<p align="center">
  <img src="assets/logo.png" alt="FamShield" width="132">
</p>

<h1 align="center">FamShield Agent</h1>

<p align="center">
  A Windows endpoint protection and management agent built with Rust.
</p>

<p align="center">
  <a href="README.md">Tiếng Việt</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="docs/README.md">Technical documentation</a> ·
  <a href="https://github.com/nntu/famshield-agent/issues">Report an issue</a>
</p>

FamShield Agent runs on Windows and provides protection and management capabilities for families. It supports local operation without a cloud connection; the centralized management server will be deployed later.

## Choose a mode

| Mode | Best for | Cloud connection |
| --- | --- | --- |
| **Standalone (`StandaloneFree`)** | Configuring and managing the device locally. | Not required |
| **Online managed (`OnlineManaged`)** | Centralized device management once the server is deployed. | Planned for a future deployment |

Both modes use the Windows agent. Online Managed mode depends on the server service, which is planned for a future deployment.

## Key capabilities

- **High-performance DNS and network controls:** local DNS proxy integrated with a multi-upstream DNS routing engine featuring dynamic RTT learning (Dynamic Learning DNS Balancer), NUMA-aware lock-striped cache, and Windows Filtering Platform (WFP) dynamic session enforcement preventing DNS/DoH bypasses.
- **Comprehensive on-device management (Local Web Admin):** responsive, keyboard-accessible local Web Admin running on `127.0.0.1:38123` (with fallback ports), supporting two-factor authentication (2FA TOTP via Google/Microsoft Authenticator & emergency backup codes), application/domain rule management, real-time activity graphs, 5-level configurable screenshot blur, and safe report log deletion.
- **Identity & secrets protection:** Argon2id Master PIN authentication combined with machine-scoped Windows DPAPI encryption; local SQLite database managed by industrial-grade **Refinery 0.10** migrations with automatic pre-migration snapshot guards (SQLite Online Backup API) prior to schema changes.
- **Real-time app & usage tracking:** tracks active foreground application duration, periodically aggregates usage segments, and enqueues them into the telemetry outbox (`app_usage`).
- **Privacy by design:** visual evidence frames are strictly event-triggered (blocked domain/app, time extensions), processed with in-memory Gaussian Blur before disk storage or transport, and self-delete after 48 hours. Keyword detection evaluates in an ephemeral RAM buffer without persisting raw keystroke logs to disk.
- **Cloud synchronization & real-time dual-channel control:** Online Managed mode supports Telemetry Protocol v1.1 with adaptive outbox batching, HTTP connection pooling with TCP keepalive, and instantaneous remote commands via **NATS WebSocket (`wss://`) on standard port 443** (100% compatible with Cloudflare Tunnel and enterprise firewalls; strictly outbound connection requiring zero inbound NAT port forwarding).

## Architecture

The Rust workspace is split into crates by responsibility:

| Crate | Responsibility |
| --- | --- |
| `famshield-core` | Domain types, policies, and state |
| `famshield-service` | Windows Service, orchestration, and IPC |
| `famshield-session` | User-session component, notifications, and system UI |
| `famshield-dns` | Multi-upstream DNS proxy, dynamic RTT learning balancer, NUMA-aware cache, and domain policies |
| `famshield-windows` | Windows platform integration, WFP dynamic sessions, NRPT, and process monitoring |
| `famshield-local-admin` | Local Web Admin console, 2FA TOTP security, and on-device management API |
| `famshield-storage` | SQLite persistence, Refinery 0.10 migrations, pre-migration snapshot guard, and event outbox |
| `famshield-sync` | Telemetry v1.1, NATS WebSocket over Cloudflare Tunnel, HTTP connection pool, and remote command authentication |
| `famshield-updater` | Signature verification and agent updates |

## Get started on Windows

### System requirements

- Windows 10 (version 1809 or later) or Windows 11, x64.
- Administrator privileges to install the service and configure DNS.

### Install with the setup program

1. Download `FamShieldSetup-1.0.2.exe` from [GitHub Releases](https://github.com/nntu/famshield-agent/releases). If a release is not available yet, see [Build the installer](#build-the-installer) below.
2. Right-click the setup program, select **Run as administrator**, and approve the UAC prompt.
3. Follow the setup wizard. It registers the Windows Service, configures DNS for the agent, and launches the initial setup experience.
4. Create a Master PIN and complete the setup in Web Admin.

Web Admin normally opens at [http://127.0.0.1:38123](http://127.0.0.1:38123). If that port is unavailable, the agent tries fallback ports; the active port is recorded in `%ProgramData%\FamShield\admin_port.txt`.

> **Network note:** FamShield changes Windows DNS settings to enforce policies. During uninstall, the uninstaller resets adapter DNS settings to DHCP. If the device uses static DNS, record its configuration before installing or uninstalling.

### Uninstall

Run `unins000.exe` from the installation directory, usually `C:\Program Files\FamShield`, as Administrator and follow the confirmation prompts. If installed from source, you can uninstall with `.\build.ps1 uninstall` in an elevated PowerShell window.

### Install from source

Install the Rust toolchain, open PowerShell as Administrator in the repository directory, and run:

```powershell
.\build.ps1 install
.\build.ps1 status
```

### Build the installer

Install Rust, Node.js, and Inno Setup 6, then run this from PowerShell in the repository directory:

```powershell
.\build.ps1 package
```

The installer is generated at `installer\Output\FamShieldSetup-1.0.2.exe`.

## Development and testing

Common repository checks:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Documentation

- [Installation & Initial Setup Guide for Parents](docs/17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md)
- [Inno Setup Packaging & Architecture Overview](installer/README.md)
- [Architecture and specifications](docs/README.md)
- [Contribution guidelines](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [OpenAPI contract](contracts/openapi.yaml)
- [Policy JSON Schema](contracts/policy.schema.json)

## License

FamShield Agent is released under the [MIT License](LICENSE). Unless a separate license notice says otherwise, this license applies to the source code and documentation in this repository. Third-party libraries and assets remain subject to their respective licenses. The license does not grant rights to the FamShield name or logo.

## Contributing

Bug reports, improvement proposals, and code contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request, and use [GitHub Issues](https://github.com/nntu/famshield-agent/issues) to start a discussion.
