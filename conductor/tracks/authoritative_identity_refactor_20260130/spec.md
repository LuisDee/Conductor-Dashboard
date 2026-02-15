# Specification: Authoritative Identity & Sequential Guardrail Refactor

## 1. Background
A critical bug occurred where a restricted instrument (`102120`) was auto-approved. 
The system failed because the user request used the Bloomberg ID `"102120 KS"`, while the restricted list contained the raw ticker `"102120"`. 
The Risk Classifier performed a shallow exact-match lookup on only one identifier and concluded the trade was LOW risk and eligible for auto-approval, ignoring the Advisory System's later warning.

## 2. Technical Goal
Unify the system's identity resolution and risk assessment logic. 
Transition from an ambiguous `ticker`-based model to an **`inst_symbol` anchor** (internal) and **`isin`** (regulatory) model. 
Implement a **Sequential Pipeline** where policy violations discovered by the Advisory System act as a physical circuit breaker for status transitions.

## 3. Scope of Changes

### 3.1 Identity Resolution (`inst_symbol`)
- Implement a tiered "Search & Enrich" logic:
    - **Tier 1**: `oracle_bloomberg`
    - **Tier 2**: `map_inst_symbol`
    - **Tier 3**: `product`
- Every request must be resolved to an `inst_symbol`.
- **Upgrade Step**: If found in T2 or T3, the system must re-lookup the `inst_symbol` in T1 to pull the `isin` and full description.

### 3.2 Database Schema
- **`pad_request`**: 
    - Add `inst_symbol` (indexed).
    - Drop `ticker` column (ambiguous).
- **`restricted_security`**:
    - Add `inst_symbol` (indexed).
    - Match on `inst_symbol` or `isin`.
    - Drop `ticker` column.

### 3.3 Risk & Advisory Logic
- **Categorical Scorer**: 
    - Implement "Any HIGH = HIGH" rule.
    - Restricted List match = HIGH Factor.
    - 2+ MEDIUM Factors = Overall MEDIUM.
- **Sequential Flow**:
    1. Resolve Identity -> Enriched Metadata.
    2. Check Restricted List (Single Source of Truth).
    3. Run Advisory (Passive).
    4. Run Scorer (Categorical).
- **The Veto Gate**:
    - If `AdvisoryResult.should_advise_reject` is `True`, the Orchestrator MUST force the status to `pending_compliance` and block auto-approval.

### 3.4 Cleanup
- Fully deprecate and delete `src/pa_dealing/agents/orchestrator/risk_classifier.py` (legacy numeric math).
- Update all unit and integration tests to use the categorical scorer.

## 4. Constraints & Requirements
- **Passive Advisory**: The Advisory System must not query the DB; it receives flags from the Orchestrator.
- **Auditability**: Rejections must log the exact identifiers (`inst_symbol`, `isin`) that triggered the match.
- **Case Sensitivity**: All database lookups must use `func.upper()` for identifier matching.
- **No Data Loss**: The Alembic migration must migrate existing `ticker` data to `inst_symbol` before dropping columns.

## 5. Success Criteria
- Request for `102120 KS` correctly resolves to `inst_symbol="102120"`.
- Restricted lookup finds the match via `inst_symbol`.
- Scorer sets level to **HIGH**.
- Status is **PENDING COMPLIANCE** (not auto-approved).
- Dashboard shows ISIN or "Not Available".

## 6. Frontend Consistency (Added 2026-02-01)
- **Data Exposure**: All dashboard endpoints (`get_pending_approvals`, `get_holding_periods`, etc.) must return `isin` and `inst_symbol`.
- **Visibility**: All main dashboard views ("My Requests", "Pending Approvals", "Holding Period Calendar") must explicitly display the ISIN and Internal Reference for every line item.

## 7. UX Refinements (Added 2026-02-01)
- **ISIN Missing Text**: Explicitly state "ISIN: Not Available For This Product" instead of just "Not Available".
- **Status Subtext**:
    - Pending states: Show "Awaiting Approval" (unified).
    - Approved states: Show "Awaiting Execution" with amber/warning styling (to prompt user action).
