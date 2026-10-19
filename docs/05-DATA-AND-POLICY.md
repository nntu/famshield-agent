# Data Model and Policy Semantics

PostgreSQL is authoritative for tenants, users, children, devices, policy versions, assignments, commands, received events, aggregates, releases, and audit. SQLite is authoritative only for the endpoint's device identity, last verified policy, local command journal, usage checkpoints, and unsent outbox.

## Server entities

- `tenants`: security and billing boundary.
- `users`, `tenant_memberships`, `roles`: guardian identity and permissions.
- `children`: managed person record with minimal profile data.
- `devices`: enrollment, assignment, credential state, capabilities, versions, and health.
- `policies`, `policy_versions`: mutable policy identity and immutable published content.
- `policy_assignments`: desired policy for a child or device with effective time.
- `commands`, `command_results`: ordered, expiring device actions.
- `events`: immutable normalized endpoint events partitioned by occurrence time.
- `applications`, `device_applications`, `application_classifications`: normalized application families, device observations, and audited guardian or catalog classification.
- `app_usage_daily`, `category_usage_daily`, `domain_activity_daily`: derived application, game-category, and domain reporting aggregates.
- `agent_releases`: signed manifests, channels, rollout percentage, and rollback state.
- `audit_log`: append-only administrative and security actions.

Every tenant-owned table includes `tenant_id`; unique keys and foreign keys include the tenant where practical. Deleting a guardian must not delete audit records. Device revocation immediately prevents sync while preserving reports according to retention policy.

## Endpoint SQLite tables

- `agent_meta(key, value)` stores schema, installation, and boot metadata.
- `policies(version, content_hash, envelope, verified_at, active)` stores verified policy.
- `outbox(event_id, type, occurred_at, payload, attempts, next_attempt_at)` stores durable events.
- `commands(command_id, sequence, payload, state, result)` makes execution idempotent.
- `usage_segments(segment_id, child_id, rule_id, start_utc, duration_ms, state)` checkpoints usage.
- `application_inventory(application_id, identity_hash, signer, product, category, category_source, first_seen, last_seen)` stores observed software without command lines or window titles.
- `active_processes(process_key, session_id, application_id, parent_key, started_mono, last_seen_mono, foreground)` supports running-state and launcher relationship tracking.
- `visual_evidence(evidence_id, event_id, occurred_at, trigger, payload, size_bytes, uploaded, expires_at)` stores ephemeral blurred frames, subject to a strict 10 MiB aggregate outbox quota and 48-hour automated expiration.
- `local_admin_auth(id, hash_algorithm, salt, credential_hash, failed_attempts, locked_until, updated_at)` stores DPAPI-secured Master PIN authentication for Local Mode.
- `local_rules(rule_id, rule_type, payload, enabled, created_at, updated_at)` stores user-defined rules created during Local Mode operation.
- `network_journal(setting, previous_value, desired_value, state)` supports safe restoration.

In **Local Mode**, telemetry events, usage checkpoints, and domain queries accumulate in SQLite and serve as the direct data source for the local dashboard. A local retention worker prunes logs older than the configured limit (default 30 days) without sending data to any external network. In **Online Mode**, events are ingested into the durable outbox and dispatched to the Go server in batches via mTLS, purging acknowledged records.

Migrations are append-only, transactional where supported, and tested from every released schema. Unknown future fields must survive policy caching even when the active agent ignores an optional field.


## Policy envelope

The payload follows [`../contracts/policy.schema.json`](../contracts/policy.schema.json). A published version is immutable and includes:

- identity, schema version, monotonically increasing policy version, issue time, and tenant;
- child and device applicability;
- schedules and timezone behavior;
- application identity rules and hybrid WDAC/AppLocker/user-mode enforcement actions;
- domain groups, categories, SafeSearch, WFP user-mode external DNS and DoH blocking, and DNS failure behavior;
- smart visual verification settings (enabled, triggers, blur level, daily quota, ephemeral retention);
- telemetry collection, retention, idle threshold, and warning settings;
- signature metadata outside the canonical payload.

The agent evaluates times in the policy's IANA timezone. DST gaps and repeated times use explicit library behavior covered by tests. Daily quotas reset at local midnight; monotonic duration prevents wall-clock changes from adding usage. A policy replacement does not reset already accumulated daily usage unless an explicit signed reset command is present.

## Decision semantics

All decisions return a machine-readable reason code. Application-specific rules override application-family and category rules; therefore one game may receive an exception without disabling the policy for all games. The full application precedence is defined in the agent specification. Domain evaluation order is:

1. essential system domains;
2. temporary allowance;
3. SafeSearch rewrite;
4. prohibited bypass infrastructure;
5. explicit allow or deny rule according to policy mode;
6. category and keyword rule;
7. schedule and quota;
8. policy default action.

Policy validation rejects duplicate identifiers, invalid domains, empty executable identities, overlapping schedule windows with contradictory actions, negative quotas, unsupported schema versions, and payloads over the configured limit.

## Event types

V1 accepts `agent.health`, `policy.applied`, `app.discovered`, `app.lifecycle`, `app.usage`, `app.blocked`, `domain.observed`, `domain.blocked`, `time.requested`, `visual.verification`, `command.completed`, and `security.alert`. `app.lifecycle` is coalesced process presence and is not usage time. Payloads exclude secrets, full command lines, window titles, installation user paths, and raw DNS packets.

`visual.verification` events contain event-triggered, privacy-blurred frame metadata (trigger reason, application/domain ID, timestamp, and asset reference). Image payloads are capped at 150 KiB and are subject to an automated 48-hour purge policy across endpoint and cloud storage.

Domain events are aggregated locally per minute before entering the outbox. Server reports may aggregate further but retain the original normalized event until retention expiry. Default endpoint retention is 30 days or until successful upload plus seven days; default server event retention is 90 days. Tenant policy may shorten these values.

