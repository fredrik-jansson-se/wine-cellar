# Data Model: Food Pairings Feature

**Feature**: 001-food-pairings | **Date**: 2026-03-09

---

## New Table: `wine_food_pairings`

```sql
CREATE TABLE wine_food_pairings (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  wine_id INTEGER NOT NULL,
  food    TEXT    NOT NULL COLLATE NOCASE,
  FOREIGN KEY (wine_id) REFERENCES wines(wine_id),
  UNIQUE (wine_id, food)           -- case-insensitive via COLLATE NOCASE
);

CREATE INDEX wine_food_pairings_wine_id ON wine_food_pairings(wine_id);
CREATE INDEX wine_food_pairings_food    ON wine_food_pairings(food COLLATE NOCASE);
```

### Field Rules

| Field   | Type    | Constraints                          |
|---------|---------|--------------------------------------|
| id      | INTEGER | PK, autoincrement                    |
| wine_id | INTEGER | NOT NULL, FK → wines(wine_id)        |
| food    | TEXT    | NOT NULL, max 100 chars (app layer), COLLATE NOCASE |

### Validation Rules

- `food` MUST NOT be empty or whitespace-only (validated in handler before DB call).
- `food` MUST be at most 100 characters (validated in handler; returns 400 if exceeded).
- `(wine_id, food)` MUST be unique case-insensitively (enforced by DB UNIQUE constraint; handler returns 409/400 on conflict).

### State Transitions

- **Add pairing**: `INSERT INTO wine_food_pairings (wine_id, food) VALUES ($1, $2)` — fails with constraint error on duplicate.
- **Remove pairing**: `DELETE FROM wine_food_pairings WHERE id = $1 AND wine_id = $2`.
- **Wine deleted**: `DELETE FROM wine_food_pairings WHERE wine_id = $1` inside `delete_wine` transaction.

---

## Modified: `delete_wine` transaction (`src/db.rs`)

The existing transaction gains one additional statement before the `wines` delete:

```sql
DELETE FROM wine_food_pairings WHERE wine_id = $1;
```

---

## New Rust Types

### `src/db.rs`

```rust
pub(crate) struct FoodPairing {
    pub id: i64,
    pub food: String,
}
```

### `src/db.rs` — new functions

| Function | SQL pattern |
|----------|-------------|
| `get_wine_food_pairings(db, wine_id) -> Vec<FoodPairing>` | `SELECT id, food FROM wine_food_pairings WHERE wine_id=$1 ORDER BY id` |
| `add_food_pairing(db, wine_id, food: &str) -> Result<FoodPairing>` | `INSERT INTO wine_food_pairings (wine_id, food) VALUES ($1, $2) RETURNING id, food` |
| `remove_food_pairing(db, pairing_id, wine_id) -> Result<()>` | `DELETE FROM wine_food_pairings WHERE id=$1 AND wine_id=$2` |
| `search_wines_by_food(db, pattern: &str) -> Vec<WineWithPairings>` | JOIN `wines` on `wine_food_pairings` WHERE `food LIKE $1` |

### `src/db.rs` — search result type

```rust
pub(crate) struct WineWithPairings {
    pub wine_id: i64,
    pub name: String,
    pub year: i64,
    pub matched_pairings: Vec<String>,   // all pairings for the wine (for context display)
}
```

> Note: `matched_pairings` is fetched in a second query per wine (or via aggregation) to keep compatible with `sqlx::query!` compile-time checking. Since result sets are small (≤ 500 wines), two queries is acceptable.

---

## Entity Relationships

```
wines (existing)
  1 ──< wine_food_pairings
           id, wine_id (FK), food TEXT COLLATE NOCASE
```
