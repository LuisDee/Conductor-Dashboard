# Plan: God Module Decomposition

**Track Effort**: L (1-3 months)
**Objective**: Split 3 god modules (repository.py 2,953 LOC, pad_service.py 2,748 LOC, handlers.py 3,192 LOC) into focused sub-modules with backward compatibility during transition.

---

## Executive Summary

This refactor decomposes three critical god modules:
1. **repository.py** (2,953 LOC, 46 functions, 13 importers) → 4 domain repositories
2. **pad_service.py** (2,748 LOC, 53 methods, 6 importers) → 4 service modules
3. **handlers.py** (3,192 LOC) → Extract business logic to service layer

**Key Strategy**: Re-export from `__init__.py` during transition to maintain backward compatibility, then remove after all callers updated.

---

## Phase 1: Repository Split (repository.py → 4 domain modules)

### 1.0 Pre-Flight Analysis

**Verified Caller Files (13 total)**:
```
src/pa_dealing/services/pad_service.py
src/pa_dealing/services/trade_document_processor.py
src/pa_dealing/agents/database/agent.py
src/pa_dealing/agents/orchestrator/agent.py
src/pa_dealing/agents/orchestrator/risk_scoring_service.py
src/pa_dealing/agents/slack/chatbot.py
src/pa_dealing/agents/slack/handlers.py
src/pa_dealing/api/routes/dashboard.py
src/pa_dealing/api/routes/pdf_reconciliation.py
src/pa_dealing/api/routes/requests.py
src/pa_dealing/agents/monitoring/jobs.py (inferred)
src/pa_dealing/services/email_ingestion.py (inferred)
src/pa_dealing/services/graph_email_poller.py (inferred)
```

**Test Files to Update**:
```
tests/unit/test_audit_events.py
tests/unit/test_conflict_detection.py
tests/unit/test_contract_note_api.py
tests/unit/test_contract_note_upload.py
tests/integration/test_database_tools.py
tests/integration/test_external_resolution_integration.py
tests/integration/test_position_lookup.py
tests/integration/test_restricted_lookup_comprehensive.py
tests/integration/test_security_confirmation_flow.py
tests/integration/test_security_matching.py
tests/test_instrument_lookup.py
```

### 1.1 Create Employee Repository

**File**: `src/pa_dealing/db/employee_repository.py`

**Functions to Move** (Lines 65-179, ~114 LOC):
```python
# L65-106: async def get_employee_by_email(session, email) -> EmployeeInfo | None
# L107-131: async def get_employee_by_id(session, employee_id) -> EmployeeInfo | None
# L133-177: async def get_manager_chain(session, employee_id) -> ManagerChainInfo | None
```

**Shared Dependencies**:
- Models: `OracleEmployee`, `OracleCostCentre`
- Schemas: `EmployeeInfo`, `ManagerChainInfo`
- SQL Fragments: `EMPLOYEE_CONTACT_LEFT_JOIN` (from `sql_fragments.py`)

**Implementation Steps**:

1. Create `src/pa_dealing/db/employee_repository.py`:
```python
"""Employee repository - HR data access layer."""
from sqlalchemy.ext.asyncio import AsyncSession
from .models import OracleEmployee, OracleCostCentre
from .schemas import EmployeeInfo, ManagerChainInfo
from .sql_fragments import EMPLOYEE_CONTACT_LEFT_JOIN

# Move functions here
```

2. Update `src/pa_dealing/db/__init__.py` (BACKWARD COMPAT):
```python
# Re-export for backward compatibility (REMOVE in Phase 1.6)
from .employee_repository import (
    get_employee_by_email,
    get_employee_by_id,
    get_manager_chain,
)
```

3. Update direct importers (update in place, no rush):
```python
# OLD: from pa_dealing.db.repository import get_employee_by_email
# NEW: from pa_dealing.db.employee_repository import get_employee_by_email
```

**Caller Impact**:
- `pad_service.py`: Lines 85, 100-101 (via `db_tools.get_employee_by_email/by_id`)
- `slack/chatbot.py`: Employee lookup logic
- `slack/handlers.py`: Employee resolution in payloads
- `orchestrator/agent.py`: Employee context
- Test files: Direct imports need updating

**Verification Gate**:
```bash
# Run before proceeding to 1.2
pytest tests/unit/test_*.py -k employee
pytest tests/integration/test_database_tools.py
# All existing tests MUST pass (148 total)
```

### 1.2 Create Instrument Repository

**File**: `src/pa_dealing/db/instrument_repository.py`

**Functions to Move** (Lines 1610-2152, ~542 LOC):
```python
# L1610-1635: async def search_bloomberg(session, term) -> list[str]
# L1637-1659: async def search_map_inst_symbol(session, term) -> list[str]
# L1661-1690: async def search_product(session, term) -> list[str]
# L1692-1737: async def resolve_instrument_identity(session, identifier) -> InstrumentInfo | None
# L1739-1745: async def search_instruments(session, term) -> InstrumentLookupResult
# L1747-2151: async def _search_instruments(session, term) -> InstrumentLookupResult (LARGE FUNCTION - 404 LOC!)
# L2153-2191: def _resolve_desk_name(cost_centre_name) -> str
```

**Shared Dependencies**:
- Models: `OracleBloomberg`, `OracleMapInstSymbol`, `OracleProduct`, `ProductUsage`
- Schemas: `InstrumentInfo`, `InstrumentLookupResult`
- External: `get_external_resolver` from `instruments/external_resolver`
- Fuzzy Cache: `ensure_cache_fresh`, `search_fuzzy` from `instruments/fuzzy_cache`

**CRITICAL NOTE**: `_search_instruments` is 404 LOC and should be considered for further decomposition in a follow-up track.

**Implementation Steps**:

1. Create `src/pa_dealing/db/instrument_repository.py`:
```python
"""Instrument repository - security/instrument lookup layer."""
from sqlalchemy.ext.asyncio import AsyncSession
from pa_dealing.instruments.external_resolver import get_external_resolver
from pa_dealing.instruments.fuzzy_cache import ensure_cache_fresh, search_fuzzy
from .models import OracleBloomberg, OracleMapInstSymbol, OracleProduct, ProductUsage
from .schemas import InstrumentInfo, InstrumentLookupResult

# Move functions here
```

