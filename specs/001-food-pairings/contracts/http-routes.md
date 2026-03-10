# HTTP Route Contracts: Food Pairings Feature

**Feature**: 001-food-pairings | **Date**: 2026-03-09

All routes follow the existing Wine Cellar conventions:
- Responses are HTML fragments (Maud markup) rendered server-side.
- Errors are returned as Bootstrap alert HTML via `AppError`.
- HTMX attributes on elements drive partial swaps; full-page navigation is used for the initial `/pairings/search` render only.

---

## Wine Pairings Management

### `GET /wines/{wine_id}/pairings`

**Description**: Render the food-pairings edit page for a wine.

**Path params**: `wine_id: i64`

**Response**: Full `#main` replacement — page header, current pairings list with remove buttons, add-pairing inline form.

**Errors**:
- 404 if wine not found.

---

### `POST /wines/{wine_id}/pairings`

**Description**: Add a food pairing to a wine.

**Path params**: `wine_id: i64`

**Form body** (`application/x-www-form-urlencoded`):
```
food = <string, 1–100 chars, required>
```

**Response** (HTMX partial): Updated `#food-pairings-list` fragment with the new pairing appended.

**Errors**:
- 400 if `food` is empty, whitespace-only, or exceeds 100 characters.
- 400 (or 409 message) if the pairing already exists on this wine (case-insensitive).
- 404 if wine not found.

---

### `DELETE /wines/{wine_id}/pairings/{pairing_id}`

**Description**: Remove a specific food pairing by its ID.

**Path params**: `wine_id: i64`, `pairing_id: i64`

**Response** (HTMX partial): Empty fragment (the HTMX `hx-swap="delete"` removes the target element from the DOM). Alternatively, the updated list is returned.

**Errors**:
- 404 if pairing not found or does not belong to the wine.

---

## Food Search

### `GET /pairings/search`

**Description**: Render the food-pairing search page (full `#main` replacement).

**Query params**: none (initial empty state)

**Response**: Full page with search input and empty results placeholder. Search input is wired to `hx-get="/pairings/search/results"`.

---

### `GET /pairings/search/results`

**Description**: Return wine recommendation results for a food search (HTMX partial).

**Query params**:
```
q = <search term, optional>
```

**Behaviour**:
- `q` absent or whitespace-only → prompt fragment ("Enter a food to find matching wines").
- `q` present and non-empty → list of matching wines (name, year, matched food pairings).

**Response** (HTMX partial): `#search-results` div content.

**SQL pattern**:
```sql
-- escape %, _, \ in q before binding
SELECT DISTINCT w.wine_id, w.name, w.year
FROM wines w
JOIN wine_food_pairings fp ON fp.wine_id = w.wine_id
WHERE fp.food LIKE '%' || $1 || '%' ESCAPE '\'
ORDER BY w.name, w.year
```

**Errors**: None expected; empty result set is rendered as "no results found" message.

---

## Navigation Integration

The wine table page (`GET /wines` → `#main`) gains a link:

```html
<a hx-get="/pairings/search" hx-target="#main" hx-target-error="#error">
  Food Pairings Search
</a>
```

The wine table row action dropdown gains a "Pairings" item linking to `GET /wines/{wine_id}/pairings`.
