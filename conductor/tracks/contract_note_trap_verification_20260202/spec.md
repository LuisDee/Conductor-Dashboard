# Specification: Contract Note Trap Verification & Robustness Hardening

## Goal
Harden the PDF extraction pipeline to be broker-agnostic, robust against known financial "traps", and capable of handling duplicate documents via a many-to-many linkage architecture. Utilizing **Instructor** for structured extraction and self-healing validation.

## Core Requirements

### 1. Hardened LLM Extraction (Broker-Agnostic via Instructor)
-   **Approach:** Use intelligence over configuration. Prompts must handle date formats (US vs UK) and currency normalization dynamically.
-   **Standard:** Use **GBX** (pence) as the standard market data code for UK stocks.
-   **Constraint:** No per-broker hardcoded logic in Python. All variations must be handled via prompt engineering and Pydantic validators using the **Instructor** library.

### 2. Document Classification & Trade Detection
-   **Classifications:**
    -   `CONTRACT_NOTE`: Single trade confirmation.
    -   `ACTIVITY_STATEMENT`: Periodic statement (with "Trades" section).
    -   `ACCOUNT_STATEMENT`: Periodic statement (Positions only, NO "Trades" section).
    -   `OTHER`: Non-trade documents (holdings mirrors, tax forms).
-   **Logic:** If `ACTIVITY_STATEMENT` but `has_trades_section=False`, reclassify/flag as `ACCOUNT_STATEMENT` and skip trade extraction.

### 3. "Ingestion First" Robustness
-   **Process Flow:** **1. Store document record FIRST (always) → 2. Commit/Flush → 3. Extract trades.**
-   **Requirement:** Document logging must never fail even if extraction fails. If extraction fails, the document record should remain in the database with a `status='failed'` or `pending_extraction`, rather than being rolled back.

### 4. Trade-Level Deduplication
-   **Solution:** Unique constraint on a composite key in a new `trade` table.
-   **Key Definition:**
    -   Primary: `broker_ref` (if available).
    -   Fallback: `fingerprint(broker, account, date, symbol, direction, quantity, price)`.
-   **Behavior:** `ON CONFLICT DO NOTHING` (or link to existing record).

### 5. Document-Trade Linkage (Many-to-Many)
-   **Architecture:**
    -   `GCSDocument`: Registry of ALL ingested files.
    -   `Trade`: Canonical list of unique trades.
    -   `TradeDocumentLink`: Junction table linking documents to trades.
-   **Flow:**
    1.  Ingest PDF.
    2.  Extract Trades.
    3.  For each trade:
        -   If NEW: Insert `Trade`, Insert `TradeDocumentLink(is_primary=True)`.
        -   If EXISTS: Insert `TradeDocumentLink(is_primary=False)` (link only).

### 6. Trap Verification (The 5 Cases)
Robustness against specific edge cases:
-   **Case 1 (Holdings Mirror):** Classify as `OTHER` or `ACCOUNT_STATEMENT` (no trades).
-   **Case 2 (Entity Ambiguity):** 
    -   **Detection:** Extract `entity_type` (Individual vs Trust).
    -   **Routing:** Flag for `MANUAL_REVIEW` if:
        1.  User matching confidence score < 0.8.
        2.  **OR** Extracted entity type (e.g., Trust) ≠ registered user type (Individual).
-   **Case 3 (GBX Trap):** 154.2 GBX -> 1.542 GBP.
-   **Case 4 (Date Ambiguity):** 01/02/2026 -> Jan 2nd (US broker) vs Feb 1st (UK broker).
-   **Case 5 (Cancellation):** Detect `is_cancelled=True` and flag for manual review.

## Architecture & Schema

### Database Changes
-   **New Tables:** `trade`, `trade_document_link`.
-   **Migration:** Migrate data from `parsed_trade` to new structure.

### Prompt Engineering
-   **Centralized Rules:** `FINANCIAL_EXTRACTION_RULES` updated with explicit constraints for Currency (GBX), Dates (ISO8601), and Cancellations.
-   **Entity Type:** Add `account_type` (Individual, Trust, Corporate) to extraction schema.