2. Update `src/pa_dealing/db/__init__.py` (BACKWARD COMPAT):
```python
# Re-export for backward compatibility (REMOVE in Phase 1.6)
from .instrument_repository import (
    search_bloomberg,
    search_map_inst_symbol,
    search_product,
    resolve_instrument_identity,
    search_instruments,
)
```

**Caller Impact**:
- `pad_service.py`: Lines 2208-2226 (via `db_tools.search_instruments/resolve_instrument_identity`)
- `trade_document_processor.py`: Instrument matching
- `slack/chatbot.py`: Instrument autocomplete
- `api/routes/requests.py`: Instrument validation
- Test files: `test_instrument_lookup.py`, `test_external_resolution_integration.py`, etc.

**Verification Gate**:
```bash
pytest tests/test_instrument_lookup.py
pytest tests/integration/test_external_resolution_integration.py
pytest tests/integration/test_security_matching.py
# Verify external resolver integration still works
```

### 1.3 Create Compliance Repository

**File**: `src/pa_dealing/db/compliance_repository.py`

**Functions to Move** (Lines 405-1127, ~722 LOC):
```python
# L405-465: async def check_mako_positions(session, employee_id, identifier) -> ConflictCheckResult
# L467-557: async def check_restricted_list_comprehensive(session, identifier) -> RestrictedCheckResult
# L559-572: async def check_restricted_list(session, identifier) -> bool
# L574-598: async def _lookup_security_by_identifier(session, identifier) -> str | None (INTERNAL)
# L600-700: async def check_holding_period(session, employee_id, identifier) -> HoldingPeriodResult
# L702-774: async def check_employee_position(session, employee_id, identifier) -> dict
# L776-872: async def get_all_employee_positions(session, employee_id) -> list
# L874-909: async def get_breaches(session, employee_id, severity, limit) -> list
# L2193-2306: async def get_mako_position_info(session, identifier) -> MakoPositionInfo | None
# L2308-2390: async def get_employee_trade_history(session, employee_id, days) -> list[EmployeeTradeRecord]
# L2392-2527: def calculate_conflict_risk(...) -> ConflictRiskResult (PURE FUNCTION - 135 LOC)
# L1592-1608: async def get_compliance_config(session) -> ComplianceConfig
```

**Shared Dependencies**:
- Models: `OraclePosition`, `RestrictedSecurity`, `PADBreach`, `PADRequest`, `ComplianceConfig`
- Schemas: `ConflictCheckResult`, `RestrictedCheckResult`, `HoldingPeriodResult`, `MakoPositionInfo`, `EmployeeTradeRecord`, `ConflictRiskResult`

**Implementation Steps**:

1. Create `src/pa_dealing/db/compliance_repository.py`:
```python
"""Compliance repository - conflict checks, holding periods, restrictions."""
from sqlalchemy.ext.asyncio import AsyncSession
from .models import OraclePosition, RestrictedSecurity, PADBreach, PADRequest, ComplianceConfig
from .schemas import (
    ConflictCheckResult, RestrictedCheckResult, HoldingPeriodResult,
    MakoPositionInfo, EmployeeTradeRecord, ConflictRiskResult
)

# Move functions here
```

2. Update `src/pa_dealing/db/__init__.py` (BACKWARD COMPAT):
```python
# Re-export for backward compatibility (REMOVE in Phase 1.6)
from .compliance_repository import (
    check_mako_positions,
    check_restricted_list_comprehensive,
    check_restricted_list,
    check_holding_period,
    check_employee_position,
    get_all_employee_positions,
    get_breaches,
    get_mako_position_info,
    get_employee_trade_history,
    calculate_conflict_risk,
    get_compliance_config,
)
```

**Caller Impact**:
- `pad_service.py`: Lines ~1190-1270 (via `db_tools.check_*` methods)
- `orchestrator/risk_scoring_service.py`: Risk calculation
- `slack/handlers.py`: Pre-trade checks
- `monitoring/jobs.py`: Breach monitoring
- Test files: `test_conflict_detection.py`, `test_position_lookup.py`, `test_restricted_lookup_comprehensive.py`

**Verification Gate**:
```bash
pytest tests/unit/test_conflict_detection.py
pytest tests/integration/test_position_lookup.py
pytest tests/integration/test_restricted_lookup_comprehensive.py
# Verify compliance checks still work
```

### 1.4 Create PAD Repository

**File**: `src/pa_dealing/db/pad_repository.py` (rename from `repository.py`)

**Functions Remaining** (~1,575 LOC):
```python
# PAD Request CRUD
# L179-272: async def get_or_create_security(...)
# L274-311: async def get_all_requests(...)
# L313-346: async def get_pad_requests(...)
# L348-403: def _request_to_info(req) -> PADRequestInfo
# L911-948: async def get_recent_pad_requests(...)
# L1030-1126: async def pre_trade_check(...)
# L1128-1249: async def submit_pad_request(...)
# L1251-1296: async def check_recent_executed_buy(...)
# L1298-1329: async def record_decision_outcome(...)
# L1331-1447: async def update_pad_status(...) (LARGE - 116 LOC)
# L1449-1507: async def create_auto_approval(...)
# L1509-1536: async def get_pad_request_by_thread(...)
# L1538-1559: async def get_pad_request_by_id(...)
# L1561-1590: async def get_recent_request_count(...)

# Execution tracking
# L950-1028: async def record_execution(...) (LARGE - 78 LOC)

# Contract notes
# L2529-2589: async def create_contract_note_upload(...)
# L2591-2602: async def get_contract_note_history(...)
# L2604-2616: async def get_active_contract_note(...)
# L2618-2738: async def get_trade_history(...) (LARGE - 120 LOC)

# Audit events
# L2740-2783: async def insert_audit_event(...)
# L2785-2835: async def get_recent_audit_events(...)
# L2837-2869: async def get_audit_event_by_id(...)
# L2871-2906: async def get_audit_event_stats(...)
# L2908-2953: async def get_audit_event_sparkline(...)
```

**Implementation Steps**:

1. Rename `repository.py` → `pad_repository.py`:
```bash
git mv src/pa_dealing/db/repository.py src/pa_dealing/db/pad_repository.py
```

2. Update imports in `pad_repository.py` to reference other new repositories:
```python
# Add cross-repository imports if needed
from .employee_repository import get_employee_by_id
from .instrument_repository import resolve_instrument_identity
from .compliance_repository import check_holding_period
```

