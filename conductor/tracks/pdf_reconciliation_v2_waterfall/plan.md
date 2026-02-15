# Implementation Plan: PDF Reconciliation Waterfall Matrix

## Phase 1: Backend Logic Fixes
- [ ] Refactor `UserMatcher.match_by_email` to fetch all candidates and apply gates.
- [ ] Remove `await` crash from `TradeDocumentProcessor`.
- [ ] Deprecate `verify_trade` and integrate its logic into the new gated matcher.
- [ ] Add "Total Value" check (`proceeds` vs `estimated_value`) to the economics gate.

## Phase 2: API & Schema Updates
- [ ] Add `linked_request_id` to `PDFHistoryItem` schema.
- [ ] Implement `match_status` filtering in `list_pdf_history` (join with ParsedTrade).
- [ ] Add `match_matrix` to API response for detailed UI explanation.

## Phase 3: Dashboard UI Overhaul
- [ ] Update `PDFHistory.tsx` to use the new `match_status` filter.
- [ ] Move "Raw Extracted Data" to the top of the details view.
- [ ] Create "Match Matrix" visualization in the sidebar/modal.
- [ ] Link "Matched" badge directly to the request page.

## Phase 4: Data Cleanup & Verification
- [ ] Run cleanup script to unlink bad matches (e.g. META -> WMI).
- [ ] Verify with test emails (Activity Statement with multiple trades).
