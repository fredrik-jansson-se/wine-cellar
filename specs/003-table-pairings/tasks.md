# Tasks: Show Pairings in Wine Table

**Input**: Design documents from `/specs/003-table-pairings/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/routes.md ✅, quickstart.md ✅

**Organization**: Single user story (P1). All changes are confined to `src/web/markup.rs`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the build baseline before making changes.

- [x] T001 Verify clean build baseline: run `cargo clippy -- -D warnings` and `cargo build` from repo root

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: No new infrastructure is needed — the `wine_food_pairings` table, `db::get_wine_food_pairings` function, and all routes already exist.

> ✅ No tasks required. Proceed directly to User Story 1.

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 — View Food Pairings in Main Table (Priority: P1) 🎯 MVP

**Goal**: Replace the grapes column in the wine overview table with a food pairings column, reusing the existing `db::get_wine_food_pairings` query.

**Independent Test**: Load the wine list page; verify the column header reads "Pairings", wines with food pairings display them as a `<ul>` list, wines without pairings show an empty cell, and the grape filter still narrows rows correctly.

### Implementation for User Story 1

- [x] T002 [US1] Rename the column header from `"Grapes"` to `"Pairings"` in the `wine_table_html` function in `src/web/markup.rs`
- [x] T003 [US1] Add a `db::get_wine_food_pairings(db, wine_id)` call inside `wine_table_row` in `src/web/markup.rs` (after the existing `get_wine_grapes` and `wine_inventory_events` calls)
- [x] T004 [US1] Replace the grapes `<ul>/<li>` render block in the table cell with a pairings `<ul>/<li>` render block using the `Vec<FoodPairing>` returned by T003; keep the existing grapes fetch and grape-filter logic untouched in `src/web/markup.rs`

**Checkpoint**: User Story 1 is fully functional and independently testable at this point.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Ensure code quality and validate the end-to-end behaviour.

- [x] T005 [P] Run `cargo fmt` to ensure formatting is correct
- [x] T006 [P] Run `cargo clippy -- -D warnings` and resolve any warnings in `src/web/markup.rs`
- [x] T007 Run `cargo build` and `cargo test` to confirm no regressions
- [ ] T008 Manual smoke test per quickstart.md: start `cargo run`, open `http://localhost:20000`, verify pairings column header, pairings display, empty-cell behaviour, and grape filter

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Nothing to do.
- **User Story 1 (Phase 3)**: Depends on Phase 1 baseline check.
  - T002 and T003 can be done in either order (different parts of the same function flow; T004 depends on T003).
- **Polish (Phase 4)**: Depends on Phase 3 completion. T005 and T006 can run in parallel.

### User Story Dependencies

- **User Story 1 (P1)**: Only story — entire feature scope.

### Within User Story 1

- T002 (header rename) is independent of T003/T004 and can be done first.
- T003 (add query call) must complete before T004 (render pairings cell).
- T004 completes the story.

### Parallel Opportunities

- T005 (`cargo fmt`) and T006 (`cargo clippy`) can run in parallel after T004.

---

## Parallel Example: User Story 1

```bash
# T002 and T003 touch different lines — can be done in the same edit pass:
Edit wine_table_html: "Grapes" → "Pairings"
Edit wine_table_row: add get_wine_food_pairings call + render pairings cell

# Polish tasks in parallel:
cargo fmt
cargo clippy -- -D warnings
```

---

## Implementation Strategy

### MVP (User Story 1 Only — entire feature)

1. Complete Phase 1: verify baseline builds.
2. Complete Phase 3: two-edit change in `src/web/markup.rs`.
3. Complete Phase 4: fmt, clippy, test, smoke test.
4. **Done** — feature complete.

---

## Notes

- All changes are in `src/web/markup.rs` only. No migrations, no new routes, no JS.
- The grape fetch (`get_wine_grapes`) and grape-filter suppress logic MUST NOT be removed (FR-005).
- The pairings `<ul>/<li>` pattern mirrors the existing grape list format.
- Column header decision: `"Pairings"` (not `"Food Pairings"`) — matches other short headers per research.md Q3.
