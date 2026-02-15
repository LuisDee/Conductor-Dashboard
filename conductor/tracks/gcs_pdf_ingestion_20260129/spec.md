# GCS PDF Polling & Ingestion System - Technical Specification

## Overview

This specification defines a production-ready system for polling PDFs from Google Cloud Storage, processing them through an AI parser, and storing results with full end-to-end traceability. The system must support viewing the original PDF from the UI for any parsed trade record.

---

## System Context

### Current State
- PDFs arrive in GCS via a mailbox service that extracts email attachments
- PDFs contain trade confirmations that need parsing
- Manual upload process exists but needs automation
- Metadata available: `sender_email` (set by mailbox service on GCS object)

### Target State
- Automated polling of GCS for new PDFs
- Idempotent processing with deduplication
- Full traceability: `Trade in UI` → `View PDF` → `Retrieve exact document`
- Robust failure handling with retry capability

---

## Architecture

### High-Level Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Mailbox    │────▶│     GCS      │────▶│   Poller     │────▶│  AI Parser   │
│   Service    │     │  incoming/   │     │   Service    │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                                                │                      │
                                                ▼                      ▼
                                          ┌──────────────┐     ┌──────────────┐
                                          │     GCS      │     │  PostgreSQL  │
                                          │   archive/   │     │  documents   │
                                          │              │     │   trades     │
                                          └──────────────┘     └──────────────┘
                                                │                      │
                                                └──────────┬───────────┘
                                                           ▼
                                                    ┌──────────────┐
                                                    │     UI       │
                                                    │  View PDF    │
                                                    └──────────────┘
```

### GCS Bucket Structure

```
gs://{BUCKET_NAME}/
├── incoming/                           # Raw PDFs from mailbox service
│   ├── attachment_abc123.pdf           # Unprocessed
│   └── attachment_def456.pdf           # Unprocessed
│
├── processing/                         # Currently being processed (in-flight)
│   └── {document_id}.pdf               # Claimed by a worker
│
├── archive/                            # Successfully processed (immutable)
│   └── {YYYY}/
│       └── {MM}/
│           ├── {document_id}.pdf       # e.g., 550e8400-e29b-41d4-a716.pdf
│           └── {document_id}.pdf
│
└── failed/                             # Failed processing (requires investigation)
    └── {YYYY}/
        └── {MM}/
            └── {document_id}.pdf       # With error metadata attached
```

### Path Convention

| Prefix | Purpose | Naming | Retention |
|--------|---------|--------|-----------|
| `incoming/` | Landing zone from mailbox | Original filename | Transient |
| `processing/` | In-flight (claimed by worker) | `{document_id}.pdf` | Transient |
| `archive/{YYYY}/{MM}/` | Processed successfully | `{document_id}.pdf` | Permanent |
| `failed/{YYYY}/{MM}/` | Processing failed | `{document_id}.pdf` | Until resolved |

---

## Data Model

### PostgreSQL Schema

```sql
-- Extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- DOCUMENTS TABLE
-- Represents a single PDF file ingested into the system
-- ============================================================================
CREATE TABLE documents (
    -- Primary identifier (UUID) - used throughout system for traceability
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Original file information
    original_filename TEXT NOT NULL,
    original_gcs_path TEXT NOT NULL,          -- gs://bucket/incoming/original_name.pdf

    -- Metadata from mailbox service
    sender_email TEXT,

    -- GCS deduplication key - CRITICAL for idempotency
    -- This is the GCS object generation number, unique per object version
    gcs_generation BIGINT UNIQUE NOT NULL,

    -- Archive location (deterministic from id + received_at, but stored for convenience)
    archive_gcs_path TEXT,                    -- gs://bucket/archive/2025/01/{id}.pdf

    -- Timestamps
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processing_started_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,

    -- Status tracking
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,

    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'parsed', 'failed', 'rejected'))
);

-- Index for poller to find pending work
CREATE INDEX idx_documents_pending ON documents(status, received_at)
    WHERE status IN ('pending', 'failed');

-- Index for lookups by generation (dedup check)
CREATE INDEX idx_documents_generation ON documents(gcs_generation);

-- Index for date-based queries
CREATE INDEX idx_documents_received ON documents(received_at);


