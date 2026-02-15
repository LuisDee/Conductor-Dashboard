# Implementation Plan: Instrument Matching Overhaul

## Phase 1: Consistency Fix (handlers.py)

**Files to modify:**
- `src/pa_dealing/agents/slack/handlers.py` (lines 2416, 2521, 2571)

**Change:**
```python
# BEFORE (uses bloomberg)
security_identifier=pad_request.bloomberg or pad_request.ticker,

# AFTER (uses inst_symbol for consistency with chatbot)
security_identifier=pad_request.inst_symbol or pad_request.ticker,
```

**Note:** The `inst_symbol` field must exist on PADRequest model. If not, use:
```python
security_identifier=pad_request.ticker or pad_request.bloomberg,
```
And ensure chatbot also stores `ticker` as primary identifier.

---

## Phase 2: Matching Config Module

**Create:** `src/pa_dealing/matching/__init__.py`
**Create:** `src/pa_dealing/matching/config.py`

```python
"""
Centralized configuration for instrument matching.
All scoring weights, thresholds, and rules defined here.
"""
from pydantic import BaseModel, Field
from enum import Enum


class MatchDecision(str, Enum):
    MATCH = "match"           # High confidence, return single result
    CLARIFY = "clarify"       # Medium confidence, ask user
    NO_MATCH = "no_match"     # No results found


class ScoringWeights(BaseModel):
    """Points awarded for different match types."""
    # Exact matches (strongest signals)
    exact_isin: int = 100          # ISIN is globally unique
    exact_ticker: int = 95         # Exact ticker match
    exact_inst_symbol: int = 90    # Exact inst_symbol match
    exact_sedol: int = 85          # Exact SEDOL match
    exact_bloomberg_code: int = 80 # Exact bloomberg code (before space)

    # Partial matches (weaker signals)
    contains_inst_symbol: int = 30
    contains_bloomberg: int = 25
    contains_description: int = 20

    # Tier bonuses (slight preference for primary sources)
    tier_1_bonus: int = 3  # Bloomberg table
    tier_2_bonus: int = 2  # Mapping table
    tier_3_bonus: int = 1  # Product table


class ConsistencyPenalties(BaseModel):
    """Penalties for field inconsistencies."""
    # THE KEY FIX: Penalize when inst_symbol matches but bloomberg doesn't
    inst_symbol_bloomberg_mismatch: int = -40

    # Reward when multiple fields agree on the search term
    cross_field_agreement: int = 25

    # Slight penalty for missing unique identifiers
    missing_isin_sedol: int = -5


class ConfidenceThresholds(BaseModel):
    """Thresholds for auto-match vs clarification."""
    # Score gap between #1 and #2 (as ratio)
    high_confidence_gap: float = 0.40   # Above = auto-match
    low_confidence_gap: float = 0.15    # Below = always clarify

    # Minimum absolute score
    min_match_score: float = 50.0

    # Penalty when top 2 have identical descriptions
    same_description_penalty: float = 0.20


class MatchingRules(BaseModel):
    """General matching behavior."""
    max_candidates: int = 10
    max_clarification_options: int = 5
    case_sensitive: bool = False
    bloomberg_code_separator: str = " "


class InstrumentMatchingConfig(BaseModel):
    """Master configuration."""
    scoring: ScoringWeights = Field(default_factory=ScoringWeights)
    penalties: ConsistencyPenalties = Field(default_factory=ConsistencyPenalties)
    thresholds: ConfidenceThresholds = Field(default_factory=ConfidenceThresholds)
    rules: MatchingRules = Field(default_factory=MatchingRules)


# Singleton - loaded from YAML config file
_config: InstrumentMatchingConfig | None = None


def get_config() -> InstrumentMatchingConfig:
    """Load config from YAML file (cached singleton)."""
    global _config
    if _config is None:
        _config = load_config()
    return _config


def load_config(path: str | None = None) -> InstrumentMatchingConfig:
    """Load config from YAML file."""
    import yaml
    from pathlib import Path

    if path is None:
        # Default location
        path = Path(__file__).parent.parent.parent.parent / "config" / "instrument_matching.yaml"

    if Path(path).exists():
        with open(path) as f:
            data = yaml.safe_load(f)
        return InstrumentMatchingConfig(**data)

    # Fall back to defaults if no config file
    return InstrumentMatchingConfig()


CONFIG = get_config()  # Backwards compatible import
```

---

## Phase 3: Weighted Matcher Module

**Create:** `src/pa_dealing/matching/matcher.py`

