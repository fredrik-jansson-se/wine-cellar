# Data Model: Single Editable Comment

## Entities

### Wine (updated)

Stored in `wines` table.

| Column | Type | Nullable | Notes |
|--------|------|----------|-------|
| wine_id | INTEGER PK AUTOINCREMENT | NO | unchanged |
| name | TEXT NOT NULL | NO | unchanged |
| year | INT NOT NULL | NO | unchanged |
| image | BLOB | YES | unchanged |
| **comment** | **TEXT** | **YES** | **new — the single note text; NULL = no note** |
| **comment_updated_at** | **DATETIME** | **YES** | **new — set on every save, NULL when no note** |

**Validation rules**:
- `comment` may be any non-empty text or NULL (empty string from the form is coerced to NULL in the handler — saves an empty value → clears the note, fulfilling FR-005).
- No length constraint enforced at DB level; UI textarea has no `maxlength` per spec assumption.
- `comment_updated_at` is always set to `NOW()` (server time) when `comment` is written; it is
  set to NULL when `comment` is cleared to NULL.

**State transitions**:

```
No Note (comment IS NULL)
    │
    │ user enters text + saves
    ▼
Has Note (comment IS NOT NULL, comment_updated_at set)
    │
    ├──► user edits text + saves  →  Has Note (updated_at refreshed)
    │
    └──► user clears text + saves →  No Note (comment = NULL, updated_at = NULL)
```

---

### wine_comments table (removed)

Dropped by migration `20260310000001_single_comment.sql`. All existing rows are concatenated
(oldest `dt` first, separated by `\n\n`) into `wines.comment` before the table is dropped.

---

## Rust Structs (src/db.rs)

### Wine (updated)

```rust
pub(crate) struct Wine {
    pub wine_id: i64,
    pub name: String,
    pub year: i64,
    pub has_image: bool,
    pub comment: Option<String>,            // new
    pub comment_updated_at: Option<chrono::NaiveDateTime>,  // new
}
```

### WineComment (removed)

The existing `WineComment` struct is deleted. All functions that use it are removed:
- `wine_comments()`
- `last_wine_comment()`
- `add_wine_comment()`

### New / changed DB functions

| Function | Signature | Notes |
|----------|-----------|-------|
| `wines()` | `-> Vec<Wine>` | Updated SELECT to include `comment`, `comment_updated_at` |
| `get_wine()` | `-> Wine` | Updated SELECT to include `comment`, `comment_updated_at` |
| `set_wine_comment()` | `(db, wine_id, text: Option<&str>, dt: NaiveDateTime) -> ()` | UPDATE wines SET comment=$2, comment_updated_at=$3; pass None to clear |
| `delete_wine()` | unchanged signature | Remove `DELETE FROM wine_comments` line |

---

## Migration

File: `migrations/20260310000001_single_comment.sql`

```sql
-- Add new columns
ALTER TABLE wines ADD COLUMN comment TEXT;
ALTER TABLE wines ADD COLUMN comment_updated_at DATETIME;

-- Migrate existing comments: concatenate oldest-first per wine
-- DATA IMPACT: wine_comments rows are irreversibly consolidated; the source table is dropped.
UPDATE wines
SET
    comment = (
        SELECT group_concat(comment, char(10) || char(10))
        FROM (
            SELECT comment
            FROM wine_comments
            WHERE wine_id = wines.wine_id
            ORDER BY dt ASC
        )
    ),
    comment_updated_at = (
        SELECT MAX(dt)
        FROM wine_comments
        WHERE wine_id = wines.wine_id
    )
WHERE wine_id IN (SELECT DISTINCT wine_id FROM wine_comments);

-- Drop old table
DROP TABLE wine_comments;
```
