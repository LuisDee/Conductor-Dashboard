# Track Plan: Detailed E2E Testing and Initial UAT

## Phase 1: Test Suite Audit & Remediation

- [x] Task: Audit Existing E2E Tests
    - [x] Subtask: Run all tests in `tests/` inside the Docker environment and identify failures.
    - [x] Subtask: Review `test_e2e_workflow.py`, `test_slack_mock.py`, and `test_auto_approve.py` for obsolescence.
- [x] Task: Remediate Slack & Workflow Tests
    - [x] Subtask: Update E2E tests to include the "Insider Information" declaration.
    - [x] Subtask: Fix escalation tests to align with new MAR/MiFID detection logic.
    - [x] Subtask: Ensure `slack-mock` interactions match the updated Block Kit payloads.
- [x] Task: Prune Obsolete Tests
    - [x] Subtask: Remove tests that bypass authorization or rely on deprecated logic.
- [x] Task: **NEW** Create MAR Compliance Test Suite (`test_mar_compliance.py`)
    - [x] Subtask: Add tests for MAR trigger detection (instruments traded by Mako in last 3 months).
    - [x] Subtask: Add tests for MiFID breach inference (AI must not approve if MAR/MiFID inferred - per spec FR6).
    - [x] Subtask: Add tests for prohibited instrument blocking (derivatives, leveraged, spread bets, ETF options).
    - [x] Subtask: Add tests for restricted list enforcement.
    - [x] Subtask: Verify AI escalation when instrument recently traded by Mako (spec: "AI must always escalate if request relates to instrument traded in last 3 months by Mako").
- [x] Task: **NEW** Add Positive Insider Information Test Cases
    - [x] Subtask: Add test where user does NOT check the insider info checkbox → request BLOCKED (handlers.py:544-553).
    - [x] Subtask: Add test for chatbot path where user refuses to confirm no insider info → request blocked.
    - [x] Subtask: Add test verifying blocked insider info requests are logged in audit trail.
    - [x] Subtask: Ensure existing tests cover the happy path (checkbox checked → proceeds).
- [x] Task: Conductor - User Manual Verification 'Phase 1: Test Suite Audit' (Protocol in workflow.md)

## Phase 2: Live-Simulation Manual UAT

- [x] Task: Employee Persona Simulation
    - [x] Subtask: Manually submit "Standard", "Restricted", and "MAR-Flagged" requests via Slack.
    - [x] Subtask: Verify "Insider Info" checkbox behavior and blocking logic.
    - [x] Subtask: Interact with Chatbot to verify policy-aware responses.
- [x] Task: Manager & Approval Simulation
    - [x] Subtask: Verify Slack notifications for different risk levels.
    - [x] Subtask: Approve/Decline requests and verify the dynamic message updates (Green/Red emojis).
- [x] Task: Compliance & SMF16 Simulation
    - [x] Subtask: Access Dashboard and verify high-density Request Table rendering.
    - [x] Subtask: Test filtering/searching by employee and security.
    - [x] Subtask: Perform SMF16 intervention on a HIGH-risk blocked request (per spec FR4: "High → block unless SMF16 intervention").
    - [x] Subtask: Verify SMF16 exception granting flow (spec Section 10: "One click to forward to SMF16 with explanation if exception needed").
- [x] Task: Conductor - User Manual Verification 'Phase 2: Manual UAT' (Protocol in workflow.md)

## Phase 3: Dashboard & Accuracy Verification

- [x] Task: **NEW** Seed "Messy Data" for Edge Case UAT
    - [x] Subtask: Add employees with Unicode characters in names (e.g., "José García", "François Müller").
    - [x] Subtask: Add securities with long names (50+ chars) and special characters.
    - [x] Subtask: Add edge-case trade values (boundary values: exactly 10k, 50k thresholds).
    - [x] Subtask: Add requests with timezone edge cases (submitted near midnight UTC).
    - [x] Subtask: Add historical requests with missing optional fields (null manager, no justification).
    - [x] Subtask: Add related-party scenarios with various relationship types.
- [x] Task: Verify Accuracy Metrics
    - [x] Subtask: Ensure the `/api/accuracy-metrics` endpoint reflects the UAT decisions.
    - [x] Subtask: Verify the "Accuracy Metrics" dashboard page displays data correctly.
- [x] Task: Audit Log Validation
    - [x] Subtask: Verify all manual UAT actions (Submissions, Decisions, Overrides) are in the Audit Log.
    - [x] Subtask: Check "Event Insight" pills for clarity and completeness.
- [x] Task: Final Regression & Polish
    - [x] Subtask: Run full regression suite (Unit + E2E + Playwright) to ensure 100% pass rate.
    - [x] Subtask: Fix any UI layout issues discovered during "Messy Data" UAT.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Final Verification' (Protocol in workflow.md)

---

## Appendix: Spec Clarifications

### SMF16 Role (from PAD spec)
Per the spec, SMF16 is NOT just a final sequential approver but provides **intervention/exception** capability:
- **FR4**: "High → block unless SMF16 intervention"
- **Section 2 (Out of Scope Phase 1)**: "Automated exception granting (handled by SMF16)"
- **Section 10**: "One click to forward to SMF16 with explanation if exception needed"

This means HIGH-risk requests should be **blocked by default** and only proceed if SMF16 actively intervenes to grant an exception.

### MAR/MiFID Requirements (from PAD spec)
- **FR6 Guardrail**: "AI must not approve if breach of MAR/MiFID inferred"
- **FR3**: "AI must always escalate if request relates to instrument traded in last 3 months by Mako"
- **Accuracy Target**: 95% correct classification of prohibited cases, 97% AI-detected holding period breaches