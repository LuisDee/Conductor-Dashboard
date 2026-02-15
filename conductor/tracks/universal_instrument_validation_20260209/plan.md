# Implementation Plan: Universal Instrument Validation & Interface Sync ✅ COMPLETE

## Phase 1: The Shared Core & Routing Refined ✅ COMPLETE
**Goal:** Extract silod logic into a reusable service and ensure correct escalation gating.

1.  **Develop `ValidationService`** (`src/pa_dealing/services/validation_service.py`):
    *   ✅ Migrate `assess_justification_quality` logic from `chatbot.py`.
    *   ✅ Implement `check_identifier_presence` (the One-of-Four check).
    *   ✅ Add `get_instructional_hint(draft_state)` to unify guidance strings.
2.  **Refine Escalation Logic**:
    *   ✅ **Risk Scorer:** Modify `SimplifiedRiskScorer.aggregate_risk` to route `HIGH` risk to `ApprovalRoute.COMPLIANCE` (automated SMF16 bypass).
    *   ✅ **Repository:** Update `update_pad_status` to remove the automated `requires_smf16` status transition.
3.  **Update Pydantic Models**:
    *   ✅ Integrate `ValidationService` into `SubmitPADRequest` using `@model_validator(mode='after')`.
    *   ✅ Ensure any submission without an identifier fails with a `400 Bad Request`.
4.  **Audit Migration**:
    *   ✅ Ensure `isin` and `sedol` fields are consistently handled in `PADRequest` and `InstrumentInfo` schemas.

## Phase 2: Chatbot "Hardening" & Extraction ✅ COMPLETE
**Goal:** Enhance the LLM's ability to capture high-quality data.

1.  **Pydantic Extraction Tool**:
    *   ✅ Update `set_security` tool signature to include `isin: str | None`, `sedol: str | None`, etc.
    *   ✅ Update the Chatbot's `SYSTEM_PROMPT` with "Extraction Hierarchy" instructions:
        *   "Always look for 12-char strings for ISIN."
        *   "Look for 'Bloomberg' or 'Ticker' labels."
2.  **State Machine Refactor**:
    *   ✅ Update `DraftRequest` in `session.py` to store all four identifiers.
    *   ✅ Modify `_get_missing_fields` to implement the "One-of-Four" requirement.
    *   ✅ Update `confirm_selection` to copy all identifiers from the `selected_candidate` to the `DraftRequest`.
3.  **Conversational "Clarification" Turn**:
    *   ✅ Implement a new instructional hint: `IDENTIFIER_REQUIRED`.
    *   ✅ Update `chatbot.py` to recognize when a user provides a name but no identifier is resolved.

## Phase 3: Web UI Alignment & Manual Escalation ✅ COMPLETE
**Goal:** Bring the Web UI up to the Chatbot's compliance standard and add manual escalation.

1.  **Escalation Endpoint**:
    *   ✅ Add `POST /api/requests/{request_id}/escalate` in `src/pa_dealing/api/routes/requests.py`.
    *   ✅ This endpoint will manually move a request from `pending_compliance` to `pending_smf16`.
2.  **Frontend Update (NewRequest.tsx)**:
    *   ✅ Expose `POST /api/requests/validate` for real-time justification coaching.
    *   ✅ Disable "Submit" if the "One-of-Four" rule is not met.
3.  **Frontend Update (Approvals)**:
    *   ✅ **PendingApprovals.tsx**: Add "Escalate to SMF16" button to the actions column for compliance users.
    *   ✅ **RequestDetail.tsx**: Add "Escalate to SMF16" action button if status is `pending_compliance`.

## Phase 4: Verification & Testing ✅ COMPLETE
1.  **Unit Tests**:
    *   ✅ Test `ValidationService` identifier checks.
    *   ✅ Test `SimplifiedRiskScorer` to verify `HIGH` risk now routes to `COMPLIANCE`.
2.  **Integration Tests**:
    *   ✅ Test the new `/escalate` API endpoint.
    *   ✅ Test Slack outbox queuing for manual escalation messages.
3.  **E2E Tests (Slack)**:
    *   ✅ Verify the "One-of-Four" follow-up prompt when no identifier is extracted.

## Phase 5: Documentation & Hygiene ✅ COMPLETE
1.  **Gemini Skill**: ✅ Update `instrument-lookup` and `slack-outbox` skills.
2.  **Developer Guide**: ✅ Document the new manual escalation workflow.