-- ============================================================================
-- TRADES TABLE
-- Represents parsed trade data extracted from a document
-- One document can produce multiple trades (e.g., batch confirmations)
-- ============================================================================
CREATE TABLE trades (
    id SERIAL PRIMARY KEY,

    -- Link to source document - CRITICAL for UI traceability
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE RESTRICT,

    -- Parsed trade fields (adjust based on your actual schema)
    ticker TEXT,
    isin TEXT,
    cusip TEXT,
    quantity NUMERIC,
    price NUMERIC,
    notional NUMERIC,
    trade_date DATE,
    settlement_date DATE,
    counterparty TEXT,
    direction TEXT,                           -- 'buy' or 'sell'
    asset_class TEXT,
    currency TEXT,
    exchange TEXT,

    -- Parser metadata
    parsed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confidence_score NUMERIC,                 -- AI parser confidence (0-1)
    raw_extracted_data JSONB,                 -- Full extraction for debugging

    -- Manual review fields
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    review_status TEXT DEFAULT 'pending',     -- pending, approved, rejected

    CONSTRAINT valid_direction CHECK (direction IN ('buy', 'sell')),
    CONSTRAINT valid_review_status CHECK (review_status IN ('pending', 'approved', 'rejected'))
);

-- Index for document lookups (UI: get PDF for this trade)
CREATE INDEX idx_trades_document ON trades(document_id);

-- Index for trade queries
CREATE INDEX idx_trades_date ON trades(trade_date);
CREATE INDEX idx_trades_ticker ON trades(ticker);


-- ============================================================================
-- DOCUMENT_PROCESSING_LOG TABLE (Optional - for detailed audit trail)
-- ============================================================================
CREATE TABLE document_processing_log (
    id SERIAL PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,                 -- 'received', 'processing_started', 'parsed', 'failed', 'retried'
    event_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    details JSONB,                            -- Event-specific details
    worker_id TEXT                            -- Which pod/instance processed it
);

CREATE INDEX idx_processing_log_document ON document_processing_log(document_id);
```

### Key Fields Explained

| Field | Purpose | Why It Matters |
|-------|---------|----------------|
| `documents.id` (UUID) | System-wide unique identifier | Used in GCS path, FK from trades, API lookups |
| `documents.gcs_generation` | GCS object generation number | **Deduplication key** - unique even if file is overwritten |
| `documents.archive_gcs_path` | Where the PDF lives after processing | Could be derived, but explicit is clearer |
| `trades.document_id` | FK to source document | **Enables "View PDF" in UI** |

---

## Deduplication Strategy

### Why GCS Generation?

GCS assigns a unique `generation` number to each object version:
- Immutable once assigned
- Changes if object is overwritten (even with same content)
- Globally unique within the bucket

```python
blob = bucket.get_blob("incoming/file.pdf")
print(blob.generation)  # e.g., 1706547823456789
```

### Deduplication Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      POLL incoming/                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                 ┌────────────────────────┐
                 │  For each PDF blob:    │
                 │  Get blob.generation   │
                 └────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │  SELECT 1 FROM documents      │
              │  WHERE gcs_generation = ?     │
              └───────────────────────────────┘
                              │
               ┌──────────────┴──────────────┐
               │                             │
          [EXISTS]                      [NOT EXISTS]
               │                             │
               ▼                             ▼
     ┌─────────────────┐          ┌─────────────────────┐
     │  Already known  │          │  New document       │
     │  Skip or clean  │          │  Process it         │
     └─────────────────┘          └─────────────────────┘
```

### Idempotency Guarantees

```sql
-- This INSERT is idempotent due to UNIQUE constraint on gcs_generation
INSERT INTO documents (id, original_filename, gcs_generation, ...)
VALUES (gen_random_uuid(), 'file.pdf', 1706547823456789, ...)
ON CONFLICT (gcs_generation) DO NOTHING
RETURNING id;

-- If RETURNING is empty, document was already processed
```

---

## Poller Service Specification

### Configuration

```yaml
# config.yaml
gcs:
  bucket_name: "your-pdf-bucket"
  prefixes:
    incoming: "incoming/"
    processing: "processing/"
    archive: "archive/"
    failed: "failed/"

polling:
  interval_seconds: 30
  batch_size: 10                    # Max PDFs to process per cycle
  max_retries: 3
  retry_backoff_minutes: [5, 15, 60]

database:
  connection_string: "${DATABASE_URL}"
  pool_size: 5

parser:
  endpoint: "${AI_PARSER_URL}"
  timeout_seconds: 120
```

### Core Algorithm

