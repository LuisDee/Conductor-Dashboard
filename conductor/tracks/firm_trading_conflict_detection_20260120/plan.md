# Implementation Plan: Firm Trading Conflict Detection & Enrichment

## Phase 1: Data Source Integration ✅
- [x] Task: Verify ProductUsage table synced to dev database
- [x] Task: Create SQLAlchemy model for ProductUsage
- [x] Task: Create SQLAlchemy model for PortfolioGroup
- [x] Task: Create SQLAlchemy model for HistoricPosition (Mock)

## Phase 2: 3-Tier Instrument Lookup Completion ✅
- [x] Task: Implement short-circuit waterfall orchestration (Instrument Identity Refactor track)
- [x] Task: Implement Tier 1 (Bloomberg), Tier 2 (Mapping), and Tier 3 (Product) search logic

## Phase 3: Conflict Detection Service ✅
- [x] Task: Implement firm trading query logic in `repository.py`
- [x] Task: Add full desk name resolution via PortfolioGroup/MetaData/CostCentre join chain
- [x] Task: Define `ConflictResult` and `MakoPositionInfo` structures

## Phase 4: Risk Engine Integration ✅
- [x] Task: Update risk engine to call conflict detection logic
- [x] Task: Integrate "Desk Match" detection into the Advisory System
- [x] Task: Ensure conflict fields populated during PADRequest submission

## Phase 5: User Interface Updates ✅
- [x] Task: Implement side-by-side "Conflict Search" view in Dashboard
- [x] Task: Add conflict flags to dashboard summary counts
- [x] Task: Display conflict warnings in Slack via Advisory System

## Phase 6: Testing & Validation ✅
- [x] Task: Integration tests for position lookup and conflict risk calculation (`test_position_lookup.py`)
- [x] Task: Verified functional performance of bulk conflict queries