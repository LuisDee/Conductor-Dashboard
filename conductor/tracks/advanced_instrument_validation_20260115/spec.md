# Specification: Advanced Instrument Validation (3-Tier Lookup)

## Goal
Implement a robust 3-tier fallback search system for instrument lookups, replicating the exact logic from the core PA Dealing implementation. 

## Data Architecture (The Superset Model)
- **Master List (`oracle_product`):** The source of truth for ALL tradable instruments (internal + external).
- **External Subset (`oracle_bloomberg`):** Contains securities specifically sourced from Bloomberg.
- **Mapping Layer (`oracle_map_inst_symbol`):** Links exchange-specific symbols to internal symbols.

## Search Hierarchy (Short-Circuit Logic)
1. **Tier 1 (Bloomberg):** Search `oracle_bloomberg` (ticker, description, isin, sedol).
2. **Tier 2 (MapInstSymbol):** Fallback search in `oracle_map_inst_symbol` by `exch_symbol`.
3. **Tier 3 (Product):** Final fallback search in `oracle_product` (catch-all for internal/unlisted).

## Database Schema (Oracle Fidelity)
### `oracle_map_inst_symbol`
- **Constraint:** `UNIQUE (symbol_type, exchange_id, inst_type, exch_symbol)`
- **No Serial PK:** Match Oracle source exactly to simplify composite UPSERT logic.
- **FKs:** Links to `oracle_exchange(id)` and `oracle_inst_type(inst_type)`.

### `oracle_product`
- **Columns:** `inst_symbol` (PK), `description`, `inst_type`, `is_deleted`.
- **Relationship:** Linked to `oracle_bloomberg` via `inst_symbol`.

## UX Rules
- **Visibility:** Tier sources are HIDDEN from the user to keep the conversation simple.
- **Logging:** Every lookup MUST log the `source_tier` and `search_term` for audit and debugging.
- **Exception:** If Tier 3 finds an `INTERNAL_` prefixed security, the Agent may explicitly mention it is an "Internal Security."

## Reference Files (External)
- Implementation: `/home/coder/repos/data/backoffice-web/compliance_portal/pages/pa_dealing/filters.py`
- Models: `/home/coder/repos/data/backoffice-web/database_models/models/tables/table_models.py`