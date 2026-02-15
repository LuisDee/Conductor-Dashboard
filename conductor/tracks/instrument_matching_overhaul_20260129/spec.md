# Instrument Matching Overhaul & Consistency Fix

## Problem Statement

### Bug 1: Field Inconsistency (IBGM → IEGM)
User confirms "IBGM_L_EUR" but final approval shows "IEGM" because:
- **Chatbot stores** (line 453): `security_identifier = inst_symbol or ticker`
- **Handlers display** (lines 2416, 2521, 2571): `security_identifier = bloomberg or ticker`

For IBGM_L_EUR record: `inst_symbol="IBGM_L_EUR"`, `bloomberg="IEGM LN"` → displays "IEGM"

### Bug 2: No Confidence-Based Disambiguation
Two records with identical descriptions ("ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR") both match search term "IBGM":
- `IBGM_L` (bloomberg="IBGM LN")
- `IBGM_L_EUR` (bloomberg="IEGM LN")

Current scoring doesn't penalize the mismatch where `inst_symbol` contains "IBGM" but `bloomberg` contains "IEGM".

### Bug 3: No ISIN Passthrough
ISIN is available in bloomberg records but not consistently returned/stored when user confirms selection.

---

## Research Summary

Based on industry best practices ([Data Ladder Fuzzy Matching](https://dataladder.com/fuzzy-matching-101/), [AWS Entity Resolution](https://aws.amazon.com/blogs/industries/resolve-imperfect-data-with-advanced-rule-based-fuzzy-matching-in-aws-entity-resolution/)):

- **Tiered thresholds**: Auto-match above 0.90, review 0.75-0.89, reject below
- **Multi-field matching**: Combining fields adds context and strengthens confidence
- **Cross-field agreement**: Penalize when fields disagree, reward when they agree
- **Deterministic over ML**: Rules-based produces fewer incorrect matches with full explainability

---

## Solution Overview

### Part 1: Consistency Fix (Quick Win)
Use `inst_symbol` consistently throughout the system - what user confirms is what gets displayed.

### Part 2: Centralized Scoring Config
Create `src/pa_dealing/matching/config.py` with Pydantic models for configurable weights.

### Part 3: Weighted Matcher
Create `src/pa_dealing/matching/matcher.py` implementing deterministic scoring with:
- Cross-field consistency checks (penalize inst_symbol/bloomberg mismatch)
- Confidence thresholds (auto-match vs clarification)
- ISIN passthrough when available

### Part 4: Integration
Update chatbot and repository to use the new matcher.

---

## Design Decisions (User Confirmed)

1. **Primary Identifier**: `inst_symbol` - what user confirms is what gets displayed everywhere
2. **Confirmation**: Always confirm with user (safest for compliance) - no auto-accept
3. **Config Format**: YAML file at `config/instrument_matching.yaml` for easy tuning

---

## Success Criteria

1. User confirms "IBGM_L" → approval shows "IBGM_L" (not "IEGM")
2. IBGM_L ranked higher than IBGM_L_EUR for search "IBGM" (due to -40 penalty)
3. Identical descriptions trigger clarification request
4. ISIN returned when available from bloomberg
5. Scoring weights configurable via Pydantic config
6. All existing tests pass

---

## Sources

- [Data Ladder - Fuzzy Matching 101](https://dataladder.com/fuzzy-matching-101/) - Weight assignment, multi-field matching
- [AWS Entity Resolution](https://aws.amazon.com/blogs/industries/resolve-imperfect-data-with-advanced-rule-based-fuzzy-matching-in-aws-entity-resolution/) - Rule-based vs ML matching
- [Multimodal - Confidence Scoring](https://www.multimodal.dev/post/using-confidence-scoring-to-reduce-risk-in-ai-driven-decisions) - Tiered thresholds
- [Wikipedia - Record Linkage](https://en.wikipedia.org/wiki/Record_linkage) - Probabilistic weighting theory
