# Contract Note User Matching & Verification - Architecture Summary

## Current State Analysis

### What We Have (Completed)
From the `gcs_pdf_ingestion_20260129` track, we have a fully functional PDF ingestion pipeline:

1. **GCS Infrastructure**
   - Bucket: `cmek-encrypted-bucket-europe-west2-roe18`
   - Folder structure: `incoming/` → `processing/` → `archive/` or `failed/`
   - Deduplication via `gcs_generation` (unique constraint)
   - Atomic claiming via blob rename

2. **PDF Processing Pipeline**
   - `GCSPDFPoller` service polls and orchestrates
   - `DocumentAgent` (AI parser) extracts trade data using Gemini
   - Database tables: `gcs_document`, `parsed_trade`, `document_processing_log`
   - Worker tracking for multi-pod safety

3. **Metadata Capture**
   - `sender_email` extracted from GCS metadata (`x-goog-meta-sender-email`)
   - Original filename preserved
   - Full audit trail via `DocumentProcessingLog`

### What's Missing (This Track)

1. **User Matching**
   - Currently no link between PDF and employee
   - `sender_email` captured but not used for matching
   - No fuzzy name matching from PDF content
   - No link to PAD requests awaiting confirmation

2. **Document Classification**
   - System assumes all PDFs are contract notes
   - No handling for activity statements vs contract notes
   - No confidence scoring or routing logic

3. **Enhanced Extraction**
   - No account holder name extraction
   - Missing broker name and account details
   - No support for activity statement "Trades" section parsing

## Implementation Plan

### Phase 1: Schema Extensions

#### 1.1 Extend GCSDocument
```sql
ALTER TABLE padealing.gcs_document ADD COLUMN email_source VARCHAR(20);
-- Values: 'broker', 'user', 'manual', 'unknown'
```

#### 1.2 Extend ParsedTrade
```sql
ALTER TABLE padealing.parsed_trade ADD COLUMN extracted_account_holder VARCHAR(200);
ALTER TABLE padealing.parsed_trade ADD COLUMN normalized_account_holder VARCHAR(200);
ALTER TABLE padealing.parsed_trade ADD COLUMN matched_employee_id BIGINT;
ALTER TABLE padealing.parsed_trade ADD COLUMN matched_request_id INTEGER;
ALTER TABLE padealing.parsed_trade ADD COLUMN match_method VARCHAR(20);
-- Values: 'email', 'name_exact', 'name_fuzzy', 'manual'
ALTER TABLE padealing.parsed_trade ADD COLUMN match_confidence NUMERIC(3,2);
```

### Phase 2: Document Type Classification

#### 2.1 Document Types
- **CONTRACT_NOTE**: Single trade confirmation
  - Identifiers: "Contract Note", "Trade Confirmation"
  - Contains: Single execution with price/qty/direction

- **ACTIVITY_STATEMENT**: Monthly statement from Interactive Brokers
  - Identifiers: "Activity Statement", "Account Statement"
  - Contains: Account overview + optional "Trades" section
  - If "Trades" section exists → trades occurred
  - If no "Trades" section → positions only (no action needed)

#### 2.2 Enhanced Extraction Schema
```python
class DocumentClassification(BaseModel):
    document_type: Literal["CONTRACT_NOTE", "ACTIVITY_STATEMENT", "OTHER"]
    confidence_score: float  # 0.0-1.0
    has_trades_section: bool | None  # For activity statements
    account_holder_name: str | None
    broker_name: str | None

class ExtractedTradeData(BaseModel):
    # Existing fields...
    account_holder_name: str | None  # NEW
    broker_name: str | None  # NEW
```

### Phase 3: User Matching Pipeline

#### 3.1 Matching Flow
```
PDF arrives → Extract metadata
    │
    ├─► Email-based matching (Priority 1)
    │   └─► sender_email is @mako.com?
    │       └─► YES: matched_employee_id = lookup_by_email()
    │
    ├─► Name-based matching (Priority 2)
    │   └─► Extract account_holder_name from PDF
    │       └─► Query candidate pool (approved requests only)
    │           └─► Fuzzy match against forename/surname/pref_name
    │
    └─► Manual review queue
        └─► No confident match found
```

