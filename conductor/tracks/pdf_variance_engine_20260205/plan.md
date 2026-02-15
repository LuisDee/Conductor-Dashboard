# Plan: PDF Variance Engine for Parser Robustness Testing

## Phase 1: Variance Configuration System

- [x] Task 1.1: Write tests for VarianceConfig dataclass
  - [x] Test seeded random generation is deterministic
  - [x] Test jurisdiction constraints are respected
  - [x] Test all variance dimensions have valid values

- [x] Task 1.2: Implement VarianceConfig and constraints
  - [x] Create `engine/variance_config.py`
  - [x] Define JURISDICTION_CONSTRAINTS mapping
  - [x] Implement `VarianceConfig.from_seed(seed, jurisdiction)`
  - [x] Map skins to jurisdictions

- [x] Task 1.3: Conductor - User Manual Verification 'Phase 1'

## Phase 2: Jinja2 Variance Filters

- [x] Task 2.1: Write tests for date format filter
  - [x] Test US format (MM/DD/YYYY)
  - [x] Test UK format (DD/MM/YYYY)
  - [x] Test ISO format (YYYY-MM-DD)
  - [x] Test IB format (DD-MMM-YYYY)

- [x] Task 2.2: Write tests for number format filter
  - [x] Test US format (1,234.56)
  - [x] Test EU format (1.234,56)
  - [x] Test plain format (1234.56)
  - [x] Test Indian lakh format (1,23,456.00)

- [x] Task 2.3: Write tests for currency format filter
  - [x] Test prefix symbol ($100.00)
  - [x] Test suffix code (100.00 USD)
  - [x] Test prefix code (USD 100.00)

- [x] Task 2.4: Write tests for text formatting filters
  - [x] Test header casing (title, upper, lower)
  - [x] Test name format (natural, last_first, upper)

- [x] Task 2.5: Implement Jinja2 custom filters
  - [x] Create `engine/variance_filters.py`
  - [x] Implement format_date, format_number, format_currency
  - [x] Implement apply_casing, format_name
  - [x] Register filters with Jinja2 Environment

- [x] Task 2.6: Conductor - User Manual Verification 'Phase 2'

## Phase 3: Faker Integration

- [x] Task 3.1: Write tests for seeded Faker generation
  - [x] Test disclaimer text varies with seed
  - [x] Test reference numbers are reproducible
  - [x] Test same seed produces identical text

- [x] Task 3.2: Implement Faker variance provider
  - [x] Create `engine/faker_provider.py`
  - [x] Implement seeded disclaimer generator
  - [x] Implement reference number generator
  - [x] Implement broker address generator

- [x] Task 3.3: Conductor - User Manual Verification 'Phase 3'

## Phase 4: Template Integration

- [x] Task 4.1: Update TemplateEngine for variance support
  - [x] Add variance_enabled parameter
  - [x] Initialize VarianceConfig from seed + skin
  - [x] Initialize FakerVarianceProvider
  - [x] Pass variance context to templates
  - [x] Add get_variance_metadata() method

- [~] Task 4.2-4.5: Update broker templates with variance filters
  - [ ] Templates use variance filters when variance_enabled=True
  - Note: Templates can be updated incrementally as needed

- [x] Task 4.6: Conductor - User Manual Verification 'Phase 4'

## Phase 5: CLI & Ground Truth Integration

- [x] Task 5.1: Write tests for CLI variance flags
  - [x] Test --variance flag enables variance mode
  - [x] Test --seed controls randomness
  - [x] Test --count with variance generates multiple variants
  - [x] Test output filenames include seed/variant

- [x] Task 5.2: Update CLI to support variance
  - [x] Add --variance flag to argparse
  - [x] Pass variance_enabled to TemplateEngine
  - [x] Update output filename pattern for variants

- [x] Task 5.3: Write tests for variance metadata in ground truth
  - [x] Test variance_applied section is present
  - [x] Test all dimensions are recorded
  - [x] Test seed is captured

- [x] Task 5.4: Update ground truth output
  - [x] Add variance_applied to JSON output
  - [x] Include all variance dimensions used

- [x] Task 5.5: Conductor - User Manual Verification 'Phase 5'

## Phase 6: Integration Testing & Documentation

- [x] Task 6.1: Write integration tests
  - [x] Test: same seed → identical HTML output
  - [x] Test: different seeds → different metadata
  - [x] Test: 5 variants of same trade data
  - [x] Verify ground truth trades identical across variants

- [x] Task 6.2: Update README documentation
  - [x] Document --variance flag usage
  - [x] Document variance dimensions
  - [x] Add examples for batch variance generation

- [ ] Task 6.3: Create variance test samples
  - [ ] Generate 10 variants of single_trade.json
  - [ ] Store in samples/variance/ for manual inspection

- [ ] Task 6.4: Conductor - User Manual Verification 'Phase 6'