```python
"""
Deterministic instrument matching with confidence scoring.
The LLM should NOT make disambiguation decisions - this module does.
"""
from dataclasses import dataclass, field
from typing import Optional
from .config import CONFIG, MatchDecision, InstrumentMatchingConfig


@dataclass
class ScoredInstrument:
    """Instrument with match score and reasons."""
    inst_symbol: str
    ticker: Optional[str] = None
    isin: Optional[str] = None
    sedol: Optional[str] = None
    bloomberg: Optional[str] = None
    description: Optional[str] = None
    tier: int = 1

    # Populated by scoring
    score: float = 0.0
    match_reasons: list[str] = field(default_factory=list)

    @property
    def bloomberg_code(self) -> str:
        """Extract code from 'IBGM LN' -> 'IBGM'."""
        if not self.bloomberg:
            return ""
        return self.bloomberg.split()[0] if self.bloomberg else ""


@dataclass
class MatchResult:
    """Result of matching process."""
    decision: MatchDecision
    top_match: Optional[ScoredInstrument]
    confidence: float  # 0.0 to 1.0
    candidates: list[ScoredInstrument]
    clarification_message: Optional[str] = None

    # ISIN passthrough - return if available
    isin: Optional[str] = None


def calculate_score(
    instrument: ScoredInstrument,
    search_term: str,
    config: InstrumentMatchingConfig = CONFIG
) -> ScoredInstrument:
    """Calculate match score for single instrument."""
    score = 0.0
    reasons = []
    term = search_term.upper()

    # Normalize fields
    inst_symbol = (instrument.inst_symbol or "").upper()
    ticker = (instrument.ticker or "").upper()
    isin = (instrument.isin or "").upper()
    sedol = (instrument.sedol or "").upper()
    bloomberg = (instrument.bloomberg or "").upper()
    bloomberg_code = instrument.bloomberg_code.upper()
    description = (instrument.description or "").upper()

    # === EXACT MATCHES ===
    if isin and isin == term:
        score += config.scoring.exact_isin
        reasons.append(f"exact_isin: +{config.scoring.exact_isin}")

    if ticker and ticker == term:
        score += config.scoring.exact_ticker
        reasons.append(f"exact_ticker: +{config.scoring.exact_ticker}")

    if inst_symbol == term:
        score += config.scoring.exact_inst_symbol
        reasons.append(f"exact_inst_symbol: +{config.scoring.exact_inst_symbol}")

    if sedol and sedol == term:
        score += config.scoring.exact_sedol
        reasons.append(f"exact_sedol: +{config.scoring.exact_sedol}")

    if bloomberg_code and bloomberg_code == term:
        score += config.scoring.exact_bloomberg_code
        reasons.append(f"exact_bloomberg_code: +{config.scoring.exact_bloomberg_code}")

    # === PARTIAL MATCHES ===
    if term in inst_symbol and inst_symbol != term:
        score += config.scoring.contains_inst_symbol
        reasons.append(f"contains_inst_symbol: +{config.scoring.contains_inst_symbol}")

    if term in bloomberg and bloomberg_code != term:
        score += config.scoring.contains_bloomberg
        reasons.append(f"contains_bloomberg: +{config.scoring.contains_bloomberg}")

    if term in description:
        score += config.scoring.contains_description
        reasons.append(f"contains_description: +{config.scoring.contains_description}")

    # === CONSISTENCY CHECKS (THE KEY FIX) ===
    inst_symbol_matches = term in inst_symbol
    bloomberg_matches = term in bloomberg if bloomberg else False

    if inst_symbol_matches and bloomberg_matches:
        # Both fields agree - strong signal
        score += config.penalties.cross_field_agreement
        reasons.append(f"cross_field_agreement: +{config.penalties.cross_field_agreement}")
    elif inst_symbol_matches and bloomberg and not bloomberg_matches:
        # PENALTY: inst_symbol matches but bloomberg doesn't
        # Catches IBGM_L_EUR (matches "IBGM") with bloomberg="IEGM LN"
        score += config.penalties.inst_symbol_bloomberg_mismatch
        reasons.append(f"inst_symbol_bloomberg_mismatch: {config.penalties.inst_symbol_bloomberg_mismatch}")

    # === MISSING IDENTIFIERS ===
    if not instrument.isin and not instrument.sedol:
        score += config.penalties.missing_isin_sedol
        reasons.append(f"missing_isin_sedol: {config.penalties.missing_isin_sedol}")

    # === TIER BONUS ===
    tier_bonuses = {1: config.scoring.tier_1_bonus, 2: config.scoring.tier_2_bonus, 3: config.scoring.tier_3_bonus}
    tier_bonus = tier_bonuses.get(instrument.tier, 0)
    if tier_bonus:
        score += tier_bonus
        reasons.append(f"tier_{instrument.tier}_bonus: +{tier_bonus}")

    instrument.score = score
    instrument.match_reasons = reasons
    return instrument


def calculate_confidence(ranked: list[ScoredInstrument], config: InstrumentMatchingConfig = CONFIG) -> tuple[float, bool]:
    """Calculate confidence and whether clarification needed."""
    if len(ranked) == 0:
        return 0.0, True

    if len(ranked) == 1:
        if ranked[0].score >= config.thresholds.min_match_score:
            return 1.0, False
        return 0.5, True

    top_score = ranked[0].score
    second_score = ranked[1].score

    if top_score < config.thresholds.min_match_score:
        return 0.0, True

    gap_ratio = (top_score - second_score) / top_score if top_score > 0 else 0

    # Identical descriptions add ambiguity
    same_desc = ranked[0].description and ranked[0].description == ranked[1].description

    confidence = gap_ratio
    if same_desc:
        confidence -= config.thresholds.same_description_penalty

    if confidence >= config.thresholds.high_confidence_gap:
        needs_clarification = False
    elif confidence <= config.thresholds.low_confidence_gap:
        needs_clarification = True
    else:
        needs_clarification = same_desc

    return max(0.0, min(1.0, confidence)), needs_clarification


def match_instruments(
    search_term: str,
    candidates: list[ScoredInstrument],
    config: InstrumentMatchingConfig = CONFIG
) -> MatchResult:
    """
    Main entry point: Score candidates and determine match decision.
    DETERMINISTIC - same inputs always produce same outputs.
    """
    if not candidates:
        return MatchResult(
            decision=MatchDecision.NO_MATCH,
            top_match=None,
            confidence=0.0,
            candidates=[],
            clarification_message=f'No instruments found matching "{search_term}"'
        )

    # Score all candidates
    scored = [calculate_score(c, search_term, config) for c in candidates]

    # Sort by score descending
    ranked = sorted(scored, key=lambda x: x.score, reverse=True)[:config.rules.max_candidates]

    # Calculate confidence
    confidence, needs_clarification = calculate_confidence(ranked, config)

    # ISIN passthrough - return from top match if available
    top_isin = ranked[0].isin if ranked else None

    if needs_clarification:
        return MatchResult(
            decision=MatchDecision.CLARIFY,
            top_match=ranked[0] if ranked else None,
            confidence=confidence,
            candidates=ranked,
            clarification_message=_generate_clarification(search_term, ranked, config),
            isin=top_isin
        )

    return MatchResult(
        decision=MatchDecision.MATCH,
        top_match=ranked[0],
        confidence=confidence,
        candidates=ranked,
        isin=top_isin
    )


def _generate_clarification(term: str, candidates: list[ScoredInstrument], config: InstrumentMatchingConfig) -> str:
    """Generate clarification message for user."""
    lines = [f'I found multiple instruments matching "{term}":']
    for i, c in enumerate(candidates[:config.rules.max_clarification_options], 1):
        lines.append(f"{i}. {c.inst_symbol} (Bloomberg: {c.bloomberg_code or 'N/A'}, Score: {c.score:.0f})")
        if c.description:
            lines.append(f"   {c.description[:50]}...")
    lines.append("\nPlease specify the exact Bloomberg code, ISIN, or select by number.")
    return "\n".join(lines)
```

