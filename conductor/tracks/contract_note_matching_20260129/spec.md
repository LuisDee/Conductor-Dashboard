# Contract Note User Matching & Verification

## Overview

Extend the GCS PDF ingestion system to:
1. **Support two document types**: Contract Notes AND Activity Statements
2. Match incoming documents to employees with **approved requests awaiting execution**
3. Extract account holder name from PDF for fuzzy matching
4. Link parsed trades to the matched PAD request
5. Auto-verify high-confidence matches with **self-healing extraction**

## Background

The GCS PDF ingestion system (track: `gcs_pdf_ingestion_20260129`) is complete:
- PDFs are polled from GCS bucket
- Documents are tracked in `gcs_document` table
- Sender email is captured from GCS metadata (`x-goog-meta-sender-email`)
- Files are archived after processing

**What's missing**: The system archives PDFs but can't identify who they belong to or link them to PAD requests.

---

## Document Types (from Compliance Team)

Based on conversation with Joana Filipova (Compliance), we receive two primary document types:

### 1. Contract Notes
- Single trade confirmation from any broker
- Sent immediately after trade execution
- May have partial names (e.g., "MR B Smith")
- Example: `/Users/luisdeburnay/Desktop/contract_note_1.pdf`

### 2. Activity Statements (Interactive Brokers)
- Monthly consolidated statements
- **Always includes full user name**
- Contains optional "Trades" section:
  - If "Trades" section exists → trades occurred, extract them
  - If "Trades" section missing → positions only, no action needed
- Example: `/Users/luisdeburnay/Desktop/ActivityStatement.202512.pdf`

### Key Fields in Activity Statements
| Column | Meaning |
|--------|---------|
| Symbol | Ticker (e.g., XBI, GLD) - NOT a currency! |
| T. Price | Trade/Transaction Price (execution price) |
| C. Price | Closing Price - **DO NOT USE** |
| Proceeds | Total value (negative=buy, positive=sell) |
| Comm/Fee | Commission charged |
| USD.SGD | Forex pair - **SKIP** (not a stock trade) |

---

## Key Insight: Narrow the Candidate Pool

Instead of matching against **all 500+ employees**, we match against a much smaller pool: **only employees who have approved requests awaiting contract notes**.

This dramatically improves matching accuracy:
- Pool size: ~5-20 people instead of 500+
- "Mr Smith" is unambiguous when only 1 Smith has an open request
- Context-aware: we're *expecting* notes from these specific people

---

## Problem Statement

### Email Source Scenarios

Documents arrive via email. The metadata email (`x-goog-meta-sender-email`) can be:

1. **From Broker**: Email comes directly from broker (e.g., `settlements@goldmansachs.com`)
   - Metadata email is NOT useful for user matching
   - Must extract name from PDF and fuzzy match against open requests

2. **From User**: User forwards document from their inbox
   - Metadata email IS the user's Mako email (e.g., `luis.deburnay-bastos@mako.com`)
   - Direct match via email → check if they have approved request

3. **Manual Upload**: User uploads via dashboard
   - No metadata email
   - Use authenticated user's identity

### Name Extraction Challenges

- Names may have titles: "Mr John Smith", "Mrs. Sarah Jones"
- Names may be partial: "J Smith", "Smith, J."
- Names may use nicknames: "Johnny" vs "John"
- Names may be reversed: "Smith, John" vs "John Smith"
- Middle names/initials: "John A. Smith"

---

## Matching Strategy

### Candidate Pool Query

```sql
SELECT DISTINCT
    pr.id as request_id,
    pr.employee_id,
    oc.forename,
    oc.surname,
    oc.pref_name,   -- nickname like "Johnny"
    oc.title,       -- "Mr", "Dr", etc.
    pr.ticker,
    pr.direction,
    pr.quantity,
    oe.email
FROM pad_request pr
JOIN bo_airflow.oracle_employee oe
    ON oe.id = pr.employee_id
LEFT JOIN bo_airflow.oracle_contact oc
    ON oc.employee_id = pr.employee_id
    AND oc.contact_group_id = 2
    AND oc.contact_type_id = 5
WHERE pr.status IN ('approved', 'auto_approved')
AND pr.execution_id IS NULL  -- No execution yet
AND pr.expires_at > NOW()     -- Not expired
```

### Matching Flow

```
PDF arrives in GCS bucket
    │
    ├─► Has @mako.com metadata email?
    │       │
    │       └─► YES → employee = get_by_email()
    │               → Does employee have approved request awaiting note?
    │                   → YES: Match found! Pass employee_id to PDF extractor
    │                   → NO: Flag for review (unexpected note)
    │
    └─► NO (broker email or missing)
            │
            ├─► Extract account holder name from PDF
            │
            └─► Fuzzy match name against candidate pool
                    │
                    ├─► Single confident match → Pass to extractor
                    ├─► Multiple matches → Use ticker/direction to disambiguate
                    └─► No match → Manual review queue
```

### Name Matching Strategy

**Key Insight**: This is a **normalization problem**, not a fuzzy matching problem. Use `nameparser` library to normalize names, then do exact matching.

#### Step 1: Normalize with nameparser
```python
from nameparser import HumanName

def normalize_name(raw_name: str) -> str:
    """Normalize to 'firstname middle lastname' lowercase, stripping titles/suffixes."""
    name = HumanName(raw_name)
    parts = [name.first, name.middle, name.last]
    return " ".join(p.lower() for p in parts if p)

# Examples:
normalize_name("Mr. John Smith")        # → "john smith"
normalize_name("Smith, John")           # → "john smith"
normalize_name("Dr. John A. Smith Jr.") # → "john a smith"
normalize_name("SMITH, JOHN")           # → "john smith"
normalize_name("Johnny Smith")          # → "johnny smith"
```

