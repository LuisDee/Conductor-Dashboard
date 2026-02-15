# Track: Price Resolution Strategy

**ID**: price_resolution_strategy_20260208
**Priority**: Low
**Tags**: chatbot, market-data, ux, future

## Overview
This track addresses the future handling of security market prices within the PA Dealing Chatbot. Currently, real-time price fetching is disabled to prevent AI hallucinations and unintentional overwriting of user-provided value estimates.

## The Problem
If the bot fetches a market price (currently via LLM lookup, which is unreliable), it may silently overwrite a user's explicit estimate (e.g., "about $2,500"). This leads to incorrect compliance records and poor UX.

## Future Goals
- Implement a reliable market data source (e.g., Bloomberg API, Reuters) to replace LLM-based price lookups.
- Define a "Conflict Resolution" logic: how should the bot handle cases where the system-calculated value differs significantly from the user's estimate?
- Ensure no silent overwrites: users must always confirm system-calculated values if they differ from their input.

## Success Criteria (TBD)
- [ ] Reliable market price data source integrated.
- [ ] Zero hallucinations for security prices.
- [ ] Explicit confirmation flow for price/value conflicts.
- [ ] Audit trail for user-provided vs. system-calculated values.
