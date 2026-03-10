# Quickstart: Food Pairings Feature

**Feature**: 001-food-pairings | **Date**: 2026-03-09

## Prerequisites

- Rust stable toolchain
- `cargo-sqlx` CLI: `cargo install sqlx-cli --no-default-features --features sqlite`
- SQLite dev database seeded: `./rebuild-db`

## Development Flow

### 1. Create the migration

```bash
cargo sqlx migrate add food_pairings
# Edit the generated file in migrations/ — see data-model.md for the DDL
```

### 2. Apply migration locally

```bash
cargo run   # migrations run automatically at startup
# OR
sqlx migrate run --database-url sqlite:data.db
```

### 3. Implement DB layer

Add to `src/db.rs`:
- `FoodPairing` struct
- `WineWithPairings` struct
- `get_wine_food_pairings`, `add_food_pairing`, `remove_food_pairing`, `search_wines_by_food`
- Update `delete_wine` transaction to delete pairings first

Then regenerate SQLx metadata:
```bash
cargo sqlx prepare
```

### 4. Implement handlers and markup

- `src/web/handlers.rs`: `add_food_pairing`, `remove_food_pairing`
- `src/web/markup.rs`: `edit_wine_pairings`, `pairings_search_page`, `pairings_search_results`
- `src/web.rs`: register the 6 new routes

### 5. Lint and format

```bash
cargo clippy -- -D warnings
cargo fmt
```

### 6. Run integration tests

```bash
cargo test
```

### 7. Manual smoke test

```bash
cargo run
# Open http://localhost:20000
# 1. Open a wine detail page → Action dropdown → Pairings
# 2. Add "grilled salmon", "aged cheddar"
# 3. Remove "aged cheddar"
# 4. Navigate to Food Pairings Search
# 5. Search "salmon" → wine appears
# 6. Search "xyz" → "no results found"
# 7. Delete the wine → pairings gone (check DB)
```

## Key Files Changed

| File | Change |
|------|--------|
| `migrations/<ts>_food_pairings.sql` | New table + indexes |
| `src/db.rs` | New structs + 4 functions + delete_wine update |
| `src/web/handlers.rs` | 2 new mutation handlers |
| `src/web/markup.rs` | 3 new markup functions |
| `src/web.rs` | 6 new routes |
| `tests/food_pairings.rs` | Integration tests |
