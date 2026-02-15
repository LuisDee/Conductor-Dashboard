# Track Specification: Web Search Fallback for Price Discovery

## Overview
Implement a secondary price discovery mechanism using Google ADK and Gemini 3 Flash to find market prices via web search when the primary EODHD API fails or returns no data. This is especially critical for derivatives, OTC, and exotic instruments where standard APIs often lack coverage.

## Goals
- Increase price validation coverage (target <5% "Unavailable" results).
- Leverage agentic web search to find real-time/recent prices from public financial sites.
- Maintain consistency with the `PriceProvider` protocol.
- Provide a high-integrity audit trail of the price discovery reasoning.

## Architecture & Design (ADK SequentialAgent Pattern)

### 1. SequentialAgent Orchestration
We will use a two-agent sequential pattern to overcome the ADK limitation of combining `output_schema` and `tools` on a single agent.

- **Agent 1: `search_agent`**
  - **Tools**: `google_search`
  - **Goal**: Execute up to 3 escalating queries to find price data.
  - **Output**: Stores raw findings in session state via `output_key`.
  
- **Agent 2: `formatter_agent`**
  - **Goal**: Read raw findings from state and structure them into a validated JSON object.
  - **Schema**: `PriceResult` Pydantic model.

### 2. PriceResult Schema (Pydantic)
```python
class PriceResult(BaseModel):
    ticker: str = Field(description="Instrument identifier as queried")
    price: Optional[float] = Field(None, description="Discovered price, or null if not found")
    currency: Optional[str] = Field(None, description="ISO 4217 code (USD, EUR, GBP)")
    source_url: Optional[str] = Field(None, description="Clickable URL where price was found. Required if price is not null")
    source_name: Optional[str] = Field(None, description="Human-readable source name (e.g. 'Yahoo Finance')")
    confidence: str = Field(..., description="high/medium/low/none")
    multiplier: str = Field("per_share", description="per_share/per_contract/per_lot/unknown")
    market_timestamp: Optional[str] = Field(None, description="Timestamp from the source")
    is_stale: bool = Field(False, description="True if market_timestamp is >24 hours old")
    reasoning: str = Field(..., description="Step-by-step explanation of what was searched and found")
```

### 3. Waterfall Integration in `service.py`
Update `validate_price` to use a waterfall logic:
1. Call `EODHDPriceProvider`.
2. If result is `None` OR instrument type is a known "API-weak" type (OTC, Options):
   - Call `WebSearchPriceProvider`.
3. If both fail, return `unavailable`.

## Implementation Rules
1. **No Hallucinations**: If no `source_url` is found, `price` MUST be null.
2. **Source Priority**: Exchanges > Financial News (Bloomberg/Reuters) > Finance Portals (Yahoo/Google) > Brokers.
3. **Escalating Search**:
   - Query 1: "{ticker} price quote"
   - Query 2: "{instrument description} latest price"
   - Query 3: "{ticker} {exchange} settlement price"
4. **Safety Net**: Post-validation logic to nullify price if `source_url` is missing.

## Deliverables
- [ ] `WebSearchPriceProvider` class in `src/pa_dealing/services/price_discovery/web_search_provider.py`.
- [ ] ADK Runner implementation with `search_agent` and `formatter_agent`.
- [ ] Waterfall logic update in `src/pa_dealing/services/price_discovery/service.py`.
- [ ] BigQuery audit logging for agent reasoning.
- [ ] Unit tests for parsing and extraction.
- [ ] Integration tests verifying EODHD -> Web Search waterfall.

## Test Strategy
- **Mocked Web Responses**: Test the `formatter_agent` with canned search results.
- **E2E Fallback Test**: Mock EODHD to fail and verify Web Search is invoked.
- **Schema Validation**: Verify `PriceResult` constraints.
