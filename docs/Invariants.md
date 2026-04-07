# Invariants

These are the rules tests should protect.

## Determinism

- the same manifest and data snapshot produce the same ledger and metrics
- replaying a stored ledger reproduces the same terminal state

## Accounting

- cash changes only through documented fills, fees, and cashflows
- position quantity changes only through documented fills
- equity reconciles on every ledger row

## Time Boundaries

- no signal reads future bars
- no order fill reads future bars
- no trailing stop uses same-bar information to tighten and trigger on that same bar under the default contract

## Data Boundaries

- core code never depends on provider-native response types
- run artifacts identify the data snapshot they used
- provider choice is visible in the run manifest

## Explainability

- every entry and exit has a reason code
- every blocked trade has a reason when filtering is active
- every corporate action that changes holdings or cash is inspectable

## Scope Discipline

- the first trustworthy version remains single-symbol and long-only
- portfolio breadth is added only after these invariants are well tested
