# Spec: Email Ingestion via Microsoft Graph Webhooks

## Overview

Contract notes and activity statements arrive via email to a monitored mailbox. This feature implements a Microsoft Graph webhook-based ingestion pipeline that receives email notifications in real-time, extracts PDF attachments, and **processes them directly** using the existing DocumentProcessorAgent and ExtractionRouter.

The architecture uses:
1. **Microsoft Graph webhooks** as the primary real-time ingestion path
2. **Shared state table** (`bo_airflow.email_ingestion_state`) for idempotency and retry coordination
3. **Background worker** for async email processing (fetch → extract → process → archive)
4. **Direct processing** via DocumentProcessorAgent + ExtractionRouter (no intermediate GCS storage)
5. **GCS archival** only AFTER successful processing (for audit trail)
6. **Subscription lifecycle management** — proactive daily renewal + reactive lifecycle handling
7. **Airflow DAG** (out of scope, separate repo) as a daily backstop for missed emails

Both the webhook path and the DAG backstop share the same email state table and SQL operations, ensuring exactly-once processing even when running concurrently.

## Architecture

```
┌─────────────────┐                              ┌─────────────────────────┐
│    Mailbox      │                              │   Subscription Mgmt     │
└────────┬────────┘                              │                         │
         │                                       │  ┌───────────────────┐  │
         │                                       │  │ Scheduled Task    │  │
         │                                       │  │ (Daily @ 9 AM)    │  │
         │                                       │  │ - Check expiry    │  │
         │                                       │  │ - Renew if <24h   │  │
         │                                       │  └─────────┬─────────┘  │
         │                                       │            │            │
         ├──── Webhook (primary) ────────┐      │  ┌─────────▼─────────┐  │
         │                               ▼      │  │ Lifecycle Handler │  │
         │                    ┌──────────────────┐ │ (Reactive)        │  │
         │                    │   FastAPI App    │ │ - reauthorization │  │
         │                    │                  │ │   → renew now     │  │
         │                    │  POST /api/graph/│ │ - removed         │  │
         │                    │    notifications │ │   → recreate now  │  │
         │                    │  POST /api/graph/│ └───────────────────┘  │
         │                    │    lifecycle ────┼──────────┘             │
         │                    └──────────┬───────┘                        │
         │                               │       └─────────────────────────┘
         │               ┌───────────────┴──────────────┐
         │               │      Background Worker       │
         │               │  1. Check state table        │
         │               │  2. Claim (atomic upsert)    │
         │               │  3. Fetch email + PDF bytes  │
         │               │  4. Process directly:        │
         │               │     → DocumentProcessorAgent │
         │               │     → ExtractionRouter       │
         │               │  5. Archive to GCS           │
         │               │  6. Mark success/failed      │
         │               └───────────────┬──────────────┘
         │                               │
         │               ┌───────────────┼───────────────┐
         │               ▼               ▼               ▼
         │        ┌──────────┐    ┌──────────┐    ┌──────────┐
         │        │ Database │    │  Trades  │    │   GCS    │
         │        │  State   │    │ Extracted│    │ Archive  │
         │        └──────────┘    └──────────┘    └──────────┘
         │
         └──── Backstop DAG (missed emails only) ─┘
```

## Functional Requirements

### FR-1: Microsoft Graph Configuration
- Add settings for Graph API credentials: `graph_tenant_id`, `graph_client_id`, `graph_client_secret`
- Add settings for mailbox: `graph_mailbox_email`, `graph_mailbox_folder` (default: Inbox)
- Store subscription_id persistently in database for renewal tracking

### FR-2: Webhook Endpoints

#### FR-2.1: Notification Endpoint (`POST /api/graph/notifications`)
- Handle validation handshake: return `validationToken` as plain text within 10 seconds
- For actual notifications: return `202 Accepted` immediately, queue for background processing
- Validate `clientState` matches configured secret

#### FR-2.2: Lifecycle Endpoint (`POST /api/graph/lifecycle`) — REACTIVE

Unlike notifications, lifecycle events require **immediate action**, not just logging:

| Event | Action |
|-------|--------|
| `reauthorizationRequired` | Trigger subscription renewal **immediately** |
| `subscriptionRemoved` | Trigger subscription recreation **immediately** |
| `missed` | Log warning (notifications were lost during downtime) |

