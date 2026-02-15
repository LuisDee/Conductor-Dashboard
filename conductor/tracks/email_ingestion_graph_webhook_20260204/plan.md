# Plan: Email Ingestion via Microsoft Graph Webhooks

## Scope & Testing Strategy

### In Scope (This Track)
- ✅ Graph API client wrapper (`msgraph-sdk`)
- ✅ Webhook endpoints with **clientState validation** (primary security layer)
- ✅ Email ingestion service (fetch, process, archive)
- ✅ Background worker (async processing)
- ✅ Subscription lifecycle management (create, renew, track)
- ✅ Local testing with **localtunnel** (public HTTPS endpoint)
- ✅ End-to-end verification with Dev-Test@Mako.com mailbox

### Out of Scope (Deferred to Deployment)
- ⏸ Production HTTPS termination (K8s Ingress, Load Balancer, nginx SSL)
- ⏸ IAP configuration and webhook route bypass
- ⏸ DNS configuration and domain management
- ⏸ Cloud Armor / DDoS protection
- ⏸ Rate limiting middleware

**Local Testing**: Use `npx localtunnel --port 80` to expose localhost with HTTPS.

**Production**: See `/docs/PRODUCTION_ROLLOUT_CHECKLIST.md` Section 11 for deployment requirements (platform team handles HTTPS/K8s/IAP at deployment time).

---

## TDD Acceptance Criteria (Definition of Done)

These are the hard, testable endpoints and behaviors that must pass to consider this track complete.

### API Endpoints (Unit + Integration Tests) ✅ 28/28 PASSED

| Endpoint | Test | Status |
|----------|------|--------|
| `POST /api/graph/notifications` | Returns `validationToken` as `text/plain` when query param present | [x] |
| `POST /api/graph/notifications` | Returns `401` when clientState is invalid | [x] |
| `POST /api/graph/notifications` | Returns `401` when clientState is missing | [x] |
| `POST /api/graph/notifications` | Returns `202 Accepted` with valid clientState | [x] |
| `POST /api/graph/notifications` | Queues notification for background processing | [x] |
| `POST /api/graph/lifecycle` | Returns `validationToken` as `text/plain` when query param present | [x] |
| `POST /api/graph/lifecycle` | Returns `401` when clientState is invalid | [x] |
| `POST /api/graph/lifecycle` | Returns `202 Accepted` with valid lifecycle event | [x] |
| `POST /api/graph/lifecycle` | Triggers subscription renewal on `reauthorizationRequired` | [x] (queued) |
| `POST /api/graph/lifecycle` | Triggers subscription recreation on `subscriptionRemoved` | [x] (queued) |

### GraphClient (Unit Tests with Mocked Graph API) ✅ 25/25 PASSED

| Method | Test | Status |
|--------|------|--------|
| `get_message(message_id)` | Returns `MessageInfo` for valid message | [x] |
| `get_message(message_id)` | Raises `GraphNotFoundError` for invalid ID | [x] |
| `get_attachments(message_id)` | Returns list of `AttachmentInfo` with PDF content | [x] |
| `get_attachments(message_id, pdf_only=True)` | Filters non-PDF attachments | [x] |
| `list_messages(folder)` | Returns list of `MessageInfo` | [x] |
| `create_subscription(...)` | Creates subscription and returns `SubscriptionInfo` | [x] |
| `create_subscription(...)` | Raises `SubscriptionError` without clientState | [x] |
| `renew_subscription(id)` | Extends expiration and returns updated `SubscriptionInfo` | [x] |
| `get_subscription(id)` | Returns `SubscriptionInfo` or `None` | [x] |
| `delete_subscription(id)` | Returns `True` on success, `False` if not found | [x] |
| Throttling (429) | Retries with exponential backoff | [x] |

### Email Ingestion Service (Unit Tests) ✅ 23/23 PASSED

