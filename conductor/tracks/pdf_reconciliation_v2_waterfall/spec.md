# Track Spec: PDF Reconciliation V2 (Waterfall Matrix)

## Overview
Overhaul the PDF ingestion matching logic to use a strict Waterfall/Gate system (Direction -> Security -> Economics) to prevent false matches and multi-trade broadcasting.

## Objectives
1.  **Strict User Anchor:** Only match requests for the identified user.
2.  **Deterministic Gates:** Implement Direction, Security (ISIN/Ticker), and Economic (Qty/Value) gates.
3.  **No-Guessing Policy:** Fall back to manual review if multiple candidates match or zero candidates match.
4.  **UI Transparency:** Expose the "Match Matrix" in the dashboard so users see why candidates failed.
5.  **Robustness:** Fix the `await` crash in the verification service.

## Core Mandates
- Do not auto-match anonymous documents.
- Remove `verify_trade` and integrate checks into the matching gates.
- Maintain a clear audit trail of matching decisions.