**Implementation:**
- Handle validation handshake (same as notifications)
- Validate `clientState` (same security as notifications)
- Return `202 Accepted` immediately
- Queue background task to handle the event:
  - `reauthorizationRequired` → call `renew_subscription()`
  - `subscriptionRemoved` → call `create_subscription()`
  - `missed` → log with affected time range for investigation

### FR-3: State Table (`bo_airflow.email_ingestion_state`)

Schema:
```sql
CREATE TABLE bo_airflow.email_ingestion_state (
    id BIGSERIAL PRIMARY KEY,
    app_name VARCHAR(100) NOT NULL,           -- 'pad_contract_notes'
    message_id VARCHAR(255) NOT NULL,         -- Graph message ID (idempotency key)
    mailbox_email VARCHAR(255) NOT NULL,
    sender_email VARCHAR(255),
    received_at TIMESTAMPTZ,
    subject TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'processing',
    error_message TEXT,
    attachments_processed INTEGER NOT NULL DEFAULT 0,
    trades_extracted INTEGER NOT NULL DEFAULT 0,
    archive_gcs_paths JSONB,                  -- Archive paths (post-processing)
    worker_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_app_message UNIQUE (app_name, message_id)
);
```

Status values: `processing`, `success`, `failed`, `skipped`

### FR-3.1: Subscription State Table (`padealing.graph_subscription_state`)

Track Graph API subscription lifecycle in our own schema:

```sql
CREATE TABLE padealing.graph_subscription_state (
    id SERIAL PRIMARY KEY,
    subscription_id VARCHAR(255) NOT NULL UNIQUE,  -- Graph subscription ID
    resource VARCHAR(500) NOT NULL,                -- e.g., users/{mailbox}/mailFolders/Inbox/messages
    notification_url VARCHAR(500) NOT NULL,
    lifecycle_url VARCHAR(500),
    expiration_datetime TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',  -- active, expired, removed
    last_renewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_graph_subscription_expiry
    ON padealing.graph_subscription_state (expiration_datetime)
    WHERE status = 'active';
```

**Status values:**
- `active`: Subscription is receiving notifications
- `expired`: Subscription expired (will be recreated)
- `removed`: Graph removed subscription (will be recreated)

**Operations:**
- `get_active_subscription()`: Returns current active subscription or None
- `upsert_subscription(subscription_id, expiration, ...)`: Insert or update subscription record
- `mark_expired(subscription_id)`: Set status to expired
- `mark_removed(subscription_id)`: Set status to removed

### FR-4: Processing Flow

The processing flow MUST use these exact SQL operations for DAG compatibility:

**Step 0 — Extract message_id from notification:**

Graph webhook notifications contain a resource path, not the message_id directly:
```json
{
  "value": [{
    "subscriptionId": "abc-123",
    "clientState": "your-secret",
    "changeType": "created",
    "resource": "users/mailbox@company.com/messages/AAMkAGI2...",
    "resourceData": {
      "@odata.type": "#Microsoft.Graph.Message",
      "id": "AAMkAGI2..."
    }
  }]
}
```

Extract `message_id` from:
1. **Preferred**: `notification.resourceData.id` (direct)
2. **Fallback**: Parse from `notification.resource` (regex: `messages/([^/]+)$`)

This extraction happens BEFORE Step 1.

**Step 1 — Check if already processed:**
```sql
SELECT 1 FROM bo_airflow.email_ingestion_state
WHERE app_name = 'pad_contract_notes'
  AND message_id = %s
  AND status = 'success'
```
If exists → skip (already processed by webhook or DAG)

**Step 2 — Claim the email (atomic upsert):**
```sql
INSERT INTO bo_airflow.email_ingestion_state
    (app_name, message_id, mailbox_email, sender_email,
     received_at, subject, status, worker_id, created_at, updated_at)
VALUES
    ('pad_contract_notes', %s, %s, %s, %s, %s, 'processing', %s, NOW(), NOW())
ON CONFLICT (app_name, message_id) DO UPDATE SET
    status = CASE
        WHEN email_ingestion_state.status IN ('failed', 'skipped')
        THEN 'processing'
        ELSE email_ingestion_state.status
    END,
    worker_id = EXCLUDED.worker_id,
    updated_at = NOW()
WHERE email_ingestion_state.status != 'success'
RETURNING status
```
If RETURNING gives nothing → someone else owns it, skip.