#### 3.2 Candidate Pool Query
```python
async def get_matching_candidates(session: AsyncSession) -> list[MatchCandidate]:
    """Get employees with approved requests awaiting confirmation."""
    query = """
    SELECT DISTINCT
        pr.id as request_id,
        pr.employee_id,
        oc.forename,
        oc.surname,
        oc.pref_name,
        oc.title,
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
    """
    # Execute and return as MatchCandidate objects
```

#### 3.3 Fuzzy Name Matching
```python
class NameMatcher:
    def match(self, extracted_name: str, candidates: list[MatchCandidate]) -> MatchResult:
        """
        Match extracted name against candidate pool.

        Handles:
        - Title stripping (Mr, Mrs, Dr, etc.)
        - Initial matching (J Smith → John Smith)
        - Nickname matching (Johnny → pref_name)
        - Reversed names (Smith, John → John Smith)
        - Middle name ignoring
        """
        normalized = self._normalize_name(extracted_name)
        scores = []

        for candidate in candidates:
            score = self._calculate_score(normalized, candidate)
            scores.append((candidate, score))

        # Return best match if confidence > 0.7
        best_match = max(scores, key=lambda x: x[1])
        if best_match[1] > 0.7:
            return MatchResult(
                employee_id=best_match[0].employee_id,
                request_id=best_match[0].request_id,
                confidence=best_match[1],
                method="name_fuzzy"
            )
        return None
```

### Phase 4: Activity Statement Handling

#### 4.1 Detection Logic
```python
async def process_activity_statement(pdf_content: bytes) -> ActivityStatementResult:
    """
    Process Interactive Brokers activity statement.

    1. Extract account holder full name
    2. Check for "Trades" section
    3. If trades exist, extract each trade
    4. Return structured data
    """
    # Use enhanced prompt for Gemini
    prompt = """
    Analyze this Interactive Brokers Activity Statement.

    1. Extract the account holder's FULL NAME
    2. Check if there's a "Trades" section
    3. If trades exist, extract EACH trade with:
       - Trade date
       - Symbol/ticker
       - Direction (BUY/SELL)
       - Quantity
       - Price
       - Settlement date

    Return as JSON...
    """
```

#### 4.2 Trades Section Parser
- Look for section headers: "Trades", "Transactions", "Trading Activity"
- If section missing → no trades occurred (positions only)
- If section present → extract all trades
- Each trade becomes a `ParsedTrade` record

### Phase 5: Integration Points

#### 5.1 Modify PDF Poller
```python
class GCSPDFPoller:
    async def _process_document(self, blob, session):
        # Existing: Extract metadata
        metadata = self.gcs_client.get_blob_metadata(blob)
        sender_email = metadata.get("sender-email")

        # NEW: Classify email source
        email_source = self._classify_email_source(sender_email)

        # NEW: Attempt user matching
        matched_employee_id = None
        if email_source == "user":
            matched_employee_id = await self._match_by_email(sender_email, session)

        # Existing: Parse PDF
        trades = await self._run_parser(pdf_content, document_id)

        # NEW: If no email match, try name matching
        if not matched_employee_id and trades:
            account_holder = trades[0].get("account_holder_name")
            if account_holder:
                matched_employee_id = await self._match_by_name(account_holder, session)

        # NEW: Link trades to matched employee/request
        if matched_employee_id:
            await self._link_trades_to_request(trades, matched_employee_id, session)
```

#### 5.2 Modify Document Agent
```python
class DocumentAgent:
    async def process_pdf(self, pdf_content: bytes) -> dict:
        # NEW: First classify document type
        classification = await self._classify_document(pdf_content)

        if classification.document_type == "ACTIVITY_STATEMENT":
            return await self._process_activity_statement(pdf_content)
        elif classification.document_type == "CONTRACT_NOTE":
            return await self._process_contract_note(pdf_content)
        else:
            return {"error": "Unsupported document type", "classification": classification}
```

