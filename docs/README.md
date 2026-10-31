# FamShield technical documentation

This directory contains product requirements, design specifications, operational guidance, and implementation plans for FamShield Agent. A document can describe a target or proposal; its presence does not mean the feature is implemented or released.

For behavior available in the current agent, check the code and the root [README](../README.md). For changes shipped in a release, check the [changelog](../CHANGELOG.md). The API and policy schemas under [`contracts/`](../contracts/) define machine-readable formats.

## Start with the document that matches your task

| Need | Document |
| --- | --- |
| Product scope, actors, requirements, and non-goals | [01 Product requirements](01-PRODUCT-REQUIREMENTS.md) |
| Components, trust boundaries, and data flows | [02 System architecture](02-SYSTEM-ARCHITECTURE.md) |
| Windows agent design | [03 Windows agent](03-WINDOWS-AGENT.md) |
| Agent API contract | [04 Control plane API](04-CONTROL-PLANE-API.md) and [`contracts/openapi.yaml`](../contracts/openapi.yaml) |
| Data and policy semantics | [05 Data and policy](05-DATA-AND-POLICY.md) and [`contracts/policy.schema.json`](../contracts/policy.schema.json) |
| Security and privacy requirements | [06 Security and privacy](06-SECURITY-AND-PRIVACY.md) |
| Test strategy and acceptance criteria | [07 Testing and acceptance](07-TESTING-AND-ACCEPTANCE.md) |
| Planned implementation work | [08 Implementation plan](08-IMPLEMENTATION-PLAN.md) |
| Migration and operational guidance | [09 Migration and operations](09-MIGRATION-AND-OPERATIONS.md) |
| Web console design | [10 Web console](10-WEB-CONSOLE.md) |
| Engineering conventions | [11 Engineering standards](11-ENGINEERING-STANDARDS.md) |
| Updates and versioning | [12 Auto-update and versioning](12-AUTO-UPDATE-AND-VERSIONING.md) |
| Deployment proposal | [13 Deployment guide](13-DEPLOYMENT-GUIDE.md) |
| Known gaps and proposed roadmap | [14 Gap analysis and roadmap](14-GAP-ANALYSIS-AND-ROADMAP.md) |
| Feature-specific proposals | [15 TOTP plan](15-PARENT-2FA-TOTP-PLAN.md), [16 DNS cache and upstream plan](16-NUMA-DNS-CACHE-AND-MULTI-UPSTREAM-PLAN.md) |
| Parent installation and first-run setup | [17 Installation guide](17-INSTALLATION-AND-INITIAL-SETUP-GUIDE.md) |
| Tenant synchronization | [18 Client tenant sync](18-CLIENT-TENANT-SYNC-GUIDE.md) |
| CLI category import | [19 CLI category import](19-CLI-CATEGORY-IMPORT-AND-TAXONOMY-GUIDE.md) |

## How to interpret these documents

- Requirements, plans, and proposals describe intended behavior. Verify implementation in the agent source before treating a statement as current product behavior.
- Use the OpenAPI and JSON Schema files for wire-format details. Update the relevant contract and its consumers together when changing a format.
- Treat the gap analysis as a dated assessment. Recheck repository state and linked projects before acting on its findings or priorities.
- Keep user-facing installation instructions aligned with the current installer, setup wizard, and root README.

## License

Unless a separate notice says otherwise, the repository uses the [MIT License](../LICENSE). Third-party dependencies and assets retain their own licenses. The license does not grant rights to the FamShield name or logo.
