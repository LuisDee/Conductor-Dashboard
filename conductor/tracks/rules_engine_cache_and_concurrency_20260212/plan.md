# Rules Engine Cache Invalidation & Risk Scorer Concurrency Fix

## Track Overview

**Track ID**: `rules_engine_cache_and_concurrency_20260212`
**Status**: `planned`
**Dependencies**: `rules_engine_ui_refactor_20260211` (in_progress)

### Problem Statement

This track fixes two critical bugs in the rules engine:

1. **Cache Invalidation Bug**: When rules are updated via `update_rule()` or `toggle_rule()`, the `PADRuleRegistry` cache is never invalidated, causing stale rule values to persist for up to 5 minutes (TTL duration). This means rule changes don't take effect immediately, creating compliance risk.

2. **Risk Scorer Concurrency Bug**: The `SimplifiedRiskScorer` is implemented as a global singleton with mutable configuration. When concurrent requests use different configs, they overwrite each other's scorer instances mid-flight, causing race conditions and incorrect risk assessments.

### Impact

- **Cache Bug**: Rule changes take up to 5 minutes to propagate, causing inconsistent behavior
- **Concurrency Bug**: Concurrent risk scoring requests with different configs produce incorrect results due to shared state

---

## Code Context

### Affected Files

1. `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/rules_engine/registry.py`
   - `PADRuleRegistry` class (lines 27-112)
   - `invalidate()` method (lines 108-111)

2. `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/rules_engine/service.py`
   - `update_rule()` function (lines 106-183)
   - `toggle_rule()` function (lines 186-244)

3. `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/agents/orchestrator/risk_scoring.py`
   - `_scorer` global singleton (line 824)
   - `get_risk_scorer()` factory (lines 828-833)
   - `SimplifiedRiskScorer` class (lines 199-820)

4. `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/agents/orchestrator/risk_scoring_service.py`
   - `score_pad_request()` function (line 174: caller of `get_risk_scorer()`)

### Existing Registry Invalidation Method

**File**: `src/pa_dealing/services/rules_engine/registry.py`

```python
107→    async def is_enabled(
108→        self, session: AsyncSession, rule_id: str
109→    ) -> bool:
110→        """Check whether a rule is enabled."""
111→        await self._ensure_loaded(session)
112→        rule = self._cache.get(rule_id)
113→        return bool(rule and rule["enabled"])
114→
115→    def invalidate(self) -> None:
116→        """Force the next access to reload from the database."""
117→        self._cache.clear()
118→        self._cache_time = 0.0
119→
120→
121→# ---------------------------------------------------------------------------
122→# Singleton accessor
123→# ---------------------------------------------------------------------------
```

**Analysis**: The `invalidate()` method exists and works correctly (clears cache dict, resets timestamp). The bug is that it's never called after rule mutations.

---

## Phase 1: Add Cache Invalidation to Rule Mutation Operations

### Objective

Call `PADRuleRegistry.invalidate()` after successful commits in `update_rule()` and `toggle_rule()`.

### Changes

#### File: `src/pa_dealing/services/rules_engine/service.py`

**Change 1.1: Add invalidation to `update_rule()`**

**Before** (lines 164-183):
```python
163→    # Cross-field validation: ensure range_slider handles remain ordered
164→    _validate_range_slider_ordering(rule_id, old_config, updates)
165→
166→    # Replace the config entirely (avoids mutation-tracking issues)
167→    rule.config = new_config
168→    rule.version = rule.version + 1
169→    rule.updated_at = datetime.utcnow()
170→    rule.updated_by = changed_by
171→    flag_modified(rule, "config")
172→
173→    await session.flush()
174→    await session.commit()
175→
176→    log.info(
177→        "pad_rule_updated",
178→        rule_id=rule_id,
179→        fields=list(updates.keys()),
180→        version=rule.version,
181→        changed_by=changed_by,
182→    )
183→
184→    return _rule_to_dict(rule)
```

**After** (lines 164-189):
```python
163→    # Cross-field validation: ensure range_slider handles remain ordered
164→    _validate_range_slider_ordering(rule_id, old_config, updates)
165→
166→    # Replace the config entirely (avoids mutation-tracking issues)
167→    rule.config = new_config
168→    rule.version = rule.version + 1
169→    rule.updated_at = datetime.utcnow()
170→    rule.updated_by = changed_by
171→    flag_modified(rule, "config")
172→
173→    await session.flush()
174→    await session.commit()
175→
176→    # Invalidate registry cache to ensure immediate propagation
177→    from .registry import get_rule_registry
178→    registry = await get_rule_registry()
179→    registry.invalidate()
180→
181→    log.info(
182→        "pad_rule_updated",
183→        rule_id=rule_id,
184→        fields=list(updates.keys()),
185→        version=rule.version,
186→        changed_by=changed_by,
187→    )
188→
189→    return _rule_to_dict(rule)
```