**Step 3 — Fetch email and attachments:**
- Call Graph API: `GET /users/{mailbox}/messages/{message_id}`
- Get attachments: `GET /users/{mailbox}/messages/{message_id}/attachments`
- Filter for PDF attachments only
- Hold PDF bytes in memory (do NOT write to GCS yet)

**Step 4 — Process PDF directly:**
For each PDF attachment:
- Call `DocumentProcessorAgent.process_pdf(pdf_bytes)` to extract trade data
- Call `ExtractionRouter.process_and_route(document_id, extraction, sender_email)` for:
  - Trade deduplication (fingerprint matching)
  - Confidence-based routing (auto-confirm vs manual review)
  - Linking to pending orders

**Step 5 — Archive to GCS (post-processing):**
- Path format: `archive/{sender_email=URL_ENCODED}/received_date={YYYY-MM-DD}/{message_hash}/{filename}.pdf`
- Use URL encoding for Hive compatibility: `quote(value, safe='-_~').replace('.', '%2E')`
- This is for audit trail only — processing already complete

**Step 6 — Mark success:**
```sql
UPDATE bo_airflow.email_ingestion_state
SET status = 'success',
    attachments_processed = %s,
    trades_extracted = %s,
    archive_gcs_paths = %s,
    updated_at = NOW()
WHERE app_name = 'pad_contract_notes' AND message_id = %s
```

**Step 6b — Mark failed (on error):**
```sql
UPDATE bo_airflow.email_ingestion_state
SET status = 'failed',
    error_message = %s,
    updated_at = NOW()
WHERE app_name = 'pad_contract_notes' AND message_id = %s
```

### FR-5: Background Processing
- Return `202 Accepted` within 3 seconds (Graph requirement)
- Queue notification for async processing
- Use existing asyncio task pattern (like outbox_worker)
- Implement exponential backoff for Graph API errors (429 throttling)

### FR-6: Security

#### FR-6.1: clientState Validation (CRITICAL)

The `clientState` is the **ONLY authentication layer** for webhook endpoints. Anyone who discovers the webhook URL could attempt to send fake notifications. The clientState secret is your sole defense.

**Requirements:**
- **Validate on EVERY notification** — not just during subscription creation
- **Reject immediately if mismatch** — return `401 Unauthorized`, do NOT queue for processing
- **Log security events** — log all clientState mismatches with source IP for security audit
- **Constant-time comparison** — use `secrets.compare_digest()` to prevent timing attacks

```python
# CORRECT: Check clientState BEFORE any processing
if not secrets.compare_digest(notification.clientState, settings.graph_client_state):
    logger.warning(f"clientState mismatch from {request.client.host}")
    raise HTTPException(status_code=401, detail="Invalid clientState")

# Only after validation: return 202 and queue
```

**Storage:**
- Store `clientState` as environment variable (`GRAPH_CLIENT_STATE`)
- Generate cryptographically secure value: `secrets.token_urlsafe(32)`
- Never log the actual clientState value

#### FR-6.2: Transport Security
- HTTPS only (enforced by K8s ingress)
- Webhook URLs must use `https://` scheme when registering subscription

## Non-Functional Requirements

### NFR-1: Idempotency
- Duplicate notifications MUST NOT result in duplicate processing
- State table's unique constraint on `(app_name, message_id)` ensures this
- Trade deduplication via ExtractionRouter's fingerprint matching provides secondary protection

### NFR-2: Concurrency Safety
- Webhook and DAG can run simultaneously without conflicts
- Atomic upsert pattern prevents race conditions

### NFR-3: Reliability
- Failed emails are retried by the backstop DAG
- No notifications lost during subscription gaps (DAG catches up via delta query)

### NFR-4: Response Time
- Webhook MUST respond within 10 seconds (Graph requirement)
- Target: respond within 3 seconds with `202 Accepted`

### FR-7: Unified PDF Archive Interface

A single, source-agnostic archiving module that all PDF ingestion paths use:

