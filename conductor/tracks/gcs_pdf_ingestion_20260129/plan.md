# Plan: GCS PDF Polling & Ingestion System

**Status:** COMPLETE ✅

## Phase 1: Database Schema & Alembic Migration

- [x] Create Alembic migration for `gcs_document` table
  - UUID primary key with `gen_random_uuid()`
  - `gcs_generation` BIGINT UNIQUE (deduplication key)
  - `original_filename`, `original_gcs_path`, `sender_email`
  - `archive_gcs_path` for post-processing location
  - Status enum: pending, processing, parsed, failed, rejected
  - `retry_count`, `error_message` for failure handling
  - Timestamps: `received_at`, `processing_started_at`, `processed_at`
  - Add `source` column to distinguish manual vs automated uploads
  - Migration: `alembic/versions/20260129_1000_add_gcs_document_ingestion.py`

- [x] Create `parsed_trade` table with `document_id` FK
  - `document_id UUID REFERENCES gcs_document(id) ON DELETE RESTRICT`
  - `request_id` FK to `pad_request` for verification/matching
  - Add `confidence_score`, `raw_extracted_data JSONB`
  - Add `match_status` and `review_status` for manual review workflow

- [x] Create `document_processing_log` table for audit trail
  - `event_type`: received, processing_started, parsed, failed, retried
  - `worker_id` for multi-pod traceability

- [x] Add indexes for efficient queries
  - `ix_gcs_document_pending` for poller work queue
  - `ix_gcs_document_generation` for dedup lookups
  - `ix_parsed_trade_document` for UI PDF lookups

- [x] Create SQLAlchemy models
  - `src/pa_dealing/db/models/document.py` with GCSDocument, ParsedTrade, DocumentProcessingLog
  - Added relationships to PADRequest and ContractNoteUpload
  - Added status/event type constants

- [x] Update ContractNoteUpload with `gcs_document_id` FK
  - Links manual uploads to GCS document tracking system

## Phase 2: GCS Integration Layer

- [x] Create `src/pa_dealing/services/gcs_client.py`
  - Initialize `google.cloud.storage.Client` (lazy)
  - Configure bucket from environment variable
  - Implement `list_incoming_pdfs()` with batch size limit
  - Implement `move_to_processing(blob, document_id)` (atomic claim)
  - Implement `move_to_archive(blob, document_id, timestamp)`
  - Implement `move_to_failed(blob, document_id, error_message)`
  - Implement `generate_signed_url(archive_path, expiration)`
  - Implement `download_as_bytes()`, `get_blob_metadata()`, `get_blob_generation()`

- [x] Add GCS configuration to environment/settings
  - `GCS_BUCKET_NAME`
  - `GCS_INCOMING_PREFIX` (default: `incoming/`)
  - `GCS_PROCESSING_PREFIX` (default: `processing/`)
  - `GCS_ARCHIVE_PREFIX` (default: `archive/`)
  - `GCS_FAILED_PREFIX` (default: `failed/`)
  - `GCS_POLL_INTERVAL_SECONDS` (default: 30)
  - `GCS_POLL_BATCH_SIZE` (default: 10)

## Phase 3: Poller Service Core

- [x] Create `src/pa_dealing/services/pdf_poller.py`
  - `GCSPDFPoller` class with config injection
  - `poll_cycle()` method returning PollCycleStats dataclass
  - `_is_already_processed(generation)` dedup check
  - `_process_pdf(blob)` full pipeline orchestration
  - `_extract_filename(gcs_path)` utility
  - `AlreadyProcessedError`, `DocumentProcessingError` exception classes

- [x] Implement idempotent document registration
  - Uses flush() to catch unique constraint violations
  - Handle race condition where another worker wins

- [x] Implement atomic claim mechanism
  - Move blob to `processing/{document_id}.pdf` via rename_blob()
  - Ensures only one worker processes each file

- [x] Integrate with existing AI parser
  - Calls `DocumentProcessorAgent.process_pdf()` if configured
  - Extract trades from parser response
  - Handle parser failures gracefully (returns empty list if no processor)

- [x] Implement failure handling
  - `_mark_failed(document_id, error_message)`
  - Move blob to `failed/{YYYY}/{MM}/{document_id}.pdf`
  - Attach error metadata to GCS object

## Phase 4: Runner Script & Scheduling

- [x] Create `scripts/ops/run_pdf_poller.py`
  - Parse CLI arguments (--interval, --batch-size, --once, --recovery)
  - Initialize poller with config
  - Run poll loop with configurable interval
  - Graceful shutdown on SIGTERM/SIGINT via GracefulShutdown class
  - Log stats after each cycle

- [x] Create orphan recovery job
  - `recover_orphaned_documents()` function
  - Find documents stuck in `processing` status > timeout minutes
  - Reset to `pending` with incremented retry count

- [x] Add retry logic
  - `retry_failed_documents()` function
  - Respect backoff intervals [5, 15, 60] minutes
  - Max 3 retries before permanent failure

## Phase 5: API Endpoints for PDF Viewer

- [x] Create `src/pa_dealing/api/routes/documents.py`
  - `GET /api/documents` - list documents with filtering (status, date range, pagination)
  - `GET /api/documents/{document_id}` - get document details with trades
  - `GET /api/documents/{document_id}/pdf` - get signed URL by document ID
  - `GET /api/documents/trades/{trade_id}/pdf` - get signed URL for trade's source PDF
  - `POST /api/documents/{document_id}/retry` - manual retry trigger
  - `GET /api/documents/stats/summary` - document processing statistics