**Rationale**:
- Import placed after commit to avoid circular imports at module level
- `invalidate()` called immediately after commit succeeds
- If commit fails (exception), invalidation is skipped (correct behavior)
- No parameters needed - invalidation is idempotent

**Change 1.2: Add invalidation to `toggle_rule()`**

**Before** (lines 229-244):
```python
229→    rule.enabled = enabled
230→    rule.version = rule.version + 1
231→    rule.updated_at = datetime.utcnow()
232→    rule.updated_by = changed_by
233→
234→    await session.flush()
235→    await session.commit()
236→
237→    log.info(
238→        "pad_rule_toggled",
239→        rule_id=rule_id,
240→        enabled=enabled,
241→        changed_by=changed_by,
242→    )
243→
244→    return _rule_to_dict(rule)
```

**After** (lines 229-250):
```python
229→    rule.enabled = enabled
230→    rule.version = rule.version + 1
231→    rule.updated_at = datetime.utcnow()
232→    rule.updated_by = changed_by
233→
234→    await session.flush()
235→    await session.commit()
236→
237→    # Invalidate registry cache to ensure immediate propagation
238→    from .registry import get_rule_registry
239→    registry = await get_rule_registry()
240→    registry.invalidate()
241→
242→    log.info(
243→        "pad_rule_toggled",
244→        rule_id=rule_id,
245→        enabled=enabled,
246→        changed_by=changed_by,
247→    )
248→
249→    return _rule_to_dict(rule)
```

**Rationale**: Same as Change 1.1 - ensures `enabled` flag changes take effect immediately.

### Caller Impact Analysis

**Q**: Who calls `update_rule()` and `toggle_rule()`?
**A**: These are called by:
1. **REST API endpoints** (web UI rule editor)
2. **Admin scripts** (bulk rule updates)

**Breaking changes**: None. These are internal service functions with no external API contract changes.

**Behavioral changes**:
- Rules now take effect immediately instead of within 5 minutes
- This is the **correct** behavior - faster propagation is always better for compliance

### Rollback Strategy

**If Phase 1 causes issues**:
1. Remove the 3 new lines from each function (lines 176-178 in `update_rule()`, lines 237-239 in `toggle_rule()`)
2. Redeploy
3. Cache will continue using TTL-based invalidation (5 minutes)

**Rollback risk**: Very low - `invalidate()` is a simple, idempotent operation.

---

## Phase 2: Add Integration Test for Cache Invalidation

### Objective

Verify that rule updates immediately propagate to the registry cache.

### Test Specification

**File**: `tests/integration/test_rules_engine_cache_invalidation.py` (new file)