```python
"""
GCS PDF Poller - Core Processing Logic

This module implements the main polling loop with:
- Idempotent processing via gcs_generation
- Atomic claim mechanism for multi-pod safety
- Structured error handling with retry support
"""

import uuid
from datetime import datetime
from typing import Optional
from google.cloud import storage
from google.cloud.storage import Blob
import psycopg2
from psycopg2.extras import RealDictCursor


class GCSPDFPoller:
    """
    Production-ready GCS poller for PDF ingestion.

    Guarantees:
    - Each PDF processed exactly once (via gcs_generation dedup)
    - Safe for multi-pod deployment (via atomic claim)
    - Full traceability (document_id links GCS path to trades)
    """

    def __init__(self, config: dict):
        self.config = config
        self.storage_client = storage.Client()
        self.bucket = self.storage_client.bucket(config['gcs']['bucket_name'])
        self.db_pool = self._create_db_pool()
        self.worker_id = self._get_worker_id()

    def poll_cycle(self) -> dict:
        """
        Execute one polling cycle.

        Returns:
            dict with keys: processed, skipped, failed
        """
        stats = {'processed': 0, 'skipped': 0, 'failed': 0}

        # List incoming PDFs
        prefix = self.config['gcs']['prefixes']['incoming']
        blobs = list(self.bucket.list_blobs(prefix=prefix, max_results=self.config['polling']['batch_size'] * 2))

        for blob in blobs:
            if not blob.name.endswith('.pdf'):
                continue

            # Check if already known
            if self._is_already_processed(blob.generation):
                # Cleanup orphan (shouldn't be in incoming/)
                self._cleanup_orphan(blob)
                stats['skipped'] += 1
                continue

            # Process the PDF
            try:
                self._process_pdf(blob)
                stats['processed'] += 1
            except Exception as e:
                self._handle_failure(blob, e)
                stats['failed'] += 1

        return stats

    def _is_already_processed(self, generation: int) -> bool:
        """Check if this exact file version was already processed."""
        with self.db_pool.getconn() as conn:
            with conn.cursor() as cur:
                cur.execute(
                    "SELECT 1 FROM documents WHERE gcs_generation = %s",
                    (generation,)
                )
                return cur.fetchone() is not None

    def _process_pdf(self, blob: Blob) -> str:
        """
        Process a single PDF through the full pipeline.

        Steps:
        1. Generate document_id
        2. Register in database (idempotent)
        3. Claim by moving to processing/
        4. Run AI parser
        5. Insert parsed trades
        6. Move to archive/
        7. Update status to 'parsed'

        Returns:
            document_id (UUID string)
        """
        document_id = str(uuid.uuid4())
        now = datetime.utcnow()

        with self.db_pool.getconn() as conn:
            try:
                with conn.cursor() as cur:
                    # Step 1: Register document (idempotent via gcs_generation)
                    cur.execute("""
                        INSERT INTO documents (
                            id, original_filename, original_gcs_path,
                            sender_email, gcs_generation, status,
                            processing_started_at
                        )
                        VALUES (%s, %s, %s, %s, %s, 'processing', NOW())
                        ON CONFLICT (gcs_generation) DO NOTHING
                        RETURNING id
                    """, (
                        document_id,
                        self._extract_filename(blob.name),
                        f"gs://{self.bucket.name}/{blob.name}",
                        (blob.metadata or {}).get('sender_email'),
                        blob.generation
                    ))

                    result = cur.fetchone()
                    if result is None:
                        # Already processed by another worker
                        raise AlreadyProcessedError(f"Generation {blob.generation} already exists")

                    document_id = result[0]
                    conn.commit()

                # Step 2: Move to processing/ (atomic claim)
                processing_path = f"{self.config['gcs']['prefixes']['processing']}{document_id}.pdf"
                self.bucket.rename_blob(blob, processing_path)
                processing_blob = self.bucket.blob(processing_path)

                # Step 3: Download and parse
                pdf_content = processing_blob.download_as_bytes()
                parsed_trades = self._run_ai_parser(pdf_content, document_id)

                # Step 4: Insert trades
                with conn.cursor() as cur:
                    for trade in parsed_trades:
                        cur.execute("""
                            INSERT INTO trades (
                                document_id, ticker, isin, quantity, price,
                                trade_date, settlement_date, counterparty,
                                direction, confidence_score, raw_extracted_data
                            )
                            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                        """, (
                            document_id,
                            trade.get('ticker'),
                            trade.get('isin'),
                            trade.get('quantity'),
                            trade.get('price'),
                            trade.get('trade_date'),
                            trade.get('settlement_date'),
                            trade.get('counterparty'),
                            trade.get('direction'),
                            trade.get('confidence_score'),
                            Json(trade.get('raw_data'))
                        ))

                # Step 5: Move to archive/
                archive_path = self._get_archive_path(document_id, now)
                self.bucket.rename_blob(processing_blob, archive_path)

                # Step 6: Update document status
                with conn.cursor() as cur:
                    cur.execute("""
                        UPDATE documents
                        SET status = 'parsed',
                            processed_at = NOW(),
                            archive_gcs_path = %s
                        WHERE id = %s
                    """, (f"gs://{self.bucket.name}/{archive_path}", document_id))

                conn.commit()
                return document_id

            except AlreadyProcessedError:
                conn.rollback()
                raise
            except Exception as e:
                conn.rollback()
                self._mark_failed(conn, document_id, str(e))
                raise

    def _get_archive_path(self, document_id: str, timestamp: datetime) -> str:
        """Generate deterministic archive path from document_id and timestamp."""
        return f"{self.config['gcs']['prefixes']['archive']}{timestamp:%Y/%m}/{document_id}.pdf"

    def _run_ai_parser(self, pdf_content: bytes, document_id: str) -> list[dict]:
        """
        Call AI parser service.

        Args:
            pdf_content: Raw PDF bytes
            document_id: For logging/correlation

        Returns:
            List of parsed trade dictionaries
        """
        # Implementation depends on your AI parser interface
        # Example:
        # response = requests.post(
        #     self.config['parser']['endpoint'],
        #     files={'pdf': pdf_content},
        #     headers={'X-Document-ID': document_id},
        #     timeout=self.config['parser']['timeout_seconds']
        # )
        # return response.json()['trades']
        raise NotImplementedError("Implement AI parser integration")

    def _mark_failed(self, conn, document_id: str, error_message: str):
        """Mark document as failed and move to failed/ prefix."""
        with conn.cursor() as cur:
            cur.execute("""
                UPDATE documents
                SET status = 'failed',
                    error_message = %s,
                    retry_count = retry_count + 1
                WHERE id = %s
            """, (error_message[:500], document_id))

        # Move blob to failed/
        try:
            processing_path = f"{self.config['gcs']['prefixes']['processing']}{document_id}.pdf"
            processing_blob = self.bucket.blob(processing_path)
            if processing_blob.exists():
                failed_path = f"{self.config['gcs']['prefixes']['failed']}{datetime.utcnow():%Y/%m}/{document_id}.pdf"
                self.bucket.rename_blob(processing_blob, failed_path)

                # Add error metadata
                failed_blob = self.bucket.blob(failed_path)
                failed_blob.metadata = {
                    'error': error_message[:500],
                    'failed_at': datetime.utcnow().isoformat(),
                    'document_id': document_id
                }
                failed_blob.patch()
        except Exception:
            pass  # Best effort

        conn.commit()

    @staticmethod
    def _extract_filename(gcs_path: str) -> str:
        """Extract filename from GCS path."""
        return gcs_path.split('/')[-1]


class AlreadyProcessedError(Exception):
    """Raised when attempting to process an already-processed document."""
    pass
```