---

## Phase 3b: YAML Config File

**Create:** `config/instrument_matching.yaml`

```yaml
# =============================================================================
# INSTRUMENT MATCHING CONFIGURATION
# =============================================================================
# Modify values here to tune matching behavior without code changes.
# =============================================================================

scoring:
  # Exact matches - highest value
  exact_isin: 100          # ISIN is globally unique identifier
  exact_ticker: 95         # Exact ticker match
  exact_inst_symbol: 90    # Exact inst_symbol match
  exact_sedol: 85          # Exact SEDOL match
  exact_bloomberg_code: 80 # Exact bloomberg code (before space)

  # Partial matches - lower value
  contains_inst_symbol: 30
  contains_bloomberg: 25
  contains_description: 20

  # Tier bonuses (prefer primary sources)
  tier_1_bonus: 3  # Bloomberg table
  tier_2_bonus: 2  # Mapping table
  tier_3_bonus: 1  # Product table

penalties:
  # THE KEY FIX: Penalize mismatched fields
  # Catches IBGM_L_EUR (inst_symbol contains "IBGM") with bloomberg="IEGM LN"
  inst_symbol_bloomberg_mismatch: -40

  # Reward when search term appears in BOTH inst_symbol AND bloomberg
  cross_field_agreement: 25

  # Slight penalty for incomplete data
  missing_isin_sedol: -5

thresholds:
  # Gap between #1 and #2 scores (as ratio of top score)
  high_confidence_gap: 0.40   # Above = high confidence
  low_confidence_gap: 0.15    # Below = always clarify

  # Minimum absolute score to consider a match
  min_match_score: 50.0

  # Reduce confidence when top 2 have identical descriptions
  same_description_penalty: 0.20

rules:
  max_candidates: 10           # Max results to evaluate
  max_clarification_options: 5 # Max options to show user
  case_sensitive: false        # Case-insensitive matching
  bloomberg_code_separator: " " # How to extract code from "IBGM LN"
```

