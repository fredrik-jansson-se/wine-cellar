# Data Model: Show Pairings in Wine Table

**Phase 1 output for** `003-table-pairings`

---

## Entities Involved

This feature involves no schema changes. All relevant tables and Rust types already exist.

### Wine (`wines` table / `db::Wine`)

| Field | Type | Notes |
|-------|------|-------|
| `wine_id` | `i64` | Primary key |
| `name` | `String` | Display name |
| `year` | `i64` | Vintage year |
| `has_image` | `bool` | Derived from `image IS NOT NULL` |
| `comment` | `Option<String>` | Single editable note |
| `comment_updated_at` | `Option<NaiveDateTime>` | Last edit time |

### FoodPairing (`wine_food_pairings` table / `db::FoodPairing`)

| Field | Type | Notes |
|-------|------|-------|
| `id` | `i64` | Primary key, auto-increment |
| `food` | `String` | Food item text (max 100 chars, unique per wine case-insensitively) |
| `wine_id` | `i64` (FK) | References `wines.wine_id` |

### Relationship

- **Wine** has zero or more **FoodPairings** (one-to-many).
- Fetched per-row via `db::get_wine_food_pairings(db, wine_id)` — returns `Vec<FoodPairing>` ordered by insertion.

---

## Data Flow in `wine_table_row`

```
Request: GET /wine-table-body?grape_filter=...
  │
  ├─ db::get_wine_grapes(wine_id)       → Vec<String>   (for grape filter logic — unchanged)
  ├─ db::wine_inventory_events(wine_id) → Vec<WineInvEvent>  (for bottle count — unchanged)
  └─ db::get_wine_food_pairings(wine_id) → Vec<FoodPairing>  (NEW: for pairings column display)
        │
        └─ rendered as <ul><li>…</li></ul> in the pairings table cell
```

---

## Validation Rules (unchanged)

- Empty pairings list → empty `<td>` cell (no error, no placeholder text).
- Many pairings → all rendered; no truncation at this layer.
- Grape filter applies to grape data only; pairings data has no filter at this stage.
