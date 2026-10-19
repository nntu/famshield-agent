# Migration, Deployment, and Operations

Migration replaces the C# service without losing policy, history, network ownership, or recovery capability. Cutover is transactional because the legacy and Rust DNS services cannot both bind local port 53.

## Legacy data import

The importer reads from the existing `%ProgramData%\FamShield` layout using least privilege. It supports `config.json`, security and license state where legally and technically compatible, usage history, visit logs, temporary allowances, and network ownership markers. Rust models map legacy fields into a new policy draft and emit a migration report for unsupported or ambiguous values.

DPAPI data is decrypted only on the original machine under an authorized service identity. Secrets that cannot be migrated safely are discarded and must be re-entered; they are never uploaded as diagnostic content. The importer copies source data to a restricted backup and does not mutate it before successful cutover.

## Cutover sequence

1. Verify server enrollment, signed target policy, disk space, rollback package, and administrator authorization.
2. Put the legacy agent into migration stop mode and flush logs.
3. Capture owned DNS, firewall, browser policy, and service settings.
4. Import data into a staged SQLite database and validate counts and policy.
5. Install and start the Rust service without enabling enforcement.
6. Run self-tests and server heartbeat.
7. Stop and disable the legacy service, transfer network ownership, then enable Rust enforcement.
8. Confirm DNS, application policy, session helper, event upload, and health.
9. Retain rollback assets for the configured stabilization period.

Failure before step 7 leaves the legacy product active. Failure after step 7 triggers network restoration and re-enables the legacy service when compatible. Never leave adapters pointing to a resolver that is not listening.

## Deployment environments

Use separate development, staging, and production tenants, databases, certificate authorities, signing keys, storage, and release channels. Production signing operations require restricted identities and auditable approval. Database migrations run as an explicit deployment step and remain backward compatible with the currently supported server and agent versions.

## Observability

Server metrics cover request latency and errors, enrollment, active devices, policy lag, command age, event ingestion lag, rejected events, worker failures, database saturation, and update rollout. Endpoint heartbeat reports subsystem state, last policy success, outbox depth, database health, resolver health, session-helper state, update state, and anonymized failure codes.

Logs use request, tenant, device, batch, policy, and command correlation identifiers while redacting credentials and private payloads. Alerts distinguish fleet-wide service incidents from one-device configuration failures.

## Backup and recovery

PostgreSQL uses encrypted backups with tested point-in-time recovery. Policy versions, audit logs, device revocation, and release metadata have documented recovery objectives. Restoring a database must not reactivate expired enrollment codes, revoked credentials, withdrawn releases, or completed commands.

Endpoint SQLite uses transactions and integrity checks rather than cloud backup. The outbox tolerates duplicate upload after recovery. A repair operation preserves evidence, creates a diagnostic result, and never silently resets enrollment or policy.

## Support and uninstall

Diagnostic bundles require guardian authorization, use an explicit allowlist, redact secrets, and expire server-side. Uninstall requires an authorized local or central operation, stops enforcement, restores only FamShield-owned changes, removes credentials and scheduled tasks, and retains or deletes telemetry according to the selected data-removal option. The server records retirement separately from telemetry deletion.
