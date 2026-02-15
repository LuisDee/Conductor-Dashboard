# Technical Specification: Universal Instrument Validation & Interface Sync

## 1. Executive Summary
This specification defines the unification of the Personal Account Dealing (PAD) submission pipeline across both the Slack Chatbot and the Web UI. It addresses the current discrepancy where validation logic is siloed, leading to "logic drift." By centralizing compliance heuristics and enforcing a strict "One-of-Four" identifier constraint, we ensure high-quality data for risk scoring and regulatory reporting. Additionally, it refines the escalation workflow to ensure all high-risk requests are first reviewed by Compliance before manual escalation to SMF16.

## 2. Problem Statement
*   **Inconsistent Validation:** The Chatbot provides "justification coaching" and interactive disambiguation that the Web UI lacks.
*   **Data Quality Gaps:** Requests are currently submitted with ambiguous descriptions (Outcome 4) without forcing standard identifiers, making automated restricted list and conflict checks unreliable.
*   **Automated Escalation Risk:** Automated rules currently raise directly to SMF16, bypassing the Compliance team's initial review gate.
*   **Architectural Drift:** Changes to compliance policy currently require updates in two distinct codebases (Chatbot vs. Web API).

## 3. The "One-of-Four" Identifier Constraint
To satisfy MiFID II/SEC audit standards and ensure high-confidence risk scoring, every submission **MUST** contain at least one of the following "High-Quality Identifiers":

1.  **ISIN** (International Securities Identification Number) - 12 chars, e.g., GB0002374006.
2.  **SEDOL** (Stock Exchange Daily Official List) - 7 chars, e.g., 0237400.
3.  **Bloomberg Ticker/Code** - e.g., TSLA US Equity.
4.  **Exchange Ticker** - e.g., TSLA.

### 3.1 The "Identity Upgrade" Protocol
The system will attempt to "upgrade" user-provided identifiers using the existing 3-tier lookup:
*   If a user provides a **Ticker**, the system attempts to resolve it to a database record and automatically attaches the **ISIN** and **SEDOL**.
*   The constraint is only considered "failed" if the resolution returns **Outcome 4 (Unverified)** AND no manual identifier was provided.

## 4. Architectural Solution: Headless Compliance Service
We will implement a centralized `ValidationService` to act as the "Single Source of Truth" (SSoT) for both interfaces.

### 4.1 `ValidationService` API
Located at `src/pa_dealing/services/validation_service.py`:

*   `validate_identifiers(isin, sedol, bloomberg, ticker) -> ValidationResult`:
    *   Enforces the "One-of-Four" rule.
    *   Returns boolean and a user-facing guidance message if missing.
*   `assess_justification_quality(text) -> JustificationGrade`:
    *   Grades text as `GOOD` or `WEAK` based on length and keyword density (e.g., "rebalancing", "pension", "bonus").
*   `detect_instrument_type_risk(inst_type, is_derivative, is_leveraged) -> RiskProfile`:
    *   Standardizes the mapping of instrument types to compliance risks (e.g., Options = High Risk).

## 5. Interface Synchronization Logic

### 5.1 Chatbot Implementation (Conversational Gate)
*   **Extraction:** LLM will use structured Pydantic output to extract identifiers from the very first message.
*   **Proactive Prompting:** If a search results in "Outcome 4", the chatbot will shift from fuzzy search to a **Specific Identifier Request** state:
    > *"I've found no record of '[Name]'. To proceed, I need one of the following identifiers to verify this is not restricted: ISIN, SEDOL, Bloomberg, or Ticker."*
*   **Coaching:** If justification is `WEAK`, the chatbot delivers the "soft-gate" re-prompt before the `show_preview` tool is enabled.

### 5.2 Web UI Implementation (Synchronized API)
*   **Real-time Feedback:** A new `/api/requests/validate` endpoint will allow the Web UI to show a "Justification Strength" meter and "Identifier Required" warnings *before* the user clicks Submit.
*   **Disambiguation:** The Web UI will adopt the Chatbot's disambiguation logic—if a ticker is ambiguous, the UI must show a selection list instead of defaulting to the first match.

## 6. Compliance & Escalation Protocols
*   **Outcome 1 (Exact Internal Match):** Eligible for Auto-Approval.
*   **Outcome 2 (External Match):** Eligible for Auto-Approval with "New Security" flag.
*   **Outcome 3 (Internal Partial Match):** Requires Manager Approval.
*   **Outcome 4 (Unverified):** **BLOCKED** from Auto-Approval. Requires a "Manual Review Justification" and is routed directly to **Compliance**.
*   **Manual Escalation Gate:** Nothing raises directly to SMF16. All HIGH risk or Outcome 4 requests route to `pending_compliance`. Only a Compliance Officer can manually trigger escalation to **SMF16 (Compliance Head)** via Slack or Dashboard.
*   **Audit Trail:** Every request must log the `resolution_outcome`, the `external_provider`, and whether the user bypassed a "Weak Justification" warning.

## 7. Industry Best Practices (MiFID II Alignment)
*   **Layered Validation:** Frontend (UX feedback) -> Backend Schema (Integrity) -> Service Layer (Compliance Policy).
*   **Non-Repudiation:** The audit trail captures the exact identifier used for the restricted list check at the moment of submission.
*   **SSoT:** All validation heuristics are versioned in one Python module, preventing "Policy Drift" between the Slack app and the Dashboard.
