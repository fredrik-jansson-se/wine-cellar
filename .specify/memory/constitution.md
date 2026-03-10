<!--
SYNC IMPACT REPORT
==================
Version change: (unversioned template) → 1.0.0
Added sections:
  - Core Principles (I–IV): Code Quality, Testing Standards, UX Consistency, Performance
  - Additional Constraints (security, image handling, error surfacing)
  - Development Workflow (build gates, review, deployment)
  - Governance

Modified principles: N/A (initial ratification)
Removed sections: N/A (initial ratification)

Templates reviewed:
  ✅ .specify/templates/plan-template.md — Constitution Check section present; gates
       will reference the four principles defined here.
  ✅ .specify/templates/spec-template.md — User Scenarios & Requirements sections
       align with UX Consistency and Testing Standards principles.
  ✅ .specify/templates/tasks-template.md — Phase structure supports incremental
       delivery; test-optional flag aligns with Testing Standards principle.
  ✅ .specify/templates/constitution-template.md — source template; no changes needed.

Deferred TODOs:
  - None. All fields resolved from project context.
-->

# Wine Cellar Constitution

## Core Principles

### I. Code Quality

All Rust source MUST pass `cargo clippy` (zero warnings) and `cargo fmt` before any
build or merge. Code MUST follow idiomatic Rust 2024 edition conventions: `snake_case`
functions, `CamelCase` types, 4-space indentation.

Non-trivial functions (multi-step logic, non-obvious side effects, performance-sensitive
paths) MUST carry doc comments explaining intent, not mechanics.

Modules MUST remain focused on a single responsibility: `db` for all database queries,
`web` for HTTP concerns, `web/markup` for HTML generation. Cross-cutting concerns
(error types, shared state) live in their own files rather than being duplicated.

Dead code and unused imports MUST NOT be committed. `#[allow(...)]` suppressions require
an inline comment justifying the exception.

### II. Testing Standards

All database queries MUST use `sqlx::query!` macros for compile-time SQL validation.
Changing any SQL query REQUIRES running `cargo sqlx prepare` to regenerate `.sqlx/`
offline metadata before committing.

Integration tests that touch the database MUST use a disposable SQLite file (not the
development `data.db`) and MUST run migrations before executing assertions.

New handler logic that branches on user input or database state SHOULD have at least
one integration test covering the primary success path and one covering an error path.

Tests live in `src/` (unit) or `tests/` (integration). No test file may rely on
shared mutable global state.

### III. User Experience Consistency

The UI MUST remain server-side rendered via Maud templates. No client-side routing or
SPA frameworks may be introduced. JavaScript is permitted only for progressive
enhancement (e.g., image crop UI in `js/edit-image.js`).

All dynamic partial updates MUST use HTMX attributes (`hx-get`, `hx-post`,
`hx-target`, `hx-swap`). Full-page reloads are acceptable only for initial navigation.

Visual components MUST use Bootstrap 5 classes and follow existing modal, alert, table,
and form patterns established in `src/web/markup.rs`. Custom CSS is permitted only when
Bootstrap utilities are insufficient; custom styles MUST be scoped and documented.

Error states rendered to the browser MUST use the `AppError` HTML fragment (Bootstrap
alert) pattern from `src/web/error.rs`. Raw error strings or stack traces MUST NOT
be sent to the client.

### IV. Performance Requirements

HTTP handlers MUST return the first byte within 200 ms at p95 under normal single-user
load on development hardware. Queries that scan full tables require justification and,
where feasible, an indexed alternative.

The SQLite database MUST remain in WAL mode. Synchronous full-table scans on the
`wines` table are acceptable only during initial render; subsequent interaction MUST
use targeted queries by primary key or indexed column.

Image uploads MUST be validated at the `MAX_UPLOAD_BYTES` (10 MB) limit before
processing. Processed images MUST be resized to at most 512×512 PNG before storage.
Storing originals alongside processed images is prohibited.

All handlers and DB functions MUST be instrumented with `#[tracing::instrument]` so
latency is observable without code changes. OpenTelemetry spans MUST carry enough
attributes to diagnose slow paths without re-deploying.

## Additional Constraints

**Security**: User-supplied filenames and multipart field values MUST be validated or
discarded before use. SQL is NEVER constructed via string concatenation; `sqlx::query!`
macros are the only permitted query mechanism.

**Input limits**: Body size limits MUST be enforced at the Axum middleware layer (not
only in handler logic) to prevent unbounded memory allocation.

**Migrations**: Schema changes MUST be expressed as forward-only SQLx migration files.
Rollback migrations are not required but destructive migrations (DROP, column removal)
MUST include a comment explaining the data impact.

## Development Workflow

**Build gate**: `cargo clippy -- -D warnings` MUST pass before `cargo build`. CI
(or the developer) MUST run `cargo fmt --check` to reject unformatted code.

**Database reset**: Use `./rebuild-db` to reset the local SQLite database and reseed
sample data. Never commit `data.db` or `backup.sql`.

**Environment**: All configuration is loaded from `.env` at startup via `dotenv`.
Secrets and connection strings MUST NOT be hardcoded in source.

**Container**: Production builds use `cargo build --release` inside the Dockerfile.
The image MUST NOT include `.env`, `data.db`, or development tooling.

**Review**: PRs MUST include a description of what changed and why. Constitution Check
in the plan template MUST be completed before implementation begins on any non-trivial
feature.

## Governance

This constitution supersedes all informal conventions and ad-hoc practices. Amendments
require:

1. A written rationale describing the problem the amendment solves.
2. Version bump per semantic versioning: MAJOR for principle removal/redefinition,
   MINOR for new principle or section, PATCH for clarifications.
3. A sync-impact comment at the top of this file listing affected templates and docs.
4. Update to `LAST_AMENDED_DATE`.

All feature plans (`/speckit.plan`) MUST include a Constitution Check section that
gates work on compliance with the four core principles above.

Compliance is reviewed at PR time. Any violation MUST either be fixed or documented
in the plan's Complexity Tracking table with a written justification.

**Version**: 1.0.0 | **Ratified**: 2026-03-09 | **Last Amended**: 2026-03-09
