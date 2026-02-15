# Refactoring Opportunities

## 1. Redundant Seeding Logic
- **Issue:** Three different seeding scripts exist (`seed_data.py`, `seed_dev_database.py`, `seed_v2_test_data.py`) with overlapping functionality but different implementation styles (SQLAlchemy vs direct psycopg2).
- **Action:** Consolidate into a unified `scripts/seed/` module. Prefer SQLAlchemy (via `src/pa_dealing/db`) over raw `psycopg2` to maintain a single source of truth for schema interactions.

## 2. Inconsistent DB Connection Management
- **Issue:** Scripts like `seed_dev_database.py` hardcode DB connection parameters and use `psycopg2`, while the main app uses `src/pa_dealing/db/engine.py` and Pydantic settings.
- **Action:** Refactor all scripts to import `get_session` or `engine` from `src/pa_dealing/db` and configuration from `src/pa_dealing/config`.

## 3. Script Categorization (Cruft Removal)
- **Issue:** The `scripts/` directory is flat and contains one-off debug scripts mixed with critical ops tools.
    - Keep/Organize: `run_api.py`, `run_monitoring.py`, `init_db.sql`
    - Archive/Delete: `reproduce_attr_error.py`, `check_manager_field.py`
    - Debug: `verify_slack_blocks.py`, `check_slack_capabilities.py`
- **Action:** Create subdirectories `scripts/ops`, `scripts/db`, `scripts/debug`.

## 4. Architectural Boundaries (Agents vs Core)
- **Issue:** `src/pa_dealing/agents/database` seems to duplicate or confuse responsibilities with `src/pa_dealing/db`.
- **Action:** Verify if `agents/database` contains AI-specific tools. If it's just wrappers, move to `src/pa_dealing/db/tools.py`. Ensure `src/pa_dealing/agents` is strictly for AI/Business Logic actors.

## 5. Dangerous Redundancy: PADService vs DB Tools
- **Issue:** `src/pa_dealing/services/pad_service.py` (Audit-aware) wraps `src/pa_dealing/agents/database/tools.py` (Raw DB ops).
- **Risk:** Developers or Agents might import `db_tools` directly, bypassing audit logging.
- **Action:**
    1. Rename `agents/database/tools.py` to `db/repository.py` (Internal use only).
    2. Enforce that *only* `PADService` imports the repository.
    3. All external consumers (API, Agents) must use `PADService`.

## 6. Monolithic Models
- **Issue:** `src/pa_dealing/db/models.py` is too large (600+ lines).
- **Action:** Split into `models/core.py`, `models/pad.py`, `models/market.py`, `models/compliance.py`.

## 7. Potential Dead Code (Google ADK)
- **Issue:** `src/pa_dealing/main.py` initializes `google-adk`. The project seems to have moved to a custom FastAPI/React dashboard.
- **Action:** Confirm if `google-adk` is still required. If not, remove `main.py` and the dependency to clean up the entry point.