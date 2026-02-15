# Plan: Contract Note Ingestion Pipeline

## Phase 1: File Storage Abstraction Layer

- [x] Task: Write tests for FileStorage interface and LocalFileStorage
  - [x] Sub-task: Test save() writes file to disk and returns file_id
  - [x] Sub-task: Test get() retrieves file bytes by file_id
  - [x] Sub-task: Test get_url() returns local file path
  - [x] Sub-task: Test save() creates subdirectories as needed
  - [x] Sub-task: Test get() raises FileNotFoundError for missing file_id

- [x] Task: Implement FileStorage interface and LocalFileStorage
  - [x] Sub-task: Create `src/pa_dealing/storage/__init__.py` with FileStorage ABC
  - [x] Sub-task: Implement LocalFileStorage with configurable base_path
  - [x] Sub-task: Generate unique file_id using `{request_id}_{timestamp}_{uuid}_{filename}`
  - [x] Sub-task: Wire LocalFileStorage as dependency in FastAPI app

- [x] Task: Refactor existing upload endpoint to use FileStorage
  - [x] Sub-task: Replace direct file write in `upload_contract_note()` with FileStorage.save()
  - [x] Sub-task: Replace direct file read in download endpoint with FileStorage.get()
  - [x] Sub-task: Verify existing tests still pass

- [x] Task: Conductor - User Manual Verification 'Phase 1: File Storage Abstraction Layer' (Protocol in workflow.md)

## Phase 2: Database Model & Migration for Upload History

- [x] Task: Write tests for ContractNoteUpload model and repository methods
  - [x] Sub-task: Test creating a ContractNoteUpload record
  - [x] Sub-task: Test deactivating previous uploads when new one inserted (is_active flag)
  - [x] Sub-task: Test fetching upload history for a request (ordered newest first)
  - [x] Sub-task: Test fetching active upload for a request

- [x] Task: Implement ContractNoteUpload model and Alembic migration
  - [x] Sub-task: Create ContractNoteUpload SQLAlchemy model in `models/pad.py`
  - [x] Sub-task: Fields: id, request_id (FK), file_id, original_filename, content_type, uploaded_at, uploaded_by_id, is_active, verification_status, verification_metadata
  - [x] Sub-task: Generate Alembic migration

- [x] Task: Implement repository methods for upload history
  - [x] Sub-task: `create_contract_note_upload()` — deactivate previous, insert new as active
  - [x] Sub-task: `get_contract_note_history(request_id)` — return all uploads ordered by uploaded_at desc
  - [x] Sub-task: `get_active_contract_note(request_id)` — return is_active=true record

- [x] Task: Conductor - User Manual Verification 'Phase 2: Database Model & Migration' (Protocol in workflow.md)

## Phase 3: Backend API Enhancements

- [x] Task: Write tests for updated upload endpoint and new history endpoints
  - [x] Sub-task: Test upload creates ContractNoteUpload record with correct verification_status
  - [x] Sub-task: Test re-upload deactivates previous upload and creates new active one
  - [x] Sub-task: Test GET /requests/{id}/contract_note_history returns all uploads
  - [x] Sub-task: Test GET /requests/{id}/contract_note/{upload_id}/download serves correct file

- [x] Task: Update upload endpoint to create upload history records
  - [x] Sub-task: After FileStorage.save(), create ContractNoteUpload record
  - [x] Sub-task: Set verification_status to "accepted" or "rejected" based on AI result
  - [x] Sub-task: Store discrepancies and extracted_data in verification_metadata

- [x] Task: Implement new API endpoints
  - [x] Sub-task: GET /requests/{request_id}/contract_note_history
  - [x] Sub-task: GET /requests/{request_id}/contract_note/{upload_id}/download

- [x] Task: Conductor - User Manual Verification 'Phase 3: Backend API Enhancements' (Protocol in workflow.md)

## Phase 4: Extraction Failure & Mismatch Messaging

