# Track Specification: Interactive Manual UAT Walkthrough

## Overview
This track involves executing the `MANUAL_TESTING_SCRIPT.md` in a "Real Slack" environment to validate the end-to-end functionality of the PA Dealing system. The goal is to verify that the implementation matches the product requirements and to identify/fix any gaps in the user experience.

## Functional Requirements
- **Interactive Execution**: Walk through Scenarios 1 to 6 of the manual testing script sequentially.
- **Slack Verification**:
    - Confirm DMs and channel notifications are routed correctly.
    - Verify Block Kit UI (summaries, buttons, modals) renders and behaves as expected.
    - Test "Test Mode" (`SLACK_TEST_MODE=true`) to ensure all notifications reach the designated tester.
- **Dashboard Verification**:
    - Verify role-based views (Manager, Compliance, SMF16).
    - Ensure status transitions (e.g., `pending_manager` -> `pending_compliance`) are reflected in the UI.
- **Audit & Breaches**:
    - Verify the Audit Trail captures all lifecycle events.
    - Confirm Breach detection triggers and resolution flow works.
- **Issue Handling**:
    - Fix blocking bugs or "quick wins" immediately.
    - Document non-blocking issues in a `UAT_FINDINGS.md` file for future work.

## Acceptance Criteria
- [ ] Successfully completed all 6 scenarios in the testing script.
- [ ] Slack notifications verified for all major lifecycle events.
- [ ] Manual verification that Audit logs match the actions taken.
- [ ] List of "To-be-fixed" items documented and prioritized.

## Out of Scope
- Implementing major new features not defined in the original `product.md`.
- Deep refactoring of backend architecture (unless required for a blocking fix).
