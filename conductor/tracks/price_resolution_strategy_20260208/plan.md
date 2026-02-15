# Implementation Plan - Price Resolution Strategy

This track is currently in the "Future Backlog" state. Detailed implementation will begin once a reliable market data provider is selected.

## Phase 1: Research & Discovery (TBD)
- [ ] Evaluate market data APIs (Bloomberg, Reuters, EODHD etc.) for real-time price lookups.
- [ ] Research cost and latency implications.
- [ ] Map required fields (Price, Currency, Bid/Ask spread if relevant).

## Phase 2: Core Architecture (TBD)
- [ ] Design the `MarketDataProvider` abstraction.
- [ ] Implement the selected provider client.
- [ ] Add state tracking to `DraftRequest` for user-provided vs. system-calculated values.

## Phase 3: Conflict Resolution UX (TBD)
- [ ] Implement the confirmation flow: "I found a market price of X, which makes your total Y. Is this correct?"
- [ ] Add logic to detect "Significant Differences" (>N%) to trigger alerts.
- [ ] Update final preview to clearly indicate data sources.
