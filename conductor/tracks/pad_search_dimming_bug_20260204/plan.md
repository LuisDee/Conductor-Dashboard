# Implementation Plan: PAD Search Row Dimming Bug

## Phase 1: Diagnostic & Verification
- [x] Add console logs to `PADSearch.tsx` inside `rowClassName` to verify match logic.
- [x] Verify if `opacity-40` is actually present in the browser's computed styles for non-matching rows. (Fixed by switching to robust `.row-dimmed` class).
- [x] Inspect the `Table` component in the browser to see if `background: white` is masking the opacity. (Fixed in `Table.tsx`).

## Phase 2: Robust Matching (ID-based)
- [x] Refactor `PADSearch.tsx` to use a `Set<string>` of IDs instead of `WeakSet<object>`.
- [x] Use a composite key (e.g., `${row.id}-${row.inst_symbol}-${row.last_trade_date}`) if `row.id` is not unique due to date fan-out.

## Phase 3: CSS & Component Fix
- [x] Define a `.row-dimmed` class in `index.css` with `opacity: 0.4 !important;`.
- [x] Update `Table.tsx` to ensure inline background styles don't override the dimmed state.
- [x] Update `PADSearch.tsx` to use the new `.row-dimmed` class.

## Phase 4: Verification
- [x] Verify fix with multiple filters.
- [x] Ensure hover state still works (or is appropriately dimmed) for non-matching rows. (Fixed by conditional hover background in `Table.tsx`).