# Spec: PDF Variance Engine for Parser Robustness Testing

## Overview

Add a variance injection system to the Mock PDF Generator that produces visually different PDFs from identical trade data. The primary goal is to test the AI parser's ability to recognize and extract trades sections regardless of visual formatting differences.

The same trade data (AAPL, 100 shares, BUY @ $235.50) should generate multiple PDFs where:
- Dates appear as "02/03/2026", "03-Feb-2026", "2026-02-03"
- Numbers appear as "23,550.00", "23.550,00", "23550.00"
- Headers say "Trade Date", "TRADE DATE", "trade date"
- Currency shows "$235.50", "235.50 USD", "USD 235.50"

The parser should extract **identical ground truth** from all variants.

## Problem Statement

50% of document parsing is **finding the trades section** in varying layouts. Current templates have fixed formatting, limiting parser robustness testing. We need controlled variance to:

1. Test date format parsing (US vs UK vs ISO)
2. Test number format parsing (US vs EU locale)
3. Test section header detection (casing, labels)
4. Test currency recognition (symbol vs code placement)
5. Generate regression test suites for parser improvements

## Functional Requirements

### FR1: Variance Configuration via Seed
Generate reproducible variance using a seed value:
```bash
python -m tools.mock_pdf_generator.generate \
  --input trades.json \
  --skin ib_activity \
  --seed 42 \
  --variance
```

Same seed + same input = identical PDF every time.

### FR2: Variance Dimensions (Priority Order)

| Dimension | Variants | Parser Impact |
|-----------|----------|---------------|
| Date format | MM/DD/YYYY, DD/MM/YYYY, DD-MMM-YYYY, YYYY-MM-DD | Critical |
| Number format | 1,234.56 (US), 1.234,56 (EU), 1234.56 (plain) | Critical |
| Currency placement | $100, 100 USD, USD 100.00 | High |
| Header casing | Title Case, UPPER CASE, lower case | Medium |
| Header labels | Quantity/Qty/Shares/Units, Price/Rate/Exec Price | Medium |
| Name format | Natural, LAST, First, Last First Upper | Medium |
| Security ID display | Ticker only, ISIN only, Both, CUSIP | Medium |
| Table style | Bordered, Horizontal lines, Zebra stripes, Minimal | Low |

### FR3: Jurisdiction Constraints
Variance combinations must be realistic per jurisdiction:
```python
JURISDICTION_CONSTRAINTS = {
    "us": {
        "date_formats": ["us", "iso", "ib"],  # MM/DD, YYYY-MM-DD, DD-MMM
        "number_formats": ["us"],              # 1,234.56 only
        "currency_positions": ["prefix_symbol", "suffix_code"],
    },
    "uk": {
        "date_formats": ["uk", "iso"],         # DD/MM, YYYY-MM-DD
        "number_formats": ["us"],              # UK uses US number format
        "currency_positions": ["prefix_symbol"],
    },
    "eu": {
        "date_formats": ["uk", "iso"],
        "number_formats": ["eu"],              # 1.234,56
        "currency_positions": ["suffix_code", "prefix_code"],
    },
    "india": {
        "date_formats": ["uk", "iso"],
        "number_formats": ["india"],           # 1,23,456.00 (lakhs)
        "currency_positions": ["prefix_symbol"],
    },
}
```

### FR4: Variance Metadata in Ground Truth
Each generated PDF's ground truth must include variance applied:
```json
{
  "document_type": "ACTIVITY_STATEMENT",
  "broker_skin": "ib_activity",
  "account_holder": "Luis De Burnay-Bastos",
  "trades": [...],
  "variance_applied": {
    "seed": 42,
    "date_format": "uk",
    "number_format": "us",
    "currency_position": "suffix_code",
    "header_casing": "upper",
    "header_labels": {"qty": "SHARES", "price": "RATE"},
    "name_format": "last_first",
    "security_id_display": "both",
    "table_style": "horizontal"
  }
}
```

### FR5: Faker Integration for Realistic Text
Use Faker (seeded) to generate:
- Disclaimer paragraphs (varying lengths/styles)
- Broker addresses and contact info
- Reference numbers and confirmation IDs
- Footnote text variations

```python
fake = Faker()
Faker.seed(seed)
disclaimer = fake.paragraph(nb_sentences=5)
ref_number = fake.bothify("CN-####-????-##")
```

### FR6: Batch Variance Generation
Generate multiple variants in one command:
```bash
# Generate 5 variants with seeds 42-46
python -m tools.mock_pdf_generator.generate \
  --input trades.json \
  --skin ib_activity \
  --seed 42 \
  --count 5 \
  --variance
```

Output:
```
output/
├── Luis_De_Burnay-Bastos_ib_activity_v42.pdf
├── Luis_De_Burnay-Bastos_ib_activity_v42_ground_truth.json
├── Luis_De_Burnay-Bastos_ib_activity_v43.pdf
├── Luis_De_Burnay-Bastos_ib_activity_v43_ground_truth.json
└── ...
```

### FR7: Jinja2 Filters for Variance
Custom filters for templates:
```jinja
{{ trade.trade_date | format_date(variance.date_format) }}
{{ trade.price | format_number(variance.number_format) }}
{{ trade.price | format_currency(variance.currency_position, trade.currency) }}
{{ "Trade Date" | apply_casing(variance.header_casing) }}
{{ account_holder | format_name(variance.name_format) }}
```

## Non-Functional Requirements

### NFR1: Reproducibility
Given the same seed, input, and skin, output must be byte-identical.

### NFR2: Performance
Variance injection should add <100ms overhead per PDF.

### NFR3: Backwards Compatibility
Existing CLI without `--variance` flag works unchanged.

## Acceptance Criteria

1. `--seed 42 --variance` produces deterministic output
2. Same trade data with seeds 42, 43, 44 produces visually different PDFs
3. Ground truth JSON includes full `variance_applied` metadata
4. Parser extracts identical trade data from all variants
5. Faker-generated text varies with seed but is reproducible
6. Unit tests cover all variance dimensions
7. Integration test: 10 variants → parser → identical results

## Out of Scope

- Column order variance (requires major template refactor)
- Page orientation variance (portrait/landscape)
- Multi-page layout variance
- Scanned/degraded image simulation
