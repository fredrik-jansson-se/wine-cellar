# Feature Specification: Single Editable Wine Comment

**Feature Branch**: `002-single-editable-comment`
**Created**: 2026-03-10
**Status**: Draft
**Input**: User description: "Today it's possible to have multiple wine comments. Make this into a single comment and make it editable."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Edit Wine Comment (Priority: P1)

A user viewing a wine's detail page sees a note field. If a note exists, it is displayed. The user clicks an edit control, the note becomes editable in a textarea with **Save** and **Cancel** buttons. Clicking Save updates the note immediately; clicking Cancel discards changes and returns to the read view.

**Why this priority**: This is the core of the feature — replacing the current multi-comment model with a single, editable note. Without this, the feature has no value.

**Independent Test**: Navigate to any wine, click the edit button on the note field, change the text, save, and confirm the new text appears without a page reload.

**Acceptance Scenarios**:

1. **Given** a wine with an existing comment, **When** the user opens the wine detail view, **Then** the comment text is displayed in the comment section.
2. **Given** a wine with an existing comment, **When** the user clicks the edit button and changes the text and saves, **Then** the comment is updated and the new text is shown immediately.
3. **Given** a wine with no comment, **When** the user opens the wine detail view, **Then** a placeholder or empty state is shown indicating no note has been written.
4. **Given** a wine with no comment, **When** the user clicks the edit button, enters text, and saves, **Then** the new comment is stored and displayed.

---

### User Story 2 - Clear a Comment (Priority: P2)

A user wants to remove an existing note from a wine. They edit the comment, delete all text, and save. The comment is cleared and the wine returns to a "no comment" state.

**Why this priority**: Completing the edit experience — without the ability to clear a comment, users who want to remove notes have no recourse.

**Independent Test**: Edit an existing comment, erase all text, save, and confirm the empty/placeholder state is shown.

**Acceptance Scenarios**:

1. **Given** a wine with an existing comment, **When** the user edits and clears all text and saves, **Then** the comment is removed and the empty state placeholder is shown.

---

### Edge Cases

- What happens when the user edits a comment but clicks Cancel or navigates away without saving? (Changes are discarded; the original note is unchanged.)
- What happens when a very long comment is entered? (The field should accept reasonable-length notes; display should handle wrapping gracefully.)
- What happens to existing wines that currently have multiple comments? (See Assumptions — all existing comments are concatenated into a single note during migration; no content is lost.)
- What happens when saving fails (DB/network error)? An inline error message is shown within the comment section; edit mode remains open so the user can retry without losing their edited text.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Each wine MUST have at most one note at any time (stored as `comment` column on `wines` table).
- **FR-002**: The note MUST be displayed on the wine detail view when it exists. UI label: "Note".
- **FR-003**: Users MUST be able to edit the note inline on the wine detail view without navigating to a separate page. Edit mode is triggered by a dedicated edit button or pencil icon adjacent to the note. Edit mode MUST present both a **Save** button (commits the change) and a **Cancel** button (discards the change and returns to read view).
- **FR-004**: Users MUST be able to save an updated note and see the result immediately without a full page reload.
- **FR-005**: Users MUST be able to clear a note by saving an empty value.
- **FR-006**: When no note exists, the wine detail view MUST show an empty/placeholder state that invites the user to add a note (e.g., "Add a note…").
- **FR-007**: The system MUST preserve content from all existing comments per wine during migration by concatenating them (oldest first, separated by a blank line) into the new single note field.
- **FR-008**: If saving a note fails, an inline error message MUST be shown within the note section and edit mode MUST remain open.
- **FR-009**: When a note exists, its last-modified timestamp (`comment_updated_at`) MUST be displayed below the note text in the read view.

### Key Entities

- **Wine Comment**: A single free-text note stored as a `comment` column on the `wines` table (nullable text). Attributes: text content, last-modified timestamp (`comment_updated_at`). A wine has zero or one comment.
- **Wine**: The wine entry that owns the comment. The comment is stored directly on the `wines` row — no separate comment table. Relationship: one wine to zero-or-one comment (NULL = no comment).
- **`wine_comments` table**: Removed via migration. All existing comments per wine are concatenated (oldest first, separated by a blank line) into the new `wines.comment` column.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view, add, update, or clear a wine comment in under 30 seconds without leaving the wine detail view.
- **SC-002**: 100% of wines display at most one comment at any time (no wine shows multiple comment entries).
- **SC-003**: Saving a comment change is reflected on screen within 2 seconds of the user submitting the edit.
- **SC-004**: Users can complete the full edit-comment workflow (open edit, change text, save, confirm result) in a single interaction without a full page reload.

## Clarifications

### Session 2026-03-10

- Q: How should the single-comment constraint be enforced at the database level? → A: Add a `comment` text column directly to the `wines` table; drop the `wine_comments` table via migration. Existing comments are concatenated (oldest first) into the new column.
- Q: What should the UI do if saving a comment fails (DB/network error)? → A: Show an inline error message within the comment section; keep edit mode open so the user can retry.
- Q: Which term should be used in UI copy — "comment" or "note"? → A: Use "note" in all UI copy (buttons, placeholders, headings); keep "comment" in code and DB identifiers.
- Q: Is there an explicit Cancel button in edit mode to discard changes? → A: Yes — edit mode shows both a Save and a Cancel button; Cancel discards changes and returns to the read view.
- Q: Should the note's last-modified timestamp be shown in the UI? → A: Yes — display it below the note text in the read view.
- Q: What triggers entering edit mode — dedicated button or clicking on note text? → A: A dedicated edit button or pencil icon adjacent to the note.

## Assumptions

- **Migration**: Wines that currently have multiple comments will be migrated to a single note by concatenating all existing comments in chronological order (oldest first), separated by a blank line. No comment content is lost.
- **Comment length**: No strict maximum length is enforced beyond practical UI limits; very long comments are displayed with wrapping.
- **Unsaved changes**: If a user starts editing and navigates away (e.g., closes the edit mode without saving), the original comment is preserved unchanged.
- **Timestamps**: The note's last-modified time (`comment_updated_at`) is tracked and displayed below the note text in the read view.
- **Access control**: All users of the wine cellar have equal ability to view and edit comments (no per-user ownership or permission distinctions), consistent with the single-user personal nature of the app.
