# Tasks: Food Pairings for Wines

**Input**: Design documents from `/specs/001-food-pairings/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/http-routes.md ✓, quickstart.md ✓

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to ([US1], [US2])
- Exact file paths included in all descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the SQLite migration that all subsequent work depends on.

- [x] T001 Create `migrations/<timestamp>_food_pairings.sql` using `cargo sqlx migrate add food_pairings` and populate it with the `wine_food_pairings` DDL from data-model.md: `CREATE TABLE wine_food_pairings (id INTEGER PRIMARY KEY AUTOINCREMENT, wine_id INTEGER NOT NULL, food TEXT NOT NULL COLLATE NOCASE, FOREIGN KEY (wine_id) REFERENCES wines(wine_id), UNIQUE (wine_id, food))` plus `CREATE INDEX wine_food_pairings_wine_id` and `CREATE INDEX wine_food_pairings_food`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core Rust types and cascade-delete behaviour that MUST exist before any handler or markup can be written.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T002 Add `pub(crate) struct FoodPairing { pub id: i64, pub food: String }` and `pub(crate) struct WineWithPairings { pub wine_id: i64, pub name: String, pub year: i64, pub matched_pairings: Vec<String> }` to `src/db.rs`
- [x] T003 Update the `delete_wine` transaction in `src/db.rs` to execute `DELETE FROM wine_food_pairings WHERE wine_id = $1` before the wines delete statement, consistent with how comments and grapes are currently removed

**Checkpoint**: Migration file exists, Rust structs compile, cascade delete wired — user story implementation can now begin.

---

## Phase 3: User Story 1 — Add Food Pairings to a Wine (Priority: P1) 🎯 MVP

**Goal**: A wine detail page lets the owner add free-text food pairing labels and remove them; pairings persist in SQLite and are displayed immediately via HTMX partial swap.

**Independent Test**: Open a wine detail page → click Pairings in the action dropdown → add "grilled salmon" and "aged cheddar" → verify both appear → remove "aged cheddar" → verify only "grilled salmon" remains → delete the wine → verify no orphaned rows in `wine_food_pairings`.

### Implementation for User Story 1

- [x] T004 [P] [US1] Implement `get_wine_food_pairings(db: &SqlitePool, wine_id: i64) -> anyhow::Result<Vec<FoodPairing>>` in `src/db.rs` using `sqlx::query!("SELECT id, food FROM wine_food_pairings WHERE wine_id = $1 ORDER BY id", wine_id)`
- [x] T005 [P] [US1] Implement `add_food_pairing(db: &SqlitePool, wine_id: i64, food: &str) -> anyhow::Result<FoodPairing>` in `src/db.rs` using `sqlx::query!("INSERT INTO wine_food_pairings (wine_id, food) VALUES ($1, $2) RETURNING id, food", wine_id, food)`
- [x] T006 [P] [US1] Implement `remove_food_pairing(db: &SqlitePool, pairing_id: i64, wine_id: i64) -> anyhow::Result<()>` in `src/db.rs` using `sqlx::query!("DELETE FROM wine_food_pairings WHERE id = $1 AND wine_id = $2", pairing_id, wine_id)`
- [x] T007 [US1] Implement `add_food_pairing` Axum handler in `src/web/handlers.rs`: extract `wine_id` path param and `food` form field; validate food is non-empty/non-whitespace and ≤ 100 chars (return `AppError::bad_request` otherwise); call `db::add_food_pairing`; on SQLITE_CONSTRAINT_UNIQUE error return 400 "already exists"; on success call `db::get_wine_food_pairings` and return the `food_pairings_list` markup partial
- [x] T008 [US1] Implement `remove_food_pairing` Axum handler in `src/web/handlers.rs`: extract `wine_id` and `pairing_id` path params; call `db::remove_food_pairing`; return empty `maud::Markup` so HTMX `hx-swap="delete"` removes the element, or return the updated `food_pairings_list` partial
- [x] T009 [P] [US1] Implement `food_pairings_list(pairings: &[FoodPairing], wine_id: i64) -> Markup` partial in `src/web/markup.rs`: renders a `<ul id="food-pairings-list">` with one `<li>` per pairing showing the food label and a delete button using `hx-delete="/wines/{wine_id}/pairings/{pairing.id}"` with `hx-target` set to the `<li>` element and `hx-swap="delete"`; shows "No pairings added yet." when the list is empty
- [x] T010 [US1] Implement `edit_wine_pairings(wine_id: i64, pairings: &[FoodPairing]) -> Markup` full-page function in `src/web/markup.rs`: renders the `#main` replacement with a page header, the `food_pairings_list` partial, and an inline form `POST /wines/{wine_id}/pairings` containing a text input for the food name and a submit button; HTMX `hx-post` targets `#food-pairings-list` with `hx-swap="innerHTML"`
- [x] T011 [US1] Register `GET /wines/:wine_id/pairings` (calls `db::get_wine_food_pairings` then renders `edit_wine_pairings`), `POST /wines/:wine_id/pairings`, and `DELETE /wines/:wine_id/pairings/:pairing_id` routes in the Axum router in `src/web.rs`
- [x] T012 [US1] Add a "Pairings" item to the per-row action dropdown in the wine table markup in `src/web/markup.rs`, linking to `GET /wines/{wine_id}/pairings` via `hx-get` targeting `#main`
- [x] T013 [US1] Write integration tests in `tests/food_pairings.rs`: (a) add a pairing and verify it is returned by `get_wine_food_pairings`; (b) remove a pairing and verify it is gone; (c) add a duplicate pairing (same wine, same food, different case) and verify the DB returns a UNIQUE constraint error; (d) add a whitespace-only food name in the handler and verify 400 is returned; (e) delete a wine and verify no rows remain in `wine_food_pairings` for that wine

