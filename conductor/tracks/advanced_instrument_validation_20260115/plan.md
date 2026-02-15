# Implementation Plan: Advanced Instrument Validation (3-Tier Lookup)

## Goal
Implement a robust 3-tier fallback search system for instrument lookups, replicating the exact logic from the production PA Dealing implementation (`filters.py:182-226`) and following the `AGENT_3TIER_IMPLEMENTATION_GUIDE.md` precisely.

## Phase 1: Database Expansion (Schema Fidelity) ✅
- [x] Align SQLAlchemy models with production Django schemas:
    - [x] `OracleBloomberg` (mapped to `bloomberg` table): Search fields: `description`, `sedol`, `isin`, `reuters`, `inst_symbol`.
    - [x] `OracleMapInstSymbol` (mapped to `map_inst_symbol` table): Search fields: `exch_symbol`, `inst_symbol`.
    - [x] `OracleProduct` (mapped to `product` table): Search fields: `description`, `inst_symbol` + `is_deleted` filter.
- [x] Generate and apply Alembic migration for local environment.
- [x] Verify `alembic/env.py` continues to ignore these tables in multi-schema environments (Dev/Prod).

## Phase 2: Logic Implementation (Short-Circuit Waterfall) ✅
- [x] Implement `lookup_instrument(session, term: str) -> dict` in `src/pa_dealing/db/repository.py`:
    - [x] **Tier 1 (Bloomberg):** Search `description`, `sedol`, `isin`, `reuters`, `inst_symbol` using `ILIKE '%term%'`.
    - [x] **Short-Circuit 1:** If results found in Tier 1, STOP and return them.
    - [x] **Tier 2 (MapInstSymbol):** If Tier 1 empty, search `exch_symbol`, `inst_symbol` using `ILIKE '%term%'`.
    - [x] **Short-Circuit 2:** If results found in Tier 2, STOP and return them.
    - [x] **Tier 3 (Product):** If Tier 1 & 2 empty, search `description`, `inst_symbol` using `ILIKE '%term%'`.
    - [x] **Critical Filter:** Apply `is_deleted == 'N' OR is_deleted IS NULL` to Tier 3.
- [x] **Distinct Results:** Ensure `inst_symbol` results are unique across queries.
- [x] **Return Format:** Implemented structured return with status, inst_symbols, source_tier, count.

## Phase 3: Agent & Tool Integration ✅
- [x] Refactor the AI Agent tool wrapper (`lookup_instrument_tool`) to use the new structured return format.
- [x] Update Agent prompt to interpret the `status` field:
    - [x] `exact_match`: Proceed.
    - [x] `ambiguous`: Ask user to disambiguate.
    - [x] `not_found`: Inform user.
    - [x] **Constraint:** Agent MUST NOT explain the tier source to the user.

## Phase 4: Data Seeding & Validation ✅
- [x] Test data available via dev database connection (bo_airflow schema).
- [x] Validation suite covers all tier scenarios.

## Phase 5: Production Readiness ✅
- [x] Verify connectivity to Dev database (`bo_airflow` schema).
- [x] Log `source_tier` for all lookups to support compliance audit trails.

## Track Complete ✅

**Final Results (2026-01-25):**
- 3-tier instrument lookup fully implemented
- Models: OracleBloomberg, OracleMapInstSymbol, OracleProduct
- Short-circuit waterfall logic working
- Integrated with chatbot and database agent
- Connected to dev database with live data
