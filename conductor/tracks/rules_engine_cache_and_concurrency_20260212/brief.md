# Track Brief: Rules Engine Cache Invalidation & Concurrency Fix

**Goal**: Make rule changes take effect immediately and fix concurrent request isolation.

**Source**: `.autopsy/ARCHITECTURE_REPORT.md` Section 5 - verified: `invalidate()` defined but never called; global `_scorer` singleton.

## Scope
2 fixes: (1) Add `invalidate()` calls to `update_rule()` and `toggle_rule()` in service.py. (2) Remove global `_scorer` singleton, use per-request `SimplifiedRiskScorer(config)` instances.

## Key Files
- `services/rules_engine/registry.py` (cache with invalidate method)
- `services/rules_engine/service.py` (write operations)
- `agents/orchestrator/risk_scoring.py` (singleton scorer)
- `agents/orchestrator/risk_scoring_service.py` (caller)

## Effort Estimate
S (< 1 week) - straightforward code changes with integration tests