### Multi-Pod Safety

The system is safe for horizontal scaling because:

1. **Database registration is atomic**: `ON CONFLICT (gcs_generation) DO NOTHING` ensures only one worker wins
2. **GCS rename is atomic**: Moving to `processing/` claims the file
3. **If a worker crashes mid-processing**: File remains in `processing/` with a registered DB record

#### Orphan Recovery (Scheduled Job)

```python
def recover_orphaned_processing_files():
    """
    Find documents stuck in 'processing' status for too long.
    Run via scheduled job (e.g., every 15 minutes).
    """
    with db.cursor() as cur:
        # Find documents stuck in processing for > 10 minutes
        cur.execute("""
            SELECT id, gcs_generation
            FROM documents
            WHERE status = 'processing'
            AND processing_started_at < NOW() - INTERVAL '10 minutes'
        """)

        for doc_id, generation in cur.fetchall():
            # Reset to pending for retry
            cur.execute("""
                UPDATE documents
                SET status = 'pending',
                    retry_count = retry_count + 1
                WHERE id = %s AND status = 'processing'
            """, (doc_id,))

            # Move file back to incoming/ if it exists in processing/
            processing_blob = bucket.blob(f"processing/{doc_id}.pdf")
            if processing_blob.exists():
                bucket.rename_blob(processing_blob, f"incoming/{doc_id}.pdf")
```