---

## Phase 4: Integration with Repository

**File:** `src/pa_dealing/db/repository.py`

**Changes:**
1. Import matcher module
2. Convert InstrumentInfo to ScoredInstrument
3. Use `match_instruments()` instead of current scoring
4. Return `MatchResult` with confidence and ISIN

**Location:** `lookup_instrument()` function (lines 1600-1820)

```python
# At end of lookup_instrument(), replace current scoring with:
from pa_dealing.matching.matcher import match_instruments, ScoredInstrument

# Convert to ScoredInstrument
scored_candidates = [
    ScoredInstrument(
        inst_symbol=info.inst_symbol,
        ticker=info.ticker,
        isin=info.isin,
        sedol=info.sedol,
        bloomberg=info.bloomberg,
        description=info.description,
        tier=info.tier_source_number,
    )
    for info in instruments
]

result = match_instruments(term, scored_candidates)
```

---

## Phase 5: Integration with Chatbot

**File:** `src/pa_dealing/agents/slack/chatbot.py`

**Changes:**
1. Use `MatchResult` from matcher
2. If `result.decision == CLARIFY`, show clarification message with scores
3. If `result.decision == MATCH`, still confirm (compliance requirement) but can show confidence
4. Store ISIN when available from top match

**Note:** Always confirm with user per design decision - no auto-accept.

**Location:** `update_draft()` function (lines 500-566)

```python
# When storing confirmed selection (line 452-465):
updates.update({
    "security_identifier": selected_candidate.get('inst_symbol') or selected_candidate.get('ticker'),
    "bloomberg_ticker": selected_candidate.get('bloomberg'),
    "ticker": selected_candidate.get('inst_symbol'),  # Use inst_symbol as primary
    "isin": selected_candidate.get('isin'),  # NEW: Store ISIN
    "security_description": selected_candidate['description'],
    # ... rest unchanged
})
```

---

## Phase 6: ISIN Passthrough

**Ensure ISIN flows through entire pipeline:**

1. **DraftRequest model** (`session.py`): Add `isin: Optional[str] = None`
2. **Chatbot storage** (above): Store `isin` from candidate
3. **Orchestrator** (`agent.py`): Pass `isin` to `submit_pad_request()`
4. **PADRequest storage** (`repository.py`): Store `isin` on request
5. **Handlers display** (`handlers.py`): Include ISIN in approval if available

---

## File Changes Summary

| File | Changes |
|------|---------|
| `config/instrument_matching.yaml` | NEW - Scoring weights and thresholds |
| `src/pa_dealing/matching/__init__.py` | NEW - Module init |
| `src/pa_dealing/matching/config.py` | NEW - Pydantic config models + YAML loader |
| `src/pa_dealing/matching/matcher.py` | NEW - Weighted matcher with confidence |
| `src/pa_dealing/agents/slack/handlers.py` | Lines 2416, 2521, 2571 - Use inst_symbol |
| `src/pa_dealing/agents/slack/chatbot.py` | Lines 452-465 - Store ISIN, use matcher |
| `src/pa_dealing/agents/slack/session.py` | Add `isin` field to DraftRequest |
| `src/pa_dealing/db/repository.py` | Lines 1750-1800 - Use matcher module |

---

## Verification Steps

### Unit Tests
1. Test `calculate_score()` with IBGM_L vs IBGM_L_EUR scenarios
2. Test consistency penalty applies when inst_symbol contains "IBGM" but bloomberg contains "IEGM"
3. Test confidence calculation with identical descriptions
4. Test ISIN passthrough

### Integration Tests
1. Search "IBGM" → should rank IBGM_L higher due to cross-field agreement
2. Confirm IBGM_L → approval should show "IBGM_L" not "IEGM"
3. Search with ISIN → should return exact match with high confidence