#### Step 2: Match Logic (Exact First, Fuzzy Fallback)
```python
def match_user(extracted_name: str, candidates: list[MatchCandidate]) -> MatchResult | None:
    normalized_extracted = normalize_name(extracted_name)

    # PRIORITY 1: Exact match on normalized full name
    for candidate in candidates:
        full_name = f"{candidate.forename} {candidate.surname}"
        if normalize_name(full_name) == normalized_extracted:
            return MatchResult(candidate, confidence=1.0, method="name_exact")

        # Also check pref_name (nickname)
        if candidate.pref_name:
            pref_full = f"{candidate.pref_name} {candidate.surname}"
            if normalize_name(pref_full) == normalized_extracted:
                return MatchResult(candidate, confidence=0.95, method="name_exact")

    # PRIORITY 2: Fuzzy fallback for typos only (high threshold)
    from rapidfuzz import fuzz
    best_match, best_score = None, 0
    for candidate in candidates:
        full_name = normalize_name(f"{candidate.forename} {candidate.surname}")
        score = fuzz.token_set_ratio(normalized_extracted, full_name)
        if score > best_score:
            best_score, best_match = score, candidate

    if best_score >= 95:  # Very high threshold - only catches typos
        return MatchResult(best_match, confidence=best_score/100, method="name_fuzzy")

    return None  # No match → manual review

# Matching examples after normalization:
# "Mr Smith" normalized → "smith" (no first name extracted)
#   → Falls back to surname-only match against candidate pool
# "J Smith" → "j smith"
#   → May match "john smith" via fuzzy if only 1 Smith in pool
# "Smith, John" → "john smith" ✓ exact match
```

#### Why This Works Better
| PDF Name | After normalize_name() | Match Method |
|----------|----------------------|--------------|
| `Mr. John Smith` | `john smith` | Exact |
| `Smith, John` | `john smith` | Exact |
| `SMITH, JOHN A.` | `john a smith` | Exact |
| `Dr. J. Smith Jr.` | `j smith` | Fuzzy (if unambiguous) |
| `Johnny Smith` | `johnny smith` | Exact (via pref_name) |

---

## Requirements

### Functional Requirements

1. **Candidate Pool Loading**
   - Query employees with `approved` or `auto_approved` requests
   - Include forename, surname, pref_name from OracleContact
   - Cache for performance (invalidate on request status change)

2. **Email-Based Matching** (Priority 1)
   - If metadata email is `@mako.com`, resolve to employee
   - Verify employee has an approved request awaiting note
   - If no approved request, flag as unexpected

3. **Name Extraction from PDF**
   - Use AI parser to extract account holder name
   - Store raw and normalized versions
   - Non-blocking: continue with manual review if extraction fails

4. **Fuzzy Name Matching** (Priority 2)
   - Match extracted name against candidate pool only
   - Handle title prefixes, initials, nicknames, reversed names
   - Score matches by confidence

5. **Request Linkage**
   - Once employee matched, find their approved request(s)
   - If multiple requests, use ticker/direction to disambiguate
   - Link ParsedTrade to PADRequest

6. **Auto-Verification**
   - High-confidence matches auto-verify
   - Discrepancies → flag for compliance review

### Non-Functional Requirements

- Extraction should not block ingestion (fail gracefully)
- Matching decisions must be auditable
- Manual review queue for unmatched documents

---

## Schema Additions

### GCSDocument (existing)
```python
sender_email: str | None       # Already exists - from x-goog-meta-sender-email
email_source: str | None       # NEW: 'broker', 'user', 'manual', 'unknown'
```

### ParsedTrade (existing)
```python
extracted_account_holder: str | None   # NEW: Raw name from PDF
normalized_account_holder: str | None  # NEW: After title stripping
matched_employee_id: int | None        # NEW: FK to employee if matched
matched_request_id: int | None         # NEW: FK to pad_request if linked
match_method: str | None               # NEW: 'email', 'name_exact', 'name_fuzzy', 'manual'
match_confidence: float | None         # NEW: 0.0-1.0
```

### ExtractedTradeData (Pydantic model) - MAJOR ENHANCEMENT
See "Enhanced Extraction Schema" section below.

---

## PDF Extraction Pipeline Refactor

### Current Problem

The current PDF parser:
1. Assumes all uploads are contract notes
2. Uses plain `litellm.acompletion()` with `response_format={"type": "json_object"}` - **NO schema enforcement**
3. Has no retry logic on validation failures
4. Has hardcoded `confidence_score=0.5`
5. Cannot distinguish activity statements from contract notes

### Solution: Instructor + LiteLLM Integration

**Key Change**: Replace raw LiteLLM calls with `instructor.from_litellm()` for:
- Schema-enforced structured output
- Automatic validation retries with error feedback
- Self-healing extraction (model corrects its own mistakes)