```python
class PDFArchiver:
    def archive_processed_pdf(
        self,
        pdf_bytes: bytes,
        sender_email: str,        # From auth, Graph API, or hive path
        received_date: date,      # From Graph API, upload time, or hive path
        unique_id: str,           # Trade key (e.g., "ldeburna-20260204-AAPL-BUY-100")
        filename: str,            # Original filename
        source: str,              # "graph_webhook" | "manual_upload" | "poller"
    ) -> str:
        """Archive a processed PDF. Returns the GCS path."""
```

**Path format** (same for all sources):
```
archive/
  sender_email={URL_ENCODED}/
    received_date={YYYY-MM-DD}/
      {unique_id}/
        {filename}.pdf
```

**URL encoding**: `quote(value, safe='-_~').replace('.', '%2E')`

**Unique ID options** (in order of preference):
1. **Trade key**: `{username}-{date}-{ticker}-{direction}-{qty}` (deterministic, meaningful)
2. **Message ID hash**: First 16 chars of SHA256 (for Graph webhook if trade key unavailable)
3. **UUID**: Random fallback

**Source-specific parameter mapping**:

| Parameter | Graph Webhook | Manual Upload | Poller (legacy) |
|-----------|---------------|---------------|-----------------|
| `sender_email` | `message.from.address` | `request.user.email` | Hive path / metadata |
| `received_date` | `message.receivedDateTime` | `datetime.now()` | Hive path / metadata |
| `unique_id` | Trade key or msg hash | Trade key | `gcs_generation` |
| `source` | `"graph_webhook"` | `"manual_upload"` | `"poller"` |

**Blob metadata** (set on all uploads):
- `x-goog-meta-sender-email`: Original sender email
- `x-goog-meta-source`: Ingestion source
- `x-goog-meta-processed-at`: ISO timestamp
- `x-goog-meta-trade-key`: Trade key if available

### FR-8: Subscription Lifecycle Management

We own the full subscription lifecycle — no external DAG dependency.

#### FR-8.1: Graph Client Methods

Add subscription management to `services/graph_client.py`:

```python
class GraphClient:
    async def create_subscription(
        self,
        resource: str,              # users/{mailbox}/mailFolders/Inbox/messages
        notification_url: str,
        lifecycle_url: str,
        client_state: str,
        expiration_minutes: int = 4230,  # Max for messages: ~3 days
    ) -> SubscriptionInfo:
        """Create a new Graph subscription."""

    async def renew_subscription(
        self,
        subscription_id: str,
        expiration_minutes: int = 4230,
    ) -> SubscriptionInfo:
        """Extend an existing subscription's expiration."""

    async def get_subscription(
        self,
        subscription_id: str,
    ) -> SubscriptionInfo | None:
        """Get subscription details (or None if not found/expired)."""

    async def delete_subscription(
        self,
        subscription_id: str,
    ) -> bool:
        """Delete a subscription. Returns True if successful."""
```

**Subscription parameters:**
- `changeType`: `"created"` (new messages only)
- `resource`: `users/{mailbox_email}/mailFolders/{folder_id}/messages`
- `notificationUrl`: Public HTTPS endpoint for notifications
- `lifecycleNotificationUrl`: Public HTTPS endpoint for lifecycle events
- `clientState`: Shared secret for validation
- `expirationDateTime`: Max 4230 minutes (~3 days) for message resources

#### FR-8.2: Scheduled Renewal Task (Proactive)

Add a new job to the existing `MonitoringScheduler` (uses APScheduler):

```python
class JobType(str, Enum):
    GRAPH_SUBSCRIPTION_RENEWAL = "graph_subscription_renewal"

class MonitoringService:
    async def graph_subscription_renewal(self) -> JobResult:
        """
        Daily check: renew Graph subscription if expiring within 24 hours.

        1. Query graph_subscription_state for active subscription
        2. If expiration < 24 hours away → renew via Graph API
        3. If no active subscription exists → create new one
        4. Update database with new expiration
        """
```

**Schedule:** Daily at 9:00 AM (via `CronTrigger(hour="9", minute="0")`)

