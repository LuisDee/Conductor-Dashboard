# Plan: Contract Note User Matching & Verification

**Status:** Complete (2026-02-02)

**Depends on:** `gcs_pdf_ingestion_20260129` (COMPLETE)

---

## Overview

This track has two major components:

1. **User Matching** (Phases 1-5): Match incoming PDFs to employees with approved requests
2. **Extraction Pipeline Refactor** (Phases 6-10): Structured extraction with classification and routing

---

# PART A: USER MATCHING

---

## Phase 1: Candidate Pool Service

**Goal**: Build service to query employees with approved requests awaiting contract notes.

### Implementation

Create `src/pa_dealing/services/contract_note_candidates.py`:

```python
from dataclasses import dataclass
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from pa_dealing.db.models.core import OracleContact
from pa_dealing.db.models.pad import PADRequest

@dataclass
class AwaitingNoteCandidate:
    """Employee with an approved request awaiting contract note."""
    employee_id: int
    request_id: int
    forename: str | None
    surname: str | None
    pref_name: str | None  # Nickname
    ticker: str | None
    direction: str
    quantity: int
    google_email: str | None

async def get_candidates_awaiting_notes(
    session: AsyncSession,
) -> list[AwaitingNoteCandidate]:
    """
    Get all employees with approved requests awaiting contract notes.

    Returns small pool (~5-20) for fuzzy matching.
    """
    stmt = (
        select(
            PADRequest.id.label("request_id"),
            PADRequest.employee_id,
            OracleContact.forename,
            OracleContact.surname,
            OracleContact.pref_name,
            PADRequest.ticker,
            PADRequest.direction,
            PADRequest.quantity,
            PADRequest.google_email,
        )
        .join(
            OracleContact,
            (OracleContact.employee_id == PADRequest.employee_id)
            & (OracleContact.contact_group_id == 2)
            & (OracleContact.contact_type_id == 5),
        )
        .where(PADRequest.status.in_(["approved", "auto_approved"]))
    )
    result = await session.execute(stmt)
    return [AwaitingNoteCandidate(**row._asdict()) for row in result]
```

### Tasks

- [x] Create `AwaitingNoteCandidate` dataclass
- [x] Create `get_candidates_awaiting_notes()` function
- [x] Add unit tests with mock approved requests
- [x] Verify query joins correctly with OracleContact

### Acceptance Criteria

- [x] Returns only employees with approved/auto_approved requests
- [x] Includes forename, surname, pref_name for fuzzy matching
- [x] Includes request details (ticker, direction, quantity) for disambiguation

---

## Phase 2: Email-Based Matching

**Goal**: Match documents where sender email is a Mako employee.

### Implementation

Create `src/pa_dealing/services/contract_note_matcher.py`:

```python
from dataclasses import dataclass, field
from typing import Literal

@dataclass
class MatchResult:
    employee_id: int | None
    request_id: int | None
    match_method: Literal["email", "name_exact", "name_fuzzy", "manual", "none"]
    confidence: float  # 0.0-1.0
    reason: str  # Human-readable explanation
    candidates: list[AwaitingNoteCandidate] = field(default_factory=list)

class ContractNoteMatcherService:
    """
    Matches incoming contract notes to employees with approved requests.
    """

    async def match_by_email(
        self,
        metadata_email: str | None,
        candidates: list[AwaitingNoteCandidate],
        session: AsyncSession,
    ) -> MatchResult | None:
        """
        Try to match via metadata email.

        Returns:
            MatchResult if email is @mako.com and employee has approved request
            None if email is not @mako.com (try name matching instead)
        """
        if not metadata_email or not metadata_email.endswith("@mako.com"):
            return None  # Not a user forward, try name matching

        # Find employee by email
        identity = await self.identity_provider.get_by_email(metadata_email)
        if not identity or not identity.employee_id:
            return MatchResult(
                None, None, "none", 0.0,
                f"Could not resolve employee for {metadata_email}"
            )

        # Check if employee has approved request
        matching = [c for c in candidates if c.employee_id == identity.employee_id]
        if not matching:
            return MatchResult(
                identity.employee_id, None, "none", 0.0,
                f"Employee {metadata_email} has no approved requests awaiting notes",
                candidates=[]
            )

        if len(matching) == 1:
            return MatchResult(
                identity.employee_id,
                matching[0].request_id,
                "email",
                0.95,
                f"Matched via email {metadata_email}"
            )

        # Multiple requests - return for disambiguation
        return MatchResult(
            identity.employee_id,
            None,
            "email",
            0.7,
            f"Employee has {len(matching)} approved requests - needs disambiguation",
            candidates=matching
        )
```

### Tasks

- [x] Create `MatchResult` dataclass
- [x] Create `ContractNoteMatcherService` class
- [x] Implement `match_by_email()` method
- [x] Handle case: email resolves but no approved request
- [x] Handle case: multiple approved requests for same employee
- [x] Add unit tests

### Acceptance Criteria

- [x] @mako.com emails trigger email matching
- [x] Non-@mako.com emails skip to name matching
- [x] Returns high confidence when single request found
- [x] Returns lower confidence with candidates when multiple requests

---

## Phase 3: Name Normalization

**Goal**: Normalize extracted account holder names for fuzzy matching.

### Implementation

