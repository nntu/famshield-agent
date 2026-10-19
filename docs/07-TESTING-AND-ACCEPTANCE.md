# Testing Strategy and Release Acceptance

Testing must prove behavior across policy logic, Windows lifecycle events, unreliable networks, upgrades, and tenant boundaries. A successful compile or isolated happy-path test is not sufficient for an endpoint security product.

## Test layers

### Unit and property tests

The Rust `core` crate tests domain normalization, precedence, schedule boundaries, DST behavior, quotas, temporary allowances, policy validation, and reason codes without Windows dependencies. Property and fuzz tests cover DNS packet parsing, domain patterns, IPC frames, policy JSON, and event payloads. Core decision branches target at least 90 percent line coverage.

Go unit tests cover authorization, idempotency, policy publication, command transitions, retention, and error mapping. Repository tests run against PostgreSQL rather than replacing transaction behavior with mocks.

### Contract tests

Generate fixtures from `policy.schema.json` and validate them in both Rust and Go. Test every OpenAPI request and response example. Compatibility tests prove that an older supported agent ignores optional additions, rejects unsupported required behavior, and retains the last valid policy.

### Windows integration tests

Automated Windows virtual-machine tests must verify:

- service install, start, stop, crash recovery, upgrade, rollback, and uninstall;
- standard-user inability to alter protected state;
- logon, logoff, lock, unlock, idle, sleep, resume, and Fast User Switching;
- foreground-time accounting within a five-second tolerance for controlled scenarios;
- running, foreground, and accumulated usage states remain distinct when an application moves to the background;
- game launchers, renderer child processes, crash-restarted games, and multiple instances do not double-count gameplay;
- guardian category override takes precedence over catalog classification, and application-specific rules take precedence over the all-games rule;
- a stopped or disconnected device cannot leave stale applications displayed as currently running;
- application rule precedence and structured enforcement failures;
- IPv4 and IPv6 DNS over UDP and TCP, malformed packets, SafeSearch, direct-DNS blocking, and network restoration;
- offline policy, queued events, reconnect, and duplicate batch delivery;
- named-pipe authentication and malformed or oversized messages.

### Server and web integration tests

Test enrollment replay, revoked devices, expired commands, command redelivery, batch partial acceptance, pagination, concurrent policy publication, audit atomicity, and cross-tenant denial. Browser tests cover RBAC, policy editing validation, device assignment, temporary allowances, status, reports, and accessible error states.

## Performance and endurance gates

- Run the Windows agent for at least 24 hours through network changes, sleep/resume, and application churn without unbounded handle, task, memory, database, or outbox growth.
- Handle DNS bursts without blocking the service control path or corrupting responses.
- Upload the maximum event batch within configured request limits and process duplicate delivery without duplicate rows.
- Keep idle agent resource usage within a release budget recorded by CI; a regression requires an explicit approval rather than an undocumented fixed number.

## Security gates

Release tests include request replay, policy tampering, update tampering, invalid certificate, cross-tenant identifiers, path traversal, unsafe archive contents, IPC impersonation, oversized payloads, SQL injection, and authorization matrix coverage. Fuzz regressions become permanent corpus cases.

## Acceptance scenarios

V1 release requires end-to-end evidence for:

1. Enroll device, assign child and policy, enforce while online.
2. Disconnect for 24 hours, preserve policy and usage, then reconnect without duplicate events.
3. Apply and expire a temporary application or domain allowance.
4. Discover an unknown application, classify it as a game, apply an all-games schedule, and synchronize the result.
5. Measure only active, unlocked, non-idle foreground application and game time while ignoring a background launcher.
6. Block configured domains while accurately labeling allowed-domain telemetry.
7. Upgrade successfully and automatically roll back a deliberately unhealthy build.
8. Revoke a device so it cannot synchronize while local state remains recoverable for authorized uninstall.

Every release artifact records test results, schema versions, supported upgrade paths, and known enforcement limitations.
