# Implementation Plan: Web Search Fallback for Price Discovery

## Phase 1: Models and ADK Setup
- [ ] Update `src/pa_dealing/services/price_discovery/models.py` with Pydantic `ADKPriceResult`.
- [ ] Add conversion logic from `ADKPriceResult` to the existing `PriceResult` dataclass.
- [ ] Add required enums: `ConfidenceLevel`, `OptionMultiplier`.

## Phase 2: Agent Construction
- [ ] Create `src/pa_dealing/services/price_discovery/agents.py`.
- [ ] Implement `search_agent` with `google_search` and escalating prompt.
- [ ] Implement `formatter_agent` with Pydantic output schema.
- [ ] Implement `PriceDiscoveryRunner` using `SequentialAgent`.

## Phase 3: Provider and Service Integration
- [ ] Create `src/pa_dealing/services/price_discovery/web_search_provider.py`.
- [ ] Implement `WebSearchPriceProvider` (ADK implementation).
- [ ] Create `src/pa_dealing/services/price_discovery/waterfall_provider.py`.
- [ ] Update `service.py` to use `WaterfallPriceProvider`.
- [ ] Add post-extraction safety validation (Price/Source URL check).

## Phase 4: Audit and Logging
- [ ] Update audit log details to include `reasoning` and `source_url`.
- [ ] Ensure `gemini-3-flash-preview` is configured with "low" thinking level by default.

## Phase 5: Verification
- [ ] `tests/services/price_discovery/test_web_search_provider.py`
- [ ] `tests/services/price_discovery/test_waterfall_logic.py`