3. Update `src/pa_dealing/db/__init__.py`:
```python
# Export pad repository functions
from .pad_repository import (
    submit_pad_request,
    update_pad_status,
    get_pad_request_by_id,
    get_pad_request_by_thread,
    record_execution,
    # ... all other PAD functions
)
```

**Caller Impact**:
- `pad_service.py`: Majority of calls (Lines 350-900+)
- `slack/handlers.py`: Lines 1058, 1113-1115, 2240, 2416 (via `db_tools.*`)
- `api/routes/requests.py`: PAD CRUD operations
- `api/routes/dashboard.py`: Request listing
- `trade_document_processor.py`: Execution recording
- Test files: All PAD-related tests

### 1.5 Update All Caller Files (13 files)

**Priority Order** (update one file at a time, test after each):

1. **Low-risk: Test utilities first**
   - `tests/utils/async_helpers.py`

2. **Medium-risk: Scripts and debugging tools**
   - `scripts/debug/test_security_lookup.py`
   - `scripts/debug/verify_3tier_lookup.py`
   - `scripts/debug/verify_security_lookup.py`

3. **High-risk: Core services** (requires careful review)
   - `src/pa_dealing/services/pad_service.py` (UPDATE: Change `from .repository import` → specific imports)
   - `src/pa_dealing/services/trade_document_processor.py`

4. **Critical: Agent layer** (requires integration testing)
   - `src/pa_dealing/agents/database/agent.py`
   - `src/pa_dealing/agents/orchestrator/agent.py`
   - `src/pa_dealing/agents/orchestrator/risk_scoring_service.py`
   - `src/pa_dealing/agents/slack/chatbot.py`
   - `src/pa_dealing/agents/slack/handlers.py`

5. **Critical: API routes** (requires API testing)
   - `src/pa_dealing/api/routes/dashboard.py`
   - `src/pa_dealing/api/routes/pdf_reconciliation.py`
   - `src/pa_dealing/api/routes/requests.py`

**Update Pattern** for each file:
```python
# BEFORE
from pa_dealing.db.repository import get_employee_by_email, submit_pad_request

# AFTER (specific imports)
from pa_dealing.db.employee_repository import get_employee_by_email
from pa_dealing.db.pad_repository import submit_pad_request
```

**Verification After Each Update**:
```bash
# Run tests for the updated module
pytest tests/unit/test_<related>.py -v
pytest tests/integration/test_<related>.py -v
```

### 1.6 Remove Backward Compatibility Re-Exports

**ONLY AFTER** all 13 caller files and all test files have been updated:

1. Remove re-exports from `src/pa_dealing/db/__init__.py`:
```python
# DELETE the temporary re-exports added in 1.1-1.4
# from .employee_repository import ...  # REMOVE
# from .instrument_repository import ... # REMOVE
# from .compliance_repository import ... # REMOVE
# from .pad_repository import ...        # REMOVE
```

2. Verify no remaining imports from old module:
```bash
# Should return ZERO results
grep -r "from pa_dealing.db.repository import" src/ --include="*.py"
grep -r "from pa_dealing.db import repository" src/ --include="*.py"
```

**Final Phase 1 Verification Gate**:
```bash
# All tests must pass
pytest tests/ -v --tb=short
# Expected: 148 test files, 100% pass rate

# Line count verification
wc -l src/pa_dealing/db/employee_repository.py      # ~120 LOC
wc -l src/pa_dealing/db/instrument_repository.py    # ~550 LOC
wc -l src/pa_dealing/db/compliance_repository.py    # ~730 LOC
wc -l src/pa_dealing/db/pad_repository.py           # ~1600 LOC
# Total: ~3000 LOC (matches original)

# No module exceeds 800 LOC target (except pad_repository at ~1600)
# pad_repository.py will be further split in Phase 4 (future track)
```

**Rollback Strategy for Phase 1**:
- If failure in step 1.1-1.4: Delete new file, restore from git
- If failure in step 1.5: Revert individual file import changes
- If failure in step 1.6: Re-add re-exports to `__init__.py`

---

## Phase 2: PAD Service Split (pad_service.py → 4 service modules)

### 2.0 Pre-Flight Analysis

**Verified Caller Files (6 total)**:
```
src/pa_dealing/api/dependencies.py
src/pa_dealing/api/routes/audit.py
src/pa_dealing/api/routes/breaches.py
src/pa_dealing/api/routes/dashboard.py
src/pa_dealing/api/routes/reports.py
src/pa_dealing/api/routes/requests.py
```

**Test Files to Update**:
```
tests/unit/test_dashboard_duplication_fix.py
tests/unit/test_dashboard_redesign.py
tests/unit/test_dashboard_summary_counts.py
tests/unit/test_audit_events.py
tests/e2e/test_ux_overhaul_journey.py
```

**Current State** (after Phase 1):
- PADService imports from new repositories: `employee_repository`, `instrument_repository`, `compliance_repository`, `pad_repository`
- All API routes depend on `PADService` instance (injected via FastAPI dependencies)

### 2.1 Create Submission Service

**File**: `src/pa_dealing/services/submission_service.py`

**Methods to Move** (Lines 331-440, ~109 LOC):
```python
# L331-349: async def submit_request(self, employee_id, trade_input) -> PADSubmitResult
# L350-439: async def _submit_request_with_session(self, session, employee_id, trade_input) -> PADSubmitResult (INTERNAL - 89 LOC)
```

**Shared Dependencies**:
- Audit Logger: `self._audit` (inject in __init__)
- Repository calls: `db_tools.submit_pad_request`, `db_tools.check_recent_executed_buy`
- Schemas: `PADRequestInput`, `PADSubmitResult`
- Models: `PADRequest`

**Implementation**:

1. Create `src/pa_dealing/services/submission_service.py`:
```python
"""Submission service - PAD request submission and validation."""
from pa_dealing.audit import AuditLogger, get_audit_logger
from pa_dealing.db import get_session
from pa_dealing.db.pad_repository import submit_pad_request, check_recent_executed_buy
from pa_dealing.db.schemas import PADRequestInput, PADSubmitResult

class SubmissionService:
    def __init__(self, audit_logger: AuditLogger | None = None):
        self._audit = audit_logger or get_audit_logger()

    async def submit_request(self, employee_id: int, trade_input: PADRequestInput) -> PADSubmitResult:
        """Submit PAD request with validation and audit logging."""
        # Move implementation here
```

