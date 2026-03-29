# HTTP Route Contracts: Show Pairings in Wine Table

**Feature**: `003-table-pairings`

## No New Routes

This feature introduces no new HTTP routes. All existing routes remain unchanged in
signature, behavior, and response format.

## Affected Route (display only)

| Method | Path | Handler | Change |
|--------|------|---------|--------|
| `GET` | `/wine-table-body` | `wine_table_body` | Response HTML: pairings column cell content changes from grape names to food pairing names. Column header changes from "Grapes" to "Pairings". All other columns and behaviors unchanged. |

The HTMX contract (`hx-target="#wineTableBody"`, `hx-swap` behavior, query parameter
`grape_filter`) is **unchanged**.