**Checkpoint**: User Story 1 is fully functional — add, remove, duplicate rejection, cascade delete, and "No pairings" state all work.

---

## Phase 4: User Story 2 — Search Food and Get Wine Recommendations (Priority: P2)

**Goal**: A dedicated `/pairings/search` page accepts a food keyword and returns a live-updating list of wines whose pairings match (substring, case-insensitive); special characters are escaped; empty query shows a prompt.

**Independent Test**: Navigate to `/pairings/search` → search "salmon" → verify only wines with a "salmon" pairing appear → search "xyz" → verify "no results found" message → clear the field → verify the prompt reappears → search "sal" → same wines as "salmon" appear (partial match).

### Implementation for User Story 2

- [x] T014 [P] [US2] Implement `search_wines_by_food(db: &SqlitePool, q: &str) -> anyhow::Result<Vec<WineWithPairings>>` in `src/db.rs`: escape `%` → `\%`, `_` → `\_`, `\` → `\\` in `q`; run `SELECT DISTINCT w.wine_id, w.name, w.year FROM wines w JOIN wine_food_pairings fp ON fp.wine_id = w.wine_id WHERE fp.food LIKE '%' || $1 || '%' ESCAPE '\' ORDER BY w.name, w.year`; for each result run a second query `SELECT food FROM wine_food_pairings WHERE wine_id = $1 ORDER BY id` to populate `matched_pairings`
- [x] T015 [P] [US2] Implement `pairings_search_page() -> Markup` in `src/web/markup.rs`: renders the `#main` replacement with a page title, a search text input wired with `hx-get="/pairings/search/results"` `hx-target="#search-results"` `hx-trigger="input changed delay:500ms, keyup[key=='Enter']"` `name="q"`, and an empty `<div id="search-results">` containing the prompt text "Enter a food to find matching wines"
- [x] T016 [US2] Implement `pairings_search_results(wines: &[WineWithPairings], query: &str) -> Markup` partial in `src/web/markup.rs`: if `wines` is empty render "No wines found matching '{query}'"; otherwise render a list of Bootstrap cards each showing wine name, year, and the matched food pairings; if query is blank render the prompt "Enter a food to find matching wines"
- [x] T017 [US2] Implement `pairings_search` handler for `GET /pairings/search` in `src/web/handlers.rs`: renders the full `pairings_search_page` markup with no query
- [x] T018 [US2] Implement `pairings_search_results` handler for `GET /pairings/search/results` in `src/web/handlers.rs`: extract optional query param `q`; trim whitespace; if blank return prompt fragment; otherwise call `db::search_wines_by_food` and return `pairings_search_results` partial
- [x] T019 [US2] Register `GET /pairings/search` and `GET /pairings/search/results` routes in the Axum router in `src/web.rs`
- [x] T020 [US2] Add a "Food Pairings Search" link to the wine table page header area in `src/web/markup.rs` (next to "Add Wine") using `hx-get="/pairings/search"` `hx-target="#main"`
- [x] T021 [US2] Write integration tests in `tests/food_pairings.rs`: (a) seed two wines with pairings, search "salmon", verify only the matching wine is returned; (b) search "xyz", verify empty result; (c) search "sal" (partial), verify the salmon wine is returned; (d) search with empty/whitespace query, verify handler returns the prompt fragment; (e) search with `%` in the term, verify it is treated as a literal and no SQL wildcard expansion occurs