2. Update `src/pa_dealing/services/__init__.py`:
```python
from .submission_service import SubmissionService

__all__ = ["SubmissionService", ...]
```

3. Update `PADService` (backward compat delegation):
```python
# In PADService.__init__
self._submission_service = SubmissionService(audit_logger=self._audit)

async def submit_request(self, employee_id: int, trade_input: PADRequestInput) -> PADSubmitResult:
    """Delegate to submission service."""
    return await self._submission_service.submit_request(employee_id, trade_input)
```

**Caller Impact**:
- `api/routes/requests.py`: Submission endpoint
- `slack/handlers.py`: Slack submission flow
- Tests: `test_ux_overhaul_journey.py`, submission-related tests

**Verification Gate**:
```bash
pytest tests/unit -k submit
pytest tests/e2e/test_ux_overhaul_journey.py -k submit
```

### 2.2 Create Approval Service

**File**: `src/pa_dealing/services/approval_service.py`

**Methods to Move** (Lines 441-831, ~390 LOC):
```python
# L441-510: async def approve_request(self, request_id, approver_email, notes) -> dict
# L512-576: async def decline_request(self, request_id, decliner_email, reason) -> dict
# L832-905: async def escalate_to_smf16(self, request_id, escalator_email, reason) -> dict
# L1432-1564: async def get_pending_approvals(self, approver_id, status, limit) -> list[dict] (133 LOC - LARGE)
```

**Shared Dependencies**:
- Audit Logger: `self._audit`
- Repository calls: `db_tools.update_pad_status`, `db_tools.record_decision_outcome`, `db_tools.get_pad_request_by_id`
- Schemas: `PADStatusUpdate`, `PADUpdateResult`
- Models: `PADRequest`, `PADApproval`

**Implementation**:

1. Create `src/pa_dealing/services/approval_service.py`:
```python
"""Approval service - PAD approval workflows and state transitions."""
from pa_dealing.audit import AuditLogger, get_audit_logger
from pa_dealing.db import get_session
from pa_dealing.db.pad_repository import (
    update_pad_status, record_decision_outcome, get_pad_request_by_id
)
from pa_dealing.db.schemas import PADStatusUpdate, PADUpdateResult

class ApprovalService:
    def __init__(self, audit_logger: AuditLogger | None = None):
        self._audit = audit_logger or get_audit_logger()

    async def approve_request(self, request_id: int, approver_email: str, notes: str | None = None) -> dict:
        """Approve PAD request with audit trail."""
        # Move implementation here
```

2. Update `src/pa_dealing/services/__init__.py`:
```python
from .approval_service import ApprovalService
```

3. Update `PADService` (backward compat delegation):
```python
# In PADService.__init__
self._approval_service = ApprovalService(audit_logger=self._audit)

async def approve_request(self, request_id: int, approver_email: str, notes: str | None = None) -> dict:
    return await self._approval_service.approve_request(request_id, approver_email, notes)

async def decline_request(self, request_id: int, decliner_email: str, reason: str) -> dict:
    return await self._approval_service.decline_request(request_id, decliner_email, reason)

async def escalate_to_smf16(self, request_id: int, escalator_email: str, reason: str) -> dict:
    return await self._approval_service.escalate_to_smf16(request_id, escalator_email, reason)
```

**Caller Impact**:
- `api/routes/requests.py`: Approval endpoints
- `slack/handlers.py`: Lines ~1050-1150 (approval button handlers)
- Tests: Approval workflow tests

**Verification Gate**:
```bash
pytest tests/ -k "approve or decline or escalate"
```

### 2.3 Create Execution Service

**File**: `src/pa_dealing/services/execution_service.py`

**Methods to Move** (Lines 578-752, ~174 LOC):
```python
# L578-751: async def record_execution(self, request_id, execution_details) -> dict (173 LOC - LARGE)
# L753-830: async def manual_chase(self, request_id) -> dict
# L1628-1713: async def get_execution_tracking(self, status, limit) -> list[dict]
```

**Shared Dependencies**:
- Audit Logger: `self._audit`
- Repository calls: `db_tools.record_execution`, `db_tools.get_pad_request_by_id`
- Models: `PADExecution`, `PADRequest`

**Implementation**:

1. Create `src/pa_dealing/services/execution_service.py`:
```python
"""Execution service - trade execution tracking and contract note linking."""
from pa_dealing.audit import AuditLogger, get_audit_logger
from pa_dealing.db import get_session
from pa_dealing.db.pad_repository import record_execution, get_pad_request_by_id

class ExecutionService:
    def __init__(self, audit_logger: AuditLogger | None = None):
        self._audit = audit_logger or get_audit_logger()

    async def record_execution(self, request_id: int, execution_details: dict) -> dict:
        """Record trade execution with contract note linking."""
        # Move implementation here
```

2. Update `PADService` (backward compat delegation):
```python
# In PADService.__init__
self._execution_service = ExecutionService(audit_logger=self._audit)

async def record_execution(self, request_id: int, execution_details: dict) -> dict:
    return await self._execution_service.record_execution(request_id, execution_details)
```

**Caller Impact**:
- `trade_document_processor.py`: Contract note reconciliation
- `api/routes/requests.py`: Execution endpoints
- Tests: Execution tracking tests

**Verification Gate**:
```bash
pytest tests/ -k execution
```

### 2.4 Create Monitoring Service

**File**: `src/pa_dealing/services/monitoring_service.py`