**Logic:**
```
IF no active subscription in DB:
    → create_subscription() via Graph API
    → store in graph_subscription_state
ELSE IF expiration_datetime < NOW + 24 hours:
    → renew_subscription() via Graph API
    → update graph_subscription_state
ELSE:
    → subscription healthy, no action needed
```

#### FR-8.3: Reactive Lifecycle Handling

The lifecycle endpoint (FR-2.2) triggers immediate action:

| Event | Reactive Action |
|-------|-----------------|
| `reauthorizationRequired` | Call `renew_subscription()` immediately |
| `subscriptionRemoved` | Call `create_subscription()` immediately, mark old as `removed` |
| `missed` | Log warning, no subscription action (emails were lost) |

**Two-Layer Protection:**
1. **Proactive**: Daily scheduled task renews before expiry
2. **Reactive**: Lifecycle events catch edge cases (credential rotation, Graph-side issues)

#### FR-8.4: Bootstrap CLI Script

For initial deployment, provide a CLI command:
```bash
python -m pa_dealing.cli.create_graph_subscription
```

This script:
1. Validates all Graph API settings are configured
2. Checks if active subscription already exists in DB
3. If not: creates subscription via Graph API
4. Stores subscription_id and expiration in `graph_subscription_state`
5. Exits with clear success/failure message

**When to run:**
- After deploying webhook endpoints (first time only)
- After the scheduled task and reactive handlers take over

## Out of Scope

- **Airflow DAG implementation** — Separate repo, uses same SQL operations for email processing
- **Delta query implementation** — DAG uses delta query to catch missed emails during subscription gaps
- **Manual upload endpoint** — Uses unified archive interface but endpoint is separate track

## In Scope (Clarification)

- **Full subscription lifecycle management** — Create, renew, track, and recover subscriptions
- **Subscription state storage** — `padealing.graph_subscription_state` table
- **Scheduled renewal task** — Daily APScheduler job integrated with existing MonitoringScheduler
- **Reactive lifecycle handling** — Immediate renewal/recreation on Graph events
- **Bootstrap CLI script** — One-time script for initial deployment
- **Webhook validation handshake** — Required for subscription creation to succeed

## Dependencies

- `msgraph-sdk` — Microsoft Graph API client
- `azure-identity` — OAuth client credentials flow
- Existing `DocumentProcessorAgent` for PDF extraction
- Existing `ExtractionRouter` for trade routing/dedup
- Existing `GCSClient` for archive uploads
- Existing asyncio task pattern from `outbox_worker.py`
- Existing `MonitoringScheduler` (APScheduler) for scheduled renewal task
- Existing `MonitoringService` pattern for job implementation

## Acceptance Criteria

### Webhook & Processing
- [ ] Graph webhook receives email notifications and returns 202 within 3 seconds
- [ ] Validation handshake works (plain text token response)
- [ ] message_id correctly extracted from `resourceData.id` or resource path
- [ ] State table prevents duplicate processing
- [ ] PDFs processed directly via DocumentProcessorAgent (no intermediate GCS)
- [ ] Trades extracted and routed via ExtractionRouter
- [ ] PDFs archived to GCS AFTER successful processing
- [ ] Lifecycle events are logged
- [ ] Failed emails have `status='failed'` for DAG retry

### Security
- [ ] clientState validated on EVERY notification (not just first)
- [ ] Invalid clientState returns 401 and logs security event
- [ ] clientState comparison uses constant-time `secrets.compare_digest()`

### Subscription Management
- [ ] `graph_subscription_state` table created and model implemented
- [ ] GraphClient has `create_subscription()`, `renew_subscription()`, `get_subscription()` methods
- [ ] Scheduled renewal job runs daily via MonitoringScheduler
- [ ] Subscription renewed proactively when < 24h remaining
- [ ] Lifecycle handler triggers immediate renewal on `reauthorizationRequired`
- [ ] Lifecycle handler triggers immediate recreation on `subscriptionRemoved`
- [ ] Bootstrap CLI script creates initial subscription and stores in DB

### Unified Archive
- [ ] Unified `PDFArchiver` interface created and used by Graph webhook
- [ ] Archive paths use trade key as unique_id when available
- [ ] Same archive interface can be called from manual upload (integration-ready)

### Quality
- [ ] >80% test coverage on new code
