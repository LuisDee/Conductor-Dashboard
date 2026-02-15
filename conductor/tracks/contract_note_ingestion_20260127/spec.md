# Spec: Contract Note Ingestion Pipeline

## Overview
Enhance the Contract Note section of the trade request detail page to support multiple PDF uploads with full history, clear visual hierarchy showing document status (accepted/rejected), detailed compliance alert messaging for mismatches and extraction failures, and an abstracted file storage layer ready for GCS migration.

**Jira Reference**: LDEBURNA-260127-ISHARE-2

## Functional Requirements

### FR1: Abstracted File Storage Layer
- Implement a `FileStorage` interface with `save()`, `get()`, and `get_url()` methods
- Provide `LocalFileStorage` implementation (writes to `/app/uploads` or configurable path)
- Design interface to allow future `GCSFileStorage` implementation (signed URLs)
- All contract note file operations must go through this abstraction

### FR2: Contract Note Upload History (Database)
- Create a `contract_note_uploads` table to track all uploaded documents:
  - `id` (PK), `request_id` (FK), `file_id` (storage reference), `original_filename`, `content_type`
  - `uploaded_at` (timestamp), `uploaded_by_id` (FK to employee)
  - `is_active` (boolean — latest upload = true, previous = false)
  - `verification_status` ("accepted" | "rejected")
  - `verification_metadata` (JSONB — discrepancies, extracted data, confidence)
- When a new document is uploaded: set `is_active=false` on all previous uploads for that request, then insert the new one with `is_active=true`
- `PADExecution` remains unchanged (single row per request, updated on re-upload)
- Alembic migration required

### FR3: Upload Behavior
- **No uploads exist**: Show upload area (dashed border, file input)
- **Documents exist**: Show current (active) document prominently at top, with option to upload a replacement
- User can upload a new contract note at any time (replaces active, preserves history)
- All uploads stored permanently — never deleted

### FR4: Current Document Display (Accepted State)
- When the active document is **accepted** (AI verification passed):
  - Light green background overlay
  - Checkmark/tick icon
  - "Accepted" label
  - Preview and Download buttons
- This replaces the current emerald "Note Verified" display with a clearer accepted state

### FR5: Current Document Display (Rejected State)
- When the active document is **rejected** (AI verification failed):
  - Red/warning background
  - AlertTriangle icon
  - "Rejected" label
  - **Detailed discrepancy messages** shown inline, e.g.:
    - "Direction mismatch: expected BUY but extracted SELL"
    - "Quantity mismatch: expected 1000 but extracted 900"
    - "Price deviation 8.5% exceeds 5% tolerance: expected ~150.00/share, extracted 172.50/share"
  - Preview, Download, and Re-upload buttons

### FR6: Field-Specific Null Extraction Messages
- When AI cannot extract a field from the PDF, generate a specific message:
  - "Could not extract [field_name] from contract note"
  - E.g., "Could not extract quantity from contract note"
  - These are separate from mismatch messages (extraction failure vs value mismatch)
- Apply to all verified fields: direction, quantity, price, ticker/security, ISIN
- Show these messages alongside mismatch discrepancies in the same list

### FR7: Upload History View
- Below the current/active document, show a collapsible "Previous Uploads" section
- Each historical document shows:
  - Upload date/time
  - Status badge: "Accepted" (green) or "Rejected" (red)
  - If rejected: rejection reasons (discrepancies)
  - Click to preview/download any historical PDF
- Sorted newest first

### FR8: Upload Error Toast Enhancement
- When upload completes with verification failure, the toast notification must show:
  - Summary: "Contract note uploaded but verification failed"
  - Specific discrepancies listed (field-level mismatch or extraction failure messages)
- When upload succeeds: "Contract note uploaded and accepted"

### FR9: API Enhancements
- `POST /requests/{request_id}/contract_note` — Updated to use FileStorage, create upload history record
- `GET /requests/{request_id}/contract_note_history` — New endpoint returning all uploads for a request
- `GET /requests/{request_id}/contract_note/{upload_id}/download` — Download a specific historical upload
- Existing download endpoint continues to serve the active document

## Non-Functional Requirements

- Storage abstraction must not break existing upload/download flows
- All new database operations use async SQLAlchemy
- >80% test coverage on new code
- Playwright tests for UI changes
- No changes to Slack notification logic (out of scope per user answer)

## Acceptance Criteria

1. User can upload a PDF contract note and see it displayed with accepted (green + tick) or rejected (red + details) status
2. Uploading a new document preserves the previous one in history
3. All historical uploads are viewable in a collapsible section with status badges
4. Rejected documents show field-specific messages: "expected X but extracted Y" for mismatches, "could not extract [field]" for null fields
5. Upload toast shows specific discrepancy details on failure
6. File storage uses abstracted interface (LocalFileStorage implementation)
7. Existing PADExecution model unchanged; new upload history table tracks all documents

## Out of Scope
- GCS storage implementation (interface only, local implementation)
- Breaches page detail enhancement
- Slack notification content changes
- Manual compliance accept/reject override
- Bulk upload
