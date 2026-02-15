# Track Plan: Comprehensive GUI Audit & Testing

## Phase 1: Request Detail Diagnostics (TDD) (COMPLETED)
- [x] Task: Fix Process History / Timeline
    - [x] Subtask: Write Playwright test to verify Process History visibility on `/requests/481`.
    - [x] Subtask: Debug `RequestDetail` component and `ProcessHistory` sub-component data fetching.
    - [x] Subtask: Implement fix to ensure lifecycle events (Submission, Manager Action, etc.) render correctly.
- [x] Task: Fix Contract Note Upload & Alerts
    - [x] Subtask: Write Playwright test simulating a file upload failure/success.
    - [x] Subtask: Repair the `ContractNoteUpload` component interaction logic.
    - [x] Subtask: Refactor `ComplianceAlerts` logic to only trigger "Mismatch" when a note actually exists and conflicts with request data.
- [x] Task: Conductor - User Manual Verification 'Request Detail Diagnostics' (Protocol in workflow.md)

## Phase 2: Systematic Dashboard Inventory (COMPLETED)
- [x] Task: Generate Component & Route Map
    - [x] Subtask: Crawl `dashboard/src/pages` and `dashboard/src/components` to list all unique views and interactive elements.
    - [x] Subtask: Document the expected backend data source for every critical UI element.
- [x] Task: Functional Verification Suite
    - [x] Subtask: Create a comprehensive Playwright test file (`tests/audit.spec.ts`) that visits every page and checks for "Empty State" or "Data Load" errors.
- [x] Task: Conductor - User Manual Verification 'Systematic Dashboard Inventory' (Protocol in workflow.md)

## Phase 3: Data Integrity & Interaction Audit (COMPLETED)
- [x] Task: Audit Data Accuracy (Counts/Badges)
    - [x] Subtask: Verify "Active Breaches" count on Dashboard home matches the Breaches table.
    - [x] Task: Audit Success/Error Feedback
    - [x] Subtask: Ensure Toast notifications or Inline errors appear for every write operation (Approve, Decline, Save Settings).
- [x] Task: Conductor - User Manual Verification 'Data Integrity & Interaction Audit' (Protocol in workflow.md)

## Phase 4: Final Regression & Standards (COMPLETED)
- [x] Task: Final Build & Lint
    - [x] Subtask: Run `npm run lint` and `tsc` in the dashboard directory.
    - [x] Subtask: Run full Playwright regression suite with 4+ workers as per workflow.
- [x] Task: Conductor - User Manual Verification 'Final Regression & Standards' (Protocol in workflow.md)