```python
"""Integration test: Rule updates invalidate registry cache immediately.

This test verifies the fix for the cache invalidation bug where updates
to rules via update_rule() and toggle_rule() did not invalidate the
PADRuleRegistry cache, causing stale values to persist for up to 5 minutes.
"""

import pytest
from sqlalchemy.ext.asyncio import AsyncSession

from pa_dealing.db.models.pad_rule import PadRule
from pa_dealing.services.rules_engine.registry import get_rule_registry, reset_rule_registry
from pa_dealing.services.rules_engine.service import update_rule, toggle_rule


@pytest.fixture(autouse=True)
def reset_registry():
    """Reset the singleton registry before each test."""
    reset_rule_registry()
    yield
    reset_rule_registry()


@pytest.mark.asyncio
async def test_update_rule_invalidates_cache(async_session: AsyncSession):
    """Test that update_rule() invalidates the registry cache immediately.

    Steps:
    1. Create a rule in the database
    2. Load it into the registry cache via get_value()
    3. Update the rule via update_rule()
    4. Verify registry immediately returns the new value (not cached old value)
    """
    # 1. Seed database with a rule
    rule = PadRule(
        id="RF-002",
        rule_type="risk_factor",
        name="Mako Traded",
        category="Conflict",
        description="Mako activity lookback window",
        config={"lookbackDays": 30},
        enabled=True,
        version=1,
    )
    async_session.add(rule)
    await async_session.commit()

    # 2. Load rule into cache
    registry = await get_rule_registry()
    value = await registry.get_value(async_session, "RF-002", "lookbackDays")
    assert value == 30, "Initial value should be 30"

    # 3. Update rule via service (should invalidate cache)
    await update_rule(
        session=async_session,
        rule_id="RF-002",
        updates={"lookbackDays": 90},
        changed_by="test@mako.com",
        reason="Integration test",
    )

    # 4. Verify registry immediately reflects new value (no TTL wait)
    new_value = await registry.get_value(async_session, "RF-002", "lookbackDays")
    assert new_value == 90, "Registry should immediately reflect updated value (cache was invalidated)"


@pytest.mark.asyncio
async def test_toggle_rule_invalidates_cache(async_session: AsyncSession):
    """Test that toggle_rule() invalidates the registry cache immediately.

    Steps:
    1. Create an enabled rule
    2. Load it into the registry cache via is_enabled()
    3. Disable the rule via toggle_rule()
    4. Verify registry immediately returns False for is_enabled()
    """
    # 1. Seed database
    rule = PadRule(
        id="RF-005",
        rule_type="risk_factor",
        name="Position Size",
        category="Material Trade",
        description="Position size thresholds",
        config={"lowThreshold": 50000, "highThreshold": 100000},
        enabled=True,
        version=1,
    )
    async_session.add(rule)
    await async_session.commit()

    # 2. Load rule into cache
    registry = await get_rule_registry()
    is_enabled = await registry.is_enabled(async_session, "RF-005")
    assert is_enabled is True, "Rule should initially be enabled"

    # 3. Disable rule via service (should invalidate cache)
    await toggle_rule(
        session=async_session,
        rule_id="RF-005",
        enabled=False,
        changed_by="test@mako.com",
        reason="Integration test",
    )

    # 4. Verify registry immediately reflects disabled state
    is_enabled_after = await registry.is_enabled(async_session, "RF-005")
    assert is_enabled_after is False, "Registry should immediately reflect disabled state (cache was invalidated)"


@pytest.mark.asyncio
async def test_multiple_updates_invalidate_cache(async_session: AsyncSession):
    """Test that multiple rapid updates all invalidate cache correctly."""
    # 1. Seed database
    rule = PadRule(
        id="RF-007",
        rule_type="compliance",
        name="Holding Period",
        category="Short Term Trading",
        description="Minimum holding period",
        config={"minimumHoldDays": 30},
        enabled=True,
        version=1,
    )
    async_session.add(rule)
    await async_session.commit()

    registry = await get_rule_registry()

    # 2. Rapid-fire updates
    await update_rule(async_session, "RF-007", {"minimumHoldDays": 45}, "test@mako.com", "Update 1")
    value1 = await registry.get_value(async_session, "RF-007", "minimumHoldDays")
    assert value1 == 45

    await update_rule(async_session, "RF-007", {"minimumHoldDays": 60}, "test@mako.com", "Update 2")
    value2 = await registry.get_value(async_session, "RF-007", "minimumHoldDays")
    assert value2 == 60

    await update_rule(async_session, "RF-007", {"minimumHoldDays": 90}, "test@mako.com", "Update 3")
    value3 = await registry.get_value(async_session, "RF-007", "minimumHoldDays")
    assert value3 == 90, "All rapid updates should invalidate cache and reflect latest value"
```

**Fixtures Required**:
- `async_session`: AsyncSession fixture (already exists in conftest.py)

**Test Strategy**:
1. Test `update_rule()` invalidation
2. Test `toggle_rule()` invalidation
3. Test rapid-fire updates (stress test)

**Success Criteria**:
- All tests pass
- Coverage: 100% of new invalidation code paths

### Rollback Strategy

**If Phase 2 tests fail**:
1. Investigate failure - if it's a test bug, fix the test
2. If it reveals a regression in Phase 1, rollback Phase 1 changes
3. Tests can be safely removed without affecting production code

---

## Phase 3: Remove Singleton Pattern from Risk Scorer

### Objective

Replace the global singleton `_scorer` with per-request instance creation to eliminate concurrency bugs.

### Concurrency Bug Explanation

**Current Code** (`risk_scoring.py` lines 824-833):
```python
824→# Singleton instance
825→_scorer: SimplifiedRiskScorer | None = None
826→
827→
828→def get_risk_scorer(config: RiskScoringConfig | None = None) -> SimplifiedRiskScorer:
829→    """Get the singleton risk scorer instance."""
830→    global _scorer
831→    if _scorer is None or config is not None:
832→        _scorer = SimplifiedRiskScorer(config)
833→    return _scorer
```

