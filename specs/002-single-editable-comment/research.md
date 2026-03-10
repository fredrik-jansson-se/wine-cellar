# Research: Single Editable Comment

## 1. Inline editing pattern with HTMX in this codebase

**Decision**: Use HTMX `outerHTML` swap on a named `<div id="wine-note-{wine_id}">` to toggle
between read view and edit form, with GET for read/edit views and POST for save.

**Rationale**: The existing pairings and food-search flows use the same swap-innerHTML pattern.
`outerHTML` is preferred here because the root element carries the id and must be re-rendered by
each partial; this avoids an extra wrapper div. The wine detail page already renders a full
`#main` replacement, so the note section only needs a partial swap once edit mode is entered.

**Alternatives considered**:
- *hx-swap="innerHTML"* with a static outer wrapper — works but requires two divs; outerHTML is
  cleaner for self-contained partials that re-render their own root.
- *Separate edit page* (current pattern for add-comment) — rejected; the spec explicitly requires
  inline editing without a full page reload.
- *Alpine.js / client-side toggle* — rejected by constitution (no new JS frameworks).

---

## 2. Comment storage: column on `wines` vs separate table

**Decision**: Store `comment TEXT` and `comment_updated_at DATETIME` as nullable columns directly
on the `wines` table. Drop `wine_comments` via migration.

**Rationale**: The spec mandates at most one note per wine (FR-001). A column on `wines` means
zero joins for the table listing (comment text already on the row), PK-targeted updates, and
simpler code. SQLite supports `ADD COLUMN` without a table rebuild.

**Alternatives considered**:
- *Keep `wine_comments` table with a UNIQUE constraint* — rejected; the spec explicitly says to
  drop the table. The single-row constraint would also be error-prone to enforce correctly.
- *Separate `wine_notes` table (1:1)* — rejected; same overhead as the old table with no benefit.

---

## 3. SQLite migration strategy

**Decision**: A single forward-only migration file performs:
1. `ALTER TABLE wines ADD COLUMN comment TEXT`
2. `ALTER TABLE wines ADD COLUMN comment_updated_at DATETIME`
3. Data migration: one UPDATE per wine that has comments (subquery concatenates via `group_concat`)
4. `DROP TABLE wine_comments`

**Rationale**: SQLx migrations are forward-only; all schema and data changes in one file keeps the
migration atomic and easy to reason about. SQLite's `group_concat` with an ORDER BY subquery
handles oldest-first concatenation without requiring a CTE.

**Alternatives considered**:
- *Two separate migration files* — would split schema and data changes across files, adding
  complexity without benefit.
- *Application-level data migration at startup* — rejected; mixing migration concerns into startup
  code violates the "migrations live in `migrations/`" convention.

---

## 4. Error handling for save failures (inline edit mode)

**Decision**: The save handler (`POST /wines/{id}/comment`) returns the edit form partial with an
embedded Bootstrap alert on error, so HTMX swaps the edit form in-place keeping user text intact.

**Rationale**: Matches FR-008. The existing `AppError` HTML fragment renders a Bootstrap alert but
replaces the entire `#main` div, which would lose the user's edits. For the note section, the
handler must render a bespoke error partial (alert + form with pre-filled textarea).

**Alternatives considered**:
- *`hx-target-error` pointing to a sibling div* — works but requires the error HTML to be
  completely separate from the form, making it harder to keep edit mode open with the user's text.
- *Return AppError and let the global error target catch it* — user loses their edits; rejected.

---

## 5. "Comment" dropdown action on the wine table

**Decision**: Remove the "Comment" action from the wine-table-row dropdown menu entirely. The
note is now accessed via the wine detail page.

**Rationale**: The spec replaces the navigate-away comment flow with inline editing on the detail
page. Keeping the dropdown item would navigate to a page that no longer exists.

**Alternatives considered**:
- *Repurpose the dropdown item to open the detail page* — adds confusion; the detail page already
  shows the note. Removing is cleaner.

---

## 6. Timestamp display

**Decision**: Show `comment_updated_at` formatted as a date (no time) below the note text in the
read view, e.g. "Last updated: 2026-03-10".

**Rationale**: Consistent with how `WineInvEvent.dt` is displayed in the events table
(`evt.dt.date()`). The time component adds noise in a personal app context.

**Alternatives considered**:
- *Full datetime* — more information, more visual noise for a personal app.
- *Relative time ("3 days ago")* — requires client-side JS or server clock math; out of scope.
