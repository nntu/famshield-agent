# Security and Privacy Requirements

FamShield handles information about children and controls privileged Windows settings. Security, transparency, tenant isolation, and data minimization are release requirements rather than optional hardening work.

## Threat model

Protect against a standard child account attempting to stop the agent, change DNS, rename an application, replay a temporary allowance, alter the system clock, or delete local state. Protect the server from a compromised device submitting cross-tenant identifiers, duplicate events, oversized payloads, forged commands, or stale credentials. Protect endpoints from a compromised guardian account by limiting command types and preserving audit evidence.

V1 does not claim resistance to a local administrator, kernel-level malware, firmware compromise, or physical disk extraction without platform disk encryption. These conditions must be visible in capability and health reporting.

## Device identity and transport

- Enrollment uses a one-time code and an endpoint-generated private key.
- Device traffic uses TLS 1.2 or later and mutual client authentication.
- The private key is non-exportable when platform support permits; otherwise its serialized form is DPAPI-protected with machine scope and restricted ACLs.
- Revocation is checked on every device request at the application layer.
- Policy and update artifacts are signed by keys distinct from the online API credential.
- Signing keys are versioned, rotated, and supported through an explicit trust transition.

Do not disable certificate validation, accept arbitrary roots, or ship development credentials in production builds.

## Authorization and tenant isolation

The server derives tenant and device identity from authenticated context. It never authorizes using `tenant_id`, `device_id`, or `child_id` supplied only in a request body. Guardian actions require explicit permissions. Sensitive actions such as device revocation, retention changes, emergency unlock, and release publication require recent authentication and complete audit records.

Database access should use tenant-scoped repository methods and defense-in-depth row-level controls where operationally practical. Automated tests must attempt cross-tenant access for every resource family.

## Endpoint privilege boundaries

Only the service holds device credentials, edits protected machine state, opens the database for writes, or performs enforcement. The session helper is untrusted input. Named-pipe ACLs, peer session checks, message limits, timeouts, and schema validation are mandatory. All privileged filesystem operations reject traversal and unsafe reparse-point behavior.

Network filtering and process restrictions deliberately utilize Windows user-mode APIs (User-Mode WFP via `fwpmu.dll` and native WDAC / AppLocker) running as `LocalSystem`. Third-party kernel-mode drivers (`.sys`) are strictly avoided to eliminate Blue Screen of Death (BSOD) system crash hazards, bypass expensive WHQL EV certificate overhead, and prevent false-positive conflicts with gaming anti-cheat engines (e.g., Easy Anti-Cheat, Riot Vanguard).

Service binaries, policy cache, SQLite database, backup, and update staging directories allow only `SYSTEM` and Administrators to modify them. Diagnostic exports redact tokens, private keys, email secrets, recovery material, and personal file paths not required for support.

## Privacy rules and ethical visual verification

The product collects only the information needed for configured parental-control features, complying with international child protection standards (COPPA, GDPR-K) and local regulations (Vietnam Child Law 2016, Decree 13/2023/ND-CP on Personal Data Protection):

- Prohibited collection: Keystrokes, clipboard contents, private messages, full window titles, full command lines, continuous screen recording/streaming, microphone, camera, HTTPS payload interception, and covert remote shell remain strictly prohibited.
- Smart visual verification boundaries:
  - Verification frames are strictly event-triggered (app blocked, domain blocked, or child time-extension request). Periodic or continuous background recording is forbidden.
  - Privacy blur: Captured frames undergo immediate local downscaling (max 480p) and Gaussian text blur to obscure chat text, credentials, and confidential personal reading, while retaining recognizable high-level UI geometry for guardian verification.
  - Child transparency: Capture events display an instant visual indicator in the child's session. Stealth or stalkerware operation is prohibited.
  - Ephemeral retention: Verification frames are auto-deleted after 48 hours from both endpoint outbox and server storage.
- The Windows installation, tray status, web console, and product notices must identify that management is active. Reports distinguish measured foreground application time from DNS-observed domains. Guardians can configure shorter retention and delete eligible telemetry, while security audit records follow a separate documented retention rule.


## Updates and supply chain

Release manifests contain version, channel, target architecture, artifact digest, minimum supported version, rollout constraints, and rollback metadata. Agents verify the manifest signature and artifact digest before staging. Installation is atomic, retains one known-good version, and requires the new service to pass a health handshake before deleting the previous version.

CI pins toolchains and dependencies, generates a software bill of materials, scans dependencies, and signs release provenance. Secrets never enter build logs or repository files.

## Security response

Stable reason codes cover credential rejection, policy signature failure, rollback attempt, binary tampering, repeated IPC rejection, database integrity failure, and updater verification failure. Critical events are stored locally before alert delivery. A response runbook must support credential revocation, signing-key rotation, release withdrawal, forced upgrade, and tenant notification.
