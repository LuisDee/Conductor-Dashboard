# Track: Modernize Trade Upload Pipeline

**Goal**: Establish Google Cloud Storage (GCS) as the single source of truth for all trade documentation, eliminating legacy "local disk" storage paths in Slack and ensuring the "View Contract Note" feature works consistently across all upload methods (UI and Slack).

## Problem Statement
The system currently has two disconnected "pipes" for trade confirmation:
1.  **UI Uploads**: Use the modern `TradeDocumentProcessor` and upload to GCS, BUT fail to save the GCS ID to the execution record. Result: No "View" icon.
2.  **Slack Uploads**: Use legacy logic (circa Dec 2025) that writes files to the server's local disk (`data/contract_notes/`). Result: No GCS ID, no "View" icon, and data is lost if the container is rebuilt.

## Success Criteria
- [ ] **Slack Handler**: Does NOT write to local disk. Uses `process_trade_document` to upload directly to GCS.
- [ ] **Repository**: `record_execution` and `create_contract_note_upload` accept and store `gcs_document_id`.
- [ ] **UI Handler**: Correctly passes the GCS ID to the repository.
- [ ] **Trade History**: The "View" icon appears for ALL new trades (both UI and Slack sourced).
- [ ] **Tests**: Integration tests pass with the new schema and GCS mocking.

## Architecture Change
**Before (Slack):**
`Slack -> Download -> open('local_path', 'wb') -> Parse -> DB (local_path)`

**After (Slack):**
`Slack -> Download -> io.BytesIO -> process_trade_document() -> GCS Upload -> DB (gcs_document_id)`

## Key Components
- `src/pa_dealing/agents/slack/handlers.py`: Legacy file handler.
- `src/pa_dealing/services/trade_document_processor.py`: The unified service we must use.
- `src/pa_dealing/db/repository.py`: The DB layer that needs schema updates.
- `src/pa_dealing/api/routes/requests.py`: The UI route that needs the ID fix.