---

## UI Integration - PDF Viewer

### API Endpoint

```python
"""
API endpoint for retrieving PDF viewing URL from trade ID.
"""

from fastapi import APIRouter, HTTPException
from google.cloud import storage
from datetime import timedelta

router = APIRouter()

@router.get("/api/trades/{trade_id}/pdf")
async def get_trade_pdf(trade_id: int, db = Depends(get_db)):
    """
    Get a signed URL to view the PDF for a specific trade.

    Returns:
        {
            "document_id": "550e8400-e29b-41d4-a716-446655440000",
            "original_filename": "confirmation_ABC123.pdf",
            "sender_email": "broker@example.com",
            "received_at": "2025-01-29T10:30:00Z",
            "pdf_url": "https://storage.googleapis.com/...",  # Signed URL
            "expires_in_seconds": 3600
        }
    """
    # Query document info via trade
    result = db.execute("""
        SELECT
            d.id as document_id,
            d.original_filename,
            d.sender_email,
            d.received_at,
            d.archive_gcs_path
        FROM trades t
        JOIN documents d ON t.document_id = d.id
        WHERE t.id = %s
    """, (trade_id,)).fetchone()

    if not result:
        raise HTTPException(status_code=404, detail="Trade not found")

    # Generate signed URL
    bucket = storage.Client().bucket(BUCKET_NAME)

    # Extract blob path from full GCS URI
    # gs://bucket/archive/2025/01/uuid.pdf -> archive/2025/01/uuid.pdf
    blob_path = result['archive_gcs_path'].replace(f"gs://{BUCKET_NAME}/", "")
    blob = bucket.blob(blob_path)

    signed_url = blob.generate_signed_url(
        version="v4",
        expiration=timedelta(hours=1),
        method="GET"
    )

    return {
        "document_id": str(result['document_id']),
        "original_filename": result['original_filename'],
        "sender_email": result['sender_email'],
        "received_at": result['received_at'].isoformat(),
        "pdf_url": signed_url,
        "expires_in_seconds": 3600
    }


@router.get("/api/documents/{document_id}/pdf")
async def get_document_pdf(document_id: str, db = Depends(get_db)):
    """
    Get a signed URL to view a PDF by document ID directly.
    Useful for document-centric views.
    """
    result = db.execute("""
        SELECT archive_gcs_path, original_filename, sender_email, received_at
        FROM documents
        WHERE id = %s AND status = 'parsed'
    """, (document_id,)).fetchone()

    if not result:
        raise HTTPException(status_code=404, detail="Document not found")

    # Generate signed URL (same as above)
    # ...
```

### Frontend Integration

```typescript
// React component example
interface Trade {
  id: number;
  ticker: string;
  quantity: number;
  // ... other fields
  document_id: string;
}

const TradeRow: React.FC<{ trade: Trade }> = ({ trade }) => {
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleViewPdf = async () => {
    setLoading(true);
    try {
      const response = await fetch(`/api/trades/${trade.id}/pdf`);
      const data = await response.json();
      // Open in new tab or modal
      window.open(data.pdf_url, '_blank');
    } catch (error) {
      console.error('Failed to get PDF URL', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <tr>
      <td>{trade.ticker}</td>
      <td>{trade.quantity}</td>
      {/* ... other columns */}
      <td>
        <button onClick={handleViewPdf} disabled={loading}>
          {loading ? 'Loading...' : 'View PDF'}
        </button>
      </td>
    </tr>
  );
};
```

---

## Error Handling & Retry Logic

### Failure Categories

| Category | Example | Action |
|----------|---------|--------|
| **Transient** | Network timeout, GCS rate limit | Retry with backoff |
| **Parser Error** | AI couldn't extract data | Retry, then manual review |
| **Validation Error** | Missing required fields | Move to failed, alert |
| **Permanent** | Corrupt PDF, wrong file type | Move to failed, no retry |

### Retry Configuration

