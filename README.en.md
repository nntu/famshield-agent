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

- **DNS and network controls:** local DNS proxy, domain filtering policies, and Windows Filtering Platform (WFP) enforcement.
- **On-device management:** responsive, keyboard-accessible local Web Admin with activity logs, policy configuration, and audit-only monitoring.
- **Access protection:** Master PIN authentication, local SQLite storage, and Windows DPAPI protection for secrets.
- **App and time policies:** supports application activity monitoring and configured usage limits.
- **Privacy by design:** visual evidence is event-triggered and blurred before storage; keyword detection uses an in-memory buffer and does not write raw keystroke sequences to disk.
- **Optional synchronization:** Online Managed mode supports policy synchronization, telemetry delivery, and authenticated remote commands.

## Architecture

The Rust workspace is split into crates by responsibility:

| Crate | Responsibility |
| --- | --- |
| `famshield-core` | Domain types, policies, and state |
| `famshield-service` | Windows Service, orchestration, and IPC |
| `famshield-session` | User-session component, notifications, and system UI |
| `famshield-dns` | DNS proxy and domain policy processing |
| `famshield-windows` | Windows integration, WFP, NRPT, and process monitoring |
| `famshield-local-admin` | Local Web Admin and management API |
| `famshield-storage` | SQLite database and event outbox |
| `famshield-sync` | Operating modes, synchronization, and remote command authentication |
| `famshield-updater` | Signature verification and agent updates |

## Get started on Windows

### System requirements

- Windows 10 (version 1809 or later) or Windows 11, x64.
- Administrator privileges to install the service and configure DNS.

### Install with the setup program

1. Download `FamShieldSetup-1.0.0.exe` from [GitHub Releases](https://github.com/nntu/famshield-agent/releases). If a release is not available yet, see [Build the installer](#build-the-installer) below.
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

The installer is generated at `installer\Output\FamShieldSetup-1.0.0.exe`.

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

## Contributing

Bug reports, improvement proposals, and code contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request, and use [GitHub Issues](https://github.com/nntu/famshield-agent/issues) to start a discussion.
