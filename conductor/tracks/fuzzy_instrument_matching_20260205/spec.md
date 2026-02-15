# Fuzzy Instrument Matching for Typo Detection

## Overview

The PA Dealing chatbot currently uses LIKE-based substring queries for instrument lookup across a 3-tier database search (Bloomberg → Exchange Mappings → Product). This works well for exact or partial matches but fails completely for typos (e.g., "Vodafon3" returns nothing even though "Vodafone" exists).

This creates a **compliance risk**: if the system says "not found" due to a typo, and the user confirms, auto-approval could trigger for a security that IS actually traded by Mako.

### Problem Statement

> "If someone spells Vodafone as Vodafon3, can we have confidence that the AI will recognise this and won't discount this product as not traded by Mako?"

**Current behavior:**
- User types "Vodafon3" → LIKE query `%Vodafon3%` → Zero results → "Not found"

**Required behavior:**
- User types "Vodafon3" → DB returns nothing → Fuzzy fallback finds "Vodafone" at 86% → "Did you mean Vodafone?"

## Functional Requirements

### FR-1: In-Memory Fuzzy Cache Module
- Create a shared module `src/pa_dealing/instruments/fuzzy_cache.py`
- Load all instruments from `oracle_bloomberg` table at application startup
- Pre-process descriptions, tickers, inst_symbols, and ISINs for fast matching
- Use `rapidfuzz` library (already in dependencies) with `process.extract()` for batch matching
- Expose singleton functions: `load_fuzzy_cache()`, `search_fuzzy()`, `ensure_cache_fresh()`, `get_cache_stats()`

### FR-2: Cache Refresh Strategy
- **TTL:** 24 hours (matches daily data sync)
- **Pattern:** Stale-while-revalidate
  - If cache age < 24h: serve immediately
  - If cache age 24-25h: serve stale, trigger background refresh
  - If cache age > 25h: block and refresh synchronously
- Use `asyncio.Lock` to prevent concurrent refresh storms

### FR-3: Fuzzy Search Algorithm
- Search fields in priority order:
  1. Ticker (strictest matching, `fuzz.ratio`, threshold 80%)
  2. Inst Symbol (`fuzz.ratio`, threshold 80%)
  3. ISIN (`fuzz.ratio`, threshold 85%, only if query ≥6 chars)
  4. Description (`fuzz.token_set_ratio`, threshold 70%)
- Return top 10 matches with confidence scores
- Deduplicate results across field searches

### FR-4: Repository Integration
- Modify `_search_instruments()` in `repository.py`
- Fuzzy cache is **only** invoked when all 3 DB tiers return empty results
- Add `match_type` field to `InstrumentLookupResult`: `"exact"`, `"fuzzy"`, `"verified_not_found"`
- Add `match_confidence` field to `InstrumentInfo` for fuzzy matches

### FR-5: Startup Integration
- Load fuzzy cache in `pad_api` container (FastAPI lifespan handler)
- Load fuzzy cache in `pad_slack` container (Slack listener startup)
- Log cache size, load time, and memory estimate on startup

### FR-6: Existing Flow Unchanged
- User confirmation flow ("Is this what you mean?") remains the same
- Fuzzy matches go through identical confirmation as exact matches
- `match_type` is for internal tracking/audit only

## Non-Functional Requirements

### NFR-1: Performance
- Cache load time: <5 seconds for 15k instruments
- Fuzzy search latency: <50ms (p99)
- Memory footprint: ~10-15 MB per container

### NFR-2: Reliability
- Cache failure should not break instrument lookup (graceful degradation to DB-only)
- Background refresh failures should be logged but not crash the service

### NFR-3: Observability
- Log cache load events with instrument count and duration
- Log fuzzy search invocations (term, result count, top match confidence)
- Expose `/health/cache` endpoint (optional) for monitoring

## Acceptance Criteria

> **Note:** Confidence thresholds are initial values subject to heuristic tuning based on real instrument data.

