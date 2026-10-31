# Install FamShield Agent and complete first run

This guide covers installing the Windows agent and completing its current first-run setup. The installer configures a local Windows service and opens Web Admin. Keep the recovery key shown at the end of setup; it is required to reset a forgotten Master PIN.

## Requirements

- Windows 10 version 1809 or later, or Windows 11, x64.
- Administrator access to install the service and configure DNS.
- For source installation, a Rust toolchain and PowerShell. Inno Setup 6 is additionally required to build the setup program.

## Install from a release

1. Download the latest setup program from [GitHub Releases](https://github.com/nntu/famshield-agent/releases).
2. Run it as Administrator and approve the Windows UAC prompt.
3. Follow the installer. Web Admin normally opens at `http://127.0.0.1:38123`.
4. If that address does not open, check `%ProgramData%\FamShield\admin_port.txt` for the active port.

FamShield changes Windows DNS settings to apply its local rules. Record any static DNS configuration before installation. The uninstaller resets adapter DNS to DHCP.

## Complete first-run setup

The setup wizard asks you to:

1. Create and confirm a six-digit Master PIN.
2. Choose a profile: **Balanced**, **Strict**, or **Basic**. The wizard shows the profile's rules and limits; review them before continuing.
3. Choose `StandaloneFree` or `OnlineManaged`. `OnlineManaged` requires a server URL and enrollment code; availability depends on a compatible server deployment.
4. Save the recovery key displayed after initialization. Store it somewhere private and accessible to the parent or administrator.

The recovery key can reset the Master PIN if you forget it. Treat it as a credential and do not share it with the child account.

## Install from source

Open an elevated PowerShell window at the repository root and run:

```powershell
.\build.ps1 install
.\build.ps1 status
```

To build a setup program, install Rust, Node.js, and Inno Setup 6, then run:

```powershell
.\build.ps1 package
```

The generated installer is placed in `installer\Output\`.

## Uninstall

Run `unins000.exe` from the installation directory, usually `C:\Program Files\FamShield`, as Administrator. For a source installation, run `.\build.ps1 uninstall` in an elevated PowerShell window. DNS adapter settings are reset to DHCP during uninstall; restore your recorded static settings if needed.

For current feature and release information, see the repository [README](../README.md) and [changelog](../CHANGELOG.md). Design specifications and proposals are indexed in [technical documentation](README.md).