- [x] Implement signed URL generation
  - Uses `generate_signed_url()` with configurable expiration (1-24 hours)
  - Returns URL with metadata (filename, expires_in_hours)

- [x] Add document routes to main API router
  - Updated `src/pa_dealing/api/routes/__init__.py`
  - Updated `src/pa_dealing/api/main.py`

## Phase 6: Existing Manual Upload Integration

- [x] Update existing contract note upload to use new schema
  - Added `GCSFileStorage` class in `src/pa_dealing/storage/__init__.py`
  - Added `create_manual_upload_document()` helper with `source='manual'`
  - Updated `create_contract_note_upload()` to accept `gcs_document_id` FK
  - Maintains backward compatibility (gcs_document_id is optional)

- [x] Ensure both upload paths work in parallel
  - Manual upload continues to work unchanged (LocalFileStorage default)
  - Can switch to GCSFileStorage via `set_file_storage()`
  - Poller uses `source='mailbox_poller'` automatically

## Phase 7: Monitoring & Observability

- [x] Add structured logging
  - Log document_id on all operations (in pdf_poller.py)
  - Log processing duration (poll cycle stats)
  - Log error details on failure (error_message stored in DB)

- [x] Add health check endpoint
  - `/api/health` - basic liveness (already existed)
  - `/api/ready` - checks DB, Slack, and now GCS connectivity

- [x] Create monitoring queries for dashboard
  - `/api/documents/stats/summary` - counts by status
  - Processing rate tracked via document_processing_log
  - Failed documents queryable via API with status filter

## Phase 8: Testing

- [x] Unit tests for `GCSPDFPoller` (`tests/unit/test_pdf_poller.py`)
  - Test deduplication logic (`_is_already_processed`)
  - Test poll cycle stats
  - Test parsed trade insertion
  - Test failure handling and marking
  - Test event logging
  - Test orphan recovery
  - Test retry logic with backoff
  - Test filename extraction

- [x] Unit tests for GCS client (`tests/unit/test_gcs_client.py`)
  - Test initialization and settings
  - Test blob listing with PDF filtering
  - Test generation number retrieval
  - Test move_to_processing, move_to_archive, move_to_failed
  - Test signed URL generation
  - Test path parsing (gs:// and plain formats)
  - Test singleton pattern

- [x] Integration tests (`tests/integration/test_pdf_poller_integration.py`)
  - Test full pipeline with mocked GCS (13 tests)
  - Test document-trade linkage
  - Test deduplication, failure handling, orphan recovery

- [x] API endpoint tests (`tests/integration/test_documents_api.py`)
  - Test document listing, stats summary
  - Test 404 handling for document details, PDF URL, retry
  - 5 tests covering document API routes

- [ ] E2E test (requires live GCS bucket)
  - Upload PDF to incoming/
  - Run poller
  - Verify document in DB
  - Verify trades extracted
  - Verify PDF in archive
  - Verify API returns signed URL

## Phase 9: Integration & Configuration

- [x] Integrated into existing PA Dealing codebase
  - All code lives in `src/pa_dealing/` alongside existing modules
  - Uses existing database connection and session management
  - API routes registered with main FastAPI app
  - Health check extended to include GCS connectivity

- [x] Added PDF Poller as Docker Compose service
  - Service `pdf-poller` added to `docker/docker-compose.yml`
  - Runs `scripts/ops/run_pdf_poller.py` with configurable interval
  - Depends on db (healthy) before starting
  - Mounts gcloud credentials for ADC in development

- [x] Configure GCS environment variables in deployment
  - Added to `docker/.env`:
    - `GCS_BUCKET_NAME=cmek-encrypted-bucket-europe-west2-roe18`
    - `GCS_INCOMING_PREFIX=contract_notes/incoming/`
    - `GCS_PROCESSING_PREFIX=contract_notes/processing/`
    - `GCS_ARCHIVE_PREFIX=contract_notes/archive/`
    - `GCS_FAILED_PREFIX=contract_notes/failed/`
    - `GCS_POLL_INTERVAL_SECONDS=30`
    - `GCS_POLL_BATCH_SIZE=10`

- [ ] Ensure GCS service account has required permissions (production)
  - Read/write to bucket for move operations
  - Sign URLs for secure PDF access
  - Note: Uses Workload Identity in production (automatic)

## Acceptance Criteria

### AC1: Deduplication
- [ ] Same PDF (same gcs_generation) is never processed twice
- [ ] Concurrent pollers don't create duplicate documents

### AC2: Traceability
- [ ] Every trade has a `document_id` FK
- [ ] UI can retrieve PDF for any trade via API
- [ ] Signed URLs are secure and time-limited

### AC3: Reliability
- [ ] Transient failures trigger retry with backoff
- [ ] Permanent failures are logged and moved to failed/
- [ ] Orphaned processing files are recovered

### AC4: Performance
- [ ] Poller handles batch of 10 PDFs per cycle
- [ ] Poll interval is configurable (default 30s)
- [ ] Signed URL generation is fast (<100ms)

### AC5: Monitoring
- [ ] Processing stats are logged each cycle
- [ ] Failed documents are queryable
- [ ] Health checks pass when system is healthy

---

## Next Steps

User matching and verification functionality has been moved to a separate track:

**→ See: `contract_note_matching_20260129`**

This track covers:
- Account holder name extraction from PDFs
- Email source classification (broker vs user)
- Multi-strategy user matching
- PAD request linkage
- Manual review UI
- Auto-verification
