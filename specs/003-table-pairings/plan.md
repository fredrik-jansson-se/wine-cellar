# Implementation Plan: Show Pairings in Wine Table

**Branch**: `003-table-pairings` | **Date**: 2026-03-29 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-table-pairings/spec.md`

## Summary

Replace the grapes column in the main wine overview table with a food pairings column.
The existing `wine_food_pairings` table and `db::get_wine_food_pairings` function are
reused as-is. Changes are confined to `src/web/markup.rs`: the column header label and
the per-row cell content.

## Technical Context

**Language/Version**: Rust stable (edition 2021 conventions)
**Primary Dependencies**: Axum 0.8, SQLx 0.8 (SQLite), Maud (axum feature), HTMX 2.x, Bootstrap 5
**Storage**: SQLite WAL mode — `wine_food_pairings` table (already exists, no migrations needed)
**Testing**: `cargo test` — sqlx compile-time checked queries, `#[tokio::test]` integration tests in `src/db.rs`
**Target Platform**: Linux server (single-user, self-hosted)
**Project Type**: Web service (server-side rendered)
**Performance Goals**: <200 ms p95 handler response (per constitution)
**Constraints**: No SPA/JS frameworks; HTMX for partial updates; Bootstrap 5 for all UI components
**Scale/Scope**: Single user, small collection (tens to low hundreds of wines)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
|-----------|-------|--------|
| I. Code Quality | Changes are in `markup.rs` only; no new functions; must pass `cargo clippy` and `cargo fmt` | ✅ PASS |
| II. Testing Standards | No new SQL queries; `get_wine_food_pairings` already tested; `wine_table_row` change is a read-only display swap with no new branching logic — existing test coverage is sufficient | ✅ PASS |
| III. UX Consistency | SSR via Maud; Bootstrap 5 `<ul>`/`<li>` pattern (mirrors existing grape list); HTMX attributes unchanged | ✅ PASS |
| IV. Performance | One additional `get_wine_food_pairings` call per row (same N+1 pattern already in use for grapes and inventory events); acceptable at stated scale | ✅ PASS |

**Post-design re-check**: No schema changes, no new routes, no JS — all gates remain green.

## Project Structure

### Documentation (this feature)

```text
specs/003-table-pairings/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (no new routes; documents unchanged routes)
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (affected files only)

```text
src/
└── web/
    └── markup.rs        # Only file changed:
                         #   • wine_table_html: "Grapes" header → "Pairings"
                         #   • wine_table_row: render food pairings instead of grapes
                         #     (grapes still fetched for grape filter; only display changes)
```

No migrations, no new modules, no new routes, no JS changes.

## Complexity Tracking

> No constitution violations — table not needed.