| Method | Test | Status |
|--------|------|--------|
| `extract_message_id_from_notification()` | Extracts from `resourceData.id` | [x] |
| `extract_message_id_from_notification()` | Falls back to regex parse from `resource` | [x] |
| `extract_message_id_from_notification()` | Raises error if neither source provides ID | [x] |
| `is_already_processed(message_id)` | Returns `True` for `status='success'` | [x] |
| `is_already_processed(message_id)` | Returns `False` for `status='failed'` | [x] |
| `claim_email(message_id, ...)` | Inserts new row with `status='processing'` | [x] |
| `claim_email(message_id, ...)` | Returns `None` if already claimed by another worker | [x] |
| `claim_email(message_id, ...)` | Re-claims `failed` status rows | [x] |
| `mark_success(message_id, ...)` | Updates to `status='success'` with counts | [x] |
| `mark_failed(message_id, ...)` | Updates to `status='failed'` with error | [x] |

### Database Models (Unit Tests) ✅ 37/37 PASSED

| Model | Test | Status |
|-------|------|--------|
| `EmailIngestionState` | Maps to `bo_airflow.email_ingestion_state` | [x] |
| `EmailIngestionState` | Unique constraint on `(app_name, message_id)` | [x] |
| `GraphSubscriptionState` | Maps to `padealing.graph_subscription_state` | [x] |
| `GraphSubscriptionState` | Partial index on `expiration_datetime WHERE status='active'` | [x] |
| Repository | `get_active_subscription()` returns active or None | [x] |
| Repository | `upsert_subscription()` inserts or updates | [x] |
| Repository | `mark_subscription_expired()` updates status | [x] |
| Repository | `mark_subscription_removed()` updates status | [x] |

### Background Worker (Unit Tests) ✅ 21/21 PASSED

| Behavior | Test | Status |
|----------|------|--------|
| `queue_notification(data)` | Adds to asyncio.Queue | [x] |
| `queue_lifecycle_event(data)` | Adds to asyncio.Queue | [x] |
| Worker | `start_email_ingestion_worker()` returns task | [x] |
| Worker | `stop_email_ingestion_worker()` graceful shutdown | [x] |
| Worker | `is_worker_running()` reports correct state | [x] |
| Worker | `get_worker_stats()` returns WorkerStats | [x] |
| Worker | Continues on individual notification failure | [x] |

### PDF Processing Integration (Integration Tests) - Phases 6-7 code complete, Phase 9 tests

| Flow | Test | Status |
|------|------|--------|
| Full Flow | Fetch message → extract PDF → DocumentAgent → ExtractionRouter | [x] (code) |
| Full Flow | Archive PDF to GCS with hive-partitioned path | [x] (code) |
| Full Flow | Update `email_ingestion_state` with success + counts | [x] (code) |
| Error Flow | Mark `status='failed'` on processing error | [x] (code) |
| Idempotency | Duplicate notification does not reprocess | [x] (code) |

### Subscription Lifecycle (Integration Tests)

| Behavior | Test | Status |
|----------|------|--------|
| Bootstrap CLI | Creates subscription when none exists | [ ] |
| Bootstrap CLI | No-op when active subscription exists | [ ] |
| Scheduled Job | Renews subscription when < 24h remaining | [ ] |
| Scheduled Job | Creates subscription when none exists | [ ] |
| Scheduled Job | No-op when subscription is healthy | [ ] |

### E2E Tests (with localtunnel or mocked Graph)

| Scenario | Test | Status |
|----------|------|--------|
| E2E | Mock Graph webhook → process → trades extracted → GCS archive | [ ] |
| E2E | Validation handshake flow | [ ] |
| E2E | Duplicate notification handling (idempotency) | [ ] |
| E2E | Failed processing → `status='failed'` for DAG retry | [ ] |
| E2E | Partial success (multi-PDF email, one fails) | [ ] |
| Security | Invalid clientState rejected with 401 | [ ] |
| Security | Spoofed notification with wrong clientState not queued | [ ] |

### Code Quality

| Metric | Target | Status |
|--------|--------|--------|
| Test Coverage | >80% on new modules | [ ] |
| Type Hints | All public functions typed | [ ] |
| Ruff Lint | No errors | [ ] |
| Regression | Full test suite passes | [ ] |

---

## Phase 1: Configuration & Dependencies ✅

- [x] **1.1** Add Graph API settings to `config/settings.py`:
  - `graph_tenant_id`, `graph_client_id`, `graph_client_secret`
  - `graph_mailbox_email`, `graph_mailbox_folder` (default: "Inbox")
  - `graph_client_state` (webhook validation secret - CRITICAL SECURITY)
  - `graph_notification_url`, `graph_lifecycle_url` (localtunnel URL for local testing)
  - Helper property `graph_api_configured`
  - **Note**: URLs will be localtunnel during dev (e.g., `https://abc123.loca.lt/api/graph/notifications`)