### Core Functionality
- [ ] **AC-1:** Search "Vodafon3" returns "Vodafone Group PLC" as top fuzzy match
- [ ] **AC-2:** Search "APPL" returns "AAPL" (Apple) as top fuzzy match
- [ ] **AC-3:** Search "XYZ123RandomNonsense" returns empty (verified not found)
- [ ] **AC-4:** Exact matches (e.g., "Vodafone") still work via DB lookup (no regression)
- [ ] **AC-5:** Fuzzy fallback only triggers when DB returns zero results

### Cache Behavior
- [ ] **AC-6:** Cache loads successfully at startup with correct instrument count
- [ ] **AC-7:** Cache refreshes automatically after TTL expires
- [ ] **AC-8:** Stale-while-revalidate pattern works (serves stale, refreshes in background)

### Integration
- [ ] **AC-9:** `pad_api` container loads cache at startup
- [ ] **AC-10:** `pad_slack` container loads cache at startup
- [ ] **AC-11:** `InstrumentLookupResult.match_type` correctly indicates "exact", "fuzzy", or "verified_not_found"

### Performance
- [ ] **AC-12:** Cache loads 15k instruments in <5 seconds
- [ ] **AC-13:** Fuzzy search completes in <50ms

## Documentation Requirements

### DR-1: Update Tooling Documentation
- Update `docs/tooling/instrument-lookup.md` with:
  - Updated Mermaid diagram showing fuzzy fallback layer
  - New section explaining fuzzy cache behavior
  - Updated "How It's Used" to describe fallback flow
  - New "Last Verified" date

### DR-2: Update Gemini Skill
- Update `.gemini/skills/instrument-lookup/SKILL.md` with:
  - New "Tier 4: Fuzzy Cache Fallback" section
  - Constraints for when fuzzy is used
  - Examples of typo handling
  - Integration with stale-while-revalidate pattern

## Out of Scope

- Redis or external cache service (in-memory per-container is sufficient for 15k rows)
- pg_trgm or database-level fuzzy matching
- Changes to user-facing confirmation flow
- Fuzzy re-ranking of existing DB matches (pure fallback only)
- Phonetic matching (Soundex/Metaphone) — may be added in future phase

## Technical Notes

### Why In-Memory Cache (Not Redis)?
- 15k instruments × ~300 bytes = ~5-10 MB (trivial memory cost)
- Two containers need it (pad_api, pad_slack) — 20 MB total is acceptable
- No operational overhead of additional infrastructure
- Latency: in-memory is ~1000x faster than Redis network call

### Why rapidfuzz?
- Already in project dependencies
- 10-100x faster than fuzzywuzzy (C++ implementation)
- `process.extract()` provides optimized batch matching
- Proven pattern in codebase (`identity/fuzzy_matcher.py`, `services/user_matcher.py`)

### Data Freshness
- `oracle_bloomberg` syncs daily
- 24-hour cache TTL matches sync frequency
- Stale-while-revalidate ensures no blocking on refresh

## Architecture Diagram

```
User Input: "Vodafon3"
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  EXISTING 3-TIER DB LOOKUP (unchanged)                          │
│                                                                 │
│  Tier 1: ILIKE '%Vodafon3%' → []                                │
│  Tier 2: ILIKE '%Vodafon3%' → []                                │
│  Tier 3: ILIKE '%Vodafon3%' → []                                │
└─────────────────────────────────────────────────────────────────┘
         │
         │ Zero results?
         │
        YES
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  NEW: FUZZY CACHE FALLBACK                                      │
│                                                                 │
│  process.extract("Vodafon3", cached_descriptions)               │
│  → "Vodafone Group PLC" at 86%                                  │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  Return to User                                                 │
│  "Did you mean Vodafone Group PLC (VOD US)?"                    │
│  match_type = "fuzzy"                                           │
└─────────────────────────────────────────────────────────────────┘
```