**Methods to Move** (Monitoring/Reporting/Dashboard, ~1,000 LOC):
```python
# Dashboard & Metrics
# L129-173: async def get_recent_activity(self, limit) -> list[dict]
# L174-224: async def get_dashboard_summary_counts(self, since, employee_id) -> dict
# L225-315: async def _count_* helper methods (90 LOC total)
# L2321-2479: async def get_enhanced_dashboard_summary(self, ...) -> dict (158 LOC - LARGE)
# L2481-2516: async def get_sparkline_data(self) -> dict
# L2518-2550: async def _state_based_sparkline(self, ...) -> list[dict]
# L2552-2654: async def get_request_statistics(self, days) -> dict
# L2656-2736: async def get_activity_events(self, ...) -> list[dict]
# L2738-2748: async def get_activity_event_detail(self, event_id) -> dict | None

# Reporting
# L1956-1983: async def get_activity_report(self, start_date, end_date) -> dict
# L1985-2021: async def get_breach_report(self, start_date, end_date) -> dict

# Breaches
# L1163-1168: async def get_breaches(self, employee_id, severity, limit) -> list
# L1282-1327: async def create_breach(self, ...) -> int
# L1329-1389: async def get_active_breaches(self, employee_id, severity) -> list[dict]
# L1391-1430: async def resolve_breach(self, breach_id, resolver_email, notes) -> dict
# L1826-1931: async def get_mako_conflicts(self, ...) -> dict (105 LOC)

# Data lookup (read-only)
# L1121-1156: async def get_requests_by_status(self, status, limit) -> list
# L1158-1161: async def get_all_requests(self, status, limit) -> list[dict]
# L1566-1626: async def get_requests(self, status, limit) -> list[dict]
# L1715-1824: async def get_holding_period_calendar(self, employee_id, days) -> dict (109 LOC)

# Audit log
# L2023-2122: async def search_audit_log(self, ...) -> list[dict]
# L2124-2134: async def get_audit_actors(self) -> list[str]
# L2136-2181: async def get_audit_entity_ids(self, entity_type) -> list
# L2183-2206: async def get_audit_employees(self) -> list[dict]
```

**Implementation**:

1. Create `src/pa_dealing/services/monitoring_service.py`:
```python
"""Monitoring service - dashboards, metrics, breaches, reporting."""
from pa_dealing.audit import AuditLogger, get_audit_logger
from pa_dealing.db import get_session
from pa_dealing.db.pad_repository import get_breaches, get_all_requests
from pa_dealing.db.compliance_repository import get_breaches as get_compliance_breaches

class MonitoringService:
    def __init__(self, audit_logger: AuditLogger | None = None):
        self._audit = audit_logger or get_audit_logger()

    async def get_dashboard_summary_counts(self, since: datetime | None = None, employee_id: int | None = None) -> dict:
        """Get dashboard summary with counts."""
        # Move implementation here
```

2. Update `PADService` (keep as THIN FACADE):
```python
class PADService:
    """
    Facade service - delegates to domain services.

    DEPRECATED: Use specific services directly (SubmissionService, ApprovalService, etc.)
    This class is maintained for backward compatibility only.
    """

    def __init__(self, audit_logger: AuditLogger | None = None):
        self._audit = audit_logger or get_audit_logger()
        self._submission = SubmissionService(audit_logger=self._audit)
        self._approval = ApprovalService(audit_logger=self._audit)
        self._execution = ExecutionService(audit_logger=self._audit)
        self._monitoring = MonitoringService(audit_logger=self._audit)

    # Delegate all methods to sub-services
    async def submit_request(self, *args, **kwargs):
        return await self._submission.submit_request(*args, **kwargs)

    async def approve_request(self, *args, **kwargs):
        return await self._approval.approve_request(*args, **kwargs)

    # ... etc
```

**Caller Impact**:
- `api/routes/dashboard.py`: Dashboard endpoints
- `api/routes/breaches.py`: Breach endpoints
- `api/routes/reports.py`: Reporting endpoints
- `monitoring/jobs.py`: Background monitoring
- Tests: Dashboard, breach, reporting tests

**Verification Gate**:
```bash
pytest tests/unit/test_dashboard_*.py
pytest tests/ -k breach
pytest tests/ -k report
```

### 2.5 Update All Caller Files (6 files)

**Update Strategy**: Migrate callers to use specific services instead of PADService facade.

**Priority Order**:

1. **api/dependencies.py** (inject individual services):
```python
# BEFORE
from pa_dealing.services.pad_service import PADService

async def get_pad_service() -> PADService:
    return PADService()

# AFTER
from pa_dealing.services import (
    SubmissionService, ApprovalService, ExecutionService, MonitoringService
)

async def get_submission_service() -> SubmissionService:
    return SubmissionService()

async def get_approval_service() -> ApprovalService:
    return ApprovalService()

async def get_execution_service() -> ExecutionService:
    return ExecutionService()

async def get_monitoring_service() -> MonitoringService:
    return MonitoringService()
```

2. **api/routes/requests.py** (use specific service dependencies):
```python
# BEFORE
@router.post("/submit")
async def submit_request(trade_input: PADRequestInput, service: PADService = Depends(get_pad_service)):
    return await service.submit_request(...)

# AFTER
@router.post("/submit")
async def submit_request(
    trade_input: PADRequestInput,
    service: SubmissionService = Depends(get_submission_service)
):
    return await service.submit_request(...)
```

3. Update remaining API routes (dashboard.py, breaches.py, reports.py, audit.py)

**Verification After Each Update**:
```bash
# Test API endpoints
pytest tests/integration/test_api_*.py -v
# Run API server and manual smoke test
```

### 2.6 Deprecate PADService Facade (Future Phase)

**NOT in this track** - keep `PADService` as facade for now to ease migration.

Future work:
- Mark `PADService` with `@deprecated` decorator
- Add deprecation warnings in logs
- Remove facade in next major version

**Final Phase 2 Verification Gate**:
```bash
# All tests must pass
pytest tests/ -v --tb=short

# Line count verification
wc -l src/pa_dealing/services/submission_service.py   # ~150 LOC
wc -l src/pa_dealing/services/approval_service.py     # ~450 LOC
wc -l src/pa_dealing/services/execution_service.py    # ~220 LOC
wc -l src/pa_dealing/services/monitoring_service.py   # ~1100 LOC
wc -l src/pa_dealing/services/pad_service.py          # ~150 LOC (facade only)
# Total: ~2070 LOC (down from 2748 LOC - some logic removed/simplified)

# No service exceeds 1200 LOC (monitoring_service is close - consider future split)
```

**Rollback Strategy for Phase 2**:
- If failure in step 2.1-2.4: Delete new service file, restore delegation in PADService
- If failure in step 2.5: Revert API dependency changes
- PADService facade ensures no breaking changes for callers

---

## Phase 3: Slack Handler Business Logic Extraction

### 3.0 Current State Analysis

**handlers.py** (3,192 LOC):
- Direct repository calls: 6 instances (Lines 1058, 1113-1115, 2240, 2416)
- Business logic mixed with Slack payload parsing
- Duplicates approval/decline logic from PADService

