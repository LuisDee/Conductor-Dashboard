# Specification: Advanced Slack Integration & Testing Infrastructure

## 1. Overview
This track focuses on elevating the Slack integration from a conversational bot to a full-featured Slack Application with a persistent dashboard and shortcut commands. It also introduces a robust automated testing infrastructure using a Slack Mock Server to enable high-fidelity E2E testing without external dependencies.

## 2. Functional Requirements

### 2.1 Slack Mock Infrastructure
- Integrate `slack-server-mock` as a dedicated service in `docker-compose.yml`.
- Implement `SLACK_API_BASE_URL` environment variable support in the `SlackClient`.
- Ensure the backend can route all API calls to the mock server when configured.
- Create automated E2E test scenarios that verify Slack interactions using the mock server.

### 2.2 App Home Dashboard
- Implement the `app_home_opened` event handler to render a rich "App Home" view.
- **Sections to include:**
    - **Current Standing:** Display active holding periods and any open breaches.
    - **Active Requests:** A table/list of recent PAD requests with live status.
    - **My Holdings:** Summary of securities currently held.
    - **Quick Actions:** Buttons for "Submit New Request" and "Confirm Execution".

### 2.3 Slash Commands
- Implement handlers for the following commands:
    - `/pad status`: Displays a compact summary of the user's active requests.
    - `/pad report`: (Authorized users only) Displays a summary of pending manager/compliance actions.
    - `/pad help`: Displays a quick-reference guide for all app features.

### 2.4 Rich Interactive Reporting
- Enhance approval messages to dynamically update with more context (e.g., specific risk factor details) after interactions.
- Implement periodic digest reports sent to Managers/Compliance via Slack blocks.

## 3. Technical Requirements
- **Mock Server:** [ygalblum/slack-server-mock](https://github.com/ygalblum/slack-server-mock).
- **Communication:** Slack Socket Mode for both real and mock interactions.
- **Payloads:** Extensive use of Slack Block Kit for dashboards and commands.

## 4. Acceptance Criteria
- [x] Backend successfully communicates with `slack-server-mock` when `SLACK_API_BASE_URL` is set.
- [x] Users can view their full PAD profile in the Slack "App Home" tab.
- [x] `/pad status`, `/pad report`, and `/pad help` return correct, formatted Block Kit responses.
- [x] Automated tests verify that a submission via Slack results in the correct record in the database and a notification to the manager.

## 5. Out of Scope
- Integration with non-Slack messaging platforms (e.g., Microsoft Teams).
- Real-time stock price streaming (prices will remain point-in-time estimates).
