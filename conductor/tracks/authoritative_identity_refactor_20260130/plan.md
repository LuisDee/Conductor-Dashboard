# Plan: Authoritative Identity & Sequential Guardrail Refactor

## 1. Problem Definition & Root Cause
The system currently suffers from **Parallel Confusion**, where the decision engine (Scorer) and the reporting engine (Advisory) use different logic, leading to the "102120" auto-approval.

### Core Failures:
1.  **Identity Fragility**: System matches "102120 KS" (request) vs "102120" (DB) and fails due to exact-string mismatch.
2.  **Blind Scoring**: `SimplifiedRiskScorer` is unaware of the Restricted List; returns LOW risk for forbidden trades.
3.  **Veto Failure**: Orchestrator ignores "CRITICAL" advisory labels during the auto-approve state transition.

---

## 2. Goal: The "Unified Guardrail" Architecture
Unify all logic under an **`inst_symbol` anchor**. Implement a **Sequential Pipeline** where policy (Advisory) acts as a physical circuit breaker for status (Approval).

---

## 3. Implementation Phases (TDD Workflow)

### Phase 0: TDD Baseline (Failing Tests)
**Goal:** Create the "Red" state by reproducing the bug.

- [x] **Reproduction Test**: Create `tests/integration/test_identity_bug_reproduction.py` to prove that `102120 KS` currently auto-approves.
- [x] **Unit Test (Lookup)**: Add failing tests for `check_restricted_list_comprehensive` in `tests/unit/test_database_tools.py`.
- [x] **Unit Test (Scorer)**: Create `tests/unit/test_categorical_rules.py` to test the "1 High / 2 Medium" logic in isolation.

### Phase 1: Database & Data Migration
**Goal:** Transition from ambiguous `ticker` to authoritative `inst_symbol`.

- [x] **Alembic Migration**:
    1.  **Add Columns**: Add `inst_symbol` (String 50, Indexed) to `pad_request` and `restricted_security`.
    2.  **Data Migration**: Run a lookup for all existing records to populate `inst_symbol` from `ticker`.
    3.  **Drop Columns**: Remove the old `ticker` column from both tables.
- [x] **SQLAlchemy Models**: Update models in `src/pa_dealing/db/models/compliance.py` and `pad.py`.

### Phase 2: Authoritative Identity Engine (`repository.py`)
**Goal:** Implement the "Search & Enrich" pipeline (T1 -> T2 -> T3).

- [x] **`lookup_instrument_comprehensive`**: 
    *   Search: Bloomberg (T1) -> Mappings (T2) -> Product (T3).
    *   **The Upgrade**: If matched in T2 or T3, re-query T1 (`oracle_bloomberg`) to fetch `isin` and Gold-Standard description.
- [x] **`check_restricted_list_comprehensive`**:
    *   **Input**: `inst_symbol`, `isin`.
    *   **Logic**: Use `func.upper()` and `or_` to check against `restricted_security`.
    *   **Audit Logging (Exact Format)**:
        ```python
        if match:
            logger.warning(
                f"RESTRICTED SECURITY DETECTED | "
                f"matched_entry={match.inst_symbol} | "
                f"identifiers_checked={[inst_symbol, isin]} | "
                f"reason={match.reason}"
            )
        ```
    *   **Verify**: Ensure Phase 0 unit tests now pass (Green).

### Phase 3: Categorical Scorer Enhancement (`risk_scoring.py`)
**Goal:** Align Scorer with Rulebook Section 1 ("Any HIGH = HIGH").

- [x] **Restricted Factor**: Add `assess_restricted_list(is_restricted: bool)` to the `SimplifiedRiskScorer`.
- [x] **Categorical Aggregation**:
    *   `if ANY factor == HIGH` -> Overall **HIGH** (Routing: SMF16).
    *   `if 2+ factors == MEDIUM` -> Overall **MEDIUM** (Routing: Compliance).
    *   `else` -> **LOW** (Eligible for Auto-Approve).
