# Track Specification: Comprehensive GUI Audit & Testing

## Overview
This track focuses on a rigorous quality assurance audit and functional cleanup of the PA Dealing Dashboard. It begins with a deep-dive into critical failures identified on the Request Detail page and expands into a system-wide audit of all pages and components to ensure data integrity and interaction consistency.

## Objectives
1. **Targeted Repair:** Diagnose and fix broken components on the Request Detail view (`/requests/{id}`), using ID 481 as the primary test case.
2. **System-wide Audit:** Systematically inventory every Dashboard page and component to identify functional gaps, data mismatches, and interaction failures.
3. **Automated Verification:** Codify the audit findings into Playwright tests to prevent future regressions.

## Functional Requirements
### Phase 1: Request Detail Diagnostics (`/requests/481`)
- **Compliance Alerts:** Fix logic that displays "Contract Note Mismatch" when no note has been uploaded or processed.
- **Contract Note Upload:** Repair the file upload component to ensure it successfully transmits files to the backend and updates the UI state.
- **Process History:** Investigate and fix the empty state of the request timeline/history component. Ensure it accurately reflects all lifecycle events (submission, approval, execution).

### Phase 2: Comprehensive Dashboard Audit
- **Inventory Phase:** List every route and sub-component in the React application.
- **Data Integrity Audit:** Verify that all UI elements (counts, badges, tables) correctly reflect the PostgreSQL/API state.
- **Interaction Flow Audit:** Verify that all buttons, filters, and form submissions trigger the correct API calls and handle success/error states gracefully.
- **Visual Consistency (Lower Priority):** Identify outliers in Tailwind styling or layout behavior that deviate from the design system.

## Technical Constraints
- **Test-Driven:** All fixes in Phase 1 must be preceded by a Playwright test reproducing the failure.
- **Containerization:** All testing must be performed against the Docker environment as per the project's `workflow.md`.

## Acceptance Criteria
- [ ] `/requests/481` displays a correct Process History timeline.
- [ ] Contract Note upload works successfully and triggers a status update.
- [ ] Compliance Alerts only show relevant, accurate warnings.
- [ ] A complete audit report/checklist of all Dashboard components is generated and addressed.
- [ ] Playwright E2E coverage is established for all major interaction flows.

## Out of Scope
- Backend performance optimization (unless directly causing UI timeouts).
- Major visual redesigns or rebranding.