Create `src/pa_dealing/services/name_normalizer.py`:

```python
import re

TITLES = ["mr", "mrs", "ms", "miss", "dr", "prof", "sir", "dame"]

def normalize_name(raw_name: str) -> tuple[str | None, str | None, str]:
    """
    Normalize account holder name.

    Args:
        raw_name: Name like "Mr John Smith" or "Smith, John"

    Returns:
        (forename, surname, original)
    """
    if not raw_name:
        return None, None, raw_name

    original = raw_name.strip()
    name = original.lower()

    # Strip title
    for title in TITLES:
        if name.startswith(title + " ") or name.startswith(title + "."):
            name = name[len(title):].lstrip(". ")
            break

    # Handle "Surname, Forename" format
    if "," in name:
        parts = [p.strip() for p in name.split(",", 1)]
        surname, forename = parts[0], parts[1] if len(parts) > 1 else None
    else:
        # Assume "Forename Surname" or "Forename M. Surname"
        parts = name.split()
        if len(parts) >= 2:
            # Last part is surname, first part is forename, middle ignored
            forename = parts[0]
            surname = parts[-1]
        elif len(parts) == 1:
            forename = None
            surname = parts[0]
        else:
            forename = None
            surname = None

    return forename, surname, original
```

### Tasks

- [x] Create `normalize_name()` function
- [x] Add unit tests for edge cases:
  - Titles: "Mr John Smith" → (john, smith)
  - Reversed: "Smith, John" → (john, smith)
  - Initial: "J. Smith" → (j, smith)
  - Middle name: "John A. Smith" → (john, smith)

### Acceptance Criteria

- [x] Handles common title prefixes
- [x] Handles reversed name format
- [x] Handles initials and middle names

---

## Phase 4: Fuzzy Name Matching

**Goal**: Match extracted name against candidate pool using fuzzy matching.

### Implementation

Add to `src/pa_dealing/services/contract_note_matcher.py`:

```python
from rapidfuzz import fuzz

def match_by_name(
    self,
    extracted_name: str | None,
    candidates: list[AwaitingNoteCandidate],
) -> MatchResult:
    """
    Fuzzy match extracted name against candidates awaiting notes.
    """
    if not extracted_name:
        return MatchResult(None, None, "none", 0.0, "No name extracted from PDF")

    forename, surname, _ = normalize_name(extracted_name)

    scored_candidates = []
    for candidate in candidates:
        score = self._score_name_match(
            forename, surname, extracted_name,
            candidate.forename, candidate.surname, candidate.pref_name
        )
        if score > 0.5:
            scored_candidates.append((candidate, score))

    scored_candidates.sort(key=lambda x: x[1], reverse=True)

    if not scored_candidates:
        return MatchResult(
            None, None, "none", 0.0,
            f"No candidates matched '{extracted_name}'",
            candidates=candidates  # Return all for manual review
        )

    best, best_score = scored_candidates[0]

    # Check if match is unambiguous
    if len(scored_candidates) == 1 or scored_candidates[1][1] < best_score - 0.2:
        method = "name_exact" if best_score > 0.95 else "name_fuzzy"
        return MatchResult(
            best.employee_id,
            best.request_id,
            method,
            best_score,
            f"Matched '{extracted_name}' to {best.forename} {best.surname}"
        )

    # Ambiguous - multiple close matches
    return MatchResult(
        None, None, "none", best_score,
        f"Ambiguous: multiple candidates match '{extracted_name}'",
        candidates=[c for c, _ in scored_candidates[:5]]
    )

def _score_name_match(
    self,
    pdf_forename: str | None,
    pdf_surname: str | None,
    pdf_raw: str,
    db_forename: str | None,
    db_surname: str | None,
    db_pref_name: str | None,
) -> float:
    """
    Score how well a PDF name matches a candidate.

    Handles:
    - Exact matches
    - Initial matches (J vs John)
    - Nickname matches (Johnny vs John via pref_name)
    - Surname-only matches (Mr Smith)
    """
    score = 0.0

    # Surname match is critical
    if pdf_surname and db_surname:
        surname_score = fuzz.ratio(pdf_surname.lower(), db_surname.lower()) / 100
        if surname_score < 0.8:
            return 0.0  # Surname must be close
        score += surname_score * 0.6

    # Forename matching
    if pdf_forename and db_forename:
        # Check initial match
        if len(pdf_forename) == 1:
            if db_forename.lower().startswith(pdf_forename.lower()):
                score += 0.3
            elif db_pref_name and db_pref_name.lower().startswith(pdf_forename.lower()):
                score += 0.3
        else:
            # Full forename comparison
            forename_score = fuzz.ratio(pdf_forename.lower(), db_forename.lower()) / 100

            # Also check against pref_name (nickname)
            if db_pref_name:
                pref_score = fuzz.ratio(pdf_forename.lower(), db_pref_name.lower()) / 100
                forename_score = max(forename_score, pref_score)

            score += forename_score * 0.4
    elif pdf_surname and not pdf_forename:
        # Surname-only match (e.g., "Mr Smith")
        score *= 0.8

    return score
```

### Tasks