```python
# ❌ CURRENT (problematic)
response = await litellm.acompletion(
    model="openai/gemini-2.0-flash-exp",
    messages=[...],
    response_format={"type": "json_object"},  # No schema!
)
data = json.loads(response.choices[0].message.content)  # Can fail
return ExtractedTradeData(**data)  # No retry on error

# ✅ NEW (self-healing)
import instructor
from litellm import acompletion

client = instructor.from_litellm(acompletion)

result = await client.chat.completions.create(
    model="gemini/gemini-2.0-flash-exp",
    response_model=ExtractedTradeData,  # Schema enforced!
    messages=[...],
    max_retries=3,  # Auto-retry with validation error feedback
)
```

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ENHANCED EXTRACTION PIPELINE                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────┐    ┌─────────────┐    ┌──────────────┐    ┌────────────────┐ │
│  │ Document │───▶│ PASS 1:     │───▶│ PASS 2:      │───▶│ Instructor     │ │
│  │  Input   │    │ Classify    │    │ Extract      │    │ Validation     │ │
│  └──────────┘    │ Doc Type    │    │ Trade Data   │    │ + Auto-Retry   │ │
│                  └─────────────┘    └──────────────┘    └────────────────┘ │
│                         │                  │                     │          │
│                         ▼                  ▼                     ▼          │
│                  ┌──────────────────────────────────────────────────────┐  │
│                  │              CONFIDENCE-BASED ROUTING                 │  │
│                  ├──────────────────────────────────────────────────────┤  │
│                  │  NOT_CONTRACT_NOTE/ACTIVITY_STMT ──▶ Route by type   │  │
│                  │  needs_human_review = true    ──────▶ Manual Review  │  │
│                  │  confidence = LOW (<0.5)      ──────▶ Manual Review  │  │
│                  │  confidence = MEDIUM (0.5-0.8) ─────▶ Auto + Audit   │  │
│                  │  confidence = HIGH (≥0.8)     ──────▶ Auto-Approve   │  │
│                  └──────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Enhanced Extraction Schema

### Design Principles (from Best Practices Research)

1. **Rich Field Descriptions**: Include examples and anti-examples in Field descriptions
2. **Capture All Identifiers**: Try to extract ANY available identifier (ticker, ISIN, SEDOL, CUSIP, Bloomberg)
3. **Self-Healing Validators**: Pydantic validators that feed errors back to LLM for correction
4. **Model-Generated Confidence**: LLM rates its own extraction quality
5. **Human Review Triggers**: Explicit flag when manual intervention needed

### ExtractedTradeData Schema