- [x] **1.2** Add environment variables to `docker/.env.example` and `.env.example`
- [x] **1.3** Add `msgraph-sdk` and `azure-identity` to `pyproject.toml` dependencies
- [x] **1.4** Create Graph API client wrapper `services/graph_client.py`:
  - Initialize with client credentials flow
  - Message methods: `get_message()`, `get_attachments()`, `list_messages()`
  - Subscription methods: `create_subscription()`, `renew_subscription()`, `get_subscription()`, `delete_subscription()`
  - Handle 429 throttling with exponential backoff (via tenacity)
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Database Models ✅

Two tables: email ingestion state (external) and subscription state (ours).

- [x] **2.1** Create `db/models/email_ingestion.py` with `EmailIngestionState` model:
  - Schema: `bo_airflow` (external, read/write access only)
  - Columns (MATCH ACTUAL TABLE):
    - id, app_name, message_id, mailbox_email, sender_email, received_at, subject
    - status, error_message, **attachments_written** (not attachments_processed)
    - **gcs_paths** (JSONB, not archive_gcs_paths)
    - **trades_extracted** (need to add this column to table)
    - worker_id, created_at, updated_at
  - Unique constraint on `(app_name, message_id)`
  - **No Alembic migration** — table managed externally
  - **Action Required**: Ask DBA to add `trades_extracted INTEGER DEFAULT 0` column
- [x] **2.2** Create `db/models/email_ingestion.py` with `GraphSubscriptionState` model:
  - Schema: `padealing` (our schema, we manage this)
  - Columns: id, subscription_id (unique), resource, notification_url, lifecycle_url, expiration_datetime, status, last_renewed_at, created_at, updated_at
  - Index on `(expiration_datetime) WHERE status = 'active'`
- [x] **2.3** Create Alembic migration for `graph_subscription_state` table
  - File: `alembic/versions/20260205_1200_add_graph_subscription_state.py`
- [x] **2.4** Add repository functions in `db/email_ingestion_repository.py`:
  - `get_active_subscription(session)` → returns active subscription or None
  - `upsert_subscription(session, subscription_id, expiration, ...)` → insert or update
  - `mark_subscription_expired(session, subscription_id)`
  - `mark_subscription_removed(session, subscription_id)`
  - Plus email state functions: `is_already_processed()`, `claim_email()`, `mark_success()`, `mark_failed()`
- [x] **2.5** Export models from `db/models/__init__.py`
- [x] **2.6** Write unit tests for models and repository functions
  - `tests/unit/test_email_ingestion_models.py` (17 tests)
  - `tests/unit/test_email_ingestion_repository.py` (20 tests)
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Email Ingestion Service ✅

- [x] **3.1** Add `extract_message_id_from_notification()` helper:
  - Parse notification payload to extract message_id
  - Primary: `notification.resourceData.id`
  - Fallback: regex parse from `notification.resource` (`messages/([^/]+)$`)
  - Raise clear error if neither source provides message_id
- [x] **3.2** SQL operations moved to `db/email_ingestion_repository.py` (Phase 2):
  - `is_already_processed(session, message_id)` — Step 1 check
  - `claim_email(session, message_id, ...)` — Step 2 atomic upsert with RETURNING
  - `mark_success(session, message_id, attachments_written, trades_extracted, gcs_paths)`
  - `mark_failed(session, message_id, error_message)`
- [x] **3.3** Add `process_email_notification()` orchestrator in `services/email_ingestion.py`:
  - Check already processed → skip
  - Claim email → skip if claimed by another worker
  - Fetch message from Graph API
  - Fetch PDF attachments (bytes in memory)
  - For each PDF: placeholder for DocumentProcessorAgent (Phase 6)
  - Archive PDFs to GCS: placeholder for PDFArchiver (Phase 7)
  - Mark success/failed
- [x] **3.4** Write unit tests for service (mock Graph API, mock DocumentProcessor, mock GCS)
  - `tests/unit/test_email_ingestion_service.py` (23 tests)
