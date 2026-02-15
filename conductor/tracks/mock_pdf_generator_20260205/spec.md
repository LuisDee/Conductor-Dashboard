# Spec: Mock PDF Generator for Parser Testing

## Overview

A standalone CLI tool that generates realistic broker PDFs (Activity Statements, Contract Notes, Trade Confirmations) from user-specified trade data. Designed for end-to-end testing of the PA Dealing document extraction pipeline — generate a PDF with known trades, submit through the chatbot, verify the parser extracts exactly what you specified.

## Functional Requirements

### FR1: Document Types
The generator MUST support three document types:
- **Activity Statements** — Monthly brokerage statements (IB-style) with trades section
- **Contract Notes** — Single or batch trade confirmations (UK broker style)
- **Trade Confirmations** — US-style trade confirms

### FR2: Broker Skins
The generator MUST support multiple broker templates ("skins"):
- **Interactive Brokers** — Activity Statement and Trade Confirmation formats
- **UK Contract Note** — Standard UK broker format with stamp duty, GBX pricing
- **Fidelity/US Brokerage** — US retail broker statement style
- **Indian Brokers** — Zerodha/ICICI style with STT, GST, stamp duty

Skins are selectable via CLI flag (e.g., `--skin ib_activity`).

### FR3: Trade Input via JSON
Users specify trades in a JSON input file:
```json
{
  "account_holder": "Luis De Burnay-Bastos",
  "account_number": "U1234567",
  "statement_date": "2026-02-05",
  "trades": [
    {
      "security": "AAPL",
      "isin": "US0378331005",
      "action": "BUY",
      "quantity": 100,
      "price": 235.50,
      "currency": "USD",
      "trade_date": "2026-02-03",
      "settlement_date": "2026-02-05"
    }
  ]
}
```

### FR4: Ground Truth Output
For each generated PDF, output a `_ground_truth.json` containing the trade-centric data that went into the PDF:
```json
{
  "document_type": "ACTIVITY_STATEMENT",
  "broker_skin": "ib_activity",
  "account_holder": "Luis De Burnay-Bastos",
  "trades": [
    {
      "security": "AAPL",
      "isin": "US0378331005",
      "action": "BUY",
      "quantity": 100,
      "price": 235.50,
      "currency": "USD",
      "trade_date": "2026-02-03",
      "settlement_date": "2026-02-05",
      "commission": 1.00,
      "net_amount": 23551.00
    }
  ]
}
```

### FR5: CLI Interface
```bash
python -m mock_pdf_generator.generate \
  --input trades.json \
  --skin ib_activity \
  --output ./output/
```

Options:
- `--input` — Path to JSON file with trade data
- `--skin` — Broker template (ib_activity, ib_confirmation, uk_contract_note, fidelity, indian)
- `--output` — Output directory for PDF + ground truth JSON
- `--seed` — Optional RNG seed for reproducible variance (header casing, disclaimers)
- `--count` — Generate multiple variants (1-10) from same input with variance

### FR6: Fee Calculation
The generator MUST compute realistic fees based on jurisdiction:
- **US**: Commission + SEC fee + TAF fee
- **UK**: Commission + Stamp duty (0.5% on buys)
- **India**: Brokerage + STT + GST + stamp duty + SEBI fee

Net amount = (quantity × price) ± fees (depending on buy/sell).

### FR7: Template Variance
Each skin supports minor variance dimensions:
- Date format (MM/DD/YYYY, DD-MMM-YYYY, DD/MM/YYYY)
- Number format (US: 1,234.56 vs EU: 1.234,56)
- Header casing ("Trade Date" vs "TRADE DATE")
- Disclaimer text (sampled from pre-generated pool)

## Non-Functional Requirements

### NFR1: Technology Stack
- **Templating**: Jinja2
- **PDF Rendering**: WeasyPrint
- **Language**: Python 3.11+
- **No external dependencies** on LLMs or network services

### NFR2: Performance
- Generate a single PDF in <2 seconds
- Batch of 10 PDFs in <15 seconds

### NFR3: Standalone
- No integration with existing PA Dealing services
- Self-contained module under `tools/mock_pdf_generator/`

## Acceptance Criteria

1. `python -m mock_pdf_generator.generate --input sample.json --skin ib_activity` produces a valid PDF
2. Generated PDF visually resembles real IB Activity Statement
3. Ground truth JSON contains all input trades with computed fees
4. At least 4 broker skins implemented (IB activity, IB confirm, UK contract note, Fidelity)
5. Fee calculations are mathematically correct (net = gross ± fees)
6. Unit tests cover fee calculation and JSON parsing

## Out of Scope

- Chaos injection / edge case traps (separate future track)
- Integration with DocumentAgent or ExtractionRouter
- Automated parser accuracy testing
- PDFs with >50 trades (keep simple for testing)
- Scanned/rotated/degraded appearance simulation
