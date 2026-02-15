# Spec: God Module Decomposition

## Problem Statement
5 files exceed 2,300 LOC each, making them difficult to understand, test, review, and modify without introducing regressions. Business logic embedded in Slack handlers duplicates API route logic and creates maintenance burden.

## Source
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "HIGH: God Modules Exceed Maintainability Thresholds" + "HIGH: Business Logic in Slack Handlers"
- `.autopsy/REVIEW_REPORT.md` - HIGH findings in batch-3 (repository) and batch-7 (handlers)

## Findings (Verified Against Code)

### God Module Sizes
| Module | LOC | Location |
|--------|-----|----------|
| `handlers.py` | 3,192 | `agents/slack/handlers.py` |
| `repository.py` | 2,953 | `db/repository.py` |
| `ui.py` | 2,784 | `agents/slack/ui.py` |
| `pad_service.py` | 2,748 | `services/pad_service.py` |
| `chatbot.py` | 2,334 | `agents/slack/chatbot.py` |

### Business Logic in Handlers
- **File:** `agents/slack/handlers.py` lines 1058, 1113-1123
- `_process_approval()` makes direct repository calls (`db_tools.get_pad_request_by_id`, `db_tools.update_pad_status`) instead of going through PADService
- Approval logic (auth checks, duplicate detection, status transitions) lives in handler, not service layer
- Same logic must exist in API routes for dashboard-initiated approvals

## Requirements

### Phase 1: Repository Split
Split `repository.py` (2,953 LOC, ~43 functions) into:
1. `employee_repository.py` - Employee lookup, org-chart, visibility
2. `pad_repository.py` - PAD CRUD, approval, execution, status transitions
3. `instrument_repository.py` - Instrument search, restricted list, holdings
4. `compliance_repository.py` - Holding periods, conflicts, breaches, audit
5. `db/session.py` - Shared session and engine utilities

### Phase 2: PAD Service Split
Split `pad_service.py` (2,748 LOC) into:
1. `submission_service.py` - Trade submission, validation
2. `approval_service.py` - Approval workflows, decline, status transitions
3. `execution_service.py` - Execution tracking, contract note linking
4. `monitoring_service.py` - Overdue detection, breach detection, notifications

### Phase 3: Slack Handler Extraction
Extract business logic from `handlers.py` (3,192 LOC):
1. Move approval/decline logic into `approval_service.py` (from Phase 2)
2. Slim handlers to: parse Slack payload -> call service -> format Slack response
3. Verify both Slack and API paths call identical service methods

### Out of Scope (Future)
- `ui.py` (2,784 LOC) - Slack Block Kit templates, cosmetic not logic
- `chatbot.py` (2,334 LOC) - Conversational flow, complex but focused

## Acceptance Criteria
- [ ] No module exceeds 800 LOC (target: <500 LOC per module)
- [ ] All imports updated across callers
- [ ] Business logic in Slack handlers extracted to service layer
- [ ] Both Slack and API approval paths use same service methods
- [ ] All existing tests pass (zero regressions)
- [ ] New focused test files for each extracted module