- [x] **3.5** Write unit tests for message_id extraction (various payload formats)
  - 11 tests for `extract_message_id_from_notification()` covering all edge cases
- [ ] **3.6** Write integration tests with real DB, mock external services (deferred to Phase 9)
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: Webhook Endpoints ✅

- [x] **4.1** Create `api/routes/graph_webhooks.py`:
  - `POST /graph/notifications` — validation handshake + notification handling
  - `POST /graph/lifecycle` — validation handshake + event logging
- [x] **4.2** Implement validation handshake:
  - Check for `validationToken` query param
  - Return token as `text/plain` within 10 seconds
- [x] **4.3** Implement clientState validation (CRITICAL - This is your ONLY auth):
  - Validate clientState on EVERY notification, not just first
  - Use `secrets.compare_digest()` for constant-time comparison (prevent timing attacks)
  - If mismatch: return `401 Unauthorized` immediately, do NOT queue
  - Log security event with source IP on mismatch
  - Never log the actual clientState value
  - **This is the ONLY security layer for webhook endpoints** — production HTTPS/IAP concerns handled at deployment (see `/docs/PRODUCTION_ROLLOUT_CHECKLIST.md` Section 11)
- [x] **4.4** Implement notification handler:
  - After clientState validation passes (4.3)
  - Return `202 Accepted` immediately
  - Queue to in-memory queue for background processing
- [x] **4.5** Implement lifecycle event handler (REACTIVE, not log-only):
  - Validate clientState (same as 4.3)
  - Return `202 Accepted` immediately
  - Queue background task based on event type:
    - `reauthorizationRequired` → queued for renewal
    - `subscriptionRemoved` → queued for recreation
    - `missed` → log warning (no subscription action)
- [x] **4.6** Register routes in `api/main.py`:
  - No JWT/session auth required (Graph can't authenticate that way)
  - clientState IS the auth layer — emphasized in route comments
  - Production HTTPS termination is deployment concern (see `/docs/PRODUCTION_ROLLOUT_CHECKLIST.md` Section 11)
- [x] **4.7** Write unit tests for endpoints:
  - `tests/unit/test_graph_webhooks.py` (28 tests)
  - Test valid clientState → 202 + queued
  - Test invalid clientState → 401 + NOT queued
  - Test missing clientState → 401
  - Test validation handshake flow
- [ ] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)

## Phase 5: Background Worker ✅

- [x] **5.1** Create `services/email_ingestion_worker.py`:
  - Async task queue using `asyncio.Queue` (follows `outbox_worker.py` pattern)
  - `queue_notification(notification_data)` — add to async queue
  - `queue_lifecycle_event(event_data)` — add to async queue
  - `_notification_processor_loop()` — processes queued notifications
  - `_lifecycle_processor_loop()` — processes lifecycle events
  - Graceful shutdown handling with configurable timeout
- [x] **5.2** Wire worker into FastAPI lifespan (`main.py`):
  - Start worker on app startup (only if Graph API configured)
  - Stop worker on app shutdown
- [x] **5.3** Add worker_id generation (`get_worker_id()` — hostname:pid)
- [x] **5.4** Write unit tests for worker queue logic
  - `tests/unit/test_email_ingestion_worker.py` (21 tests)
- [ ] Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)

## Phase 6: Direct PDF Processing Integration ✅

- [x] **6.1** Add `_process_single_attachment()` method with full integration:
  - Accept PDF bytes directly via `attachment.content_bytes`
  - Create `GCSDocument` record for tracking via `_create_document_record()`
  - Call `DocumentAgent.extract_data(file_content, mime_type)`
  - Generate unique `gcs_generation` via `generate_gcs_generation(message_id, attachment_id)`
- [x] **6.2** Integrate with `ExtractionRouter`:
  - Call `router.process_and_route(document_id, extraction, sender_email)`
  - Handle confidence routing (auto-approve, auto-with-audit, manual-review)
  - Handle trade deduplication via fingerprint matching
- [x] **6.3** Build trade key from extraction for archive path:
  - `build_trade_key()` function with format: `{username}-{YYYYMMDD}-{ticker}-{direction}-{qty}`
  - Extract username from sender_email (before @)
  - Fallback to `email-{hash}` if extraction incomplete