```python
from enum import Enum
from decimal import Decimal
from datetime import date
from pydantic import BaseModel, Field, field_validator, model_validator
from typing import Literal


class ConfidenceLevel(str, Enum):
    HIGH = "high"      # ≥0.8 → Auto-approve
    MEDIUM = "medium"  # 0.5-0.8 → Auto + audit flag
    LOW = "low"        # <0.5 → Manual review


class DocumentType(str, Enum):
    CONTRACT_NOTE = "CONTRACT_NOTE"
    ACTIVITY_STATEMENT = "ACTIVITY_STATEMENT"
    OTHER = "OTHER"


class ExtractedTradeData(BaseModel):
    """
    Trade data extracted from a contract note or activity statement.

    IMPORTANT: This schema uses rich descriptions to guide extraction.
    The descriptions become part of the LLM prompt context.
    """

    # === DOCUMENT CLASSIFICATION ===
    document_type: DocumentType = Field(
        description="""Classify the document type:
        - CONTRACT_NOTE: Single trade confirmation with execution details
        - ACTIVITY_STATEMENT: Monthly/periodic statement (e.g., Interactive Brokers)
        - OTHER: Holdings summary, fund factsheet, or unrecognized format"""
    )

    has_trades_section: bool | None = Field(
        None,
        description="""For ACTIVITY_STATEMENT only:
        Does the document contain a "Trades" section with actual transactions?
        True = trades occurred, extract them
        False = positions only, no trades to extract"""
    )

    # === ACCOUNT INFORMATION (CRITICAL FOR MATCHING) ===
    account_holder_name: str | None = Field(
        None,
        description="""Full name of the account holder/client.
        Look for: "Account Holder", "Client Name", "Name", header area
        Examples: "John Smith", "Mr. J. Smith", "SMITH, JOHN A."
        This is CRITICAL for matching to PAD requests."""
    )

    account_number: str | None = Field(
        None,
        description="""Account/portfolio number if shown.
        Examples: "U1234567", "ABC-12345"
        DO NOT confuse with SEDOL or other security identifiers."""
    )

    broker_name: str | None = Field(
        None,
        description="""Name of the broker/firm.
        Examples: "Interactive Brokers", "Goldman Sachs", "Charles Schwab"
        Usually in header or footer of document."""
    )

    # === SECURITY IDENTIFIERS (TRY TO GET ALL AVAILABLE) ===
    ticker: str | None = Field(
        None,
        description="""Stock ticker symbol.
        Examples: AAPL, MSFT, GLD, XBI
        Note: GLD is a gold ETF ticker, NOT a currency!
        Look in: "Symbol", "Ticker", "Stock" columns"""
    )

    symbol: str | None = Field(
        None,
        description="""Any symbol/code if ticker format unclear.
        Use this for non-standard identifiers."""
    )

    isin: str | None = Field(
        None,
        description="""ISIN - International Securities Identification Number.
        MUST be exactly 12 characters.
        Format: 2-letter country + 9 alphanumeric + 1 check digit
        Examples: US0378331005 (Apple), GB00B80QG614
        DO NOT confuse account numbers with ISIN."""
    )

    sedol: str | None = Field(
        None,
        description="""SEDOL - Stock Exchange Daily Official List identifier.
        MUST be exactly 7 alphanumeric characters.
        Examples: B80QG61, 2046251
        Common in UK broker documents.
        DO NOT confuse account numbers with SEDOL."""
    )

    cusip: str | None = Field(
        None,
        description="""CUSIP - Committee on Uniform Securities Identification Procedures.
        MUST be exactly 9 characters.
        Examples: 037833100 (Apple), 594918104
        Common in US broker documents."""
    )

    bloomberg_ticker: str | None = Field(
        None,
        description="""Bloomberg terminal ticker format.
        Examples: AAPL US Equity, VOD LN Equity
        Format usually: TICKER + EXCHANGE + "Equity" """
    )

    security_description: str | None = Field(
        None,
        description="""Full name/description of the security.
        Examples: "Apple Inc.", "SPDR Gold Shares", "iShares S&P 500 ETF"
        Helps identify security when ticker/ISIN unavailable."""
    )

    # === TRADE DETAILS ===
    execution_date: date | None = Field(
        None,
        description="""Date the trade was executed.
        Look for: "Trade Date", "Execution Date", "Date/Time" column
        Format as YYYY-MM-DD"""
    )

    execution_time: str | None = Field(
        None,
        description="""Time of trade execution if available.
        Format: HH:MM:SS
        Example: 10:31:41"""
    )

    settlement_date: date | None = Field(
        None,
        description="""Settlement date if shown.
        Usually T+1 or T+2 from execution date.
        Format as YYYY-MM-DD"""
    )

    direction: str | None = Field(
        None,
        description="""Trade direction - MUST be exactly "BUY" or "SELL".
        Indicators for BUY: "Buy", "Purchase", "Bought", negative proceeds
        Indicators for SELL: "Sell", "Sale", "Sold", positive proceeds"""
    )

    quantity: Decimal | None = Field(
        None,
        description="""Number of shares/units traded.
        Preserve decimals for fractional shares (e.g., 132.449)
        Always positive number for both buys and sells.
        Remove commas: "1,000" → 1000
        Examples: 26, 100, 132.449"""
    )

    price: Decimal | None = Field(
        None,
        description="""Execution price per share/unit.

        EXTRACT: "T. Price", "Trade Price", "Execution Price", "Price"

        DO NOT EXTRACT (wrong values):
        - "C. Price" (closing price)
        - "Avg Cost", "Average Cost"
        - "Book Cost per Unit"
        - "Current Price", "Market Price"

        Remove currency symbols: "$395.97" → 395.97
        Examples: 395.97, 131.45, 0.5632"""
    )

    currency: str | None = Field(
        None,
        description="""3-letter ISO currency code.
        Examples: USD, GBP, EUR, SGD, JPY
        Look near price or in header."""
    )

    proceeds: Decimal | None = Field(
        None,
        description="""Total cash proceeds from trade.

        NEGATIVE for purchases (cash outflow): -10,295.22 means bought $10,295.22 worth
        POSITIVE for sales (cash inflow): 9,521.21 means sold and received $9,521.21

        Should approximately equal: quantity × price (with sign based on direction)"""
    )

    commission: Decimal | None = Field(
        None,
        description="""Broker commission/fee.
        Usually NEGATIVE or zero.
        Examples: -1.09 means $1.09 fee charged
        If no commission shown, use 0.00
        Look for: "Comm/Fee", "Commission", "Fee" """
    )

    broker_reference: str | None = Field(
        None,
        description="""Broker's reference/confirmation number.
        Examples: "REF123456", "CONF-2025-001"
        Useful for reconciliation."""
    )

    # === CONFIDENCE & REVIEW FLAGS ===
    confidence: ConfidenceLevel = Field(
        default=ConfidenceLevel.MEDIUM,
        description="""Your confidence in this extraction:

        HIGH: All values clearly readable, math checks out (qty × price ≈ proceeds)
        MEDIUM: Some values inferred or partially unclear, but reasonable
        LOW: Values guessed, OCR issues, math doesn't add up, or critical fields missing

        Be honest - LOW confidence triggers manual review which is better than bad data."""
    )

    confidence_notes: str | None = Field(
        None,
        description="""Explain any issues or uncertainties:
        Examples:
        - "price partially obscured, used visible digits"
        - "assumed BUY based on negative proceeds"
        - "could not find ISIN, only ticker available"
        - "multiple trades in document, extracted first one" """
    )

    needs_human_review: bool = Field(
        default=False,
        description="""Set to True if ANY of these apply:
        - confidence is LOW
        - Critical fields missing (account_holder_name, direction, quantity)
        - Math doesn't add up (proceeds ≠ qty × price within 5%)
        - Document appears to have multiple trades
        - Unusual format or potential OCR issues"""
    )

    review_reasons: list[str] = Field(
        default_factory=list,
        description="""List specific reasons for human review:
        Examples:
        - "missing account_holder_name"
        - "proceeds math off by 15%"
        - "could not determine direction"
        - "document contains 3 trades, only extracted first" """
    )

    # === VALIDATORS FOR SELF-HEALING ===

    @field_validator("isin")
    @classmethod
    def validate_isin_format(cls, v):
        """ISIN must be exactly 12 characters."""
        if v is not None and len(v) != 12:
            raise ValueError(f"ISIN must be exactly 12 characters, got {len(v)}: '{v}'")
        return v

    @field_validator("sedol")
    @classmethod
    def validate_sedol_format(cls, v):
        """SEDOL must be exactly 7 characters."""
        if v is not None and len(v) != 7:
            raise ValueError(f"SEDOL must be exactly 7 characters, got {len(v)}: '{v}'")
        return v

    @field_validator("cusip")
    @classmethod
    def validate_cusip_format(cls, v):
        """CUSIP must be exactly 9 characters."""
        if v is not None and len(v) != 9:
            raise ValueError(f"CUSIP must be exactly 9 characters, got {len(v)}: '{v}'")
        return v

    @field_validator("direction")
    @classmethod
    def validate_direction(cls, v):
        """Direction must be BUY or SELL."""
        if v is not None and v.upper() not in ("BUY", "SELL"):
            raise ValueError(f"Direction must be 'BUY' or 'SELL', got '{v}'")
        return v.upper() if v else v

    @model_validator(mode="after")
    def check_proceeds_math(self):
        """Validate that proceeds ≈ quantity × price."""
        if self.quantity and self.price and self.proceeds:
            expected = float(self.quantity) * float(self.price)
            if self.direction == "BUY":
                expected = -expected

            actual = float(self.proceeds)
            tolerance = abs(expected * 0.05) + 10  # 5% + $10 for fees

            if abs(actual - expected) > tolerance:
                # Don't fail, but flag for review
                self.needs_human_review = True
                self.review_reasons.append(
                    f"Proceeds ({actual:.2f}) doesn't match qty×price ({expected:.2f})"
                )
        return self

    @model_validator(mode="after")
    def check_critical_fields(self):
        """Flag if critical fields are missing."""
        missing = []
        if not self.account_holder_name:
            missing.append("account_holder_name")
        if not self.direction:
            missing.append("direction")
        if not self.quantity:
            missing.append("quantity")
        if not (self.ticker or self.isin or self.sedol or self.symbol):
            missing.append("security identifier (ticker/isin/sedol)")

        if missing:
            self.needs_human_review = True
            self.review_reasons.append(f"Missing critical fields: {', '.join(missing)}")

        return self
```

