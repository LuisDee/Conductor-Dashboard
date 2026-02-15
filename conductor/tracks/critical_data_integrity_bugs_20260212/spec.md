# Spec: Critical Data Integrity Bug Fixes

## Problem Statement
The autopsy review identified 8 logic bugs causing data corruption, incorrect compliance decisions, and broken audit trails. All have been verified against the actual codebase.

## Source
- `.autopsy/REVIEW_REPORT.md` - Findings #3, #4, #5, #6 + HIGH patterns
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "HIGH: HIGH Risk Routes to COMPLIANCE Instead of SMF16"

## Findings (All Verified Against Code)

### 1. Double Base64-Decode Corrupts PDFs (CRITICAL)
- **File:** `src/pa_dealing/services/graph_email_poller.py` line 353
- **Issue:** `base64.b64decode(attachment.content_bytes)` but Graph SDK `AttachmentInfo.content_bytes` is already `bytes`, not base64 string.
- **Impact:** Every PDF processed via email polling is corrupted.
- **Fix:** `pdf_bytes = attachment.content_bytes` (remove decode)

### 2. Breach Auto-Resolution Never Works (CRITICAL)
- **File:** `src/pa_dealing/services/pad_service.py` line 622
- **Issue:** `not PADBreach.resolved` uses Python `not` on SQLAlchemy Column. Always evaluates to `False`, matching 0 rows.
- **Impact:** Contract note mismatch breaches never auto-resolve, creating compliance liability.
- **Fix:** Replace with `not_(PADBreach.resolved)` (already imported)

### 3. FX Rate Fallback to 1.0 (CRITICAL)
- **File:** `src/pa_dealing/services/currency_service.py` lines 56-67
- **Issue:** Missing FX rate silently converts at 1:1 (100 JPY becomes 100 GBP).
- **Impact:** Risk scoring uses 150x overstated values, incorrect compliance decisions.
- **Fix:** Raise `CurrencyConversionError` instead of fallback

### 4. Approval Routing Logic vs Docstrings Mismatch (RECLASSIFIED - INVESTIGATION)
- **File:** `src/pa_dealing/agents/orchestrator/risk_scoring.py` line 694
- **Issue:** Code sets `approval_route = ApprovalRoute.COMPLIANCE` for HIGH risk. Docstrings (lines 15, 680) say HIGH -> SMF16. Neither may be correct.
- **Expected business logic (confirmed 2026-02-12):**
  - The approval chain is **cumulative**, not a single destination:
    - LOW → auto-approve
    - MEDIUM → manager approval
    - HIGH → manager → compliance
    - (future) CRITICAL/SMF16 → manager → compliance → SMF16
  - SMF16 is only reached when compliance manually escalates, not via automated routing.
  - All routing tiers should be **configurable via the rules engine**.
- **Current code gap:** The code sets a single `ApprovalRoute` enum per risk level. It does not model a multi-step approval chain. It is unclear whether the existing workflow already implements manager-then-compliance sequencing elsewhere.
- **Fix:** When this phase is picked up:
  1. Investigate whether a multi-step approval chain already exists in the workflow (check `pad_service.py`, orchestrator agent, and Slack approval handlers)
  2. If it exists, verify the routing enum maps to the correct chain
  3. If it doesn't exist, this becomes a workflow refactor (separate track)
  4. Update docstrings to match the confirmed business logic above
  5. Make routing tiers configurable via the rules engine

### 5. recover_orphaned_documents Ignores Timeout (HIGH)
- **File:** `src/pa_dealing/services/pdf_poller.py` line 463
- **Issue:** `cutoff = datetime.now(UTC).replace(tzinfo=None)` uses current time instead of `now - timeout_minutes`. Parameter is ignored.
- **Impact:** Active document processing gets interrupted, causing duplicate trade extractions.
- **Fix:** `cutoff = datetime.now(UTC).replace(tzinfo=None) - timedelta(minutes=timeout_minutes)`

### 6. Range Slider Invalid Threshold Ordering (CRITICAL)
- **File:** `dashboard/src/components/ui/DualRangeSlider.tsx` line 96
- **Issue:** Low handle capped at `maxValue - step` (slider max) not the current high handle position. Low can exceed high.
- **Impact:** Risk thresholds become inverted (trades below 100k HIGH, above 900k MEDIUM).
- **Fix:** Validate `low <= high` constraint during drag

### 7. Wrong ActionType for Audit Entries (HIGH)
- **File:** `src/pa_dealing/services/pad_service.py` lines 1319, 1413
- **Issue:** Breach creation and resolution both use `ActionType.PAD_REQUEST_VIEWED`. Correct types `BREACH_DETECTED` and `BREACH_RESOLVED` exist in the enum.
- **Impact:** Breach lifecycle events invisible in audit log when filtering by type.
- **Fix:** Use `ActionType.BREACH_DETECTED` (line 1319) and `ActionType.BREACH_RESOLVED` (line 1413)

### 8. Boolean HTML Selects Produce Strings (HIGH)
- **File:** `dashboard/src/pages/NewRequest.tsx` lines 194-198
- **Issue:** HTML `<select>` produces strings "true"/"false", not booleans. Code uses `String(data.isDerivative) === 'true'` workaround. Insider info field at line 198 has inverted semantics.
- **Impact:** Type system says boolean but runtime is string. Insider info compliance field semantically inverted.
- **Fix:** Use React Hook Form Controller with value transformers

## Acceptance Criteria
- [ ] PDFs processed via email polling are not corrupted
- [ ] Breach auto-resolution resolves matching breaches
- [ ] Missing FX rates raise explicit errors (no silent fallback)
- [ ] HIGH risk routes to correct approval tier (aligned with spec)
- [ ] Orphan recovery respects timeout_minutes parameter
- [ ] Range slider enforces low <= high constraint
- [ ] Breach audit entries use correct ActionTypes
- [ ] Boolean selects produce actual booleans
- [ ] All existing tests pass + new tests for each fix