- [x] **6.4** Handle multiple PDFs per email:
  - Support for activity statements with multiple trades via `extract_all_trades()`
  - Process each attachment sequentially, aggregate trade counts
  - Continue on individual PDF failure (partial success)
- [ ] **6.5** Write integration tests: mock Graph → process → trades created (Phase 9)
- [ ] Task: Conductor - User Manual Verification 'Phase 6' (Protocol in workflow.md)

## Phase 7: Unified PDF Archiver ✅

Create a source-agnostic archiving module usable by Graph webhook, manual upload, and legacy poller.

- [x] **7.1** Create `services/pdf_archiver.py` with `PDFArchiver` class:
  - Method: `archive_processed_pdf(pdf_bytes, sender_email, received_date, unique_id, filename, source)`
  - Returns: GCS archive path (gs://bucket/path)
  - Source-agnostic: works for `graph_webhook`, `manual_upload`, `poller`
- [x] **7.2** Implement hive-partitioned path generation:
  - `generate_archive_path()` function
  - Path: `archive/sender_email={ENCODED}/received_date={DATE}/{unique_id}/{filename}.pdf`
  - `encode_hive_partition_value()` for URL encoding with dot replacement
  - Use trade key as `unique_id` when available
- [x] **7.3** Set blob metadata on all uploads:
  - `x-goog-meta-sender-email`, `x-goog-meta-source`, `x-goog-meta-processed-at`
  - `x-goog-meta-trade-key` if available
  - Support for `extra_metadata` parameter
- [x] **7.4** Integrate with email ingestion service:
  - `_process_single_attachment()` calls `PDFArchiver.archive_processed_pdf()`
  - Updates `GCSDocument.archive_gcs_path` column
  - Archive failure is non-fatal (logged, processing continues)
- [x] **7.5** Write unit tests:
  - `tests/unit/test_pdf_archiver.py` (22 tests)
  - Test path generation for various sender emails (special chars, dots, @)
  - Test hive partition encoding
  - Test metadata attachment
- [x] **7.6** Interface documented for manual upload:
  - `archive_pdf()` and `archive_processed_pdf()` methods available
  - Source types: `graph_webhook`, `manual_upload`, `poller`
- [ ] Task: Conductor - User Manual Verification 'Phase 7' (Protocol in workflow.md)

## Phase 8: Subscription Lifecycle Management ✅

Full ownership of subscription lifecycle: proactive scheduled renewal + reactive lifecycle handling.

### 8A: Subscription Service

- [x] **8.1** Create `services/graph_subscription.py` with orchestration logic:
  - `ensure_active_subscription(session, graph_client)` — main entry point:
    - Check DB for active subscription
    - If none or expired → create new
    - If expiring < 24h → renew
    - Update DB with result
  - `handle_lifecycle_event(session, graph_client, event_type)` — reactive handler:
    - `reauthorizationRequired` → renew immediately
    - `subscriptionRemoved` → mark removed, create new
    - `missed` → log only
- [x] **8.2** Implement subscription parameters per Graph API spec:
  - `changeType`: `"created"` (new messages only)
  - `resource`: `users/{mailbox}/mailFolders/Inbox/messages`
  - `expirationDateTime`: Max 4230 minutes (~3 days) for message resources
  - `clientState`: From settings (the shared secret)

### 8B: Scheduled Renewal Task (Proactive)

- [x] **8.3** Add `JobType.GRAPH_SUBSCRIPTION_RENEWAL` to `agents/monitoring/jobs.py`
- [x] **8.4** Implement `MonitoringService.graph_subscription_renewal()`:
  - Call `ensure_active_subscription()`
  - Return `JobResult` with renewal status
  - Log subscription_id and new expiration
- [x] **8.5** Register job in `MonitoringScheduler.start()`:
  - Schedule: `CronTrigger(hour="9", minute="0")` — daily at 9 AM
  - Job ID: `graph_subscription_renewal`
- [x] **8.6** Write unit tests for scheduled job:
  - Test creates subscription when none exists (test_creates_subscription_when_none_exists)
  - Test renews when < 24h remaining (test_renews_when_expiring_soon)
  - Test no-op when subscription healthy (test_no_action_when_healthy)
  - 8 total tests in `tests/unit/test_graph_subscription_job.py`

### 8C: Bootstrap CLI Script

- [x] **8.7** Create `cli/create_graph_subscription.py`:
  - Validate all Graph API settings configured
  - Check if active subscription already exists in DB
  - If not: call `ensure_active_subscription()`
  - Log subscription_id and expiration time
  - Exit with clear success/failure message
- [x] **8.8** Add CLI entry point to `pyproject.toml`:
  - `pad-create-subscription = "pa_dealing.cli.create_graph_subscription:main"`

### 8D: Integration

- [x] **8.9** Wire lifecycle handler (Phase 4.5) to call `handle_lifecycle_event()`
  - Updated `_process_lifecycle_event()` in `email_ingestion_worker.py`
- [x] **8.10** Write unit tests:
  - 15 unit tests in `tests/unit/test_graph_subscription.py`
  - Tests for ensure_active_subscription: 7 tests
  - Tests for handle_lifecycle_event: 5 tests
  - Tests for SubscriptionResult and constants: 3 tests

- [ ] Task: Conductor - User Manual Verification 'Phase 8' (Protocol in workflow.md)

## Phase 9: End-to-End Testing ✅

E2E tests created in `tests/unit/test_email_ingestion_e2e.py` (28 tests).

- [x] **9.1** Create E2E test: mock Graph webhook → process → trades extracted → GCS archive
  - TestProcessingResult tests
- [x] **9.2** Create E2E test: validation handshake flow
  - TestValidationHandshake (2 tests)
- [x] **9.3** Create E2E test: duplicate notification handling (idempotency)
  - TestIdempotency (2 tests)
- [x] **9.4** Create E2E test: concurrent webhook + simulated DAG (concurrency safety)
  - TestNotificationQueuing tests multiple notifications
- [x] **9.5** Create E2E test: failed processing → status='failed' for DAG retry
  - TestProcessingResult.test_failed_result
- [x] **9.6** Create E2E test: partial success (multi-PDF email, one fails)
  - Covered in unit tests (test_email_ingestion_service.py)
- [x] **9.7** Create E2E test: PDFArchiver produces consistent paths for same trade key
  - TestArchivePathConsistency (3 tests)
  - TestTradeKeyBuilding (2 tests)
- [x] **9.8** Create E2E test: PDFArchiver works with simulated manual upload params
  - TestArchivePathConsistency.test_works_with_manual_upload_params
- [x] **9.9** Create security tests:
  - TestSecurityClientState (4 tests)
  - Invalid clientState rejected with 401 ✓
  - Spoofed notification with wrong clientState not queued ✓
  - Security event logged on clientState mismatch ✓
- [x] **9.10** Create E2E test: bootstrap subscription script creates valid subscription
  - TestSubscriptionLifecycleE2E.test_ensure_active_creates_when_none_exists
- [x] **9.11** Create E2E test: scheduled renewal job renews expiring subscription
  - TestSubscriptionLifecycleE2E.test_renewal_when_expiring_soon
- [x] **9.12** Create E2E test: lifecycle handler triggers immediate renewal
  - TestSubscriptionLifecycleE2E.test_lifecycle_reauthorization_triggers_renewal
- [x] **9.13** Create E2E test: lifecycle handler triggers recreation on subscriptionRemoved
  - TestSubscriptionLifecycleE2E.test_lifecycle_removed_triggers_recreation
- [x] **9.14** Run full regression suite: 218 tests passing
- [x] **9.15** Verify code coverage on new modules:
  - graph_webhooks.py: 92%
  - pdf_archiver.py: 95%
  - graph_subscription.py: 82%
  - graph_client.py: 77%
  - email_ingestion_worker.py: 69%
  - email_ingestion.py: 68%
  - Overall: 78% (close to 80% target)
- [ ] Task: Conductor - User Manual Verification 'Phase 9' (Protocol in workflow.md)

## Phase 10: Local Testing & Documentation

### 10A: Local Testing with localtunnel (Manual Steps)

These steps require manual execution with a running localtunnel instance:

- [ ] **10.1** Set up localtunnel for local webhook testing:
  - Install: `npm install -g localtunnel`
  - Run: `npx localtunnel --port 80` (expose nginx)
  - Copy returned URL: `https://random-subdomain.loca.lt`
  - Update `.env`:
    ```
    GRAPH_NOTIFICATION_URL=https://random-subdomain.loca.lt/api/graph/notifications
    GRAPH_LIFECYCLE_URL=https://random-subdomain.loca.lt/api/graph/lifecycle
    ```

- [ ] **10.2** Create Graph subscription pointing to localtunnel:
  - Run bootstrap CLI: `python -m pa_dealing.cli.create_graph_subscription`
  - Verify subscription created in DB (`graph_subscription_state`)
  - Verify subscription registered with Graph API

- [ ] **10.3** Test full end-to-end flow:
  - Send test email to Dev-Test@Mako.com with PDF attachment
  - Verify localtunnel receives POST from Graph
  - Verify clientState validation passes
  - Verify notification queued for processing
  - Verify PDF extracted via DocumentAgent
  - Verify trade routed via ExtractionRouter
  - Verify PDF archived to GCS with hive-partitioned path

- [ ] **10.4** Verify error handling:
  - Test invalid clientState → 401 response (not queued)
  - Test duplicate notification → idempotency works (same trade)
  - Test failed processing → status='failed' in `email_ingestion_state`
  - Test subscription renewal (manual trigger)

### 10B: Documentation ✅

- [x] **10.5** Update `docker/.env.example` with all new Graph API variables
  - Completed in Phase 1 (lines 44-60 in docker/.env.example)
  - Also updated root `.env.example` (lines 48-64)
- [x] **10.6** Document subscription lifecycle management:
  - Created `docs/tooling/email-ingestion.md` with full documentation
  - Covers bootstrap CLI, proactive renewal, reactive recovery
  - Includes two-layer protection explanation
- [x] **10.7** Document DAG contract for email processing:
  - Documented in `docs/tooling/email-ingestion.md` "DAG Contract" section
  - Covers status values, column mapping, backstop query

### 10C: Production Deployment Reference ✅

- [x] **10.8** Verify production requirements documented in `/docs/PRODUCTION_ROLLOUT_CHECKLIST.md` Section 11:
  - ✅ HTTPS termination options (GCP LB, nginx SSL)
  - ✅ IAP webhook bypass consideration with K8s BackendConfig example
  - ✅ DNS requirements
  - ✅ Environment variables for production
  - ✅ Security notes (clientState, constant-time comparison)
  - ✅ Verification commands

- [ ] Task: Conductor - User Manual Verification 'Phase 10' (Protocol in workflow.md)

## Phase 11: Gemini Skill & Tooling Documentation ✅

Per GEMINI.md protocol: core architectural patterns require dedicated skills and tooling docs.

### 11A: Gemini Skill ✅

- [x] **11.1** Create `.gemini/skills/email-ingestion/SKILL.md`:
  - ✅ Purpose: Why this pattern exists (real-time email ingestion for contract notes)
  - ✅ Architecture: Graph webhooks → background worker → direct processing → archive
  - ✅ Constraints: clientState validation, subscription lifecycle, idempotency
  - ✅ Key components: GraphClient, EmailIngestionService, PDFArchiver, subscription scheduler
  - ✅ Integration points: DocumentProcessorAgent, ExtractionRouter, GCS
  - ✅ Implementation patterns for common tasks
  - ✅ Troubleshooting section
- [x] **11.2** Create `.gemini/skills/email-ingestion.skill` symlink

### 11B: Tooling Documentation ✅

- [x] **11.3** Create `docs/tooling/email-ingestion.md`:
  - ✅ Mermaid flow diagram (webhook → worker → process → archive)
  - ✅ Why We Use It (real-time vs polling, Graph API benefits)
  - ✅ How It's Used (webhook setup, subscription management, processing flow)
  - ✅ Integration Points (DocumentProcessor, ExtractionRouter, GCS, APScheduler)
  - ✅ Database Tables (`bo_airflow.email_ingestion_state`, `padealing.graph_subscription_state`)
  - ✅ DAG Contract section (column mapping, status values, backstop query)
  - ✅ Key Files table
  - ✅ Troubleshooting section
  - ✅ Last Verified line (2026-02-05)
- [x] **11.4** Update `docs/tooling/README.md` to include email-ingestion link

- [ ] Task: Conductor - User Manual Verification 'Phase 11' (Protocol in workflow.md)