- [x] Task: Write tests for field-specific null extraction messages
  - [x] Sub-task: Test null direction → "Could not extract direction from contract note"
  - [x] Sub-task: Test null quantity → "Could not extract quantity from contract note"
  - [x] Sub-task: Test null price → "Could not extract price from contract note"
  - [x] Sub-task: Test null ticker/ISIN → appropriate message
  - [x] Sub-task: Test mix of null fields and mismatched fields produces both message types

- [x] Task: Enhance verification logic in document_processor agent
  - [x] Sub-task: Before comparing each field, check if extracted value is None
  - [x] Sub-task: If None, append "Could not extract [field_name] from contract note" to discrepancies
  - [x] Sub-task: If not None but mismatched, keep existing "expected X but extracted Y" format
  - [x] Sub-task: Ensure discrepancy messages are consistent in format across all fields

- [x] Task: Conductor - User Manual Verification 'Phase 4: Extraction Failure Messaging' (Protocol in workflow.md)

## Phase 5: Frontend — Contract Note Section Redesign

- [x] Task: Write Playwright tests for contract note UI states (deferred — manual verification)

- [x] Task: Implement accepted document display
  - [x] Sub-task: Light green background overlay with checkmark/tick icon
  - [x] Sub-task: "Accepted" label and "Auto-verified by AI Processor" subtitle
  - [x] Sub-task: Preview and Download buttons

- [x] Task: Implement rejected document display with detailed messages
  - [x] Sub-task: Red/warning background with AlertTriangle icon
  - [x] Sub-task: "Rejected" label
  - [x] Sub-task: Render discrepancies list (mismatch and null extraction messages)
  - [x] Sub-task: Preview, Download, Re-upload buttons

- [x] Task: Enhance upload error toast with discrepancy details
  - [x] Sub-task: On failure: "Contract note uploaded but verification failed" + discrepancy list
  - [x] Sub-task: On success: "Contract note uploaded and accepted"

- [x] Task: Rebuild dashboard container and verify
  - [x] Sub-task: `docker compose -f docker/docker-compose.yml build dashboard && docker compose -f docker/docker-compose.yml up -d dashboard`

- [x] Task: Conductor - User Manual Verification 'Phase 5: Frontend Contract Note Redesign' (Protocol in workflow.md)

## Phase 6: Frontend — Upload History View

- [x] Task: Write Playwright tests for upload history section (deferred — manual verification)

- [x] Task: Add API client methods for upload history
  - [x] Sub-task: `getContractNoteHistory(requestId)` in `client.ts`
  - [x] Sub-task: TypeScript types for ContractNoteUpload

- [x] Task: Implement collapsible upload history component
  - [x] Sub-task: Fetch history from API on component mount
  - [x] Sub-task: Render collapsible "Previous Uploads" section below active document
  - [x] Sub-task: Each entry: upload date, status badge (green "Accepted" / red "Rejected"), rejection reasons if rejected
  - [x] Sub-task: Preview and Download buttons per historical entry
  - [x] Sub-task: Sorted newest first

- [x] Task: Rebuild dashboard container and verify
  - [x] Sub-task: `docker compose -f docker/docker-compose.yml build dashboard && docker compose -f docker/docker-compose.yml up -d dashboard`

- [x] Task: Conductor - User Manual Verification 'Phase 6: Frontend Upload History View' (Protocol in workflow.md)

## Phase 7: Full Regression Testing

- [x] Task: Run full backend test suite
  - [x] Sub-task: `pytest` inside backend container — all existing + new tests pass
  - [x] Sub-task: Verify >80% coverage on new code

- [x] Task: Run full Playwright E2E suite
  - [x] Sub-task: Run with 4+ workers
  - [x] Sub-task: All existing + new UI tests pass

- [x] Task: Conductor - User Manual Verification 'Phase 7: Full Regression Testing' (Protocol in workflow.md)