```python
RETRY_CONFIG = {
    'max_retries': 3,
    'backoff_minutes': [5, 15, 60],  # Exponential-ish backoff
    'retryable_errors': [
        'ConnectionError',
        'TimeoutError',
        'RateLimitError',
        'ParserTemporaryError'
    ]
}

def should_retry(document: dict, error: Exception) -> bool:
    """Determine if a failed document should be retried."""
    if document['retry_count'] >= RETRY_CONFIG['max_retries']:
        return False

    error_type = type(error).__name__
    return error_type in RETRY_CONFIG['retryable_errors']


def get_retry_delay(retry_count: int) -> int:
    """Get delay in minutes before next retry."""
    backoffs = RETRY_CONFIG['backoff_minutes']
    index = min(retry_count, len(backoffs) - 1)
    return backoffs[index]
```

### Failed Document Recovery

```sql
-- Query for documents needing retry
SELECT id, original_filename, error_message, retry_count
FROM documents
WHERE status = 'failed'
AND retry_count < 3
AND processed_at < NOW() - (
    CASE retry_count
        WHEN 0 THEN INTERVAL '5 minutes'
        WHEN 1 THEN INTERVAL '15 minutes'
        ELSE INTERVAL '60 minutes'
    END
);

-- Manual retry trigger
UPDATE documents
SET status = 'pending',
    error_message = NULL
WHERE id = '{document_id}';
```

---

## Monitoring & Observability

### Key Metrics

```python
# Prometheus metrics example
from prometheus_client import Counter, Histogram, Gauge

# Counters
documents_received = Counter('documents_received_total', 'Total PDFs received')
documents_processed = Counter('documents_processed_total', 'Total PDFs successfully processed')
documents_failed = Counter('documents_failed_total', 'Total PDFs that failed processing', ['error_type'])
trades_extracted = Counter('trades_extracted_total', 'Total trades extracted from PDFs')

# Histograms
processing_duration = Histogram('document_processing_seconds', 'Time to process a PDF')
parser_duration = Histogram('ai_parser_seconds', 'Time for AI parser to extract trades')

# Gauges
pending_documents = Gauge('documents_pending', 'Documents waiting to be processed')
processing_documents = Gauge('documents_processing', 'Documents currently being processed')
```

### Health Check Queries

```sql
-- Documents by status (dashboard)
SELECT status, COUNT(*)
FROM documents
GROUP BY status;

-- Processing rate (last hour)
SELECT
    date_trunc('minute', processed_at) as minute,
    COUNT(*) as processed
FROM documents
WHERE processed_at > NOW() - INTERVAL '1 hour'
GROUP BY 1
ORDER BY 1;

-- Failed documents needing attention
SELECT id, original_filename, error_message, retry_count, processed_at
FROM documents
WHERE status = 'failed'
ORDER BY processed_at DESC
LIMIT 20;

-- Average trades per document
SELECT AVG(trade_count)
FROM (
    SELECT document_id, COUNT(*) as trade_count
    FROM trades
    GROUP BY document_id
) sub;
```

### Alerting Rules

```yaml
# Alert if processing backlog grows
- alert: PDFProcessingBacklog
  expr: documents_pending > 100
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "PDF processing backlog growing"

# Alert if failure rate spikes
- alert: PDFProcessingFailureRate
  expr: rate(documents_failed_total[5m]) / rate(documents_received_total[5m]) > 0.1
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "PDF processing failure rate above 10%"

# Alert if no documents processed recently (stalled)
- alert: PDFProcessingStalled
  expr: rate(documents_processed_total[15m]) == 0 AND documents_pending > 0
  for: 15m
  labels:
    severity: critical
  annotations:
    summary: "PDF processing appears stalled"
```

---

## Deployment

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pdf-poller
spec:
  replicas: 2  # Safe to scale horizontally
  selector:
    matchLabels:
      app: pdf-poller
  template:
    metadata:
      labels:
        app: pdf-poller
    spec:
      serviceAccountName: pdf-poller-sa  # With GCS permissions
      containers:
      - name: poller
        image: your-registry/pdf-poller:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: GCS_BUCKET
          value: "your-pdf-bucket"
        - name: POLL_INTERVAL_SECONDS
          value: "30"
        - name: AI_PARSER_URL
          value: "http://ai-parser-service:8080"
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

### Required IAM Permissions

```yaml
# GCS permissions for service account
roles:
  - roles/storage.objectViewer   # Read incoming/
  - roles/storage.objectCreator  # Write to archive/, failed/
  - roles/storage.objectAdmin    # Delete/move objects (for rename operations)

# Scoped to specific bucket
resource: "projects/_/buckets/your-pdf-bucket"
```

