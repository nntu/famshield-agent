# Changelog Management Rules (2-Tier Architecture)

**Scope**: All AI coding agents and human contributors modifying or releasing features.

## Core Principle: 2-Tier Changelog Architecture

To optimize readability, preserve deep architectural documentation, and minimize LLM context token usage, the repository strictly enforces a **2-tier changelog structure**:

1. **Root Summary ([`CHANGELOG.md`](../../CHANGELOG.md))**:
   - Executive / User-Facing Summary following [Keep a Changelog](https://keepachangelog.com/).
   - Every official release retains only **3–7 concise bullet points** highlighting user-facing, administrative, and critical architectural milestones.
   - Must include a direct link to the technical deep-dive:
     `> 📖 **Chi tiết kỹ thuật & mã nguồn thay đổi:** [docs/releases/vX.Y.Z.md](docs/releases/vX.Y.Z.md)`
   - The `## [Unreleased]` section lives at the top of this root file.

2. **Detailed Release Archive ([`docs/releases/vX.Y.Z.md`](../../docs/releases/))**:
   - Complete technical release specifications, schema migrations, IPC frame contracts, compiler flags, internal struct names, and affected file paths.
   - Never deletes or summarizes away deep technical findings.

## Execution Workflow

### 1. During Fast Inner Loop / Feature Development
- When implementing a feature, refactoring, fixing a bug, or updating security/telemetry:
- Append changes directly to the `## [Unreleased]` section in [`CHANGELOG.md`](../../CHANGELOG.md).
- Use standard Keep a Changelog categories:
  - `### Added`: New capabilities, APIs, crates, or UI components.
  - `### Changed`: Behavioral changes, performance tuning, refactorings.
  - `### Deprecated`: Features slated for removal.
  - `### Removed`: Deleted capabilities or obsolete files.
  - `### Fixed`: Bug fixes, spin-loop eliminations, memory leak resolutions.
  - `### Security`: Crypto updates, ACL hardening, DPAPI protections.

### 2. During Release Cut (Version Bump)
1. **Create Detailed Spec**: Create [`docs/releases/vX.Y.Z.md`](../../docs/releases/) containing the full in-depth notes, internal details, and migration steps.
2. **Convert Unreleased**: Rename `## [Unreleased]` in [`CHANGELOG.md`](../../CHANGELOG.md) to `## [X.Y.Z] - YYYY-MM-DD`.
3. **Summarize Root Entry**: Condense the root entry to 3–7 key bullet points and prepend the link to `docs/releases/vX.Y.Z.md`.
4. **Preserve History**: Never wipe out or overwrite past releases in either `CHANGELOG.md` or `docs/releases/`.

## Single Source of Truth
- Master AI Guide: [`../../AGENTS.md`](../../AGENTS.md)
- Root Changelog: [`../../CHANGELOG.md`](../../CHANGELOG.md)
- Releases Directory: [`../../docs/releases/`](../../docs/releases/)