**The Bug**:

Consider two concurrent requests:

```
Time →
T0: Request A calls get_risk_scorer(config_A)
    → _scorer = SimplifiedRiskScorer(config_A)
T1: Request B calls get_risk_scorer(config_B)
    → _scorer = SimplifiedRiskScorer(config_B)  # OVERWRITES Request A's scorer!
T2: Request A calls scorer.score_request()
    → Uses config_B instead of config_A! ❌
```

**Why This Happens**:
- Line 831: `if _scorer is None or config is not None`
- When `config is not None`, it **always** replaces the global singleton
- Concurrent requests with different configs clobber each other's scorer instances
- The scorer has mutable state (`self.config`) that gets shared across requests

**Real-World Scenario**:
- Request A: High-risk employee, uses custom config with stricter thresholds
- Request B: Low-risk employee, uses default config
- Request A's risk assessment ends up using Request B's config → **incorrect risk level**

### Changes

#### File: `src/pa_dealing/agents/orchestrator/risk_scoring.py`

**Change 3.1: Remove singleton pattern**

**Before** (lines 820-833):
```python
820→        )
821→
822→
823→
824→# Singleton instance
825→_scorer: SimplifiedRiskScorer | None = None
826→
827→
828→def get_risk_scorer(config: RiskScoringConfig | None = None) -> SimplifiedRiskScorer:
829→    """Get the singleton risk scorer instance."""
830→    global _scorer
831→    if _scorer is None or config is not None:
832→        _scorer = SimplifiedRiskScorer(config)
833→    return _scorer
834→
```

**After** (lines 820-831):
```python
820→        )
821→
822→
823→
824→def get_risk_scorer(config: RiskScoringConfig | None = None) -> SimplifiedRiskScorer:
825→    """Create a new risk scorer instance with the given config.
826→
827→    Note: This function creates a new instance per call to avoid concurrency
828→    issues with shared mutable state. The scorer is stateless except for config,
829→    so instance creation is cheap (no DB access, no network calls).
830→    """
831→    return SimplifiedRiskScorer(config)
832→
```

**Rationale**:
- Removes global `_scorer` variable entirely
- Returns a fresh instance on every call
- No shared mutable state across requests
- Concurrency-safe by design

**Performance Impact**:
- **Negligible**: `SimplifiedRiskScorer.__init__()` is cheap (just stores config, no I/O)
- No database queries or network calls in constructor
- Config is passed in by caller (already loaded)

### Caller Impact Analysis

**File**: `src/pa_dealing/agents/orchestrator/risk_scoring_service.py`

**Current Caller** (line 174):
```python
171→    # 0. Fetch config from DB if not provided
172→    if config is None:
173→        config = await fetch_risk_scoring_config(session)
174→
175→    scorer = get_risk_scorer(config)
```

**Impact**:
- **No change required** - the call signature is identical
- Behavior change: Creates new instance instead of reusing singleton
- Correctness: This is the **correct** behavior (eliminates race conditions)

**Other potential callers**:
```bash
# Search for all calls to get_risk_scorer()
grep -r "get_risk_scorer(" src/
```

Expected: Only `risk_scoring_service.py` calls it. If others exist, verify they don't rely on singleton behavior.

### Rollback Strategy

**If Phase 3 causes issues**:
1. Restore the 9 deleted lines (824-833) containing the singleton pattern
2. Redeploy
3. Scorer will revert to singleton behavior (with original concurrency bug)

**Rollback risk**: Very low - this is a pure refactor with no functional changes to the scoring logic.

---

## Phase 4: Update Caller (Risk Scoring Service)

### Objective

Verify that `score_pad_request()` correctly creates per-request scorer instances (no code changes needed - just verification).

### Analysis

**File**: `src/pa_dealing/agents/orchestrator/risk_scoring_service.py`

**Current Code** (lines 142-174):
```python
142→async def score_pad_request(
143→    session: AsyncSession,
144→    # Trade info
145→    value: float,
146→    buysell: str,
147→    currency: str = "GBP",
148→    # ... other params ...
169→) -> dict[str, Any]:
170→    """Score a PAD request using the new 6-factor system."""
171→    # 0. Fetch config from DB if not provided
172→    if config is None:
173→        config = await fetch_risk_scoring_config(session)
174→
175→    scorer = get_risk_scorer(config)
176→
177→    # ... scoring logic ...
```

