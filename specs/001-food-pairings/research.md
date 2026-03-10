# Research: Food Pairings Feature

**Feature**: 001-food-pairings | **Date**: 2026-03-09

---

## Decision 1: Case-insensitive uniqueness in SQLite

**Decision**: Use a `UNIQUE` constraint with `COLLATE NOCASE` on the `food` column, scoped to `(wine_id, food)`.

**Rationale**: SQLite `COLLATE NOCASE` applies to ASCII letters only, which is sufficient for food labels (English food names). This enforces the unique-per-wine requirement at the database level rather than relying on application-layer checks, preventing races. SQLx compile-time macros will catch constraint violations as `sqlx::Error::Database` with error code 2067 (SQLITE_CONSTRAINT_UNIQUE).

**Alternatives considered**:
- Storing a normalised lowercase shadow column: adds write complexity with no benefit given NOCASE covers the requirement.
- Application-layer `SELECT ... lower(food) = lower($1)` before insert: subject to TOCTOU race and more code.

---

## Decision 2: Substring search via SQLite LIKE with escaped special characters

**Decision**: Search query: `SELECT ... WHERE food LIKE '%' || $escaped || '%' ESCAPE '\'`. Escape `%` → `\%`, `_` → `\_`, `\` → `\\` in the handler before binding.

**Rationale**: SQLite `LIKE` with `ESCAPE` is the idiomatic substring-match approach for the database engine in use. It avoids FTS5 setup overhead for this scale (≤ 500 wines). Escaping special chars at the application layer (before the bind parameter) satisfies FR-005 and keeps SQL injection risk zero because the pattern is still a bind parameter.

**Alternatives considered**:
- SQLite FTS5 full-text search: more powerful but significant added complexity (virtual table, triggers, tokeniser config) for a feature that only needs simple substring match.
- `INSTR()` function: does not support case-insensitive matching without `lower()` on both sides; LIKE with NOCASE column collation is simpler.

---

## Decision 3: Cascade delete via explicit transaction (not FK ON DELETE CASCADE)

**Decision**: Add `DELETE FROM wine_food_pairings WHERE wine_id = $1` to the existing `delete_wine` transaction in `db.rs`, consistent with how comments and grapes are currently deleted.

**Rationale**: The codebase already disables `FOREIGN KEY` pragmas at the application level (no `PRAGMA foreign_keys = ON` in `connect()`). Enabling FK cascades would require a database-wide pragma change with unknown side effects on the existing schema. The explicit transaction approach is already established by `delete_wine` and is safe and auditable.

**Alternatives considered**:
- `ON DELETE CASCADE` FK constraint: clean, but requires enabling foreign key enforcement globally; not done today and out of scope.

---

## Decision 4: HTMX partial-update pattern for add/remove pairings

**Decision**: Use an inline form (POST) to add a pairing and per-pairing delete buttons (DELETE via `hx-delete`). Both swap an `#food-pairings-list` div. No modal needed.

**Rationale**: Matches the existing comment-add and grape-select patterns. The pairing list is simple enough (text label + remove button) that a full-page form would be over-engineered. HTMX `hx-swap="innerHTML"` on the list container keeps the approach consistent.

**Alternatives considered**:
- Modal dialog for adding pairings: heavier UX than needed; comments use inline forms.
- Full-page re-render on each add/remove: works but slower than HTMX partial.

---

## Decision 5: Search page as a dedicated route

**Decision**: `/pairings/search` renders a full search page. The search input triggers `hx-get="/pairings/search/results?q=..."` with `hx-trigger="input changed delay:500ms, keyup[key=='Enter']"`, replacing a `#search-results` div. Empty query shows a prompt; non-empty query shows wine cards with matched pairings highlighted.

**Rationale**: Spec requires a dedicated route (FR-004). Debounced HTMX matches the grape-filter pattern already in `wine_table_html`. The results partial returns only the results div, keeping bandwidth low.

**Alternatives considered**:
- Query param on `/wines`: would muddle the wine-table semantics and require the search field to live in the wine table header.

---

## Decision 6: Navigation link placement

**Decision**: Add a "Food Pairings Search" link to the wine table page header area (next to "Add Wine"), pointing to `/pairings/search` via a full navigation (`hx-get="/pairings/search" hx-target="#main"`).

**Rationale**: The wine table is the app's primary landing page; placing the link there gives discoverable access without a persistent nav bar (which doesn't exist yet).
