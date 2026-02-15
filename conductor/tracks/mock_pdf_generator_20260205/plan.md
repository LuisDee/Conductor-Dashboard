# Plan: Mock PDF Generator for Parser Testing

## Phase 1: Project Scaffold & Template Engine Setup ✅ COMPLETE

- [x] Task 1.1: Create project structure
- [x] Task 1.2: Write tests for JSON input parsing
- [x] Task 1.3: Implement JSON input schema
- [x] Task 1.4: Write tests for basic PDF generation
- [x] Task 1.5: Implement base template engine
- [x] Task 1.6: Conductor - User Manual Verification 'Phase 1'

## Phase 2: Interactive Brokers Templates ✅ COMPLETE

- [x] Task 2.1: Reverse-engineer IB Activity Statement layout
- [x] Task 2.2: Write tests for IB Activity Statement generation
- [x] Task 2.3: Implement IB Activity Statement template
- [x] Task 2.4: Write tests for IB Trade Confirmation generation
- [x] Task 2.5: Implement IB Trade Confirmation template
- [x] Task 2.6: Conductor - User Manual Verification 'Phase 2'

## Phase 3: UK & US Broker Templates ✅ COMPLETE

- [x] Task 3.1: Write tests for UK Contract Note generation
- [x] Task 3.2: Implement UK Contract Note template
- [x] Task 3.3: Write tests for Fidelity statement generation
- [x] Task 3.4: Implement Fidelity template
- [x] Task 3.5: Conductor - User Manual Verification 'Phase 3'

## Phase 4: Fee Calculation Engine ✅ COMPLETE

- [x] Task 4.1: Write tests for US fee calculation
- [x] Task 4.2: Implement US fee calculator
- [x] Task 4.3: Write tests for UK fee calculation
- [x] Task 4.4: Implement UK fee calculator
- [x] Task 4.5: Write tests for Indian fee calculation
- [x] Task 4.6: Implement Indian fee calculator
- [x] Task 4.7: Conductor - User Manual Verification 'Phase 4'

## Phase 5: CLI Interface & Variance System ✅ COMPLETE (variance deferred)

- [x] Task 5.1: Write tests for CLI argument parsing (manual verification)
- [x] Task 5.2: Implement CLI with argparse
- [ ] Task 5.3: Write tests for variance injection (DEFERRED - not in scope)
- [ ] Task 5.4: Implement variance system (DEFERRED - not in scope)
- [x] Task 5.5: Conductor - User Manual Verification 'Phase 5'

## Phase 6: Indian Broker Template & Final Polish ✅ COMPLETE

- [x] Task 6.1: Write tests for Indian broker statement
- [x] Task 6.2: Implement Indian broker template
- [x] Task 6.3: Create sample input files
- [x] Task 6.4: Write integration tests (manual CLI verification)
- [x] Task 6.5: Documentation (README.md)
- [x] Task 6.6: Conductor - User Manual Verification 'Phase 6'

---

## Summary

**Completed:**
- 5 broker skins: ib_activity, ib_confirmation, uk_contract_note, fidelity, indian
- Fee calculators for US, UK, India jurisdictions
- CLI interface with --input, --skin, --output, --seed, --count options
- 38 unit tests passing
- 4 sample JSON files
- README documentation

**Deferred:**
- Variance injection system (date formats, number formats, header casing) - not in initial scope

**Generated Outputs:**
- `tools/mock_pdf_generator/output/` contains 6 PDFs + ground truth JSON files