---

## Extraction Prompts

### Financial Domain Rules (Include in All Prompts)

```python
FINANCIAL_RULES = """
IMPORTANT FINANCIAL CONVENTIONS:
1. Quantities: Always POSITIVE numbers for both buys and sells
   - The 'direction' field indicates buy vs sell, not the quantity sign

2. Proceeds/Amount:
   - NEGATIVE for purchases (cash outflow): -10,295.22 means you paid $10,295.22
   - POSITIVE for sales (cash inflow): 9,521.21 means you received $9,521.21

3. Settlement Date:
   - Typically T+1 (next business day) for most equities
   - T+2 for some markets/instruments
   - If not shown, omit (don't guess)

4. Commission/Fees:
   - May be embedded in the price (net price) OR shown separately
   - Extract if visible, otherwise omit
   - Usually negative or zero

5. Price Types - CRITICAL:
   - Extract EXECUTION/TRADE price only (what was actually paid)
   - NEVER extract: closing price, average cost, book value, market price

6. Missing Data:
   - If a field is not clearly present in the document, OMIT it
   - Do NOT guess or infer values
   - It's better to have null than wrong data
"""
```

### Two-Pass Extraction Strategy

For Activity Statements (which have multiple similar-looking tables), use two passes:

**PASS 1: Section Identification**
```
Analyze this financial statement and identify ALL sections/tables present.

For each section, note:
- Section name (exactly as shown)
- Column headers
- Whether it contains trade transactions

Common sections in IBKR statements:
- "Trades" → ACTUAL TRADES (what we want)
- "Mark-to-Market Performance Summary" → NOT trades (unrealized P/L)
- "Open Positions" → NOT trades (current holdings)
- "Cash Report" → NOT trades (summary)

Return the list of sections found.
```

**PASS 2: Targeted Extraction**
```
Extract trade data ONLY from the "Trades" section, subsection "Stocks".

## EXTRACT FROM (correct section):
The "Trades" table with columns: Symbol, Date/Time, Quantity, T. Price, Proceeds, Comm/Fee
Rows like: GLD  2025-12-12, 10:31:41  26  395.9700  -10,295.22  -1.09

## DO NOT EXTRACT FROM (wrong sections - IGNORE completely):
1. "Mark-to-Market Performance Summary"
   - Has columns: Symbol, Prior Qty, Current Qty, Prior Price, Current Price
   - This shows UNREALIZED gains, NOT actual trades

2. "Open Positions"
   - Has columns: Symbol, Quantity, Cost Price, Close Price
   - This shows CURRENT HOLDINGS, NOT trade history

3. "Cash Report"
   - Shows: Starting Cash, Deposits, Withdrawals, Trades (Summary)
   - This is a SUMMARY, NOT individual trades

4. "Forex" subsection under Trades
   - Rows with symbols like "USD.SGD" or "EUR.USD"
   - We only want STOCK trades

## Column Mapping:
- "T. Price" = Trade Price (USE THIS)
- "C. Price" = Closing Price (DO NOT USE)
- "Proceeds" = Total value (negative=buy, positive=sell)

If a row has a currency pair as Symbol (USD.SGD), SKIP it.
```

### Contract Note Extraction Prompt

```
Extract trade details from this contract note / trade confirmation.

## ACCOUNT INFORMATION (CRITICAL):
Find the account holder's FULL NAME - usually in header or "Client" field.
Also note account number and broker name if visible.

## SECURITY IDENTIFIERS (extract ALL available):
- Ticker/Symbol (e.g., AAPL, MSFT)
- ISIN (exactly 12 characters, e.g., US0378331005)
- SEDOL (exactly 7 characters, e.g., B80QG61)
- CUSIP (exactly 9 characters)
- Full security description

## TRADE DETAILS:
- Direction: "BUY" or "SELL" (infer from context if not explicit)
- Quantity: exact number of shares (preserve decimals)
- Price: EXECUTION price per share (NOT average cost, NOT closing price)
- Currency: 3-letter code (USD, GBP, EUR)
- Dates: execution date and settlement date

## CONFIDENCE:
- Rate your extraction confidence: HIGH, MEDIUM, or LOW
- Note any issues in confidence_notes
- Set needs_human_review=true if uncertain about critical fields

Return as JSON matching the ExtractedTradeData schema.
```

---

## Confidence-Based Routing

| Confidence | needs_human_review | Routing | Action |
|------------|-------------------|---------|--------|
| HIGH | false | Auto-Approve | Process and verify against PAD request |
| HIGH | true | Manual Review | Something flagged despite high confidence |
| MEDIUM | false | Auto + Audit Flag | Process but mark for compliance review |
| MEDIUM | true | Manual Review | Medium confidence + issues |
| LOW | * | Manual Review | Low confidence always needs review |
| * | true | Manual Review | Explicit review flag always triggers |

### Operational Note: Threshold Tuning

Start conservative (more manual review), then loosen thresholds after reviewing false positives/negatives:

