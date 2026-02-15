# Implementation Plan: Interactive Manual UAT Walkthrough

## Phase 1: Environment Setup & Connectivity (COMPLETED)
- [x] **Task: Verify Environment Config**
  - Check `docker/.env` for `SLACK_TEST_MODE=true` and `SLACK_API_BASE_URL` pointing to production.
  - Ensure API and Dashboard are running and accessible.
- [x] **Task: User Switching Verification**
  - Verify that switching between users (skemp, swilliam, cdavis) updates the dashboard and `X-Dev-User-Email` headers correctly.
  - **FIXED:** Implemented tab-specific user switching using `sessionStorage` and removed cross-tab synchronization.
- [x] **Task: Conductor - User Manual Verification 'Phase 1: Setup' (Protocol in workflow.md)**

## Phase 2: Scenario Execution (Chatbot Overhaul & Standard Flow)
- [ ] **Task: Scenario 1 - Standard Approval Flow (READY TO RE-TEST)**
  - Submit request via skemp (Slack), approve via swilliam (Manager), approve via cdavis (Compliance).
  - Verify Slack notifications and Audit logs at each step.
- [ ] **Task: Scenario 2 - High Risk & SMF16 Escalation**
  - Submit high-value trade (50,000 BARC), verify automatic price lookup and escalation to SMF16.
  - Fix any routing or status transition issues found.
- [ ] **Task: Conductor - User Manual Verification 'Phase 2: Standard Flows' (Protocol in workflow.md)**

## Phase 3: Negative Paths & Post-Approval
- [ ] **Task: Scenario 3 - Request Decline Flow**
  - Verify decline reason capture and notification to employee.
- [ ] **Task: Scenario 4 - Execution & Contract Notes**
  - Record execution and upload/preview PDF contract note.
  - Verify inline preview and metadata extraction (if applicable).
- [ ] **Task: Conductor - User Manual Verification 'Phase 3: Negative/Post-App' (Protocol in workflow.md)**

## Phase 4: Compliance & Audit
- [ ] **Task: Scenario 5 - Breach Detection & Resolution**
  - Trigger and resolve a breach; verify visibility for Compliance users.
- [ ] **Task: Scenario 6 - Complete Audit Trail Audit**
  - Verify a full request lifecycle is accurately reflected in the Audit page.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4: Compliance' (Protocol in workflow.md)**

## Phase 5: Finalization
- [ ] **Task: Consolidate UAT Findings**
  - Review `UAT_FINDINGS.md` and prioritize any remaining non-blocking issues.
- [ ] **Task: Conductor - User Manual Verification 'Finalization' (Protocol in workflow.md)**