**Checkpoint**: User Stories 1 and 2 both work independently and together.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Lint, format, regenerate SQLx metadata, and validate end-to-end.

- [x] T022 Run `cargo clippy -- -D warnings` from the repo root and fix all warnings in changed files (`src/db.rs`, `src/web.rs`, `src/web/handlers.rs`, `src/web/markup.rs`, `tests/food_pairings.rs`)
- [x] T023 [P] Run `cargo fmt` to format all changed source files
- [x] T024 Run `cargo sqlx prepare` to regenerate `.sqlx/` offline query metadata after all `sqlx::query!` macros are finalised
- [x] T025 Run the manual smoke test from `specs/001-food-pairings/quickstart.md` step 7: add pairings to a wine, remove one, search for it, delete the wine, verify orphan-free DB

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — create migration file immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (migration must exist for `cargo sqlx prepare` to work) — **BLOCKS all user stories**
- **User Story phases (Phase 3, 4)**: Both depend on Phase 2 completion; can then proceed in parallel or sequentially
- **Polish (Phase 5)**: Depends on all desired user story phases being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start immediately after Phase 2 — no dependency on US2
- **User Story 2 (P2)**: Can start immediately after Phase 2 — depends on US1 data existing (food pairings must be added before search has anything to find), but implementation is independent

### Within Each User Story

- DB functions (T004–T006, T014) before handlers (T007–T008, T017–T018)
- Markup partials (T009, T015–T016) can be written in parallel with DB work
- Full-page markup (T010) depends on the partial (T009)
- Routes (T011, T019) must be registered after handlers exist
- Navigation links (T012, T020) can be added any time after the target routes exist
- Integration tests (T013, T021) can be written in parallel but must pass after implementation is complete

### Parallel Opportunities

- T004, T005, T006 (US1 DB functions) can run in parallel — all different functions in the same file, no cross-dependency
- T007, T008 (US1 handlers) can run in parallel after DB functions exist
- T009 (list partial) can run in parallel with T004–T006
- T014, T015, T016 (US2 DB + markup) can all run in parallel
- T022, T023 (clippy + fmt) can run in parallel in the Polish phase

---

## Parallel Example: User Story 1

```bash
# These DB functions can be implemented simultaneously (no cross-dependencies):
Task T004: get_wine_food_pairings in src/db.rs
Task T005: add_food_pairing in src/db.rs
Task T006: remove_food_pairing in src/db.rs

# This partial markup can be written at the same time as DB work:
Task T009: food_pairings_list partial in src/web/markup.rs
```

## Parallel Example: User Story 2

```bash
# DB function and markup partials can run simultaneously:
Task T014: search_wines_by_food in src/db.rs
Task T015: pairings_search_page in src/web/markup.rs
Task T016: pairings_search_results partial in src/web/markup.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Create migration
2. Complete Phase 2: Add Rust structs + cascade delete
3. Complete Phase 3: User Story 1 (add/remove pairings UI)
4. **STOP and VALIDATE**: Manually smoke-test add, remove, duplicate rejection, cascade delete
5. Deploy/demo if ready — search can be added later

### Incremental Delivery

1. Setup + Foundational → migration + types ready
2. User Story 1 → pairings add/remove live → validate → deploy (MVP!)
3. User Story 2 → search page live → validate → deploy
4. Polish → clean build, formatted code, offline metadata committed

### Parallel Team Strategy

With two developers:

1. Both complete Phase 1 + 2 together (fast — 3 small tasks)
2. Developer A: User Story 1 (T004–T013)
3. Developer B: User Story 2 (T014–T021) — can stub the DB call until T014 is done
4. Both: Phase 5 Polish together

---

## Notes

- [P] tasks operate on different functions/files or have no blocking dependencies
- [US1]/[US2] labels map directly to the user stories in spec.md
- `cargo sqlx prepare` MUST be re-run after every change to a `sqlx::query!` macro (T024)
- Duplicate pairing errors come back as `sqlx::Error::Database` with SQLite error code 2067 — handle in handlers, not in DB functions
- LIKE escaping (`%` → `\%`, `_` → `\_`, `\` → `\\`) MUST happen in the handler/DB function before binding, not in SQL
- Commit after each phase checkpoint at minimum
