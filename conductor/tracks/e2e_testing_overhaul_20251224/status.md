# E2E Testing Infrastructure Overhaul - Status

## Track Information
- **Created**: 2024-12-24
- **Status**: PLANNING COMPLETE
- **Priority**: HIGH

## Problem Summary

The E2E tests pass in CI but fail to catch real bugs because:
1. Mock Slack server is too permissive (accepts any input, returns hardcoded responses)
2. Authorization checks are bypassed via patching
3. Timing-based tests are flaky (`asyncio.sleep()` instead of event-based waiting)
4. Critical business logic paths lack coverage (auto-approve, holding period edge cases)
5. Assertions use fragile substring matching

## Work Items Overview

| Area | Items | Priority | Status |
|------|-------|----------|--------|
| 1. Mock Server Fidelity | 1.1-1.5 | CRITICAL/HIGH | Not Started |
| 2. Timing-Based Tests | 2.1-2.3 | HIGH | Not Started |
| 3. Authorization | 3.1-3.3 | HIGH | Not Started |
| 4. Database Isolation | 4.1-4.2 | CRITICAL | Not Started |
| 5. Missing Coverage | 5.1-5.4 | MEDIUM | Not Started |
| 6. Assertions | 6.1-6.2 | MEDIUM | Not Started |

## Implementation Phases

### Phase 1: Foundation (CRITICAL)
- [x] 1.1 Stateful user registry in mock server
- [x] 1.2 Per-user DM channel tracking
- [x] 2.1 Async wait utilities
- [x] 4.1 Transaction rollback isolation

### Phase 2: Mock Fidelity (HIGH)
- [x] 1.3 Block Kit schema validation
- [x] 1.4 Error simulation endpoints
- [x] 1.5 Rate limit simulation

### Phase 3: Remove Patches (HIGH)
- [x] 3.1 Role-aware test fixtures
- [x] 3.2 Remove authorization patches
- [x] 3.3 Remove email resolution patches

### Phase 4: Coverage (MEDIUM)
- [x] 5.1 Auto-approve path tests
- [x] 5.2 Holding period edge case tests
- [x] 5.3 Concurrent submission tests
- [x] 5.4 Document processing error tests

### Phase 5: Assertions (MEDIUM)
- [x] 6.1 Structured message matchers
- [x] 6.2 Update existing tests
- [x] 4.2 Data builders/factories

### Phase 6: Timing (LOW)
- [x] 2.2 Update tests to use wait utilities
- [x] 2.3 Event-based synchronization

## Key Files

### New Files
- `tests/utils/async_helpers.py` - Async wait utilities
- `tests/utils/message_matchers.py` - Structured assertions
- `tests/factories.py` - Test data builders
- `tests/test_auto_approve.py` - Auto-approve tests
- `tests/test_holding_period.py` - Holding period tests
- `tests/test_concurrency.py` - Concurrency tests
- `tests/test_document_errors.py` - Document error tests
- `integrations/slack-mock/.../validation/block_kit.py` - Block Kit validator

### Modified Files
- `integrations/slack-mock/slack_server_mock/slack_server/slack_server.py`
- `integrations/slack-mock/slack_server_mock/servers/http/handler.py`
- `integrations/slack-mock/slack_server_mock/actor/actor.py`
- `tests/conftest.py`
- `tests/test_e2e_scenarios.py`
- `tests/test_slack_mock.py`

## Success Metrics

1. No `asyncio.sleep()` calls > 0.2s in assertions
2. No `patch(...has_role...)` calls in tests
3. No `patch(..._get_slack_user_email...)` calls in tests
4. Mock rejects invalid Block Kit structures
5. 3+ authorization failure tests
6. 5+ holding period edge case tests
7. Concurrent submission tests pass reliably
8. 30%+ test execution time improvement
9. CI runs are deterministic (no flaky failures)
10. All `find_message` replaced with structured matchers

## Notes

See `plan.md` for detailed implementation instructions for each work item.
