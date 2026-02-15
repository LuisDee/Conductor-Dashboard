# Spec: Rules Engine Cache Invalidation & Concurrency Fix

## Problem Statement
Two interrelated issues in the rules engine scoring pipeline: (1) PADRuleRegistry cache is never invalidated on write operations, causing up to 5-minute stale config windows, and (2) the singleton risk scorer is shared across concurrent async requests, creating a race condition.

## Source
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "CRITICAL: Rules Engine Cache Not Invalidated on Write" + "HIGH: Singleton Risk Scorer Retains Stale Config"
- `.autopsy/REVIEW_REPORT.md` - Additional CRITICAL patterns

## Findings (Verified Against Code)

### 1. Cache Not Invalidated on Write (CRITICAL)
- **Registry:** `services/rules_engine/registry.py` lines 108-111 defines `invalidate()` method
- **Service:** `services/rules_engine/service.py` - `update_rule()` (lines 106-183) and `toggle_rule()` (lines 186-244) commit changes but NEVER call `invalidate()`
- **Grep confirms:** `invalidate()` is defined but never called anywhere in the codebase
- **TTL:** 300 seconds (5 minutes) in registry.py
- **Impact:** Rule changes take up to 5 minutes to propagate. Compliance officer disables a rule but orchestrator keeps applying old config.

### 2. Singleton Risk Scorer Race Condition (HIGH)
- **File:** `agents/orchestrator/risk_scoring.py` lines 824-833
- **Pattern:** Global `_scorer` variable, `get_risk_scorer(config)` replaces singleton on every call with `config is not None`
- **Called from:** `risk_scoring_service.py` line 174: `scorer = get_risk_scorer(config)`
- **Impact:** Concurrent requests overwrite each other's scorer config. One request's risk scoring may use another request's (possibly default) config.

## Requirements
1. Call `PADRuleRegistry.invalidate()` after `update_rule()` commits (service.py)
2. Call `PADRuleRegistry.invalidate()` after `toggle_rule()` commits (service.py)
3. Remove global `_scorer` singleton pattern in risk_scoring.py
4. Create new `SimplifiedRiskScorer(config)` per request instead of reusing singleton
5. Add integration test: update rule -> immediately score request -> verify new config used
6. Add integration test: concurrent requests with different configs don't interfere

## Acceptance Criteria
- [ ] Rule changes take effect immediately (not delayed by cache TTL)
- [ ] `invalidate()` called after every rule write operation
- [ ] No global `_scorer` singleton; per-request scorer instances
- [ ] Concurrent requests isolated from each other's scoring config
- [ ] Integration tests verify both fixes
- [ ] All existing tests pass