**Direct DB Calls to Eliminate**:
```python
# L1058: pad_request = await db_tools.get_pad_request_by_id(session, request_id)
# L1113-1115: update_result = await db_tools.update_pad_status(...)
# L2240: req_info = await db_tools.get_pad_request_by_thread(...)
# L2416: pad_request = await db_tools.get_pad_request_by_id(session, request_id)
```

### 3.1 Extract Approval Logic to Service Layer

**Target Functions** in `handlers.py`:
- `_process_approval` (Lines ~1040-1160)
- `_process_decline` (Lines ~1170-1280)
- `_process_escalation` (Lines ~1290-1400)

**Refactoring Strategy**:

1. Ensure `ApprovalService` (from Phase 2.2) handles all approval logic:
```python
# In approval_service.py - ADD if missing
async def process_slack_approval(
    self,
    request_id: int,
    approver_email: str,
    slack_user_id: str,
    notes: str | None = None
) -> dict:
    """
    Process approval from Slack interface.

    Returns dict with keys: success, message, updated_request
    """
    # Consolidate logic from handlers._process_approval
```

2. Update `handlers.py` to delegate:
```python
# BEFORE (in _process_approval)
async with get_session() as session:
    pad_request = await db_tools.get_pad_request_by_id(session, request_id)
    # ... business logic ...
    update_result = await db_tools.update_pad_status(...)

# AFTER
from pa_dealing.services import ApprovalService

approval_service = ApprovalService()
result = await approval_service.process_slack_approval(
    request_id=request_id,
    approver_email=user_email,
    slack_user_id=slack_user["id"],
    notes=notes
)
```

**Verification**:
```bash
# Test both paths produce identical DB state
pytest tests/integration/test_approval_dual_path.py -v
```

### 3.2 Extract Submission Logic to Service Layer

**Target Functions** in `handlers.py`:
- Submission flow in chatbot interaction handlers

**Refactoring Strategy**:

1. Update `SubmissionService` to accept Slack-specific metadata:
```python
# In submission_service.py - ADD if missing
async def submit_from_slack(
    self,
    employee_email: str,
    trade_input: PADRequestInput,
    slack_thread_ts: str,
    slack_channel_id: str
) -> PADSubmitResult:
    """Submit PAD request from Slack with thread tracking."""
    # Add slack_thread_ts to submission metadata
```

2. Update handlers to delegate submission logic.

### 3.3 Eliminate Direct db_tools Calls

**Action Items**:

1. Replace `db_tools.get_pad_request_by_id` with service method:
```python
# BEFORE
pad_request = await db_tools.get_pad_request_by_id(session, request_id)

# AFTER
monitoring_service = MonitoringService()
pad_request = await monitoring_service.get_request_detail(request_id)
```

2. Replace `db_tools.get_pad_request_by_thread` with service method:
```python
# Add to monitoring_service.py if missing
async def get_request_by_thread(self, thread_ts: str) -> dict | None:
    async with get_session() as session:
        return await pad_repository.get_pad_request_by_thread(session, thread_ts)
```

3. Replace `db_tools.update_pad_status` with approval service methods.

**Verification**:
```bash
# No direct db_tools calls in handlers
grep "db_tools\." src/pa_dealing/agents/slack/handlers.py
# Should return 0 results

# Run Slack integration tests
pytest tests/integration/test_slack_*.py -v
```

### 3.4 Dual-Path Consistency Tests

**Create comprehensive integration tests**:

`tests/integration/test_approval_dual_path.py`:
```python
"""Verify Slack and API approval produce identical DB state."""
import pytest

async def test_slack_approval_matches_api_approval():
    """Slack approval should produce same DB state as API approval."""
    # Setup: Create identical PAD requests
    request_id_slack = await create_test_request()
    request_id_api = await create_test_request()

    # Execute via Slack
    await slack_handlers._process_approval(request_id_slack, approver_email, notes)

    # Execute via API
    await approval_service.approve_request(request_id_api, approver_email, notes)

    # Assert: DB state is identical
    slack_state = await get_request_state(request_id_slack)
    api_state = await get_request_state(request_id_api)

    assert slack_state == api_state
    assert slack_state["status"] == "APPROVED"
    assert slack_state["approver_email"] == approver_email
```

Similar tests for:
- Decline flow
- Escalation flow
- Submission flow

**Verification Gate**:
```bash
pytest tests/integration/test_approval_dual_path.py -v
pytest tests/integration/test_slack_*.py -v
# All must pass
```

### 3.5 Refactor handlers.py Structure

**Goal**: Slim handlers to ~800 LOC (down from 3,192 LOC).

**Handler responsibility** (after refactor):
1. Parse Slack payload
2. Extract user/request identifiers
3. Call service layer
4. Format Slack response

**Example refactored handler**:
```python
# BEFORE (~120 LOC in handler)
async def _process_approval(self, request_id, approver_email, notes):
    async with get_session() as session:
        # ... 80 LOC of business logic ...
        pad_request = await db_tools.get_pad_request_by_id(session, request_id)
        # ... validation ...
        # ... status update ...
        # ... notifications ...

# AFTER (~20 LOC in handler)
async def _process_approval(self, request_id, approver_email, notes):
    """Handle Slack approval button - delegates to service layer."""
    result = await self.approval_service.process_slack_approval(
        request_id=request_id,
        approver_email=approver_email,
        notes=notes
    )

    if result["success"]:
        return self._format_approval_success_message(result)
    else:
        return self._format_error_message(result["error"])
```

**Expected LOC Reduction**:
- handlers.py: 3,192 → ~1,200 LOC (60% reduction)
- Business logic moved to: `approval_service.py`, `submission_service.py`, `monitoring_service.py`

**Final Phase 3 Verification Gate**:
```bash
# All tests must pass
pytest tests/ -v --tb=short

# No direct repository imports in handlers
grep "from pa_dealing.db.repository import\|from pa_dealing.db import repository\|db_tools\." \
  src/pa_dealing/agents/slack/handlers.py
# Should return 0 results

# Line count reduced
wc -l src/pa_dealing/agents/slack/handlers.py  # ~1200 LOC (down from 3192)

# Integration tests pass
pytest tests/integration/test_slack_*.py -v
pytest tests/integration/test_approval_dual_path.py -v
```

