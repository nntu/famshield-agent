# Implementation Documentation Index

This directory is the source of truth for the FamShield rewrite. Implementers should read the documents in order because later contracts assume the product scope, trust boundaries, and terminology established earlier.

## Required reading order

1. [`01-PRODUCT-REQUIREMENTS.md`](01-PRODUCT-REQUIREMENTS.md) defines users, supported behavior, non-goals, and acceptance outcomes.
2. [`02-SYSTEM-ARCHITECTURE.md`](02-SYSTEM-ARCHITECTURE.md) defines component ownership, trust boundaries, and failure behavior.
3. [`03-WINDOWS-AGENT.md`](03-WINDOWS-AGENT.md) specifies the Rust service and per-session helper.
4. [`04-CONTROL-PLANE-API.md`](04-CONTROL-PLANE-API.md) specifies enrollment, synchronization, commands, and event upload.
5. [`05-DATA-AND-POLICY.md`](05-DATA-AND-POLICY.md) defines persistent entities, policy semantics, and retention.
6. [`06-SECURITY-AND-PRIVACY.md`](06-SECURITY-AND-PRIVACY.md) defines mandatory controls and prohibited collection.
7. [`07-TESTING-AND-ACCEPTANCE.md`](07-TESTING-AND-ACCEPTANCE.md) defines test layers and release gates.
8. [`08-IMPLEMENTATION-PLAN.md`](08-IMPLEMENTATION-PLAN.md) breaks delivery into independently verifiable milestones.
9. [`09-MIGRATION-AND-OPERATIONS.md`](09-MIGRATION-AND-OPERATIONS.md) defines C# migration, deployment, updates, rollback, and support.
10. [`10-WEB-CONSOLE.md`](10-WEB-CONSOLE.md) defines guardian workflows, pages, validation, and accessibility.
11. [`11-ENGINEERING-STANDARDS.md`](11-ENGINEERING-STANDARDS.md) defines repository, coding, error, dependency, and review conventions.
12. [`12-AUTO-UPDATE-AND-VERSIONING.md`](12-AUTO-UPDATE-AND-VERSIONING.md) defines Semantic Versioning, staged atomic auto-update, canary rollout, and automated watchdog rollback.
13. [`13-DEPLOYMENT-GUIDE.md`](13-DEPLOYMENT-GUIDE.md) provides comprehensive deployment methods, Docker infrastructure, operational runbooks, and clean uninstallation procedures.
14. [`14-GAP-ANALYSIS-AND-ROADMAP.md`](14-GAP-ANALYSIS-AND-ROADMAP.md) provides comprehensive gap analysis between Server and Agent ecosystems, technical matrix, and action-oriented completion roadmap.
15. [`15-PARENT-2FA-TOTP-PLAN.md`](15-PARENT-2FA-TOTP-PLAN.md) defines offline two-factor authentication (2FA / TOTP RFC 6238) for parent login via Google/Microsoft Authenticator and emergency backup codes.
16. [`16-NUMA-DNS-CACHE-AND-MULTI-UPSTREAM-PLAN.md`](16-NUMA-DNS-CACHE-AND-MULTI-UPSTREAM-PLAN.md) specifies NUMA-aware sharded DNS caching (lock striping, LRU, dynamic TTL wire rewrite, optimistic background prefetch) and multi-upstream DNS routing engine with parallel racing.
17. [`17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md`](17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md) provides a comprehensive, parent-friendly walkthrough for Windows installation, initial Setup Wizard configuration, 2FA/OTP activation, and anti-tampering clean teardown.
18. [`../HUONG-DAN-SU-DUNG-VA-TINH-NANG.md`](../HUONG-DAN-SU-DUNG-VA-TINH-NANG.md) tổng hợp toàn bộ tính năng và cẩm nang hướng dẫn sử dụng chi tiết cho phụ huynh và quản trị viên.


Machine-readable contracts live under [`contracts/`](../contracts/). The Markdown documents govern intent; schemas govern wire validation. A contract change requires updating both in the same pull request.

## Fixed V1 decisions

- The system supports Dual-Mode Operation: Local Standalone Mode (PIN/password protected on endpoint, zero cloud required) and Online Managed Mode (enrolled to Go control plane). Seamless transition between modes is supported.
- Windows endpoint components use Rust; the server API uses Go.
- The service communicates outbound over HTTPS only when enrolled in Online Mode.
- PostgreSQL is the server system of record for Online Mode; SQLite is the authoritative endpoint store, local policy database, and event outbox.
- Policy remains active offline using the last verified version.
- Application usage means active foreground time, not process lifetime.
- Website telemetry means DNS-observed domains, not browser history or full URLs.
- Network enforcement utilizes Windows Filtering Platform (WFP) user-mode APIs (`fwpmu.dll`) to block external DNS and DoH bypasses without loading third-party kernel drivers (.sys), eliminating WHQL/BSOD risks.
- Application blocking combines user-mode termination with native Windows WDAC / AppLocker policy generation and ETW process creation monitoring.
- Visual verification is permitted strictly as an event-triggered, privacy-preserving cross-check (blurred/low-resolution thumbnail on rule violation or time request) with visible notification and 48-hour ephemeral retention. Continuous screen recording, keystrokes, clipboard capture, HTTPS interception, covert surveillance, or arbitrary remote shell is prohibited.
- Architecture adheres to core Rust principles: type-driven modeling ("Parse, don't validate"), semantic newtypes, compile-time typestate state machines, zero-allocation hot paths, zero runtime panics, and guaranteed RAII resource cleanup (`Drop`) on all OS handles.



## Change control

Use an architecture decision record for changes to trust boundaries, languages, persistence engines, device authentication, policy signing, collection scope, or enforcement technology. Product text, API examples, schemas, tests, and migration behavior must stay consistent.
