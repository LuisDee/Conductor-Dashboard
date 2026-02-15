# Plan: Modernize Trade Upload Pipeline

## Phase 1: Repository & Infrastructure
Update the database layer to support GCS linkage for executions.

- [ ] **1.1 Update `record_execution`**: Modify signature in `src/pa_dealing/db/repository.py` to accept `gcs_document_id: str | None`.
- [ ] **1.2 Update `create_contract_note_upload`**: Ensure it correctly handles the GCS ID (already partially there, needs verification).
- [ ] **1.3 Update `PADExecution` model**: Ensure `gcs_document_id` column exists or is linked via the upload table (Review schema). *Note: The relationship is currently via `contract_note_upload` -> `request_id` -> `execution` OR direct `contract_note_path`. We should favor the relational link.*

## Phase 2: Fix UI Upload Pipe
Connect the missing link in the dashboard upload flow.

- [ ] **2.1 Update `api/routes/requests.py`**: In `upload_contract_note`, retrieve `output.gcs_document_id` from the processor.
- [ ] **2.2 Pass ID to DB**: Pass this ID to `create_contract_note_upload`.

## Phase 3: Refactor Slack Handler
Delete the legacy local-write logic and implement the unified processor.

- [ ] **3.1 Remove Local I/O**: Delete `os.makedirs` and `open()` calls in `handlers.py`.
- [ ] **3.2 Implement `TradeDocumentInput`**: Construct the input object from the downloaded Slack bytes.
- [ ] **3.3 Call Unified Processor**: Use `process_trade_document` to handle extraction, verification, and GCS archiving in one step.
- [ ] **3.4 Handle Response**: Use the `TradeDocumentOutput` to send the correct Slack reply (Verified/Mismatch).

## Phase 4: Verification & Cleanup
Ensure stability and clean up technical debt.

- [ ] **4.1 Fix Integration Tests**: Update `tests/integration/test_document_errors.py` to mock `process_trade_document` instead of the raw `DocumentAgent`, or ensure the raw agent mock aligns with the new schema.
- [x] **4.2 Delete Local Data**: Remove the `data/contract_notes` directory from the repository/container (optional, or add to .gitignore).
- [ ] **4.3 Manual Test**: Perform a manual upload in Slack and verify the "View" icon appears in the Dashboard.