**Rollback Strategy for Phase 3**:
- Restore original handlers.py from git
- Service layer changes in Phase 2 are backward compatible

---

## Cross-Phase Verification

**After completing ALL phases**:

```bash
# 1. Line count verification - no module exceeds 1200 LOC
find src/pa_dealing -name "*.py" -type f -exec wc -l {} \; | sort -n -r | head -20

# Expected largest modules:
# - monitoring_service.py: ~1100 LOC (acceptable)
# - pad_repository.py: ~1600 LOC (future split candidate)
# - handlers.py: ~1200 LOC (down from 3192)

# 2. All tests pass
pytest tests/ -v --tb=short --maxfail=5
# Expected: 148 test files, 100% pass rate

# 3. No circular imports
python -c "import pa_dealing; print('OK')"

# 4. Docker build succeeds
docker build -t pa-dealing:test .
docker run --rm pa-dealing:test pytest tests/unit -v

# 5. API server starts
uvicorn pa_dealing.api.main:app --reload
# Manual smoke test: Submit request, approve, check dashboard

# 6. Slack integration works
# Manual test: Submit via Slack, approve via Slack, verify DB state
```

---

## Test Migration Strategy

**Test Update Patterns**:

### Pattern 1: Direct repository imports (24 test files)
```python
# BEFORE
from pa_dealing.db.repository import get_employee_by_email, submit_pad_request

# AFTER
from pa_dealing.db.employee_repository import get_employee_by_email
from pa_dealing.db.pad_repository import submit_pad_request
```

### Pattern 2: Service layer tests (5 test files)
```python
# BEFORE
from pa_dealing.services.pad_service import PADService

# AFTER (use specific service)
from pa_dealing.services import SubmissionService, ApprovalService
```

### Pattern 3: Mock/patch updates
```python
# BEFORE
@patch("pa_dealing.db.repository.get_employee_by_email")

# AFTER
@patch("pa_dealing.db.employee_repository.get_employee_by_email")
```

**Test Files Priority**:

1. **Unit tests** (update imports only):
   - `tests/unit/test_audit_events.py`
   - `tests/unit/test_conflict_detection.py`
   - `tests/unit/test_contract_note_*.py`
   - `tests/unit/test_dashboard_*.py`

2. **Integration tests** (may need logic updates):
   - `tests/integration/test_database_tools.py`
   - `tests/integration/test_external_resolution_integration.py`
   - `tests/integration/test_security_*.py`

3. **E2E tests** (should pass unchanged if backward compat maintained):
   - `tests/e2e/test_ux_overhaul_journey.py`

**Test Verification After Each Phase**:
```bash
# Phase 1 complete
pytest tests/unit -v
pytest tests/integration -v
# Fix any failures before Phase 2

# Phase 2 complete
pytest tests/unit -v
pytest tests/integration -v
pytest tests/e2e -v
# Fix any failures before Phase 3

# Phase 3 complete
pytest tests/ -v --tb=short
# All 148 test files must pass
```

---

## Rollback Procedures

### Rollback Phase 1 (Repository Split)
```bash
# If in step 1.1-1.4 (file creation)
rm src/pa_dealing/db/employee_repository.py
rm src/pa_dealing/db/instrument_repository.py
rm src/pa_dealing/db/compliance_repository.py
git checkout src/pa_dealing/db/__init__.py

# If in step 1.5 (caller updates) - revert individual files
git checkout src/pa_dealing/services/pad_service.py
# ... repeat for each updated file

# If in step 1.6 (removed re-exports) - restore re-exports
git checkout src/pa_dealing/db/__init__.py
```

### Rollback Phase 2 (Service Split)
```bash
# Remove new service files
rm src/pa_dealing/services/submission_service.py
rm src/pa_dealing/services/approval_service.py
rm src/pa_dealing/services/execution_service.py
rm src/pa_dealing/services/monitoring_service.py

# Restore original PADService
git checkout src/pa_dealing/services/pad_service.py
git checkout src/pa_dealing/services/__init__.py

# Restore API dependencies
git checkout src/pa_dealing/api/dependencies.py
git checkout src/pa_dealing/api/routes/*.py
```

### Rollback Phase 3 (Handler Refactor)
```bash
# Restore original handlers
git checkout src/pa_dealing/agents/slack/handlers.py

# Service layer changes from Phase 2 are backward compatible - no rollback needed
```

---

## Success Metrics

**Quantitative**:
- ✅ repository.py split into 4 modules, largest <1600 LOC
- ✅ pad_service.py split into 4 services, largest <1200 LOC
- ✅ handlers.py reduced from 3,192 → ~1,200 LOC (60% reduction)
- ✅ All 148 test files pass
- ✅ No circular import errors
- ✅ Docker build succeeds

**Qualitative**:
- ✅ Clear domain boundaries (employee, instrument, compliance, pad)
- ✅ Single Responsibility Principle: each module has one concern
- ✅ Easier to test: focused unit tests per domain
- ✅ Easier to onboard: new devs can understand one domain at a time
- ✅ Reduced git conflicts: parallel work on different domains
- ✅ Foundation for microservices: clear service boundaries

---

## Risk Mitigation

### Risk 1: Breaking Changes During Transition
**Mitigation**: Re-export strategy in `__init__.py` maintains backward compatibility.

### Risk 2: Test Failures After Split
**Mitigation**: Phase gates ensure all tests pass before proceeding.

### Risk 3: Circular Import Dependencies
**Mitigation**: Cross-repository imports only flow one direction (pad → compliance → instrument, employee).

### Risk 4: Performance Regression
**Mitigation**: No architectural changes to DB queries, only code organization.

### Risk 5: Incomplete Migration
**Mitigation**: Automated checks (`grep` for old imports) before removing re-exports.

---

## Future Work (Post-Track)

### Phase 4: Further Split Large Modules (Future Track)
- `pad_repository.py` (~1600 LOC) → split into `pad_crud.py`, `execution_tracking.py`, `contract_notes.py`, `audit_events.py`
- `monitoring_service.py` (~1100 LOC) → split into `dashboard_service.py`, `reporting_service.py`, `breach_service.py`
- `_search_instruments` (404 LOC) → decompose into smaller functions