- [x] **Explicit Veto Documentation**: Add a comment block mapping HIGH factors to specific routing requirements (e.g., "HIGH Restricted -> Route to SMF16 via Categorical Scorer") to keep the "Why" in the code.
- [x] **Logic Cleanup**: Completely remove numeric scoring math.
- [x] **Verify**: Ensure Phase 0 scorer tests now pass (Green).

### Phase 4: Orchestrator Pipeline Refactor (`agent.py`)
**Goal:** Implement the "Sequential Guardrail" flow.

- [x] **`process_pad_request` rewrite**:
    1.  **Resolve Identity**: One-time call to `lookup_instrument_comprehensive`.
    2.  **Check Restricted**: One-time call to `check_restricted_list_comprehensive`.
    3.  **Run Advisory (Passive)**: Pass `is_restricted` to `detect_advisory_criteria`.
    4.  **Run Scorer (Passive)**: Pass `is_restricted` to `score_pad_request`.
    5.  **The Veto Gate**:
        ```python
        if advisory.should_advise_reject:
            risk["auto_approve_eligible"] = False
            risk["status"] = "pending_compliance"
        ```
- [x] **Audit Trail Enrichment**: Ensure the audit log records which specific advisory criterion triggered the veto (e.g., `details={"veto_reason": "restricted_list_match", "advisory_severity": "CRITICAL"}`).
- [x] **Verify**: Ensure Phase 0 reproduction test now passes (Status = PENDING COMPLIANCE).

### Phase 5: Cleanup & Deprecation
- [x] **Test Migration**: Update `test_orchestrator.py` and `test_mar_compliance.py` to use the new categorical system.
- [x] **Delete Legacy Code**: Remove `src/pa_dealing/agents/orchestrator/risk_classifier.py`.

### Phase 6: Frontend & Reporting
- [x] **UI Display**: Update `MyRequests.tsx`. 
    *   If `isin` exists: Show ISIN.
    *   Else: Show "Not Available for this product" (italicized gray).
    *   Show `inst_symbol` as "Internal Ref".
- [x] **Search Optimization**: Update the search bar in `MyRequests.tsx` to explicitly allow searching by ISIN.

### Phase 7: Missing UI Columns & Data Exposure
**Goal:** Ensure ISIN and Authoritative Identity are visible across ALL dashboard views.

- [x] **UI Update - MyRequests**: Add dedicated `ISIN` column (or ensure visibility).
- [x] **UI Update - PendingApprovals**: Add `ISIN` and `Internal Ref` to the Instrument column.
- [x] **UI Update - HoldingPeriods**: Add `ISIN` and `Internal Ref` to the Instrument column.
- [x] **Backend Update**: Ensure `get_holding_periods` endpoint returns `isin` and `inst_symbol` in its response.
- [x] **Type Definition**: Update `HoldingPeriod` interface in `dashboard/src/types/index.ts` to include `isin` and `inst_symbol`.

### Phase 8: UX Refinements (Status & ISIN)
- [x] **ISIN Display**: Update `MyRequests.tsx`, `PendingApprovals.tsx`, `HoldingPeriods.tsx` to show "ISIN: Not Available For This Product".
- [x] **Status Badge Logic**: Update `MyRequests.tsx` status column.
    - Consolidate pending states to "Awaiting Approval".
    - Add "Awaiting Execution" sub-badge for approved/auto_approved states.
    - Ensure styling matches existing yellow/orange warning style.

---

## 4. Why this Solves the Issue
1.  **No Matching Bug**: `102120 KS` is resolved to `102120` at the very first step.
2.  **Single Source of Truth**: The restricted lookup happens once, and its result is final.
3.  **Rule Enforcement**: Policy violations (Advisory) now have explicit **Veto Power** over state transitions.
4.  **Data Maturity**: Standardizes on `inst_symbol` across the entire ecosystem.