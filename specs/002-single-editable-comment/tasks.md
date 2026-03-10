# Tasks: Single Editable Comment

**Input**: Design documents from `/specs/002-single-editable-comment/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/http-endpoints.md ✅, quickstart.md ✅

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to ([US1], [US2])
- Exact file paths are included in all descriptions

---

## Phase 1: Setup

**Purpose**: Create the migration that establishes the new schema. All other work depends on this.

- [X] T001 Create migration file `migrations/20260310000001_single_comment.sql` — ADD COLUMN comment TEXT, ADD COLUMN comment_updated_at DATETIME on `wines`; data migration concatenating `wine_comments` rows oldest-first via `group_concat`; DROP TABLE wine_comments

---

## Phase 2: Foundational (DB Layer — Blocking Prerequisites)

**Purpose**: Update the `Wine` struct and all DB query functions to reflect the new schema. Every handler and markup template depends on the updated `Wine` type.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 Update `Wine` struct in `src/db.rs` to add `comment: Option<String>` and `comment_updated_at: Option<chrono::NaiveDateTime>` fields
- [X] T003 [P] Remove `WineComment` struct and `wine_comments()`, `last_wine_comment()`, `add_wine_comment()` functions from `src/db.rs`
- [X] T004 Add `set_wine_comment(db, wine_id: i64, text: Option<&str>, dt: NaiveDateTime)` function in `src/db.rs` — UPDATE wines SET comment=$2, comment_updated_at=$3 WHERE wine_id=$1; decorated with `#[tracing::instrument(skip(db))]`
- [X] T005 [P] Update `wines()` SELECT query in `src/db.rs` to include `comment` and `comment_updated_at` columns
- [X] T006 [P] Update `get_wine()` SELECT query in `src/db.rs` to include `comment` and `comment_updated_at` columns
- [X] T007 Update `delete_wine()` in `src/db.rs` to remove the `DELETE FROM wine_comments` line (table no longer exists)
- [X] T008 Run `cargo sqlx prepare` from repo root to regenerate `.sqlx/` offline query metadata; commit the updated `.sqlx/` directory

**Checkpoint**: Foundation ready — all DB functions compile, `Wine` struct carries comment fields, `.sqlx/` is current.

---

## Phase 3: User Story 1 — Edit Wine Comment (Priority: P1) 🎯 MVP

**Goal**: A user on the wine detail page can view the note, click an edit button to open an inline textarea, and Save or Cancel. Saving updates the note immediately via HTMX partial swap.

**Independent Test**: Navigate to any wine, click the edit button on the note field, change the text, click Save, and confirm the new text appears inside `#wine-note-{id}` without a full page reload.

### Implementation for User Story 1

- [X] T009 [US1] Replace old add-comment routes with `GET /wines/{id}/comment`, `GET /wines/{id}/comment/edit`, and `POST /wines/{id}/comment` routes in `src/web.rs` route table
- [X] T010 [US1] Replace `add_comment` handler with `get_note` handler in `src/web/handlers.rs` — `GET /wines/{id}/comment` fetches the wine by id and returns the note read-view partial (`note_read_view` from markup.rs); decorated with `#[tracing::instrument(skip(state))]`
- [X] T011 [US1] Add `get_note_edit` handler in `src/web/handlers.rs` — `GET /wines/{id}/comment/edit` fetches the wine by id and returns the note edit-form partial (`note_edit_form` from markup.rs); decorated with `#[tracing::instrument(skip(state))]`
- [X] T012 [US1] Add `save_comment` handler in `src/web/handlers.rs` — `POST /wines/{id}/comment`, reads `comment` from form body, calls `set_wine_comment`, on success returns `note_read_view` partial, on DB error returns `note_edit_form` partial with inline Bootstrap danger alert and the user's text pre-filled (edit mode stays open per FR-008); decorated with `#[tracing::instrument(skip(state))]`
- [X] T013 [US1] Add `note_read_view(wine: &Wine) -> Markup` function in `src/web/markup.rs` — renders `<div id="wine-note-{wine_id}">` with: note text and `comment_updated_at` formatted as date (e.g. "Last updated: 2026-03-10") when note exists; placeholder text "Add a note…" when `comment` is None; edit button with `hx-get="/wines/{id}/comment/edit"`, `hx-target="#wine-note-{id}"`, `hx-swap="outerHTML"`
- [X] T014 [US1] Add `note_edit_form(wine_id: i64, current_text: &str, error: Option<&str>) -> Markup` function in `src/web/markup.rs` — renders `<div id="wine-note-{wine_id}">` with: optional Bootstrap danger alert, `<textarea name="comment">` pre-filled with `current_text`, Save button (`hx-post="/wines/{wine_id}/comment"`, `hx-target="#wine-note-{wine_id}"`, `hx-swap="outerHTML"`), Cancel button (`hx-get="/wines/{wine_id}/comment"`, `hx-target="#wine-note-{wine_id}"`, `hx-swap="outerHTML"`)
- [X] T015 [US1] Update `wine_information` in `src/web/markup.rs` to embed the note section — call `note_read_view(&wine)` and include it in the detail view HTML; this hydrates the initial `<div id="wine-note-{id}">` on page load
- [X] T016 [US1] Remove the "Comment" dropdown action from `wine_table_row` in `src/web/markup.rs` (the route it pointed to no longer serves the add-comment form)