- [x] Add `rapidfuzz` to dependencies
- [x] Implement `match_by_name()` method
- [x] Implement `_score_name_match()` helper
- [x] Handle edge cases:
  - Initial only: "J Smith" matches "John Smith"
  - Nickname: "Johnny Smith" matches pref_name="Johnny"
  - Surname only: "Mr Smith" matches when unambiguous
- [x] Add comprehensive unit tests

### Acceptance Criteria

- [x] Surname mismatch returns 0 (must match)
- [x] Initial matches forename start ("J" matches "John")
- [x] Nickname matches via pref_name field
- [x] Ambiguous matches return multiple candidates

---

## Phase 5: Request Disambiguation

**Goal**: When employee has multiple approved requests, use trade details to pick the right one.

### Implementation

```python
def disambiguate_by_trade_details(
    self,
    candidates: list[AwaitingNoteCandidate],
    extracted_ticker: str | None,
    extracted_direction: str | None,
    extracted_quantity: int | None,
) -> AwaitingNoteCandidate | None:
    """
    When multiple requests match, use extracted trade details to disambiguate.
    """
    if not candidates:
        return None

    if len(candidates) == 1:
        return candidates[0]

    scored = []
    for c in candidates:
        score = 0

        # Ticker match (highest weight)
        if extracted_ticker and c.ticker:
            if extracted_ticker.upper() == c.ticker.upper():
                score += 3
            elif extracted_ticker.upper() in c.ticker.upper():
                score += 1

        # Direction match
        if extracted_direction and c.direction:
            if extracted_direction.upper() == c.direction.upper():
                score += 2

        # Quantity match (within 10% tolerance)
        if extracted_quantity and c.quantity:
            ratio = min(extracted_quantity, c.quantity) / max(extracted_quantity, c.quantity)
            if ratio > 0.9:
                score += 1

        scored.append((c, score))

    scored.sort(key=lambda x: x[1], reverse=True)

    # Only return if clear winner
    if scored[0][1] > scored[1][1]:
        return scored[0][0]

    return None  # Still ambiguous
```

### Tasks

- [x] Implement `disambiguate_by_trade_details()` method
- [x] Weight ticker match highest (most unique)
- [x] Add direction and quantity as secondary signals
- [x] Return None if still ambiguous
- [x] Add unit tests

### Acceptance Criteria

- [x] Ticker match strongly favored
- [x] Returns single candidate when unambiguous
- [x] Returns None when multiple candidates tie

---

# PART B: EXTRACTION PIPELINE REFACTOR

---

## Phase 6: Extraction Schemas

**Goal**: Create Pydantic schemas for structured LLM extraction.

### File Structure

```
src/pa_dealing/agents/document_processor/
├── schemas/
│   ├── __init__.py
│   ├── extraction.py      # ContractNoteExtraction, ExtractionResult
│   └── verification.py    # VerificationIssue, VerificationResult
├── prompts/
│   ├── __init__.py
│   └── extraction.py      # EXTRACTION_INSTRUCTION
```

### Implementation

Create `src/pa_dealing/agents/document_processor/schemas/extraction.py`:

```python
from pydantic import BaseModel, Field
from typing import Optional, List, Literal
from enum import Enum


class DocumentType(str, Enum):
    """Binary classification"""
    CONTRACT_NOTE = "CONTRACT_NOTE"
    OTHER = "OTHER"


class TradeDirection(str, Enum):
    BUY = "BUY"
    SELL = "SELL"


class ConfidenceLevel(str, Enum):
    """Computed from confidence_score"""
    HIGH = "HIGH"      # >= 0.8
    MEDIUM = "MEDIUM"  # 0.5 - 0.8
    LOW = "LOW"        # < 0.5


class SecurityIdentifier(BaseModel):
    """Security identifiers. UK brokers use SEDOL primarily."""
    sedol: Optional[str] = Field(None, description="7-character UK identifier")
    isin: Optional[str] = Field(None, description="12-character international identifier")
    ticker: Optional[str] = Field(None, description="Exchange ticker")
    fund_name: Optional[str] = Field(None, description="Full security name")

    def has_identifier(self) -> bool:
        return any([self.sedol, self.isin, self.ticker, self.fund_name])


class ExtractedTrade(BaseModel):
    """Trade execution details."""
    direction: Optional[TradeDirection] = Field(None, description="BUY or SELL")
    quantity: Optional[float] = Field(None, description="Exact units traded")
    price: Optional[float] = Field(None, description="Execution price per unit")
    gross_amount: Optional[float] = Field(None, description="Total before fees")
    fees: Optional[float] = Field(None, description="Total fees/commission")
    net_amount: Optional[float] = Field(None, description="Final settlement amount")
    currency: Optional[str] = Field(None, description="3-letter code (GBP, USD)")
    execution_date: Optional[str] = Field(None, description="Trade date (YYYY-MM-DD)")
    settlement_date: Optional[str] = Field(None, description="Settlement date")
    venue: Optional[str] = Field(None, description="Exchange/venue")
    broker_reference: Optional[str] = Field(None, description="Contract note reference")


class ContractNoteExtraction(BaseModel):
    """
    SCHEMA 1: What the LLM extracts (response_schema for Gemini).
    No computed fields - just what LLM sees and extracts.
    """
    document_type: DocumentType = Field(description="CONTRACT_NOTE or OTHER")
    confidence_score: float = Field(ge=0.0, le=1.0, description="LLM confidence 0.0-1.0")
    extraction_notes: str = Field(default="", description="LLM notes on quality/issues")

    broker_name: Optional[str] = Field(None, description="Broker/platform name")
    document_date: Optional[str] = Field(None, description="Document date")
    account_number: Optional[str] = Field(None, description="Client account number")
    account_holder_name: Optional[str] = Field(None, description="Account holder name for matching")

    security: Optional[SecurityIdentifier] = Field(None, description="Security identifiers")
    trade: Optional[ExtractedTrade] = Field(None, description="Trade execution details")


class ExtractionResult(ContractNoteExtraction):
    """
    SCHEMA 2: Enriched result with computed fields.
    Created AFTER LLM extraction by post-processing.
    """
    confidence_level: ConfidenceLevel = Field(default=ConfidenceLevel.LOW)
    is_contract_note: bool = Field(default=False)
    is_partial_extraction: bool = Field(default=False)
    missing_fields: List[str] = Field(default_factory=list)
    extraction_failed: bool = Field(default=False)
    error_message: Optional[str] = Field(None)

    @classmethod
    def from_extraction(cls, raw: ContractNoteExtraction) -> "ExtractionResult":
        """Create enriched result from LLM extraction."""
        # Compute confidence level
        if raw.confidence_score >= 0.8:
            conf_level = ConfidenceLevel.HIGH
        elif raw.confidence_score >= 0.5:
            conf_level = ConfidenceLevel.MEDIUM
        else:
            conf_level = ConfidenceLevel.LOW

        is_cn = (raw.document_type == DocumentType.CONTRACT_NOTE)

        # Check for missing critical fields
        missing = []
        if is_cn:
            if not raw.security or not raw.security.has_identifier():
                missing.append('security_identifier')
            if not raw.trade:
                missing.extend(['direction', 'quantity', 'price'])
            else:
                if not raw.trade.direction:
                    missing.append('direction')
                if not raw.trade.quantity:
                    missing.append('quantity')
                if not raw.trade.price:
                    missing.append('price')

        return cls(
            **raw.model_dump(),
            confidence_level=conf_level,
            is_contract_note=is_cn,
            is_partial_extraction=len(missing) > 0,
            missing_fields=missing,
            extraction_failed=False,
            error_message=None,
        )

    @classmethod
    def from_error(cls, error: Exception) -> "ExtractionResult":
        """Create failed extraction result from error."""
        return cls(
            document_type=DocumentType.OTHER,
            confidence_score=0.0,
            extraction_notes="",
            confidence_level=ConfidenceLevel.LOW,
            is_contract_note=False,
            is_partial_extraction=True,
            missing_fields=['all'],
            extraction_failed=True,
            error_message=str(error),
        )

    def requires_manual_review(self) -> bool:
        return (
            self.extraction_failed or
            not self.is_contract_note or
            self.confidence_level == ConfidenceLevel.LOW or
            self.is_partial_extraction
        )

    def requires_audit_flag(self) -> bool:
        return (
            not self.extraction_failed and
            self.is_contract_note and
            self.confidence_level == ConfidenceLevel.MEDIUM and
            not self.is_partial_extraction
        )
```

Create `src/pa_dealing/agents/document_processor/schemas/verification.py`:

```python
from pydantic import BaseModel, Field
from typing import Optional, List, Literal


class VerificationIssue(BaseModel):
    """Structured discrepancy report"""
    field: str
    severity: Literal["error", "warning"]
    rule: str  # EXACT_MATCH, FUZZY_MATCH, TOLERANCE_MATCH
    message: str
    expected: Optional[float | str] = None
    extracted: Optional[float | str] = None
    detail: Optional[dict] = None


class VerificationResult(BaseModel):
    """Validation result with structured issues"""
    is_verified: bool
    issues: List[VerificationIssue] = Field(default_factory=list)
    confidence_score: float
```

### Tasks

- [x] Create `schemas/extraction.py` with all models
- [x] Create `schemas/verification.py` with verification models
- [x] Add `schemas/__init__.py` with exports
- [x] Add unit tests for schema validation
- [x] Test `from_extraction()` and `from_error()` factory methods

### Acceptance Criteria

- [x] All Pydantic models validate correctly
- [x] `ExtractionResult.from_extraction()` computes derived fields
- [x] `ExtractionResult.from_error()` creates proper error result
- [x] `requires_manual_review()` and `requires_audit_flag()` work correctly

---

## Phase 7: Extraction Prompt

**Goal**: Create optimized LLM prompt for contract note extraction.

### Implementation

Create `src/pa_dealing/agents/document_processor/prompts/extraction.py`:

```python
EXTRACTION_INSTRUCTION = """
You are a financial document extraction specialist for UK brokerage contract notes.
Your extractions feed into automated compliance verification. Accuracy is critical.

## TASK

1. CLASSIFY: Is this a contract note (trade confirmation)?
2. EXTRACT: If contract note, extract all trade details precisely.
3. SCORE: Rate your confidence 0.0 to 1.0.

## DOCUMENT CLASSIFICATION

CONTRACT_NOTE indicators:
- Title: "Contract Note" / "Trade Confirmation" / "Execution Confirmation"
- Shows SINGLE trade execution with price, quantity, direction
- Settlement details and fees breakdown
- Broker reference/contract number

NOT a contract note (classify as OTHER):
- Holdings summaries / portfolio valuations
- Investment reports / performance summaries
- Account statements (transaction history)
- Fund factsheets

If OTHER: Set document_type="OTHER", confidence_score (how sure you are it's not a contract note), leave trade fields null.

## EXTRACTION RULES

### Account Holder Name
- Extract the account holder/client name exactly as shown
- This is used for matching to PAD requests
- Common formats: "Mr John Smith", "J. Smith", "Smith, John"

### Security Identifiers
- SEDOL: 7 alphanumeric (e.g., B80QG61) - often in parentheses
- ISIN: 12 characters (e.g., GB00B80QG614)
- DO NOT confuse account numbers (10+ digits) with SEDOL (exactly 7)

### Trade Direction
- "Purchase"/"Bought"/"Buy" → BUY
- "Sale"/"Sold"/"Sell" → SELL
- Must be EXPLICIT - do not infer

### Quantity
- Extract EXACTLY as shown, preserve decimals
- "1,543.0800 units" → 1543.08
- Never round

### Price - CRITICAL
Extract EXECUTION PRICE only (price paid for THIS trade):
- Often labeled "Price" / "Deal price" / "Execution price"
- Convert pence to pounds if needed (120.63p → 1.2063)

NEVER extract:
- "Avg unit cost" (historical average)
- "Current price" / "Market price" (today's valuation)
- "Book cost per unit" (historical cost basis)

### Dates
- Convert to YYYY-MM-DD format
- execution_date: when trade executed
- settlement_date: when settled (usually T+2)

## CONFIDENCE SCORING

Rate 0.0 to 1.0:

0.9-1.0: Clear contract note, all fields readable
0.7-0.9: Contract note confirmed, most fields present
0.5-0.7: Likely contract note, some fields unclear
0.3-0.5: Uncertain type or significant issues
0.0-0.3: Cannot reliably extract

Lower confidence if:
- Document partially cut off or blurry
- Multiple trades shown (extract first only)
- Unusual format
- Key fields ambiguous

## EXTRACTION NOTES

Use extraction_notes to explain:
- Ambiguities or assumptions made
- Quality issues (blurry, cut off)
- Multiple trades detected
- Unusual formatting

## OUTPUT

Return valid JSON matching ContractNoteExtraction schema.
Use null for fields not found.
"""
```

### Tasks

- [x] Create `prompts/extraction.py` with `EXTRACTION_INSTRUCTION`
- [x] Add `prompts/__init__.py`
- [x] Review prompt with sample contract notes
- [x] Ensure account_holder_name extraction is emphasized

### Acceptance Criteria

- [x] Prompt covers classification and extraction
- [x] Confidence scoring guidance is clear
- [x] Price extraction rules are unambiguous
- [x] Account holder name extraction is specified

---

## Phase 8: Extraction Service

**Goal**: Create async extraction service with Gemini structured output.

### Implementation

Create `src/pa_dealing/agents/document_processor/services/extraction.py`:

```python
import asyncio
import logging
from typing import Optional
import google.genai as genai
from google.genai import types

from ..schemas.extraction import (
    ContractNoteExtraction,
    ExtractionResult,
)
from ..prompts.extraction import EXTRACTION_INSTRUCTION

logger = logging.getLogger(__name__)


async def extract_contract_note(
    document_content: bytes,
    document_id: Optional[str] = None,
    mime_type: str = "application/pdf",
) -> ExtractionResult:
    """
    Extract contract note data using Gemini with structured output.

    Args:
        document_content: PDF or image bytes
        document_id: Optional ID for logging/tracking
        mime_type: Content type (application/pdf, image/png, etc.)

    Returns:
        ExtractionResult with extracted data and computed fields
    """
    log_context = {"document_id": document_id, "mime_type": mime_type}
    max_retries = 3

    for attempt in range(max_retries):
        try:
            logger.info("extraction_started", extra={**log_context, "attempt": attempt + 1})

            client = genai.Client()

            response = await client.aio.models.generate_content(
                model="gemini-2.0-flash-exp",
                contents=[
                    types.Part.from_bytes(data=document_content, mime_type=mime_type),
                    "Extract trade details from this document."
                ],
                config=types.GenerateContentConfig(
                    system_instruction=EXTRACTION_INSTRUCTION,
                    response_mime_type="application/json",
                    response_schema=ContractNoteExtraction,
                    temperature=0.1,
                )
            )

            raw_extraction = ContractNoteExtraction.model_validate_json(response.text)
            result = ExtractionResult.from_extraction(raw_extraction)

            logger.info(
                "extraction_completed",
                extra={
                    **log_context,
                    "document_type": result.document_type.value,
                    "confidence_score": result.confidence_score,
                    "confidence_level": result.confidence_level.value,
                    "is_contract_note": result.is_contract_note,
                    "is_partial": result.is_partial_extraction,
                    "missing_fields": result.missing_fields,
                    "account_holder_name": result.account_holder_name,
                }
            )

            return result

        except genai.errors.ResourceExhausted as e:
            # Rate limit - retry with backoff
            if attempt < max_retries - 1:
                backoff = 2 ** attempt  # 1s, 2s, 4s
                logger.warning(
                    "extraction_rate_limit",
                    extra={**log_context, "backoff_seconds": backoff, "attempt": attempt + 1}
                )
                await asyncio.sleep(backoff)
                continue
            logger.error("extraction_rate_limit_exhausted", extra={**log_context, "error": str(e)})
            return ExtractionResult.from_error(e)

        except genai.errors.APIError as e:
            logger.error("extraction_api_error", extra={**log_context, "error": str(e)})
            return ExtractionResult.from_error(e)

        except Exception as e:
            logger.error(
                "extraction_failed",
                extra={**log_context, "error_type": type(e).__name__, "error": str(e)}
            )
            return ExtractionResult.from_error(e)

    # Should not reach here, but safety fallback
    return ExtractionResult.from_error(Exception("Max retries exceeded"))
```

