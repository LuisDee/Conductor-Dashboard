# Implementation Plan: System Health UI Refactor

## Phase 1: Preparation & Re-naming
- [x] Revert `SystemHealth.tsx` naming and header to "System Health".
- [x] Update `Sidebar.tsx` and `App.tsx` paths back to `/system-health`. (Standardized with other compliance routes).
- [x] Ensure `compliance` users can still access the path (verified in Sidebar.tsx and App.tsx).

## Phase 2: UI Standardization (MAKO Design System)
- [x] Replace custom background colors (#162A4F) with standard MAKO card backgrounds.
- [x] Apply Montserrat font and correct weights to all headers and labels.
- [x] Update table styling to match `MyRequests.tsx` or `PendingApprovals.tsx` (verified via shared Table.tsx).
- [x] Standardize semantic colors (Success/Warning/Error) using MAKO palette.

## Phase 3: Layout Restructuring
- [x] Create a "Notifications" subsection header.
- [x] Move the outbox stats cards into a grouped header area.
- [x] Move the outbox table into a dedicated section container.
- [x] Add placeholder panels for "Database" and "Background Workers" to verify scalability.

## Phase 4: Verification
- [x] Verify page loads without blank screens.
- [x] Verify design matches official Mako screenshots.
- [x] Verify accessibility for Compliance personas in Dev Mode.
