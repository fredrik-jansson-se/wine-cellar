# Quickstart: Show Pairings in Wine Table

**Feature**: `003-table-pairings`

---

## What Changes

One file: `src/web/markup.rs`.

Two edits:

1. **Column header** (`wine_table_html`): change the `th` text from `"Grapes"` to
   `"Pairings"`.

2. **Row cell** (`wine_table_row`): add a `db::get_wine_food_pairings` call and render
   the result in the `<ul>` that currently shows grapes. The grapes fetch and grape
   filter logic are **not removed** — FR-005 requires the filter to keep working.

---

## Build & Verify

```bash
# 1. Check formatting and lints
cargo fmt
cargo clippy -- -D warnings

# 2. Build
cargo build

# 3. Run tests (no new tests needed; existing db tests cover the query)
cargo test

# 4. Manual smoke test: start the server and open the wine list
cargo run
# → open http://localhost:20000
# → verify: pairings column header shows "Pairings"
# → verify: wines with food pairings show them in the column
# → verify: wines without pairings show an empty cell
# → verify: grape filter still narrows rows correctly
```

---

## No Migrations Needed

The `wine_food_pairings` table was created in the `001-food-pairings` feature.
Run `cargo run` — migrations apply automatically at startup.
