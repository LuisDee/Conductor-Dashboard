# Track Plan: Advanced Slack Integration & Testing Infrastructure

## Phase 1: Testing Infrastructure (Slack Mock) (COMPLETED)
- [x] Task: Integrate Slack Mock Server
    - [x] Subtask: Add `slack-mock` service to `docker/docker-compose.yml` using `ygalblum/slack-server-mock`.
    - [x] Subtask: Update `SlackClient` to respect `SLACK_API_BASE_URL` for endpoint overriding.
    - [x] Subtask: Verify connectivity between `api` container and `slack-mock` container.
- [x] Task: Implement Slack E2E Test Harness
    - [x] Subtask: Write baseline tests in `tests/test_slack_mock.py` that simulate a full PAD submission using the mock server.
    - [x] Subtask: Implement helper functions to inspect mock server state (messages sent, views opened).
- [x] Task: Conductor - User Manual Verification 'Testing Infrastructure' (Protocol in workflow.md)

## Phase 2: App Home Dashboard (COMPLETED)
- [x] Task: Implement App Home Event Handler
    - [x] Subtask: Add listener for `app_home_opened` in `src/pa_dealing/agents/slack/handlers.py`.
    - [x] Subtask: Implement logic to fetch user's PAD profile (history, holdings, breaches).
- [x] Task: Build Dashboard Block Kit View
    - [x] Subtask: Design and implement the multi-section Block Kit layout for App Home.
    - [x] Subtask: Connect "Quick Action" buttons to the existing PAD submission modal.
    - [x] Subtask: Write tests to verify the dashboard renders correctly for a mock user.
- [x] Task: Conductor - User Manual Verification 'App Home Dashboard' (Protocol in workflow.md)

## Phase 3: Slash Commands (COMPLETED)
- [x] Task: Implement Slash Command Dispatcher
    - [x] Subtask: Configure `SlackSocketHandler` to process `/pad` commands.
    - [x] Subtask: Implement authorized access check for `/pad report`.
- [x] Task: Implement Individual Commands
    - [x] Subtask: Implement `/pad status` (list active requests).
    - [x] Subtask: Implement `/pad help` (interactive help guide).
    - [x] Subtask: Implement `/pad report` (manager/compliance summary).
    - [x] Subtask: Write unit tests for each command handler.
- [x] Task: Conductor - User Manual Verification 'Slash Commands' (Protocol in workflow.md)

## Phase 4: Dynamic Reporting & Notifications (COMPLETED)
- [x] Task: Enhance Interactive Updates
    - [x] Subtask: Update `_process_approval` to dynamically include risk factor details in the updated Slack message block.
    - [x] Subtask: Add "View in Dashboard" deep-links to notification blocks.
- [x] Task: Implement Periodic Digest Reports
    - [x] Subtask: Create a new monitoring job `ComplianceDigestJob` to send weekly summaries to Slack.
    - [x] Subtask: Add `compliance_digest_slack_enabled` to `ComplianceConfig`.
- [x] Task: Conductor - User Manual Verification 'Dynamic Reporting' (Protocol in workflow.md)