```python
# Week 1-2: Conservative (build confidence in system)
THRESHOLDS_INITIAL = {
    "high_confidence": 0.95,   # Almost perfect only
    "medium_confidence": 0.85,  # Most things get audited
}

# Week 3-4: After reviewing edge cases
THRESHOLDS_TUNED = {
    "high_confidence": 0.90,
    "medium_confidence": 0.80,
}

# Mature system (after months of consistent ≥95% accuracy)
THRESHOLDS_MATURE = {
    "high_confidence": 0.85,
    "medium_confidence": 0.70,
}
```

**Rule**: Only loosen thresholds when false positive rate is consistently <1%.

---

## Document Classification

### CONTRACT_NOTE indicators:
- Title contains: "Contract Note", "Trade Confirmation", "Execution Confirmation"
- Shows SINGLE trade execution with price, quantity, direction
- Settlement details and fees breakdown
- Broker reference/contract number

### ACTIVITY_STATEMENT indicators:
- Title contains: "Activity Statement", "Account Statement", "Monthly Statement"
- Multiple sections (Trades, Positions, Cash Report)
- From Interactive Brokers (standardized format)
- Period covered (e.g., "December 2025")

### NOT processable (classify as OTHER):
- Holdings summaries / portfolio valuations
- Investment reports / performance summaries
- Fund factsheets
- Documents without trade information

---

## Combined Flow: Matching + Extraction + Validation

```
PDF arrives in GCS bucket
    │
    ├─► PHASE 1: USER MATCHING
    │       │
    │       ├─► @mako.com metadata email?
    │       │       └─► YES → employee = get_by_email()
    │       │               → Has approved request? Match found!
    │       │
    │       └─► NO → Will match after extraction using account_holder_name
    │
    ├─► PHASE 2: DOCUMENT CLASSIFICATION (Pass 1)
    │       │
    │       ├─► CONTRACT_NOTE → Single-pass extraction
    │       ├─► ACTIVITY_STATEMENT → Check for Trades section
    │       │       ├─► Has Trades section → Extract trades
    │       │       └─► No Trades section → Log "positions only", archive
    │       └─► OTHER → Route to manual review
    │
    ├─► PHASE 3: EXTRACTION (Pass 2)
    │       │
    │       ├─► Use instructor.from_litellm() with schema
    │       ├─► max_retries=3 for self-healing
    │       ├─► Extract all available identifiers
    │       └─► Model provides confidence + review flags
    │
    ├─► PHASE 4: USER MATCHING (if not matched in Phase 1)
    │       │
    │       ├─► Extract account_holder_name from result
    │       ├─► Fuzzy match against candidate pool
    │       │       ├─► Single match → Link to employee
    │       │       ├─► Multiple matches → Disambiguate by ticker/direction
    │       │       └─► No match → Flag for manual matching
    │       │
    │       └─► Update parsed_trade with matched_employee_id
    │
    ├─► PHASE 5: ROUTING
    │       │
    │       ├─► needs_human_review = true → Manual Review Queue
    │       ├─► confidence = LOW → Manual Review Queue
    │       ├─► confidence = MEDIUM → Auto-Process + Audit Flag
    │       └─► confidence = HIGH → Auto-Process
    │
    └─► PHASE 6: VALIDATION (if auto-processing)
            │
            ├─► Direction: exact match against PAD request
            ├─► Quantity: exact match
            ├─► Ticker/ISIN: fuzzy match
            ├─► Price: 5% tolerance vs estimated
            │
            └─► VERIFIED / VALIDATION_FAILED / VERIFIED_WITH_AUDIT
```

---

## Implementation Phases

### Phase 0: Core Infrastructure
**Priority: P0 (Critical)**

1. Add `instructor` to dependencies (`pyproject.toml`)
2. Refactor `DocumentAgent` to use `instructor.from_litellm(acompletion)`
3. Add `max_retries=3` for self-healing extraction
4. Verify Gemini model compatibility with instructor

### Phase 1: Enhanced Schema
**Priority: P0 (Critical)**

1. Implement full `ExtractedTradeData` schema with:
   - Rich field descriptions
   - All identifier fields (ticker, ISIN, SEDOL, CUSIP, Bloomberg, symbol)
   - `account_holder_name` field
   - Confidence scoring fields (`confidence`, `confidence_notes`)
   - Human review triggers (`needs_human_review`, `review_reasons`)
2. Add Pydantic validators for self-healing:
   - ISIN/SEDOL/CUSIP format validators
   - Direction normalization
   - Proceeds math check
   - Critical fields check

### Phase 2: Document Classification
**Priority: P1 (High)**

1. Implement two-pass extraction for Activity Statements
2. Add document type classification logic
3. Create negative examples in prompts (sections to ignore)
4. Handle "no trades" case for activity statements

### Phase 3: Database Migration
**Priority: P1 (High)**

```sql
-- Alembic migration
ALTER TABLE padealing.gcs_document
ADD COLUMN email_source VARCHAR(20)
CHECK (email_source IN ('broker', 'user', 'manual', 'unknown'));

ALTER TABLE padealing.parsed_trade
ADD COLUMN extracted_account_holder VARCHAR(200),
ADD COLUMN normalized_account_holder VARCHAR(200),
ADD COLUMN matched_employee_id BIGINT REFERENCES bo_airflow.oracle_employee(id),
ADD COLUMN matched_request_id INTEGER REFERENCES pad_request(id),
ADD COLUMN match_method VARCHAR(20) CHECK (
    match_method IN ('email', 'name_exact', 'name_fuzzy', 'manual')
),
ADD COLUMN match_confidence NUMERIC(3,2) CHECK (
    match_confidence >= 0 AND match_confidence <= 1
);

CREATE INDEX ix_parsed_trade_matched_employee
ON padealing.parsed_trade(matched_employee_id);

CREATE INDEX ix_parsed_trade_matched_request
ON padealing.parsed_trade(matched_request_id);
```

