# HTTP Endpoint Contracts: Single Editable Comment

All endpoints return HTML fragments (Maud/Markup). HTMX drives partial swaps in the browser.
Error responses are Bootstrap alert HTML (AppError pattern from `src/web/error.rs`).

---

## Changed endpoints

### GET /wines/{wine_id}/comment  *(repurposed)*

**Was**: Served the old add-comment form (navigate-away flow).
**Now**: Returns the **note read-view partial** for HTMX swap.

**Path params**: `wine_id: i64`
**Response**: HTML partial — `<div id="wine-note-{wine_id}">` containing:
- If note exists: note text, last-updated date, edit button
- If no note: placeholder "Add a note…", edit button
**Target**: `#wine-note-{wine_id}` (outerHTML swap)
**Used by**: Cancel button in edit form; also used by the initial wine detail render to hydrate

---

### POST /wines/{wine_id}/comment  *(repurposed)*

**Was**: Inserted a new row in `wine_comments`.
**Now**: Saves (or clears) the single note on `wines`.

**Path params**: `wine_id: i64`
**Form body**: `comment=<text>` (empty string = clear note)
**Success response**: Note read-view partial (same as GET above)
**Error response**: Edit form partial with inline Bootstrap alert and pre-filled textarea
  (edit mode stays open so user can retry without losing text — FR-008)
**Target**: `#wine-note-{wine_id}` (outerHTML swap)

---

## New endpoints

### GET /wines/{wine_id}/comment/edit

Returns the **note edit-form partial**.

**Path params**: `wine_id: i64`
**Response**: HTML partial — `<div id="wine-note-{wine_id}">` containing:
- `<textarea name="comment">` pre-filled with current note text (or empty if no note)
- Save button (`hx-post="/wines/{wine_id}/comment"`)
- Cancel button (`hx-get="/wines/{wine_id}/comment"`)
**Target**: `#wine-note-{wine_id}` (outerHTML swap)

---

## Removed endpoints

### GET /wines/{wine_id}/comment (old add-comment form)
Replaced by the repurposed GET above.

### POST /wines/{wine_id}/comment (old add handler)
Replaced by the repurposed POST above.

---

## Unchanged endpoints (comment-adjacent)

| Method | Path | Notes |
|--------|------|-------|
| GET | /wines/{wine_id} | wine_information handler — updated internally to embed note section |
| DELETE | /wines/{wine_id} | delete_wine — updated internally to not reference wine_comments |
