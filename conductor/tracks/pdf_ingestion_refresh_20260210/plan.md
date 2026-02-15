# PDF Ingestion Refresh & Fixes

**Goal:** Enhance the PDF Ingestion feature with ad-hoc polling, better logging, classification visibility, and accuracy metrics fixes.

## 1. Shared Ingestion Service Refactor
- [x] **Extract Logic:** Refactor `scripts/ops/run_graph_email_poller.py` to move the core polling/processing loop into a reusable service function in `src/pa_dealing/services/email_ingestion_service.py` (e.g., `run_ingestion_cycle()`).
- [x] **Update Script:** Update `run_graph_email_poller.py` to import and use this service function. (Verified: `GraphEmailPoller.poll_cycle` already serves this purpose).

## 2. Ad-Hoc Poll Trigger ("Refresh Now")
- [x] **Backend Endpoint:** Create `POST /api/pdf/poll` that triggers `run_ingestion_cycle()` as a `BackgroundTasks` job. (Implemented as `POST /api/pdf-history/refresh`).
- [x] **Frontend Button:** Add "Refresh Now" button to the PDF History page (top right).
- [x] **Loading State:** Show a spinner/toast while the request is acknowledged (actual polling is async).

## 3. Activity Logging
- [x] **Verify Logging:** Check `run_ingestion_cycle` to ensure it calls `audit.log()` for every new email/document found. (Added `log_email_discovery` to `AuditLogger`).
- [x] **Enhance Log:** Add details: Email Subject, Sender, Attachment Name.

## 4. Success Rate Bug
- [x] **Investigate:** Analyze `get_ingestion_stats` logic in `src/pa_dealing/services/pdf_ingestion_service.py` (or where stats are calculated).
- [x] **Fix:** Ensure `success_rate = (processed / total) * 100` and cap at 100% if data is inconsistent, or fix the underlying counter increment logic (double counting?). (Fixed double-counting in `pdf_history.py`).

## 5. Document Classification Visibility
- [x] **API Update:** Add `classification` field to `PDFHistoryItem` response model. (Added `document_type`).
- [x] **Backend Update:** Ensure the ingestion process saves the classification (Contract Note vs Activity Statement) to the database (`contract_note` table or `pdf_ingestion_history`). (Stored in `raw_extracted_data`).
- [x] **UI Update:** Display the classification chip on the PDF History table and Detail page.