**Analysis**:
- Line 172-173: Fetches config from DB (or uses passed-in config)
- Line 175: Calls `get_risk_scorer(config)` → creates new instance
- Each invocation of `score_pad_request()` gets its own scorer instance
- **No code changes needed** - the function is already concurrency-safe

**Verification Checklist**:
- ✅ Config is request-scoped (fetched per request or passed in)
- ✅ Scorer is created per request (line 175)
- ✅ No shared mutable state across requests

### Changes

**No code changes required** - this phase is verification-only.

**Documentation Update** (optional):

Add docstring clarification to `score_pad_request()`:

```python
async def score_pad_request(
    session: AsyncSession,
    # ... params ...
) -> dict[str, Any]:
    """Score a PAD request using the new 6-factor system.

    This function is concurrency-safe: each request gets its own RiskScorer
    instance with its own config, eliminating shared mutable state.

    Args:
        session: Database session for data lookups
        config: Optional risk scoring config (fetched from DB if not provided)
        ... (other params)

    Returns:
        Risk scoring result dict with level, factors, and approval route
    """
```

### Rollback Strategy

No rollback needed - this phase is read-only verification.

---

## Phase 5: Add Concurrency Test

### Objective

Prove that concurrent requests with different configs produce correct, isolated results.

### Test Specification

**File**: `tests/integration/test_risk_scoring_concurrency.py` (new file)