---

## Testing Strategy

### Unit Tests

```python
def test_deduplication():
    """Verify same generation number is not processed twice."""
    poller = GCSPDFPoller(config)

    # First processing should succeed
    result1 = poller._process_pdf(mock_blob_gen_123)
    assert result1 is not None

    # Second processing of same generation should be skipped
    with pytest.raises(AlreadyProcessedError):
        poller._process_pdf(mock_blob_gen_123)


def test_archive_path_deterministic():
    """Verify archive path is predictable from document_id and timestamp."""
    doc_id = "550e8400-e29b-41d4-a716-446655440000"
    timestamp = datetime(2025, 1, 29, 10, 30, 0)

    path = poller._get_archive_path(doc_id, timestamp)

    assert path == "archive/2025/01/550e8400-e29b-41d4-a716-446655440000.pdf"


def test_trade_document_linkage():
    """Verify trades are correctly linked to source document."""
    # Process a PDF
    doc_id = poller._process_pdf(mock_blob)

    # Query trades
    trades = db.execute("SELECT document_id FROM trades WHERE document_id = %s", (doc_id,))

    assert len(trades) > 0
    assert all(t['document_id'] == doc_id for t in trades)
```

### Integration Tests

```python
def test_full_pipeline_e2e():
    """End-to-end test: upload PDF -> poll -> parse -> verify in DB."""
    # 1. Upload test PDF to incoming/
    test_pdf = load_test_pdf("sample_trade_confirmation.pdf")
    blob = bucket.blob("incoming/test_e2e.pdf")
    blob.metadata = {'sender_email': 'test@example.com'}
    blob.upload_from_string(test_pdf)

    # 2. Run poller
    stats = poller.poll_cycle()
    assert stats['processed'] == 1

    # 3. Verify document in DB
    doc = db.execute("SELECT * FROM documents WHERE original_filename = 'test_e2e.pdf'").fetchone()
    assert doc is not None
    assert doc['status'] == 'parsed'

    # 4. Verify trades extracted
    trades = db.execute("SELECT * FROM trades WHERE document_id = %s", (doc['id'],)).fetchall()
    assert len(trades) > 0

    # 5. Verify PDF in archive
    archive_blob = bucket.blob(doc['archive_gcs_path'].replace(f"gs://{BUCKET_NAME}/", ""))
    assert archive_blob.exists()

    # 6. Verify API endpoint works
    response = client.get(f"/api/documents/{doc['id']}/pdf")
    assert response.status_code == 200
    assert 'pdf_url' in response.json()
```

---

## Migration Plan (From Manual Upload)

### Phase 1: Parallel Operation
1. Deploy poller alongside existing manual upload
2. Both systems write to same `documents` and `trades` tables
3. Manual uploads continue to work unchanged

### Phase 2: Validation
1. Compare results: manual vs automated for same PDFs
2. Tune AI parser based on discrepancies
3. Build confidence in automated system

### Phase 3: Cutover
1. Disable manual upload for mailbox-sourced PDFs
2. Keep manual upload for ad-hoc/exception cases
3. Monitor automated pipeline closely

### Backward Compatibility

```sql
-- Add source tracking to documents table
ALTER TABLE documents ADD COLUMN source TEXT DEFAULT 'manual';
-- Values: 'manual', 'mailbox_poller', 'api_upload'

-- Existing manual uploads continue to work
-- New poller sets source = 'mailbox_poller'
```

---

## Summary Checklist

### Must Have
- [ ] GCS bucket with `incoming/`, `processing/`, `archive/`, `failed/` prefixes
- [ ] PostgreSQL `documents` table with `gcs_generation` unique constraint
- [ ] PostgreSQL `trades` table with `document_id` foreign key
- [ ] Poller service with deduplication logic
- [ ] API endpoint for PDF URL generation
- [ ] Signed URL generation for secure PDF access

### Should Have
- [ ] Prometheus metrics for monitoring
- [ ] Alerting for backlog and failure rate
- [ ] Orphan recovery job for stuck `processing/` files
- [ ] Retry logic with exponential backoff
- [ ] Document processing audit log

### Nice to Have
- [ ] Admin UI for failed document management
- [ ] Manual retry trigger via API
- [ ] Batch reprocessing capability
- [ ] Parser confidence thresholds with auto-review routing