### Tasks

- [x] Create `services/extraction.py` with `extract_contract_note()`
- [x] Implement rate limit retry with exponential backoff
- [x] Add comprehensive structured logging
- [x] Add `google-genai` to requirements.txt
- [x] Add unit tests with mocked Gemini responses

### Acceptance Criteria

- [x] Extracts using Gemini structured output
- [x] Retries on rate limit (up to 3 times)
- [x] Returns error result on failure (doesn't raise)
- [x] Logs all extraction attempts with context

---

## Phase 9: Routing & Validation Services

**Goal**: Create routing logic and validation service.

### Routing Implementation

Create `src/pa_dealing/agents/document_processor/services/routing.py`:

```python
from enum import Enum
from dataclasses import dataclass
from ..schemas.extraction import ExtractionResult, ConfidenceLevel


class RoutingAction(str, Enum):
    AUTO_APPROVE = "AUTO_APPROVE"
    AUTO_APPROVE_WITH_AUDIT = "AUTO_APPROVE_WITH_AUDIT"
    MANUAL_REVIEW = "MANUAL_REVIEW"


@dataclass
class RoutingDecision:
    action: RoutingAction
    reason: str
    can_validate: bool


def determine_routing(result: ExtractionResult) -> RoutingDecision:
    """
    Determine routing based on extraction result.

    Rules:
    1. Extraction failed → Manual Review
    2. NOT_CONTRACT_NOTE → Manual Review
    3. Partial extraction (missing fields) → Manual Review
    4. confidence < 0.5 → Manual Review
    5. confidence < 0.8 → Auto + Audit Flag
    6. confidence >= 0.8 → Auto-Approve
    """
    if result.extraction_failed:
        return RoutingDecision(
            action=RoutingAction.MANUAL_REVIEW,
            reason=f"Extraction failed: {result.error_message}",
            can_validate=False,
        )

    if not result.is_contract_note:
        return RoutingDecision(
            action=RoutingAction.MANUAL_REVIEW,
            reason=f"Document type: {result.document_type.value}",
            can_validate=False,
        )

    if result.is_partial_extraction:
        return RoutingDecision(
            action=RoutingAction.MANUAL_REVIEW,
            reason=f"Missing fields: {', '.join(result.missing_fields)}",
            can_validate=False,
        )

    if result.confidence_score < 0.5:
        return RoutingDecision(
            action=RoutingAction.MANUAL_REVIEW,
            reason=f"Low confidence: {result.confidence_score:.2f}",
            can_validate=False,
        )

    if result.confidence_score < 0.8:
        return RoutingDecision(
            action=RoutingAction.AUTO_APPROVE_WITH_AUDIT,
            reason=f"Medium confidence: {result.confidence_score:.2f}",
            can_validate=True,
        )

    return RoutingDecision(
        action=RoutingAction.AUTO_APPROVE,
        reason=f"High confidence: {result.confidence_score:.2f}",
        can_validate=True,
    )
```

### Validation Implementation

Create `src/pa_dealing/agents/document_processor/services/validation.py`:

```python
import logging
from typing import Optional
from ..schemas.extraction import ExtractedTrade, SecurityIdentifier
from ..schemas.verification import VerificationIssue, VerificationResult

logger = logging.getLogger(__name__)

QUANTITY_TOLERANCE = 0.0  # EXACT match
PRICE_TOLERANCE = 0.05    # 5% tolerance


def verify_trade(
    extracted: ExtractedTrade,
    security: Optional[SecurityIdentifier],
    request: dict,
    confidence_score: float,
) -> VerificationResult:
    """
    Validate extracted trade against request.

    Rules:
    1. Direction: EXACT match
    2. Quantity: EXACT match (0% tolerance)
    3. Ticker: FUZZY match (substring)
    4. Price: TOLERANCE match (5%)
    """
    issues = []

    # Rule 1: Direction EXACT match
    req_direction = request.get('direction', '').upper()
    ext_direction = extracted.direction.value if extracted.direction else None

    if ext_direction != req_direction:
        issues.append(VerificationIssue(
            field='direction',
            severity='error',
            rule='EXACT_MATCH',
            message='Direction mismatch',
            expected=req_direction,
            extracted=ext_direction,
        ))

    # Rule 2: Quantity EXACT match
    req_qty = request.get('quantity')
    ext_qty = extracted.quantity

    if req_qty and ext_qty:
        diff = abs(ext_qty - req_qty)
        if diff > (req_qty * QUANTITY_TOLERANCE):
            issues.append(VerificationIssue(
                field='quantity',
                severity='error',
                rule='EXACT_MATCH',
                message='Quantity mismatch',
                expected=req_qty,
                extracted=ext_qty,
            ))

    # Rule 3: Ticker FUZZY match
    req_ticker = request.get('ticker', '').upper()
    ext_ticker = ''
    if security:
        ext_ticker = (security.ticker or security.fund_name or '').upper()

    if req_ticker and ext_ticker:
        if req_ticker not in ext_ticker and ext_ticker not in req_ticker:
            issues.append(VerificationIssue(
                field='ticker',
                severity='error',
                rule='FUZZY_MATCH',
                message='Ticker mismatch',
                expected=req_ticker,
                extracted=ext_ticker,
            ))

    # Rule 4: Price TOLERANCE match
    ext_price = extracted.price
    req_estimated_value = request.get('estimated_value')

    if ext_price and req_qty and req_estimated_value:
        implied_price = req_estimated_value / req_qty
        price_diff_pct = abs(ext_price - implied_price) / implied_price

        if price_diff_pct > PRICE_TOLERANCE:
            issues.append(VerificationIssue(
                field='price',
                severity='error',
                rule='TOLERANCE_MATCH',
                message=f'Price deviation {price_diff_pct:.1%} exceeds {PRICE_TOLERANCE:.0%}',
                expected=implied_price,
                extracted=ext_price,
                detail={'tolerance': f'{PRICE_TOLERANCE * 100}%', 'deviation': f'{price_diff_pct * 100:.2f}%'}
            ))

    logger.info(
        "verification_completed",
        extra={
            "is_verified": len(issues) == 0,
            "issue_count": len(issues),
            "confidence_score": confidence_score,
        }
    )

    return VerificationResult(
        is_verified=len(issues) == 0,
        issues=issues,
        confidence_score=confidence_score,
    )
```

### Tasks

- [x] Create `services/routing.py` with `determine_routing()`
- [x] Create `services/validation.py` with `verify_trade()`
- [x] Add `services/__init__.py` with exports
- [x] Add unit tests for routing rules
- [x] Add unit tests for validation rules

### Acceptance Criteria

- [x] Routing covers all confidence thresholds
- [x] Validation preserves existing tolerances (0% qty, 5% price)
- [x] Structured issues returned for each failure

---

## Phase 10: Pipeline Integration

**Goal**: Wire everything together and integrate with PDF poller.

### Pipeline Service

Create `src/pa_dealing/agents/document_processor/services/pipeline.py`:

```python
import logging
import uuid
from dataclasses import dataclass
from typing import Optional

from ..schemas.extraction import ExtractionResult
from ..schemas.verification import VerificationResult
from .extraction import extract_contract_note
from .routing import RoutingDecision, RoutingAction, determine_routing
from .validation import verify_trade

logger = logging.getLogger(__name__)


@dataclass
class PipelineResult:
    """Complete extraction pipeline result"""
    extraction: ExtractionResult
    routing: RoutingDecision
    validation: Optional[VerificationResult] = None
    status: str = ""

    def to_response(self) -> dict:
        """Format for API response"""
        return {
            'status': self.status,
            'requires_manual_review': self.routing.action == RoutingAction.MANUAL_REVIEW,
            'flagged_for_audit': self.routing.action == RoutingAction.AUTO_APPROVE_WITH_AUDIT,
            'extraction': {
                'document_type': self.extraction.document_type.value,
                'confidence_score': self.extraction.confidence_score,
                'confidence_level': self.extraction.confidence_level.value,
                'extraction_notes': self.extraction.extraction_notes,
                'extraction_failed': self.extraction.extraction_failed,
                'error_message': self.extraction.error_message,
                'account_holder_name': self.extraction.account_holder_name,
                'security': self.extraction.security.model_dump() if self.extraction.security else None,
                'trade': self.extraction.trade.model_dump() if self.extraction.trade else None,
            },
            'routing': {
                'action': self.routing.action.value,
                'reason': self.routing.reason,
            },
            'validation': self.validation.model_dump() if self.validation else None,
        }


async def process_document(
    document_content: bytes,
    request_data: dict,
    document_id: Optional[str] = None,
    mime_type: str = "application/pdf",
) -> PipelineResult:
    """
    Complete extraction → routing → validation pipeline.
    """
    if not document_id:
        document_id = str(uuid.uuid4())

    logger.info("pipeline_started", extra={"document_id": document_id})

    # Step 1: Extract
    extraction = await extract_contract_note(
        document_content=document_content,
        document_id=document_id,
        mime_type=mime_type,
    )

    # Step 2: Route
    routing = determine_routing(extraction)

    # Step 3: Validate (if routing allows)
    validation = None
    if routing.can_validate and extraction.trade:
        validation = verify_trade(
            extracted=extraction.trade,
            security=extraction.security,
            request=request_data,
            confidence_score=extraction.confidence_score,
        )

    # Step 4: Determine final status
    if not routing.can_validate:
        status = "MANUAL_REVIEW_REQUIRED"
    elif validation and not validation.is_verified:
        status = "VALIDATION_FAILED"
    elif routing.action == RoutingAction.AUTO_APPROVE_WITH_AUDIT:
        status = "VERIFIED_WITH_AUDIT"
    else:
        status = "VERIFIED"

    logger.info(
        "pipeline_completed",
        extra={
            "document_id": document_id,
            "status": status,
            "routing_action": routing.action.value,
            "is_verified": validation.is_verified if validation else None,
        }
    )

    return PipelineResult(
        extraction=extraction,
        routing=routing,
        validation=validation,
        status=status,
    )
```

### Integration with Matching Flow

Update `src/pa_dealing/services/gcs_pdf_poller.py`:

```python
async def process_document(self, gcs_doc: GCSDocument, session: AsyncSession):
    """Process document: match user → extract → validate."""

    # 1. Load candidate pool
    candidates = await get_candidates_awaiting_notes(session)

    # 2. Try email matching first
    match_result = await self.matcher.match_by_email(
        gcs_doc.sender_email, candidates, session
    )

    # 3. Run extraction pipeline
    pdf_content = await self._download_pdf(gcs_doc.gcs_path)
    pipeline_result = await process_document(
        document_content=pdf_content,
        request_data={},  # Populated after matching
        document_id=str(gcs_doc.id),
    )

    # 4. If no email match, try name matching from extraction
    if match_result is None or match_result.match_method == "none":
        if pipeline_result.extraction.account_holder_name:
            match_result = self.matcher.match_by_name(
                pipeline_result.extraction.account_holder_name, candidates
            )

            # Disambiguate if needed
            if match_result.candidates and len(match_result.candidates) > 1:
                trade = pipeline_result.extraction.trade
                if trade:
                    disambiguated = self.matcher.disambiguate_by_trade_details(
                        match_result.candidates,
                        pipeline_result.extraction.security.ticker if pipeline_result.extraction.security else None,
                        trade.direction.value if trade.direction else None,
                        int(trade.quantity) if trade.quantity else None,
                    )
                    if disambiguated:
                        match_result = MatchResult(
                            disambiguated.employee_id,
                            disambiguated.request_id,
                            match_result.match_method,
                            match_result.confidence + 0.1,
                            f"Disambiguated to {disambiguated.ticker}"
                        )

    # 5. Re-run validation with matched request data
    if match_result and match_result.request_id and pipeline_result.routing.can_validate:
        request = await session.get(PADRequest, match_result.request_id)
        validation = verify_trade(
            extracted=pipeline_result.extraction.trade,
            security=pipeline_result.extraction.security,
            request={
                'direction': request.direction,
                'quantity': request.quantity,
                'ticker': request.ticker,
                'estimated_value': float(request.estimated_value) if request.estimated_value else None,
            },
            confidence_score=pipeline_result.extraction.confidence_score,
        )
        pipeline_result.validation = validation

    # 6. Store results
    parsed_trade = ParsedTrade(
        gcs_document_id=gcs_doc.id,
        matched_employee_id=match_result.employee_id if match_result else None,
        matched_request_id=match_result.request_id if match_result else None,
        match_method=match_result.match_method if match_result else "none",
        match_confidence=match_result.confidence if match_result else 0.0,
        extracted_account_holder=pipeline_result.extraction.account_holder_name,
        # ... other fields from extraction
    )
    session.add(parsed_trade)
```

### Tasks

- [x] Create `services/pipeline.py` with `process_document()`
- [x] Update GCS PDF poller to use new pipeline
- [x] Wire matching → extraction → validation flow
- [x] Add integration tests with real documents
- [x] Add test with holdings summary (should route to manual review)

### Acceptance Criteria

- [x] Full pipeline: match → extract → route → validate
- [x] Holdings summaries route to manual review
- [x] Contract notes validate correctly
- [x] All results persisted with audit trail

---

## Phase 11: Manual Review Queue

**Goal**: API endpoints for compliance to review unmatched/low-confidence documents.

### API Endpoints

```
GET  /api/documents/review-queue           # List documents needing review
GET  /api/documents/review-queue/stats     # Counts by status
POST /api/documents/{id}/assign-employee   # Manual employee assignment
POST /api/documents/{id}/link-request      # Manual PAD request linkage
POST /api/documents/{id}/reject            # Mark as not applicable
```

### Tasks

- [x] Create review queue query (manual_review or low confidence)
- [x] Add `POST /assign-employee` endpoint
- [x] Add `POST /link-request` endpoint
- [x] Add `POST /reject` endpoint
- [x] Audit all manual actions
- [x] Add API tests

### Acceptance Criteria

- [x] Compliance can view pending review items
- [x] Manual assignment updates match_method = 'manual'
- [x] All actions are audit logged

---

## Success Criteria

1. **User Matching**: Contract notes matched to employees via email or fuzzy name
2. **Document Classification**: Holdings summaries immediately routed to manual review
3. **Extraction Confidence**: High/medium/low routing works correctly
4. **Validation**: Direction/quantity exact, price 5% tolerance preserved
5. **Error Handling**: LLM failures gracefully handled
6. **Observability**: All extractions logged with document_id, confidence, routing

---

## Dependencies

- `rapidfuzz` - Fuzzy string matching
- `google-genai` - Gemini API client
- `pydantic` - Schema validation (already present)

---

## Out of Scope

- OCR for scanned PDFs
- Multi-trade contract notes
- Broker domain management UI
- Batch reprocessing