```python
"""Integration test: Risk scorer concurrency isolation.

This test verifies the fix for the singleton concurrency bug where concurrent
requests using get_risk_scorer() with different configs would overwrite each
other's scorer instances, causing incorrect risk assessments.
"""

import asyncio
from decimal import Decimal

import pytest

from pa_dealing.agents.orchestrator.risk_scoring import (
    ApprovalRoute,
    OverallRiskLevel,
    RiskScoringConfig,
    get_risk_scorer,
)


@pytest.mark.asyncio
async def test_concurrent_requests_with_different_configs():
    """Test that concurrent scorer instances don't interfere with each other.

    Scenario:
    - Request A: Uses strict config (low threshold = £10k)
    - Request B: Uses lenient config (low threshold = £500k)
    - Both requests score a £50k trade concurrently
    - Request A should get MEDIUM risk (£50k > £10k)
    - Request B should get LOW risk (£50k < £500k)

    If the singleton bug exists, one request would use the other's config.
    """
    # Config A: Strict thresholds (typical for high-risk employees)
    config_a = RiskScoringConfig(
        position_size_low_threshold=Decimal("10000"),
        position_size_high_threshold=Decimal("50000"),
    )

    # Config B: Lenient thresholds (typical for low-risk employees)
    config_b = RiskScoringConfig(
        position_size_low_threshold=Decimal("500000"),
        position_size_high_threshold=Decimal("1000000"),
    )

    # Shared trade value (in the "gap" between the two configs)
    trade_value = Decimal("50000")

    async def score_with_config_a():
        """Simulate Request A: strict config."""
        scorer = get_risk_scorer(config_a)
        result = scorer.score_request(
            inst_type="equity",
            value_gbp=trade_value,
        )
        return result

    async def score_with_config_b():
        """Simulate Request B: lenient config."""
        scorer = get_risk_scorer(config_b)
        result = scorer.score_request(
            inst_type="equity",
            value_gbp=trade_value,
        )
        return result

    # Run both requests concurrently
    result_a, result_b = await asyncio.gather(
        score_with_config_a(),
        score_with_config_b(),
    )

    # Verify Request A: £50k > £10k low threshold, < £50k high threshold → MEDIUM
    assert result_a.overall_level == OverallRiskLevel.MEDIUM, (
        f"Request A should be MEDIUM (trade £50k > config_a.low_threshold £10k). "
        f"Got: {result_a.overall_level}"
    )

    # Verify Request B: £50k < £500k low threshold → LOW
    assert result_b.overall_level == OverallRiskLevel.LOW, (
        f"Request B should be LOW (trade £50k < config_b.low_threshold £500k). "
        f"Got: {result_b.overall_level}"
    )


@pytest.mark.asyncio
async def test_concurrent_requests_stress_test():
    """Stress test: 100 concurrent requests with different configs."""
    async def score_with_threshold(threshold: int):
        """Score a £75k trade with a custom threshold."""
        config = RiskScoringConfig(
            position_size_low_threshold=Decimal(str(threshold)),
            position_size_high_threshold=Decimal(str(threshold * 10)),
        )
        scorer = get_risk_scorer(config)
        result = scorer.score_request(
            inst_type="equity",
            value_gbp=Decimal("75000"),
        )
        # Return (threshold, result) to verify correctness
        return (threshold, result)

    # Create 100 concurrent requests with different thresholds
    # Half should produce MEDIUM (threshold < £75k), half should produce LOW (threshold > £75k)
    tasks = []
    for i in range(100):
        threshold = 10000 + (i * 2000)  # 10k, 12k, 14k, ..., 208k
        tasks.append(score_with_threshold(threshold))

    results = await asyncio.gather(*tasks)

    # Verify each result used the correct config
    for threshold, result in results:
        if threshold < 75000:
            # Trade value (£75k) exceeds low threshold → MEDIUM
            assert result.overall_level in (OverallRiskLevel.MEDIUM, OverallRiskLevel.HIGH), (
                f"Threshold {threshold}: expected MEDIUM/HIGH, got {result.overall_level}"
            )
        else:
            # Trade value (£75k) below low threshold → LOW
            assert result.overall_level == OverallRiskLevel.LOW, (
                f"Threshold {threshold}: expected LOW, got {result.overall_level}"
            )


@pytest.mark.asyncio
async def test_concurrent_default_and_custom_configs():
    """Test mixing default config (None) and custom configs concurrently."""

    async def score_with_default():
        """Use default config (None)."""
        scorer = get_risk_scorer(None)  # Should use default RiskScoringConfig()
        result = scorer.score_request(
            inst_type="equity",
            value_gbp=Decimal("60000"),
        )
        return result

    async def score_with_custom():
        """Use custom config with very high thresholds."""
        config = RiskScoringConfig(
            position_size_low_threshold=Decimal("1000000"),
            position_size_high_threshold=Decimal("5000000"),
        )
        scorer = get_risk_scorer(config)
        result = scorer.score_request(
            inst_type="equity",
            value_gbp=Decimal("60000"),
        )
        return result

    # Run 10 iterations to catch race conditions
    for _ in range(10):
        result_default, result_custom = await asyncio.gather(
            score_with_default(),
            score_with_custom(),
        )

        # Default config: £60k is between £50k and £100k → MEDIUM
        assert result_default.overall_level == OverallRiskLevel.MEDIUM

        # Custom config: £60k < £1M → LOW
        assert result_custom.overall_level == OverallRiskLevel.LOW


@pytest.mark.asyncio
async def test_scorer_instance_isolation():
    """Test that scorer instances truly don't share state."""
    config1 = RiskScoringConfig(
        position_size_low_threshold=Decimal("10000"),
        mako_lookback_days=30,
    )
    config2 = RiskScoringConfig(
        position_size_low_threshold=Decimal("500000"),
        mako_lookback_days=90,
    )

    scorer1 = get_risk_scorer(config1)
    scorer2 = get_risk_scorer(config2)

    # Verify instances are different
    assert scorer1 is not scorer2, "get_risk_scorer() should return different instances"

    # Verify configs are different
    assert scorer1.config.position_size_low_threshold == Decimal("10000")
    assert scorer2.config.position_size_low_threshold == Decimal("500000")
    assert scorer1.config.mako_lookback_days == 30
    assert scorer2.config.mako_lookback_days == 90

    # Verify mutating one doesn't affect the other
    scorer1.config.mako_lookback_days = 999
    assert scorer2.config.mako_lookback_days == 90, "Configs should be independent"
```

**Test Strategy**:
1. **Basic concurrency test**: Two requests, different configs, verify isolation
2. **Stress test**: 100 concurrent requests with different configs
3. **Mixed default/custom configs**: Verify `None` config doesn't interfere with custom configs
4. **Instance isolation**: Verify instances are truly separate (no shared state)

**Success Criteria**:
- All tests pass
- No race conditions detected across 10+ iterations
- Each request uses its own config (verified by assertion on risk level)

### Rollback Strategy

**If Phase 5 tests fail**:
1. If tests reveal a regression in Phase 3, rollback Phase 3 changes
2. Tests can be safely removed without affecting production code
3. Re-evaluate singleton removal if concurrency issues persist

---

## Testing Strategy

### Unit Tests

**Existing Tests to Update**:

