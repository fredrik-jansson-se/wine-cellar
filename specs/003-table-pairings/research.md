# Research: Show Pairings in Wine Table

**Phase 0 output for** `003-table-pairings`

---

## Q1: Are there performance concerns from adding `get_wine_food_pairings` per table row?

**Decision**: Accept the additional per-row query; no optimization needed.

**Rationale**: `wine_table_row` already issues two queries per row (`get_wine_grapes` and
`wine_inventory_events`). Adding a third (`get_wine_food_pairings`) follows the same N+1
pattern. At the stated scale (single user, tens to low hundreds of wines) this is well
within the 200 ms p95 budget. The `wine_food_pairings` table is indexed on `wine_id`
(via the foreign key), so the lookup is a direct key scan.

**Alternatives considered**: Joining pairings into the `wines` bulk fetch and aggregating
in Rust — would reduce round-trips but would require a new `db` function, complicates
the existing code, and is premature optimization for this scale.

---

## Q2: Should `get_wine_grapes` still be called when the grape filter is empty?

**Decision**: Yes — `get_wine_grapes` must always be called, unchanged.

**Rationale**: FR-005 requires the grape filter to continue working. The filter logic in
`wine_table_row` (lines 96–103 of `markup.rs`) uses `wine_grapes` to decide whether to
suppress a row. That logic is not being touched. Only the display `<ul>` at the bottom
of the row is changed from grapes to pairings.

**Alternatives considered**: Removing the `get_wine_grapes` call entirely and making the
grape filter a no-op — explicitly out of scope per FR-005 and the spec assumptions.

---

## Q3: What label should the column header use?

**Decision**: `"Pairings"`.

**Rationale**: FR-002 states "the pairings column header MUST reflect the new content
(e.g., 'Pairings' instead of 'Grapes')". The example label "Pairings" is short, matches
the existing menu item label, and is consistent with how the feature is referred to
throughout the spec.

**Alternatives considered**: "Food Pairings" — more descriptive but breaks visual
symmetry with the other short column headers ("Name", "Year", "Bottles", "Comment").

---

## Q4: Should the grape filter UI remain visually unchanged?

**Decision**: Yes — the filter icon and input remain under the "Pairings" column header
(or it can be left under the renamed header). No change to the filter widget itself.

**Rationale**: The spec assumption states the grape filter "may remain, be repurposed,
or be removed in a follow-on feature." For this feature it stays exactly as-is.
The column header rename does mean the filter icon will appear next to "Pairings" rather
than "Grapes", which is slightly confusing, but acceptable per spec scope.

---

## Summary of Changes Required

| Location | Change |
|----------|--------|
| `markup.rs` `wine_table_html` line ~259 | `"Grapes"` → `"Pairings"` |
| `markup.rs` `wine_table_row` | Add `db::get_wine_food_pairings` call; render pairings `<ul>` in place of grapes `<ul>` |

No migrations, no new DB functions, no new routes, no JS changes.