### Manual Testing
1. In Slack: "I want to buy IBGM"
2. Verify confirmation shows IBGM_L (bloomberg="IBGM LN") ranked first
3. Confirm and submit
4. Verify approval shows "(IBGM_L)" not "(IEGM)"

---

## Test File to Create

**Create:** `tests/unit/test_instrument_matching.py`

```python
"""
Unit tests for instrument matching module.
"""
import pytest
from pa_dealing.matching import (
    ScoredInstrument,
    calculate_score,
    calculate_confidence,
    match_instruments,
    MatchDecision,
    InstrumentMatchingConfig,
)


class TestCalculateScore:
    """Tests for the scoring algorithm."""

    def test_exact_ticker_match(self):
        """Exact ticker match gets highest points."""
        inst = ScoredInstrument(inst_symbol="AAPL", ticker="AAPL")
        result = calculate_score(inst, "AAPL")
        assert result.score >= 90  # exact_ticker + exact_inst_symbol

    def test_ibgm_consistency_penalty(self):
        """IBGM_L_EUR gets penalty when bloomberg doesn't match search term."""
        # IBGM_L: inst_symbol contains IBGM, bloomberg contains IBGM
        ibgm_l = ScoredInstrument(
            inst_symbol="IBGM_L",
            bloomberg="IBGM LN",
            description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
        )
        # IBGM_L_EUR: inst_symbol contains IBGM, but bloomberg contains IEGM
        ibgm_l_eur = ScoredInstrument(
            inst_symbol="IBGM_L_EUR",
            bloomberg="IEGM LN",
            description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
        )

        score_l = calculate_score(ibgm_l, "IBGM")
        score_l_eur = calculate_score(ibgm_l_eur, "IBGM")

        # IBGM_L should score higher due to cross-field agreement
        # IBGM_L_EUR should have penalty due to inst_symbol/bloomberg mismatch
        assert score_l.score > score_l_eur.score
        assert "cross_field_agreement" in str(score_l.match_reasons)
        assert "inst_symbol_bloomberg_mismatch" in str(score_l_eur.match_reasons)

    def test_isin_exact_match(self):
        """ISIN exact match gets highest score."""
        inst = ScoredInstrument(
            inst_symbol="TEST",
            isin="US0378331005",
        )
        result = calculate_score(inst, "US0378331005")
        assert result.score >= 100


class TestCalculateConfidence:
    """Tests for confidence calculation."""

    def test_single_result_high_confidence(self):
        """Single result with high score = high confidence."""
        candidates = [ScoredInstrument(inst_symbol="AAPL", score=100)]
        confidence, needs_clarification = calculate_confidence(candidates)
        assert confidence == 1.0
        assert not needs_clarification

    def test_identical_descriptions_need_clarification(self):
        """Identical descriptions should trigger clarification."""
        candidates = [
            ScoredInstrument(
                inst_symbol="IBGM_L",
                description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
                score=80,
            ),
            ScoredInstrument(
                inst_symbol="IBGM_L_EUR",
                description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
                score=70,
            ),
        ]
        confidence, needs_clarification = calculate_confidence(candidates)
        # Same description should reduce confidence
        assert needs_clarification or confidence < 0.4


class TestMatchInstruments:
    """Integration tests for match_instruments."""

    def test_no_candidates_returns_no_match(self):
        """Empty candidates list returns NO_MATCH."""
        result = match_instruments("AAPL", [])
        assert result.decision == MatchDecision.NO_MATCH
        assert result.top_match is None

    def test_ibgm_ranks_correctly(self):
        """IBGM_L should rank higher than IBGM_L_EUR for search 'IBGM'."""
        candidates = [
            ScoredInstrument(
                inst_symbol="IBGM_L_EUR",
                bloomberg="IEGM LN",
                description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
            ),
            ScoredInstrument(
                inst_symbol="IBGM_L",
                bloomberg="IBGM LN",
                description="ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR",
            ),
        ]
        result = match_instruments("IBGM", candidates)
        assert result.top_match.inst_symbol == "IBGM_L"

    def test_isin_passthrough(self):
        """ISIN should be returned in result."""
        candidates = [
            ScoredInstrument(
                inst_symbol="AAPL",
                isin="US0378331005",
            ),
        ]
        result = match_instruments("AAPL", candidates)
        assert result.isin == "US0378331005"
```

---

## Rollback Plan

If issues are found:
1. Revert handlers.py changes (use `git checkout` on specific lines)
2. Delete `src/pa_dealing/matching/` directory
3. Delete `config/instrument_matching.yaml`
4. No database migrations needed - purely code changes