1. **`tests/unit/test_rules_engine_service.py`**
   - Add test case for `update_rule()` calling `invalidate()`
   - Add test case for `toggle_rule()` calling `invalidate()`

   ```python
   @pytest.mark.asyncio
   async def test_update_rule_invalidates_registry():
       """Test that update_rule() calls registry.invalidate()."""
       session = _mock_session_with_scalar_one(_make_rule())

       with patch("pa_dealing.services.rules_engine.service.get_rule_registry") as mock_get_registry:
           mock_registry = AsyncMock()
           mock_get_registry.return_value = mock_registry

           await update_rule(
               session=session,
               rule_id="RF-002",
               updates={"lookbackDays": 90},
               changed_by="test@mako.com",
               reason="Unit test",
           )

           # Verify invalidate() was called once
           mock_registry.invalidate.assert_called_once()


   @pytest.mark.asyncio
   async def test_toggle_rule_invalidates_registry():
       """Test that toggle_rule() calls registry.invalidate()."""
       session = _mock_session_with_scalar_one(_make_rule())

       with patch("pa_dealing.services.rules_engine.service.get_rule_registry") as mock_get_registry:
           mock_registry = AsyncMock()
           mock_get_registry.return_value = mock_registry

           await toggle_rule(
               session=session,
               rule_id="RF-002",
               enabled=False,
               changed_by="test@mako.com",
               reason="Unit test",
           )

           mock_registry.invalidate.assert_called_once()
   ```

2. **`tests/unit/test_risk_scoring.py`**
   - Add test for `get_risk_scorer()` returning new instances

   ```python
   def test_get_risk_scorer_returns_new_instances():
       """Test that get_risk_scorer() returns a new instance on each call."""
       config1 = RiskScoringConfig(position_size_low_threshold=Decimal("10000"))
       config2 = RiskScoringConfig(position_size_low_threshold=Decimal("50000"))

       scorer1 = get_risk_scorer(config1)
       scorer2 = get_risk_scorer(config2)

       # Different instances
       assert scorer1 is not scorer2

       # Different configs
       assert scorer1.config.position_size_low_threshold == Decimal("10000")
       assert scorer2.config.position_size_low_threshold == Decimal("50000")


   def test_get_risk_scorer_with_none_config():
       """Test that get_risk_scorer(None) uses default config."""
       scorer = get_risk_scorer(None)

       # Should use default RiskScoringConfig values
       assert scorer.config.position_size_low_threshold == Decimal("50000")
       assert scorer.config.position_size_high_threshold == Decimal("100000")
   ```

### Integration Tests

**New Test Files**:

1. **`tests/integration/test_rules_engine_cache_invalidation.py`** (Phase 2)
   - See Phase 2 test specification above
   - 3 test cases covering update, toggle, and rapid updates

2. **`tests/integration/test_risk_scoring_concurrency.py`** (Phase 5)
   - See Phase 5 test specification above
   - 4 test cases covering basic concurrency, stress test, mixed configs, and instance isolation

### Manual Testing Checklist

**After Phase 1 & 2 (Cache Invalidation)**:
- [ ] Update a rule via admin UI
- [ ] Verify change appears immediately in API response (no 5-minute delay)
- [ ] Toggle a rule enabled/disabled
- [ ] Verify toggle takes effect immediately

**After Phase 3 & 4 (Singleton Removal)**:
- [ ] Submit multiple concurrent PAD requests with different employee roles
- [ ] Verify each request gets correct risk level (no config cross-contamination)
- [ ] Monitor logs for any unexpected errors

**After Phase 5 (Concurrency Tests)**:
- [ ] Run integration tests locally: `pytest tests/integration/test_risk_scoring_concurrency.py -v`
- [ ] Run integration tests in CI/CD pipeline
- [ ] Verify 100% pass rate across multiple runs

---

## Deployment Plan

### Pre-Deployment Checklist

- [ ] All unit tests pass (`pytest tests/unit/`)
- [ ] All integration tests pass (`pytest tests/integration/`)
- [ ] Code review completed
- [ ] Dependency track `rules_engine_ui_refactor_20260211` is complete

### Deployment Phases

**Phase 1-2: Cache Invalidation** (Low Risk)
1. Deploy service.py changes (invalidation calls)
2. Deploy integration tests
3. Monitor logs for `pad_rule_updated` and `pad_rule_toggled` events
4. Verify cache invalidation calls succeed (no errors)

**Phase 3-5: Singleton Removal** (Low Risk)
1. Deploy risk_scoring.py changes (singleton removal)
2. Deploy concurrency tests
3. Monitor request logs for concurrent scoring operations
4. Verify no config cross-contamination (spot-check high-risk vs low-risk employees)