### Phase 4: User Matching
**Priority: P1 (High)**

1. Implement email-based matching (check @mako.com sender)
2. Implement fuzzy name matching against candidate pool
3. Add ticker/direction disambiguation for multiple matches
4. Update `parsed_trade` with matching results

### Phase 5: Routing & Review
**Priority: P2 (Medium)**

1. Implement confidence-based routing logic
2. Add `needs_human_review` trigger handling
3. Create API endpoint for unmatched documents: `GET /documents/unmatched`
4. Add manual matching endpoint: `POST /documents/{id}/match`

### Phase 6: Testing
**Priority: P0 (Critical - but last in sequence)**

Tests should be comprehensive and descriptive for Gemini debugging:

```python
class TestExtractedTradeDataValidators:
    """Test Pydantic validators for self-healing extraction."""

    def test_isin_validator_rejects_wrong_length(self):
        """
        GIVEN: An ISIN with wrong length (10 chars instead of 12)
        WHEN: Creating ExtractedTradeData with this ISIN
        THEN: Should raise ValidationError with clear message

        This tests the self-healing feedback loop - the error message
        will be sent back to the LLM for correction.
        """
        with pytest.raises(ValidationError) as exc_info:
            ExtractedTradeData(isin="US03783310")  # 10 chars

        assert "ISIN must be exactly 12 characters" in str(exc_info.value)
        assert "got 10" in str(exc_info.value)

    def test_proceeds_math_check_flags_for_review(self):
        """
        GIVEN: A trade where proceeds doesn't match qty × price
        WHEN: Validation runs
        THEN: Should set needs_human_review=True and add review reason

        Expected: Trade with qty=100, price=10, proceeds=-500 (should be -1000)
        should be flagged because 100 × 10 = 1000, not 500.
        """
        trade = ExtractedTradeData(
            document_type=DocumentType.CONTRACT_NOTE,
            direction="BUY",
            quantity=Decimal("100"),
            price=Decimal("10.00"),
            proceeds=Decimal("-500.00"),  # Wrong! Should be -1000
        )

        assert trade.needs_human_review is True
        assert any("proceeds" in r.lower() for r in trade.review_reasons)


class TestActivityStatementExtraction:
    """Test extraction from Interactive Brokers activity statements."""

    def test_activity_statement_with_trades_section(self):
        """
        GIVEN: An IBKR activity statement PDF with a "Trades" section
        WHEN: Processing through DocumentAgent
        THEN: Should:
            1. Classify as ACTIVITY_STATEMENT
            2. Set has_trades_section=True
            3. Extract the account_holder_name
            4. Extract trades from "Trades" section only
            5. NOT extract from "Mark-to-Market" or "Open Positions"

        Test file: /Users/luisdeburnay/Desktop/ActivityStatement.202512.pdf
        Expected account holder: [Name from statement]
        Expected to find trades with symbols like GLD, XBI
        Expected to NOT include forex trades (USD.SGD)
        """
        pass  # Implementation

    def test_activity_statement_without_trades_section(self):
        """
        GIVEN: An IBKR activity statement with NO "Trades" section
        WHEN: Processing through DocumentAgent
        THEN: Should:
            1. Classify as ACTIVITY_STATEMENT
            2. Set has_trades_section=False
            3. Return empty trades list
            4. NOT flag for manual review (this is expected)

        This is a normal case - monthly statement with no activity.
        """
        pass  # Implementation


class TestUserMatching:
    """Test user matching against candidate pool."""

    def test_email_matching_mako_domain(self):
        """
        GIVEN: A document with sender_email "john.smith@mako.com"
        AND: An employee with email "john.smith@mako.com" in database
        AND: That employee has an approved PAD request
        WHEN: Running user matching
        THEN: Should match to that employee with method='email'
        """
        pass

    def test_fuzzy_name_matching_title_prefix(self):
        """
        GIVEN: Extracted account_holder_name "Mr J Smith"
        AND: Candidate pool contains employee with forename="John", surname="Smith"
        WHEN: Running fuzzy name matching
        THEN: Should match with high confidence

        Tests that "Mr" title is stripped and "J" matches initial of "John".
        """
        pass

    def test_disambiguation_by_ticker(self):
        """
        GIVEN: Extracted name matches TWO employees (both named "Smith")
        AND: Extracted ticker is "AAPL"
        AND: Only one Smith has an approved request for AAPL
        WHEN: Running disambiguation
        THEN: Should match to the Smith with the AAPL request
        """
        pass
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Auto-match rate (IBKR) | 90% | Documents matched without manual intervention |
| Auto-match rate (other) | 70% | Non-IBKR documents matched automatically |
| False positive rate | <1% | Incorrect auto-matches |
| Extraction accuracy | 95% | Fields correctly extracted |
| Processing time | <30s | GCS arrival to matching complete |
| Manual review reduction | 75% | Compared to current (all manual) |

---

## Out of Scope

- OCR for scanned PDFs (assume machine-readable)
- Multi-trade contract notes beyond activity statements
- Broker domain whitelist management UI
- Matching against all employees (only match against open requests)
- pdfplumber hybrid extraction (pure LLM for now)

---

## Dependencies

### New Python Packages
```toml
[project.dependencies]
instructor = "^1.0.0"  # Structured output with validation retries
nameparser = "^1.1.0"  # Human name parsing (titles, suffixes, reversed formats)
```

### Existing (no changes needed)
- `litellm` - Already installed
- `pydantic` - Already installed
- `google-cloud-storage` - Already installed
- `rapidfuzz` - Already installed (fallback for typos only)

---

## Best Practices & Tooling Rationale

This section documents the key tooling decisions and the best practices that informed them.

### 1. Instructor for Structured Output
**Tool**: [instructor](https://python.useinstructor.com/) with `instructor.from_litellm()`

**Why**: Single-pass LLM extraction fails ~15-20% on complex financial tables. Instructor feeds Pydantic validation errors back to the model automatically, creating a self-correcting loop that dramatically improves reliability.

```python
# Source: https://python.useinstructor.com/concepts/retrying/
client = instructor.from_litellm(acompletion)
result = await client.chat.completions.create(
    response_model=ExtractedTradeData,
    max_retries=3,  # Auto-retry with error feedback
)
```

### 2. Rich Pydantic Field Descriptions
**Tool**: Pydantic `Field(description=...)` with examples

**Why**: Field descriptions become part of the prompt context and significantly improve extraction accuracy on ambiguous financial data. Google's structured output docs state: "Use the description field to provide clear instructions about what each property represents."

```python
# Source: https://ai.google.dev/gemini-api/docs/structured-output
price: Decimal | None = Field(
    description="""Execution price per share.
    EXTRACT: "T. Price", "Trade Price"
    DO NOT EXTRACT: "C. Price" (closing), "Avg Cost" """
)
```

### 3. nameparser for Name Normalization
**Tool**: [nameparser](https://nameparser.readthedocs.io/)

**Why**: Name matching is a normalization problem, not a fuzzy matching problem. `nameparser` handles titles (Mr/Mrs/Dr), suffixes (Jr/Sr/PhD), reversed formats (Smith, John), and case normalization out of the box - exactly what we need for broker PDFs.

```python
# Source: https://nameparser.readthedocs.io/
from nameparser import HumanName
normalize_name("Dr. John A. Smith Jr.")  # → "john a smith"
normalize_name("Smith, John")            # → "john smith"
```

### 4. Two-Pass Extraction for Complex Documents
**Pattern**: Section identification → Targeted extraction

**Why**: Activity statements have 10+ similar-looking tables. Single-pass extraction confuses "Trades" with "Mark-to-Market" or "Open Positions". Two-pass approach (identify sections first, then extract from correct section) prevents table confusion.

```python
# Source: Google Developers Blog - Multi-agent document processing
# https://developers.googleblog.com/en/build-a-multi-agent-system-for-sophisticated-document-processing/
# Pass 1: "List all sections/tables in this document"
# Pass 2: "Extract ONLY from the 'Trades' section"
```

### 5. Pydantic Validators for Self-Healing
**Pattern**: `@field_validator` and `@model_validator` that raise clear errors

**Why**: When validators raise `ValueError`, Instructor sends the error message back to the LLM for correction. This creates a self-healing loop - the model learns from its mistakes mid-extraction.

```python
# Source: https://python.useinstructor.com/concepts/validation/
@field_validator("isin")
def validate_isin(cls, v):
    if v and len(v) != 12:
        raise ValueError(f"ISIN must be 12 chars, got {len(v)}")  # Sent back to LLM
    return v
