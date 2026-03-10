# Implementation Plan: Food Pairings for Wines

**Branch**: `001-food-pairings` | **Date**: 2026-03-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-food-pairings/spec.md`

## Summary

Add per-wine food pairing labels that users can add/remove from the wine detail page, and a dedicated `/pairings/search` page to discover wines by food keyword. Built on SQLite with a new `wine_food_pairings` table; UI follows existing Maud + HTMX + Bootstrap 5 patterns.

## Technical Context

**Language/Version**: Rust 2024 edition (edition = "2021" in Cargo.toml, Rust stable)
**Primary Dependencies**: Axum, SQLx (SQLite), Maud, HTMX 2.x, Bootstrap 5
**Storage**: SQLite WAL — new table `wine_food_pairings`
**Testing**: `cargo test` (integration tests using a disposable SQLite file + migrations)
**Target Platform**: Linux server (single-binary, Docker-ready)
**Project Type**: Server-side rendered web service
**Performance Goals**: p95 < 200 ms for all handlers; search across ≤ 500 wines well within limit
**Constraints**: No client-side routing; no SPA; JS only for progressive enhancement; all SQL via `sqlx::query!` macros
**Scale/Scope**: Single-user cellar app; no multi-tenancy; up to ~500 wines

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Code Quality** | PASS | New functions will carry `#[tracing::instrument]`; `snake_case`/`CamelCase` observed; doc comments on non-trivial logic |
| **II. Testing Standards** | PASS | All SQL via `sqlx::query!`; `cargo sqlx prepare` required after migration; integration tests planned for add/remove/search paths |
| **III. UX Consistency** | PASS | Maud templates; HTMX partial updates; Bootstrap 5 forms and alerts; no new JS beyond progressive enhancement |
| **IV. Performance** | PASS | `wine_food_pairings.wine_id` indexed (FK); `food` column has index for LIKE search; no full-table wine scans on hot paths |

No violations. Complexity Tracking table not required.

## Project Structure

### Documentation (this feature)

```text
specs/001-food-pairings/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── http-routes.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── db.rs                   # +get_wine_food_pairings, add_food_pairing,
│                           #  remove_food_pairing, search_wines_by_food
│                           #  (delete_wine updated to cascade)
├── web.rs                  # +6 new routes (GET/POST pairings, DELETE pairing,
│                           #  GET search page, GET search results)
└── web/
    ├── handlers.rs         # +add_food_pairing, remove_food_pairing handlers
    └── markup.rs           # +edit_pairings page, pairings_search page,
                            #  pairings_search_results partial

migrations/
└── <timestamp>_food_pairings.sql   # new migration

tests/
└── food_pairings.rs        # integration tests (add, remove, duplicate, search)
```

**Structure Decision**: Single-project layout (existing pattern). No new modules; food-pairing logic slots into the existing `db`, `handlers`, `markup` split.

## Complexity Tracking

> No violations to justify.