### Phase 5: Service Layer Standardization (Future Track)
- Remove `PADService` facade entirely
- Standardize service constructor patterns
- Add service factory for dependency injection

### Phase 6: API Route Restructuring (Future Track)
- Group routes by domain (employee, instrument, compliance, pad)
- Use service layer consistently across all routes

---

## Appendix: Function Catalog

### repository.py Functions by Domain

**Employee Domain** (3 functions, ~114 LOC):
- `get_employee_by_email` (L65-106)
- `get_employee_by_id` (L107-131)
- `get_manager_chain` (L133-177)

**Instrument Domain** (7 functions, ~542 LOC):
- `search_bloomberg` (L1610-1635)
- `search_map_inst_symbol` (L1637-1659)
- `search_product` (L1661-1690)
- `resolve_instrument_identity` (L1692-1737)
- `search_instruments` (L1739-1745)
- `_search_instruments` (L1747-2151) - 404 LOC!
- `_resolve_desk_name` (L2153-2191)

**Compliance Domain** (12 functions, ~722 LOC):
- `check_mako_positions` (L405-465)
- `check_restricted_list_comprehensive` (L467-557)
- `check_restricted_list` (L559-572)
- `_lookup_security_by_identifier` (L574-598)
- `check_holding_period` (L600-700)
- `check_employee_position` (L702-774)
- `get_all_employee_positions` (L776-872)
- `get_breaches` (L874-909)
- `get_mako_position_info` (L2193-2306)
- `get_employee_trade_history` (L2308-2390)
- `calculate_conflict_risk` (L2392-2527) - 135 LOC
- `get_compliance_config` (L1592-1608)

**PAD Domain** (24 functions, ~1,575 LOC):
- PAD CRUD: `get_or_create_security`, `get_all_requests`, `get_pad_requests`, `_request_to_info`, `get_recent_pad_requests`, `get_pad_request_by_thread`, `get_pad_request_by_id`, `get_recent_request_count`
- Submission: `pre_trade_check`, `submit_pad_request`, `check_recent_executed_buy`
- Approval: `record_decision_outcome`, `update_pad_status`, `create_auto_approval`
- Execution: `record_execution`
- Contract Notes: `create_contract_note_upload`, `get_contract_note_history`, `get_active_contract_note`, `get_trade_history`
- Audit: `insert_audit_event`, `get_recent_audit_events`, `get_audit_event_by_id`, `get_audit_event_stats`, `get_audit_event_sparkline`

### pad_service.py Methods by Domain

**Submission Domain** (2 methods, ~109 LOC):
- `submit_request` (L331-349)
- `_submit_request_with_session` (L350-439) - internal

**Approval Domain** (4 methods, ~390 LOC):
- `approve_request` (L441-510)
- `decline_request` (L512-576)
- `escalate_to_smf16` (L832-905)
- `get_pending_approvals` (L1432-1564) - 133 LOC

**Execution Domain** (3 methods, ~174 LOC):
- `record_execution` (L578-751) - 173 LOC
- `manual_chase` (L753-830)
- `get_execution_tracking` (L1628-1713)

**Monitoring Domain** (~30 methods, ~1,000 LOC):
- Dashboard: `get_recent_activity`, `get_dashboard_summary_counts`, `get_enhanced_dashboard_summary`, `get_sparkline_data`, `_state_based_sparkline`, `get_request_statistics`, `get_activity_events`, `get_activity_event_detail`
- Reporting: `get_activity_report`, `get_breach_report`
- Breaches: `get_breaches`, `create_breach`, `get_active_breaches`, `resolve_breach`, `get_mako_conflicts`
- Lookups: `get_requests_by_status`, `get_all_requests`, `get_requests`, `get_holding_period_calendar`
- Audit: `search_audit_log`, `get_audit_actors`, `get_audit_entity_ids`, `get_audit_employees`

**Employee/Instrument Delegation** (7 methods, delegated):
- `get_employee`, `get_employee_by_id`, `get_manager_chain`
- `search_instruments`, `resolve_instrument_identity`, `lookup_instrument`
- `get_employee_requests`, `get_employee_positions`

---

## Appendix: Import Dependency Map

**Current State** (Phase 0):
```
API Routes → PADService → repository.py (GOD MODULE)
Slack Handlers → repository.py (DIRECT - BAD!)
Agents → repository.py
```

**Target State** (Phase 3 Complete):
```
API Routes → Domain Services (Submission, Approval, Execution, Monitoring)
  ↓
Domain Services → Domain Repositories (employee, instrument, compliance, pad)
  ↓
Domain Repositories → SQLAlchemy Models

Slack Handlers → Domain Services (NO DIRECT DB ACCESS)
Agents → Domain Services
```

**Dependency Flow** (allowed):
```
pad_repository → compliance_repository (for pre-trade checks)
pad_repository → instrument_repository (for instrument resolution)
pad_repository → employee_repository (for employee lookup)

compliance_repository → instrument_repository (for security lookup)
compliance_repository → employee_repository (for position lookup)

instrument_repository → (no dependencies)
employee_repository → (no dependencies)
```

**Forbidden Dependencies** (circular):
```
employee_repository → pad_repository (FORBIDDEN)
instrument_repository → compliance_repository (FORBIDDEN)
```

---

## Timeline Estimate

**Phase 1: Repository Split** (3-4 weeks)
- Week 1: Steps 1.1-1.2 (employee repository)
- Week 2: Steps 1.3-1.4 (instrument, compliance, pad repositories)
- Week 3: Step 1.5 (update callers)
- Week 4: Step 1.6 (remove re-exports) + buffer

**Phase 2: Service Split** (3-4 weeks)
- Week 1: Steps 2.1-2.2 (submission, approval services)
- Week 2: Steps 2.3-2.4 (execution, monitoring services)
- Week 3: Step 2.5 (update API callers)
- Week 4: Integration testing + buffer

**Phase 3: Handler Refactor** (2-3 weeks)
- Week 1: Steps 3.1-3.3 (extract business logic)
- Week 2: Step 3.4 (dual-path tests) + Step 3.5 (refactor structure)
- Week 3: Integration testing + buffer

**Total: 8-11 weeks (2-3 months)**

**Dependencies**:
- Phase 2 depends on Phase 1 (services use new repositories)
- Phase 3 depends on Phase 2 (handlers use new services)
- Each phase has independent verification gates

---

**END OF PLAN**
