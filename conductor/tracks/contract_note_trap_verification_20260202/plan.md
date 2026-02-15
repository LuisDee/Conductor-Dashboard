# Implementation Plan: Contract Note Trap Verification

## Phase 1: Infrastructure & Schema (Foundations)
- [x] **Asset Setup:**
    - [x] Create `tests/assets/contract_notes/` (if missing).
    - [x] Move `case*.pdf` files to assets directory.
- [x] **Database Schema:**
    - [x] Create migration:
        - [x] Add `dedup_key` and `account_type` to `Trade` (or create new `trade` table).
        - [x] Create `trade_document_link` table.
        - [x] Add `is_cancelled` field.
- [x] **Pydantic Schemas (src/pa_dealing/agents/document_processor/schemas.py):**
    - [x] Update `ExtractedTradeData`:
        - [x] Add `is_cancelled: bool = False`.
        - [x] Add `account_type: Literal["Individual", "Trust", "Corporate", "Other"]`.
        - [x] Add `fingerprint` property for deduplication.
    - [x] Ensure **Instructor** validation rules are present for GBX and ISO dates.

## Phase 2: Logic & Prompt Hardening (The Core)
- [x] **Prompt Engineering (src/pa_dealing/agents/document_processor/prompts.py):**
    - [x] Update `FINANCIAL_EXTRACTION_RULES`:
        - [x] Standardize on **GBX** for pence.
        - [x] Date format inference rule (Broker context).
        - [x] Cancellation detection rule.
        - [x] Account type detection (Trust vs Individual).
- [x] **Poller Hardening (src/pa_dealing/services/pdf_poller.py):**
    - [x] Refactor `_process_pdf` to ensure `GCSDocument` is committed **before** extraction starts.
    - [x] Implement robust error handling so extraction failure doesn't delete the document record.
- [x] **Routing & Matching (src/pa_dealing/services/extraction_router.py):**
    - [x] Implement "Option 3" for User Matching:
        - [x] If `match_score < 0.8` → `MANUAL_REVIEW`.
        - [x] If `extracted.account_type != "Individual"` (and user is individual) → `MANUAL_REVIEW`.

## Phase 3: Integration & Trap Verification (Testing)
- [x] **Test Case 1 (Holdings):** Verify classification as `OTHER`/`ACCOUNT_STATEMENT`.
- [x] **Test Case 2 (Entity):** Verify `MANUAL_REVIEW` on Trust account mismatch.
- [x] **Test Case 3 (GBX):** Verify 154.2 GBX -> 1.542 GBP normalization.
- [x] **Test Case 4 (Date):** Verify Broker-context date parsing.
- [x] **Test Case 5 (Cancellation):** Verify `is_cancelled=True`.
- [x] **Test Deduplication:** Verify 1 Trade / 2 Links scenario.

## Phase 4: Finalization & Docs
- [x] **Skill Creation:** Create `gemini-agent-pdf-processor` skill.
- [x] **Tooling Docs:** Update `docs/tooling/instructor.md` and `docs/tooling/extraction-router.md` with new logic.
- [x] **Migration:** Backfill logic for existing data.
