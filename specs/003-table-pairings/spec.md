# Feature Specification: Show Pairings in Wine Table

**Feature Branch**: `003-table-pairings`
**Created**: 2026-03-29
**Status**: Draft
**Input**: User description: "Show pairings in the table instead of grapes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Food Pairings in Main Table (Priority: P1)

A user opens the wine list and wants to quickly see what foods pair well with each wine without navigating to individual wine detail pages. Currently the table shows grape varieties; after this change it shows food pairings instead.

**Why this priority**: This is the entire scope of the feature. The value is immediate visibility of food pairing data in the overview table, which makes it actionable at a glance.

**Independent Test**: Can be fully tested by loading the wine list page and verifying the pairings column shows food pairings for wines that have them, and is empty for wines without.

**Acceptance Scenarios**:

1. **Given** a wine with food pairings exists, **When** the user views the main wine table, **Then** the table shows the wine's food pairings in the dedicated column (replacing grapes).
2. **Given** a wine with no food pairings exists, **When** the user views the main wine table, **Then** the pairings column for that row is empty (no error, no placeholder text required).
3. **Given** the main wine table is loaded, **When** the user scans the column headers, **Then** the column formerly labeled "Grapes" is now labeled "Pairings" (or equivalent).

---

### Edge Cases

- What happens when a wine has no food pairings? The cell is blank.
- What happens when a wine has many food pairings? The list renders all of them (same as the current grape list behavior).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The main wine overview table MUST display food pairings in place of grape varieties.
- **FR-002**: The pairings column header MUST reflect the new content (e.g., "Pairings" instead of "Grapes").
- **FR-003**: Each wine row MUST list all food pairings associated with that wine in the pairings cell.
- **FR-004**: Wine rows with no food pairings MUST display an empty cell (no error state).
- **FR-005**: The grape filter search input and all other existing table features MUST continue to work unchanged.
- **FR-006**: The "Grapes" and "Pairings" action menu items in the per-row dropdown MUST remain available for editing grape and pairing data respectively.

### Key Entities

- **Wine**: A bottle entry with name, year, inventory count, comment, associated grapes, and associated food pairings.
- **Food Pairing**: A food item associated with a wine (e.g., "steak", "brie").

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After the change, 100% of wine rows in the overview table display the food pairings column instead of the grapes column.
- **SC-002**: Page load time for the wine table is not measurably increased compared to before the change.
- **SC-003**: No existing table functionality (filtering, actions, sorting) is broken by the change.

## Assumptions

- Grape data remains editable via the existing "Grapes" action in the per-row dropdown; it is only removed from the visible table column, not from the system.
- The grape filter input (filter by grape variety) is out of scope for this feature — it may remain, be repurposed, or be removed in a follow-on feature.
- The visual format for pairings in the table (unordered list) mirrors the existing grape list format.
