# Quickstart: Single Editable Comment

## Prerequisites

- Rust stable toolchain
- `sqlx-cli` installed: `cargo install sqlx-cli --no-default-features --features sqlite`
- `.env` with `DATABASE_URL=sqlite:data.db`

## Development workflow

### 1. Reset the local DB (picks up new migration automatically)

```bash
./rebuild-db
```

This drops and recreates `data.db` and runs all migrations including
`20260310000001_single_comment.sql`.

### 2. Regenerate SQLx offline metadata after changing SQL queries

Any time you add or change a `sqlx::query!` call in `src/db.rs`:

```bash
cargo sqlx prepare
```

Commit the updated `.sqlx/` directory alongside the code change.

### 3. Run locally

```bash
cargo run
```

Open http://localhost:20000, navigate to any wine, and you should see the **Note** section with
an edit button. Clicking edit opens a textarea; Save and Cancel work inline.

### 4. Lint + format

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

Both must pass before committing.

### 5. Run tests

```bash
cargo test
```

New tests in `src/db.rs` cover `set_wine_comment` and `get_wine` (comment fields) using an
in-memory SQLite database with migrations applied.

## Key interaction flow (HTMX)

```
Wine detail page (/wines/{id})
  └── <div id="wine-note-{id}"> [read view]
        ├── note text (or "Add a note…" placeholder)
        ├── last-updated timestamp (if note exists)
        └── ✏ Edit button
              hx-get="/wines/{id}/comment/edit"
              hx-target="#wine-note-{id}"
              hx-swap="outerHTML"

  ── HTMX swaps to edit form ──

  └── <div id="wine-note-{id}"> [edit view]
        ├── <textarea name="comment">…</textarea>
        ├── Save  hx-post="/wines/{id}/comment"  hx-target="#wine-note-{id}"  hx-swap="outerHTML"
        └── Cancel hx-get="/wines/{id}/comment"  hx-target="#wine-note-{id}"  hx-swap="outerHTML"

  ── On success: HTMX swaps back to read view ──
  ── On error:   HTMX swaps to edit form + inline alert (edit mode stays open) ──
```
