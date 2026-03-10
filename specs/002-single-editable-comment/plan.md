# Implementation Plan: Single Editable Comment

**Branch**: `002-single-editable-comment` | **Date**: 2026-03-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-single-editable-comment/spec.md`

## Summary

Replace the multi-comment `wine_comments` table with a single `comment` column on `wines`, and
replace the navigate-away add-comment flow with an inline HTMX edit/save/cancel interaction on
the wine detail view. A migration concatenates existing comments and drops the old table. UI copy
uses "Note"; code/DB identifiers keep "comment".

## Technical Context

**Language/Version**: Rust (edition 2021 conventions, Rust stable; "Rust 2024 idioms" per project)
**Primary Dependencies**: Axum 0.8, SQLx 0.8 (SQLite), Maud (axum feature), HTMX 2.x, Bootstrap 5, Chrono
**Storage**: SQLite WAL mode — two new nullable columns on `wines`, `wine_comments` table dropped
**Testing**: `cargo test` (tokio runtime); integration tests use in-memory SQLite with migrations
**Target Platform**: Linux server, single-user personal app
**Project Type**: Server-side-rendered web service
**Performance Goals**: <200 ms p95 (existing requirement); comment read/write is PK-targeted
**Constraints**: No SPA, no JSON API; all updates via HTMX partial HTML swaps; Bootstrap 5 only
**Scale/Scope**: Single user; wine table ~tens of rows; no pagination concerns

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | All new Rust follows snake_case/CamelCase, `cargo clippy` gate respected; no dead code left behind — old `WineComment` struct and functions fully removed |
| II. Testing Standards | ✅ PASS | New `set_wine_comment` and `get_wine_comment` DB functions covered by integration tests (success + clear paths). `cargo sqlx prepare` required after migration. |
| III. UX Consistency | ✅ PASS | Inline edit via Maud + HTMX `hx-get`/`hx-post` swapping a named target div; Bootstrap form/button classes; no new JS; error state via `AppError` HTML fragment |
| IV. Performance | ✅ PASS | `comment` stored directly on `wines` row — zero extra join for table listing; detail view fetches wine by PK; `#[tracing::instrument]` on all new DB + handler functions |

No violations; Complexity Tracking table omitted.

## Project Structure

### Documentation (this feature)

```text
specs/002-single-editable-comment/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── http-endpoints.md
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (repository root)

```text
migrations/
└── 20260310000001_single_comment.sql   # new: ADD COLUMNS + data migration + DROP TABLE

src/
├── db.rs                # updated: Wine struct, queries, new set/get comment fns, remove old fns
├── web.rs               # updated: replace old comment routes; add /comment/edit route
└── web/
    ├── handlers.rs      # updated: remove add_comment; add get_note, get_note_edit, save_comment
    └── markup.rs        # updated: wine_table_row, wine_information, note partials; remove add_comment

tests/                   # existing integration test directory (empty today; new tests go in src/db.rs)
```

**Structure Decision**: Single-project layout unchanged. All new code slots into existing modules;
no new modules or files are created beyond the migration.
