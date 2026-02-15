# PDF Reconciliation Workbench

**Goal:** Transform the PDF History page into an active "Exception Management" dashboard with a smart, side-by-side reconciliation cockpit.

## 1. Backend Infrastructure
- [ ] **Smart Match Service (`SmartMatcher`):**
    -   Implement fuzzy logic to find candidates for a given `ParsedTrade`.
    -   **Scope:** 
        1.  `PADRequest` (Approved, Pending Execution).
        2.  `PADExecution` (Already Executed, within 90 days) - for Activity Statements/duplicates.
    -   **Scoring:** Ticker (30%), Quantity (30%), Direction (20%), Date (10%), User (10%).
- [ ] **API Endpoints:**
    -   `GET /api/pdf-history/{document_id}/candidates`: Returns ranked list of `Candidate` schema (including executed trades).
    -   `POST /api/pdf-history/{document_id}/acknowledge`: Dismiss with reason.
    -   `POST /api/pdf-history/{document_id}/reconcile`:
        -   **Payload:** `{ request_id: int, overrides: Partial<ExtractedTradeData> }`.
        -   **Logic:** Update `ParsedTrade` -> Link Document -> (If pending) Create Execution & Resolve Breaches OR (If executed) Link as secondary doc.

## 2. Dashboard UI Overhaul ("The To-Do List")
- [ ] **Filters & State:**
    -   Tabs: "Unmatched" (Default), "Errors", "All", "Acknowledged".
    -   Count Badges: Red for Errors, Amber for Unmatched.
- [ ] **Table Columns:**
    -   Add "Reason" column.
    -   Add "Actions" column: "Reconcile" (Primary), "Dismiss" (Ghost).

## 3. The "Cockpit" Modal (Side-by-Side)
- [ ] **Layout:** Split View (Resizable).
    -   **Left:** PDF Viewer.
    -   **Right:** Reconciliation Panel.
-   **Reconciliation Panel:**
    -   **Top:** Editable Extraction Form (Ticker, Qty, Price, Date).
    -   **Bottom:** Candidate List.
    -   **Filters:** "Pending" (Default) | "Executed" | "All".
    -   **List Item:** Candidate Card sorted by Fuzzy Score.
    -   **Visuals:** Diff highlighting (Green/Red) for Ticker/Qty/Price comparison.
-   **Match Action:**
    -   If Pending: "Link & Create Execution".
    -   If Executed: "Link as Secondary Document".

## 4. Audit & Safety
- [ ] **Audit Trail:**
    -   Log `MANUAL_RECONCILIATION` event.
    -   Log `ACKNOWLEDGEMENT` with user reason.
- [ ] **Undo Capability:** Add "Unlink" button in Document Detail view.