### Phase 6: Confidence-Based Routing

#### 6.1 Routing Rules
| Confidence | Action | Reason |
|------------|--------|--------|
| ≥ 0.8 | Auto-approve | High confidence |
| 0.5-0.8 | Auto + Audit flag | Medium confidence |
| < 0.5 | Manual review | Low confidence |
| N/A | Manual review | Classification failed |

#### 6.2 Manual Review Queue
- New API endpoint: `GET /documents/unmatched`
- Returns documents with `match_status = 'manual_review'`
- UI shows split view: PDF + potential matches
- Compliance can manually link or reject

### Phase 7: Testing Strategy

#### 7.1 Unit Tests (Can run locally)
```python
def test_name_matching():
    """Test fuzzy name matching logic."""
    matcher = NameMatcher()

    # Test title stripping
    assert matcher._normalize_name("Mr John Smith") == "john smith"

    # Test initial matching
    candidate = MatchCandidate(forename="John", surname="Smith")
    assert matcher._matches_initials("J Smith", candidate) == True

    # Test reversed names
    assert matcher._normalize_name("Smith, John") == "john smith"
```

#### 7.2 Integration Tests (For Gemini)
```python
async def test_activity_statement_extraction():
    """
    Test extraction from Interactive Brokers activity statement.

    GIVEN: An activity statement PDF with trades section
    WHEN: Processing through DocumentAgent
    THEN: Should extract account holder name and all trades

    Expected:
    - account_holder_name: "John Smith"
    - document_type: "ACTIVITY_STATEMENT"
    - trades: List of 3 trades
    """
    # Use real test file
    pdf_path = "/Users/luisdeburnay/Desktop/ActivityStatement.202512.pdf"
    # ... detailed assertions
```

### Phase 8: Database Migration

```sql
-- Migration: add_user_matching_fields.sql
BEGIN;

-- Extend GCSDocument
ALTER TABLE padealing.gcs_document
ADD COLUMN email_source VARCHAR(20) CHECK (
    email_source IN ('broker', 'user', 'manual', 'unknown')
);

-- Extend ParsedTrade
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

-- Add indexes for performance
CREATE INDEX ix_parsed_trade_matched_employee
ON padealing.parsed_trade(matched_employee_id);

CREATE INDEX ix_parsed_trade_matched_request
ON padealing.parsed_trade(matched_request_id);

COMMIT;
```

## Key Differences from Original Spec

### Added Based on Compliance Feedback:
1. **Activity Statement Support**
   - Not just contract notes
   - "Trades" section detection
   - Full name extraction from Interactive Brokers

2. **Email-First Matching**
   - Prioritize @mako.com sender email
   - Most reliable identification method

3. **Interactive Brokers Focus**
   - Primary source of automated statements
   - Standardized format with full names
   - Monthly statements + immediate trade notifications

### Simplified:
1. **Removed complex ticker disambiguation**
   - Focus on name/email matching first
   - Ticker matching as secondary validation

2. **Deferred OCR support**
   - All documents assumed machine-readable
   - OCR can be added later if needed

## Success Metrics

1. **Matching Accuracy**
   - Target: 90% auto-match for Interactive Brokers
   - Target: 70% auto-match for other brokers

2. **Processing Speed**
   - < 30 seconds from GCS arrival to matching

3. **Manual Review Reduction**
   - Reduce manual matching by 75%

## Risk Mitigation

1. **False Positives**
   - Never auto-approve if confidence < 0.8
   - Always flag for review if multiple matches

2. **Data Privacy**
   - Account holder names stored encrypted
   - Audit trail for all matching decisions

3. **System Failures**
   - Graceful degradation to manual review
   - Never lose a document (always moves to failed/)

## Next Steps

1. Create Alembic migration for schema changes
2. Implement `NameMatcher` class with fuzzy logic
3. Enhance `DocumentAgent` for activity statements
4. Update `GCSPDFPoller` with matching pipeline
5. Add API endpoints for manual review
6. Write comprehensive tests (unit + integration)
7. Deploy and monitor in dev environment