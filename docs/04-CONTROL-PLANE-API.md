# Control Plane API Contract

The control plane exposes versioned JSON APIs under `/v1`. Guardian requests use the product identity provider and tenant-scoped RBAC. Device requests use mutual TLS after enrollment. All timestamps are UTC RFC 3339 values, identifiers are opaque strings, and request bodies have documented size limits.

The normative endpoint skeleton is [`../contracts/openapi.yaml`](../contracts/openapi.yaml). Implementations may add response fields but may not change existing meaning without an API version change.

## Enrollment

An administrator creates a one-time enrollment code scoped to a tenant and optional child. The endpoint generates a local key pair and submits the code, public-key request, installation identifier, and capabilities. A successful response returns the device identifier, client certificate chain, server trust configuration, and initial policy reference.

Enrollment codes expire after 15 minutes, are stored hashed, and become unusable after the first successful transaction. Re-enrollment creates a new device credential and audit record; it must not silently assume the identity of a revoked device.

## Heartbeat and policy

`POST /v1/device/heartbeat` carries agent version, boot identifier, active sessions, subsystem health, current policy version, outbox depth, update state, and capability flags. The response contains server time, desired policy metadata, available release metadata, and a command-wait hint.

`GET /v1/device/policy/{version}` returns a signed immutable policy envelope. The signature covers canonical policy bytes, tenant, policy identifier, version, issued time, and content hash. Agents reject a different tenant, lower version unless explicitly authorized for rollback, invalid signature, unsupported schema, or expired emergency policy.

## Event ingestion

`POST /v1/device/events:batch` accepts a stable `batch_id` and at most 1,000 events or 1 MiB uncompressed, whichever comes first. Each event includes `event_id`, `device_id`, `child_id` when known, `occurred_at`, monotonic ordering metadata within a boot, type, schema version, and payload. Supported types include telemetry, health, blocks, and `visual.verification` metadata.

The server validates tenant ownership from the device credential rather than trusting body identifiers. A repeated event returns its previous status. The response lists accepted, duplicate, retryable, and permanently rejected identifiers. One invalid event must not reject the complete batch.

## Visual evidence upload

`POST /v1/device/evidence/{evidence_id}` handles bounded visual verification frame uploads:
- Requires device mTLS authentication.
- Request payload is limited to 200 KiB compressed JPEG.
- Payload includes `occurred_at`, `trigger_type` (`app_blocked`, `domain_blocked`, `time_requested`, `anomaly`), associated `event_id`, and blur confirmation flag.
- Server validates that `evidence_id` links to a valid in-flight event, stores the asset in tenant-isolated storage with a strict 48-hour automated deletion policy, and logs an audit record.
- Guardians retrieve evidence via `GET /v1/guardian/children/{child_id}/evidence/{evidence_id}` through a short-lived presigned URL. Every guardian view is recorded in the audit log.

## Commands


`GET /v1/device/commands?wait_seconds=25` implements authenticated long polling. V1 command types are:

- `sync_policy`
- `temporary_allow`
- `lock_session`
- `unlock_session`
- `collect_diagnostics`
- `update_agent`

Every command has an identifier, sequence, issue time, expiry, actor audit reference, type, and type-specific payload. Commands never contain shell text or executable paths supplied by a web user. `POST /v1/device/commands/{id}/result` records state transitions and a structured result. Execution is idempotent by command identifier.

## Errors, retries, and limits

Errors use this envelope:

```json
{
  "error": {
    "code": "policy_version_conflict",
    "message": "A newer policy is already assigned",
    "request_id": "opaque-id",
    "retryable": false
  }
}
```

Agents retry network errors, `408`, `429`, and `5xx` with exponential backoff and jitter. They honor `Retry-After`. Authentication, validation, unsupported schema, and revoked-device errors require state transition rather than blind retry. Guardian list APIs use cursor pagination and stable sort order.

## Server package boundaries

The Go server should separate transport, authentication, tenant authorization, domain services, repositories, asynchronous jobs, and observability. Handlers must not issue ad hoc cross-tenant SQL. Database transactions enforce idempotency, policy publication, command sequencing, and audit writes atomically.

## Guardian administration resources

Guardian APIs use `/v1/admin`, the same error envelope, cursor pagination, and optimistic concurrency through entity versions. Required resource operations are:

- `GET/POST /children` and `GET/PATCH /children/{id}` for minimal child profiles.
- `GET /devices`, `GET/PATCH /devices/{id}`, `POST /devices/{id}:revoke`, and `POST /devices/{id}/commands`.
- `POST /enrollment-codes` for one-time device enrollment.
- `GET/POST /policies`, `GET/PATCH /policies/{id}/draft`, and `POST /policies/{id}:publish`.
- `GET/PUT /assignments/{child_id}` for effective policy assignment.
- `GET /devices/{id}/applications` for discovered applications, classification, first and last seen, and recent running state.
- `PUT /devices/{id}/applications/{application_id}/classification` for an audited guardian category override.
- `GET /reports/app-usage`, `GET /reports/game-usage`, `GET /reports/domains`, and `GET /reports/blocks` with bounded date ranges.
- `GET /audit` for authorized audit review.

Draft editing validates the policy schema but has no endpoint effect. Publishing creates a new immutable version in the same transaction as the audit record. Assignment references only published versions. Device commands accept a typed payload, mandatory reason, and expiry; the server supplies sequence and audit reference.

List and report endpoints default to 50 rows and cap at 500. Report ranges default to seven days and cap at 90 days. CSV export is an asynchronous audited operation when the result may exceed a normal response limit.

Application inventory responses distinguish `running`, `foreground`, and historical `last_seen_at`. A device that has stopped reporting must never show stale process state as currently running. Each classification returns its source—guardian, signed catalog, suggested, or unknown—and catalog version when applicable.

## User Registration and Telegram Activation

To eliminate Gmail daily sending quotas and deliverability failures, FamShield utilizes Telegram Bot API for account activation and real-time alert delivery:

### 1. `POST /v1/auth/register`
- **Request Body:** `{ "email": "parent@example.com", "password": "...", "telegram_username": "optional" }`
- **Response:**
  ```json
  {
    "user_id": "usr-8f7b3e1a",
    "email": "parent@example.com",
    "status": "pending_activation",
    "telegram_bot_url": "https://t.me/FamShieldBot?start=act_8f7b3e1a90c21345",
    "activation_token": "act_8f7b3e1a90c21345",
    "activation_code": "582910",
    "expires_at": "2026-10-20T19:35:00Z"
  }
  ```

### 2. Telegram Bot Deep-Link Activation Workflow
1. User clicks `telegram_bot_url` (`https://t.me/FamShieldBot?start=act_...`).
2. Telegram opens and user clicks **START**.
3. Telegram Bot API sends update `/start act_...` to `POST /v1/telegram/webhook`.
4. Control Plane validates token, marks user `active`, pairs `telegram_chat_id`, and sends a welcome confirmation message.
5. Web Console polling `GET /v1/auth/status?user_id=...` immediately transitions to active and signs the guardian in.

### 3. `POST /v1/auth/activate`
Fallback endpoint supporting direct activation with either `token` or 6-digit `code`.