```

### 6. Explicit Negative Examples in Prompts
**Pattern**: "DO NOT EXTRACT" sections in prompts

**Why**: Financial PDFs have "distractor" tables with similar formats. Research shows that explicitly telling the model what NOT to extract eliminates table confusion more effectively than positive-only instructions.

```python
# Source: Prompt engineering research
# "DO NOT EXTRACT from 'Mark-to-Market Performance Summary' -
#  this shows unrealized P/L, NOT actual trades"
```

### 7. GCS Generation Number for Idempotency
**Pattern**: Use `blob.generation` as deduplication key with UNIQUE constraint

**Why**: GCS notifications are at-least-once delivery. Multiple notifications for the same upload can occur. The generation number is the only truly unique identifier per object version - using it prevents duplicate processing even in multi-pod deployments.

```python
# Source: GCS documentation - Object versioning
# https://cloud.google.com/storage/docs/object-versioning
gcs_generation = blob.generation  # Immutable, unique per version
# INSERT ... ON CONFLICT (gcs_generation) DO NOTHING
```

### 8. Claim Pattern for Distributed Processing
**Pattern**: Atomic rename from `incoming/` → `processing/` → `archive/`

**Why**: Standard pattern for distributed file processing. GCS rename is atomic - no two workers can claim the same file. Keeps full history for debugging and reprocessing.

```python
# Source: Distributed systems best practices
# incoming/doc.pdf → processing/{uuid}.pdf → archive/2026/02/{uuid}.pdf
#                  → errors/{uuid}.pdf (on failure)
```

### 9. Confidence-Based Routing (3-Tier)
**Pattern**: HIGH → auto, MEDIUM → auto+audit, LOW → manual

**Why**: Binary match/no-match forces bad decisions. 3-tier routing reduces human workload by 60-80% while maintaining accuracy. The middle tier lets you process quickly while building confidence in the system.

```python
# Source: Human-in-the-loop ML best practices
if confidence >= 0.90: auto_approve()
elif confidence >= 0.80: auto_with_audit_flag()
else: manual_review_queue()
```

### 10. Conservative Thresholds → Loosen Over Time
**Pattern**: Start with high human oversight, reduce after validation

**Why**: Research consistently shows: start conservative, then "gradually reduce manual review once confidence is high, perhaps moving to spot-checks after consistent ≥95% accuracy."

```python
# Week 1-2: AUTO_APPROVE ≥ 0.95 (almost everything reviewed)
# Mature:   AUTO_APPROVE ≥ 0.85 (only edge cases reviewed)
```

---

## References

- [Instructor Documentation](https://python.useinstructor.com/)
- [LiteLLM + Instructor Integration](https://docs.litellm.ai/docs/tutorials/instructor)
- [Pydantic Validators](https://docs.pydantic.dev/latest/concepts/validators/)
- Compliance conversation with Joana Filipova (2026-02-02)
- Best Practices Research: "10 Must-Do Best Practices for Financial PDF Extraction"
