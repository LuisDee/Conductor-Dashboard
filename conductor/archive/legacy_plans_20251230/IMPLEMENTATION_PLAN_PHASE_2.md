# Phase 2 Implementation Plan: Enhanced Compliance, Document Handling & Workflow Automation

## 1. Objective
Enable end-to-end handling of trade confirmations (contract notes) via Slack and Web UI, automate verification using AI, and implement a robust "chasing" strategy for all stakeholders. **Crucially, this phase includes comprehensive updates to E2E and frontend tests to ensure no regression and full coverage of new features.**

## 2. Architecture Updates

### 2.1 Storage
- Create a dedicated volume/directory for storing uploaded contract notes: `data/contract_notes/`.
- File naming convention: `{request_id}_{timestamp}_{filename}`.

### 2.2 AI Document Processing
- Create `src/pa_dealing/agents/document_processor/`
- Implement `DocumentAgent` using Gemini 2.0 Flash (or similar multimodal model) to:
  - Ingest PDF/Image bytes.
  - Extract: Execution Date, Time, Security, Direction (Buy/Sell), Quantity, Price, Currency.
  - Return structured JSON.

## 3. Detailed Tasks

### 3.1 Slack Integration (File Uploads)
- **Modify `SlackSocketHandler` (`src/pa_dealing/agents/slack/handlers.py`)**:
  - Listen for `file_shared` events.
  - Check if the file is shared in a thread associated with a `PADRequest`.
  - Download the file using `SlackClient`.
  - Trigger `DocumentAgent` to extract data.
- **Verification Logic**:
  - Compare extracted data vs. `PADRequest` (approved) details.
  - Tolerance checks (e.g., price within X%, quantity exact match).
  - If valid: Automatically record execution in DB (`PADExecution`), mark `contract_note_verified=True`.
  - If invalid/mismatch: Reply in Slack with a warning and ask for human review.

### 3.2 Backend API (`src/pa_dealing/api/`)
- **New Endpoints**:
  - `POST /requests/{id}/contract_note`: Upload endpoint (for Web UI).
  - `GET /requests/{id}/contract_note`: Download/View endpoint.
  - `GET /requests/{id}/verification`: Get AI verification details.
- **Update Execution Logic**:
  - Ensure `record_execution` can accept verification metadata.

### 3.3 Frontend (`dashboard/`)
- **Request Detail Page (`RequestDetail.tsx`)**:
  - Add "Contract Note" section.
  - If missing: Show "Upload" button.
  - If present: Show "View/Download" button and "Verification Status" badge (Green check if AI verified, Yellow if manual, Red if mismatch).
- **Execution Tracking Page**:
  - Highlight trades missing contract notes.

### 3.4 Smarter Monitoring ("Chasing")

#### Configuration
- **Use `ComplianceConfig` Table:** All time thresholds MUST be stored in the database, not hardcoded.
  - `chasing_manager_hours` (default: 24)
  - `chasing_compliance_hours` (default: 4)
  - `chasing_employee_overdue_days` (default: 5)
- **Settings Service:** Ensure `MonitoringService` loads these values dynamically.

#### A. Employee Chasing (Execution & Docs)
- **Interactive Alerts:** Send buttons: [I have Executed], [Did Not Trade].

#### B. Approver Chasing
1.  **Manager Chasing:**
    *   Fetch `chasing_manager_hours` from config.
    *   Send **Reminder DM** with re-rendered [Approve]/[Decline] buttons.
2.  **Compliance Chasing:**
    *   Fetch `chasing_compliance_hours` from config.
    *   Post **"Action Required"** summary in `#compliance` channel.
3.  **SMF16 Chasing:**
    *   Same logic, targeting SMF16 user.

## 4. Work Breakdown

### Step 1: Document Agent & Storage
- Setup local storage.
- Implement `DocumentAgent` with Gemini.
- **Test:** Create unit tests with mocked Gemini responses to verify parsing logic.

### Step 2: Slack File Handling
- Update `SlackClient` & `SlackSocketHandler`.
- Wire up extraction and DB update.

### Step 3: API & Frontend
- Add API endpoints.
- Update `RequestDetail` component.

### Step 4: Monitoring Improvements
- Implement `check_pending_approvals` job using `ComplianceConfig`.
- Implement interactive reminders.

## 5. Testing Strategy (Mandatory)

**Work is not done until these tests pass.**

### 5.1 Test Assets
- Create a directory `tests/assets/contract_notes/`.
- Generate/Fabricate PDF files:
  - `valid_execution.pdf`: Contains data matching a test scenario (e.g., BUY 100 AAPL @ $150).
  - `mismatch_execution.pdf`: Contains data that deliberately mismatches (e.g., BUY 500 AAPL).
  - `unreadable.pdf`: Corrupted or non-text file.

### 5.2 Backend E2E Tests (`tests/test_e2e_scenarios.py`)
- **Scenario: Successful Contract Note Upload**
  1.  Create & Approve a PAD Request.
  2.  Simulate file upload (API call or Slack event mock) using `valid_execution.pdf`.
  3.  **Assert:** `PADExecution` created, `contract_note_verified=True`, `verification_metadata` present.
- **Scenario: Mismatch Detection**
  1.  Create & Approve a PAD Request.
  2.  Upload `mismatch_execution.pdf`.
  3.  **Assert:** `PADExecution` created but `contract_note_verified=False`. Alert logged.

### 5.3 Frontend Playwright Tests (`dashboard/tests/`)
- **Update `pages.spec.ts`**:
  - **Verify Contract Note UI:**
    - Navigate to an approved request.
    - Check for "Upload" button presence.
    - Upload a dummy file via the UI.
    - Verify UI updates to show "View" button and status badge.
  - **Verify Status Badges:** Ensure correct color/text for "Verified" vs "Pending Review".

### 5.4 Configuration Tests
- **Test Chasing Logic:**
  - Mock `ComplianceConfig` values (e.g., set manager chase time to 0 hours).
  - Run monitoring job.
  - **Assert:** Chasing logic triggers immediately based on DB config.