### Monitoring

**Key Metrics**:
- Rule update latency (should drop from ~5min to <1sec)
- Risk scoring correctness (compare results before/after for same inputs)
- Error rate (should remain 0%)

**Alerts**:
- Spike in risk scoring errors → rollback Phase 3-5
- Cache invalidation failures → rollback Phase 1-2

---

## Success Criteria

### Phase 1-2: Cache Invalidation
- ✅ Rule updates propagate to registry within <1 second
- ✅ All integration tests pass
- ✅ No increase in error rate
- ✅ Backward compatible (no breaking changes)

### Phase 3-5: Singleton Removal
- ✅ Concurrent requests with different configs produce correct results
- ✅ Stress test (100 concurrent requests) passes
- ✅ No shared state detected across requests
- ✅ No performance regression (instance creation is cheap)

### Overall Track Success
- ✅ Both bugs fixed (cache invalidation + concurrency)
- ✅ 100% test coverage of new code paths
- ✅ Zero production incidents related to these changes
- ✅ Documentation updated (docstrings, track summary)

---

## Dependencies

### Upstream Dependencies
- ✅ `rules_engine_ui_refactor_20260211` (in_progress) - Must be complete before starting this track

### Downstream Dependencies
- None - this track is a bug fix with no breaking changes

---

## Notes

### Why the Singleton Pattern Was a Bad Idea

The original singleton pattern violated the **Shared Nothing Principle** for concurrent systems:

1. **Mutable shared state**: `_scorer` is a global variable modified by all requests
2. **Non-atomic updates**: `if _scorer is None or config is not None:` is not thread-safe
3. **Config overwriting**: Passing `config is not None` always replaces the singleton
4. **Hidden coupling**: Callers don't realize they're sharing a global instance

**Correct Pattern** (what we're implementing):
- Stateless factory function (no global state)
- Per-request instance creation (isolation by default)
- Config passed explicitly (no hidden dependencies)

### Performance Considerations

**Instance Creation Cost**:
- `SimplifiedRiskScorer.__init__()` only stores `config` parameter (no I/O)
- No database queries
- No network calls
- No expensive object initialization

**Benchmark** (rough estimate):
- Singleton approach: 1 instance creation per server boot (~1ms)
- Per-request approach: 1 instance creation per request (~0.001ms)
- Trade-off: **Correctness** (worth 0.001ms per request)

### Alternative Approaches Considered

**Alternative 1**: Keep singleton, add locking
- ❌ Adds complexity (locks, race conditions)
- ❌ Performance overhead (lock contention)
- ❌ Doesn't solve root cause (shared mutable state)

**Alternative 2**: Thread-local storage
- ❌ Python asyncio uses event loops, not threads
- ❌ Doesn't work with async/await

**Alternative 3**: Immutable singleton with config passed to methods
- ⚠️ Better than current, but still has issues:
  - Config must be passed to every method (API clutter)
  - Instance is still shared (unclear ownership)
- ✅ Per-request instances are simpler and cleaner

**Chosen Approach**: Per-request instances
- ✅ Simple (remove code, don't add it)
- ✅ Correct (no shared state)
- ✅ Fast (negligible overhead)
- ✅ Obvious (caller controls lifecycle)

---

## Appendix: Code References

### File Paths

All file paths are absolute:

- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/rules_engine/registry.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/services/rules_engine/service.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/agents/orchestrator/risk_scoring.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/src/pa_dealing/agents/orchestrator/risk_scoring_service.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/tests/unit/test_rules_engine_service.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/tests/unit/test_risk_scoring.py`
- `/Users/luisdeburnay/work/rules_engine_refactor/tests/integration/test_rules_engine_cache_invalidation.py` (new)
- `/Users/luisdeburnay/work/rules_engine_refactor/tests/integration/test_risk_scoring_concurrency.py` (new)

### Key Line Numbers

**registry.py**:
- Line 108-111: `invalidate()` method

**service.py**:
- Line 106-183: `update_rule()` function
- Line 186-244: `toggle_rule()` function
- Line 172-173: Commit before invalidation insertion point (update_rule)
- Line 234-235: Commit before invalidation insertion point (toggle_rule)

**risk_scoring.py**:
- Line 824-833: Singleton pattern (to be removed)
- Line 199-820: `SimplifiedRiskScorer` class

**risk_scoring_service.py**:
- Line 174: Caller of `get_risk_scorer()`

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-02-12 | Claude Opus 4.6 | Initial plan created |