**Checkpoint**: User Story 1 fully functional — inline edit/save/cancel works on the wine detail page; no full page reload; error state keeps edit open.

---

## Phase 4: User Story 2 — Clear a Comment (Priority: P2)

**Goal**: A user edits an existing note, deletes all text, and saves. The note is removed (stored as NULL) and the placeholder "Add a note…" state is shown.

**Independent Test**: Edit an existing comment, erase all text, click Save, and confirm the empty placeholder appears in `#wine-note-{id}` without a page reload.

### Implementation for User Story 2

- [X] T017 [US2] Add whitespace-trimming and empty-string-to-None coercion in `save_comment` handler in `src/web/handlers.rs` — trim the submitted `comment` value; if result is empty, pass `None` to `set_wine_comment` (sets `comment = NULL`, `comment_updated_at = NULL`); fulfills FR-005

**Checkpoint**: User Stories 1 and 2 both work — editing saves updated text, clearing saves NULL, placeholder displays on return to read view.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Quality gate and final validation across all changed files.

- [X] T018 [P] Run `cargo clippy -- -D warnings` and fix all warnings in `src/db.rs`, `src/web.rs`, `src/web/handlers.rs`, `src/web/markup.rs`
- [X] T019 [P] Run `cargo fmt` to ensure consistent formatting across all modified files
- [X] T020 Validate the full workflow per `specs/002-single-editable-comment/quickstart.md` — run `./rebuild-db`, `cargo run`, and manually verify: view note, edit+save, cancel discards, clear+save shows placeholder, DB error shows inline alert with text preserved

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 (migration must exist for `cargo sqlx prepare`) — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 completion
- **Phase 4 (US2)**: Depends on Phase 3 completion (T017 modifies the `save_comment` handler created in T012)
- **Phase 5 (Polish)**: Depends on Phase 4 completion

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on US2
- **US2 (P2)**: Depends on US1 (T017 extends the `save_comment` handler from T012)

### Within Each User Story

- T009 (routes) before T010–T012 (handlers) — routes must reference handler functions
- T013–T014 (markup partials) before T010–T012 (handlers call them) — or implement in same pass
- T015 (embed in detail view) after T013 (partial must exist)
- T016 (remove dropdown item) is independent within US1 — can run in parallel with T009–T015

### Parallel Opportunities Within Phase 2

- T003 (remove old code), T005 (update `wines()` query), T006 (update `get_wine()` query) can all run in parallel once T002 (struct update) is done
- T004 (new `set_wine_comment` function) can run in parallel with T003/T005/T006
- T007 (update `delete_wine`) can run in parallel with T003/T004/T005/T006

---

## Parallel Example: Phase 2 (after T002)

```
# After updating Wine struct (T002), run in parallel:
Task T003: Remove WineComment struct and old functions from src/db.rs
Task T004: Add set_wine_comment() to src/db.rs
Task T005: Update wines() SELECT in src/db.rs
Task T006: Update get_wine() SELECT in src/db.rs
Task T007: Update delete_wine() in src/db.rs

# Then sequentially:
Task T008: cargo sqlx prepare (must follow all query changes)
```

## Parallel Example: Phase 3 (US1 markup + routes can overlap)

```
# Can run in parallel (different locations in markup.rs):
Task T013: Add note_read_view partial in src/web/markup.rs
Task T014: Add note_edit_form partial in src/web/markup.rs

# After T013 and T014:
Task T010: get_note handler (calls note_read_view)
Task T011: get_note_edit handler (calls note_edit_form)
Task T012: save_comment handler (calls both partials)
Task T015: embed note_read_view in wine_information
Task T016: remove Comment dropdown (independent — can run any time in US1)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Create migration
2. Complete Phase 2: DB layer (Wine struct, queries, set_wine_comment) — **CRITICAL BLOCKER**
3. Complete Phase 3: US1 — inline edit/save/cancel on detail page
4. **STOP and VALIDATE**: Test US1 independently per quickstart.md
5. Deploy/demo if ready

### Incremental Delivery

1. Phase 1 + Phase 2 → DB layer ready
2. Phase 3 (US1) → Inline editing works → MVP
3. Phase 4 (US2) → Clearing notes works → Full feature
4. Phase 5 → Polish and validation → Ready to merge

---

## Notes

- [P] tasks = different files or sections, no unresolved dependencies
- [Story] label maps each task to its user story for traceability
- `cargo sqlx prepare` (T008) **must** be re-run any time a `sqlx::query!` macro is added or changed; commit `.sqlx/` alongside code
- `cargo clippy -- -D warnings` must pass before committing (per CLAUDE.md)
- The `save_comment` error path renders a bespoke partial (not AppError) to keep edit mode open with user's text intact — see research.md §4
- US2 (T017) is a one-line addition to `save_comment` but is tracked separately to preserve user story traceability
