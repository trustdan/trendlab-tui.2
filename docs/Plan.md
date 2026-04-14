# TrendLab Plan

This is the active roadmap for the rebuild.

## Goal

Ship a trustworthy research engine first, then layer on operator surfaces and broader research tooling.

## Locked Decisions

- Language: Rust
- TUI: ratatui
- Data provider direction: Tiingo behind a provider boundary
- Build order: truthful lab, then CLI, then TUI shell
- Workspace: multi-crate from day one
- Canonical market model types live in `trendlab-core`
- Persisted run artifacts live in a shared artifact crate, not in the CLI
- Default validation is deterministic and network-free
- Point-in-time universe support: post-v1
- Validation command target: `cargo xtask validate`

## Milestones

### M0: Planning Baseline

Purpose: freeze the operating model before code exists.

Deliverables:

- `AGENTS.md`
- planning docs for workspace, math, bar semantics, invariants, and status
- Cursor rules aligned with repo rules

Acceptance criteria:

- repo guidance is internally consistent
- implementation order is explicit
- future sessions can start from repo-native instructions

### M1: Minimal Truthful Kernel

Purpose: build the smallest order-lifecycle simulator worth trusting.

Scope:

- one symbol
- daily bars only
- long-only
- canonical daily bars supplied by fixtures
- one queued market-entry path
- one fixed protective stop exit path
- raw fills
- deterministic replay
- shared run artifact schema
- event ledger
- golden tests
- one hand-authored oracle scenario or truth table for the first truthful path

Acceptance criteria:

- same inputs produce identical outputs
- every state transition is replayable from the ledger
- no lookahead across bar boundaries
- cash, position, and equity reconcile on every row
- expected versus actual ledger rows can be diffed in tests
- at least one M1 scenario is checked against a hand-authored oracle, not only generated golden output
- default validation does not require network access

Notes:

- M1 is about truthful order lifecycle and replay, not generalized strategy composition.
- M1 starts with a fixture-authored entry-intent reference flow, not generalized signal logic.
- Close-confirmed breakout logic, stop-entry logic, and reusable strategy components are deferred.
- A minimal ledger inspection utility in `xtask` is acceptable before the CLI milestone.

### M2: Canonical Market Data Layer

Purpose: own the market model instead of inheriting provider behavior.

Scope:

- provider trait
- Tiingo adapter
- raw daily bar cache
- corporate actions store
- derived analysis series
- internal weekly and monthly resampling

Acceptance criteria:

- core simulation consumes normalized internal data, not provider-native types
- provider swap would not require core changes
- resampling semantics are documented and tested
- live provider smoke checks, if present, stay behind a separate command and outside default validation

### M3: Componentized Strategy System

Purpose: preserve fair comparison between ideas.

Scope:

- signal generators
- position managers
- execution models
- filters
- close-confirmed breakout entry model
- stop-entry breakout model

Acceptance criteria:

- strategy definitions compose from separate layers
- execution models do not mutate signal logic
- position managers only use allowed state

### M4: CLI

Purpose: make the engine operable without waiting on the TUI.

Scope:

- run command
- explain command
- diff command
- data audit command

Acceptance criteria:

- a run can be launched, inspected, and replayed from the CLI
- the CLI can surface ledger reasoning and metric inputs

### M5: TUI Shell

Purpose: restore the original keyboard-first workflow.

Scope:

- minimal shell and navigation
- results list
- chart view
- help panel
- audit panel

Acceptance criteria:

- a run can be inspected from the TUI without losing auditability
- the audit surface is first-class, not bolted on

### M6: Research Extensions

Purpose: broaden search and validation only after trust exists.

Scope:

- cross-symbol aggregation
- walk-forward validation
- bootstrap confidence
- separate leaderboards by layer
- point-in-time universe support later in this phase

Acceptance criteria:

- research features sit on top of a stable deterministic core
- extra statistical layers do not obscure run provenance

### M7: Trust Hardening And Operator Surface

Purpose: close the remaining trust and operability gaps before scheduling new market-model breadth.

Scope:

- operator-facing run specs layered on top of shared and core request types
- replay and research artifact portability plus integrity hardening
- dedicated strategy-layer fixture and oracle coverage
- honest live-provider smoke that executes a real provider fetch path outside default validation
- explicit point-in-time-universe prerequisites, but not point-in-time implementation

Acceptance criteria:

- operators can launch runs from a higher-level spec without creating CLI-owned truth
- replay and research artifacts fail explicitly on broken links or integrity drift
- strategy-layer behavior is covered by deterministic audit-grade fixtures or oracles beyond unit-only coverage
- `cargo xtask validate-live --provider tiingo` exercises a real fetch path while `cargo xtask validate` stays deterministic and network-free
- point-in-time universes remain deferred until a stable universe snapshot and historical-membership model exists

### M8: Snapshot Capture And Data Provenance

Purpose: make provider-backed market data portable and reusable before any point-in-time or broader live-data expansion is scheduled.

Scope:

- persisted market-data snapshot capture layered on top of the existing live provider boundary
- snapshot reopen and audit flows that stay in shared and data-layer ownership
- operator paths that can point runs or audits at stored snapshots instead of only fixture or ad hoc live-fetch inputs
- explicit point-in-time-universe prerequisites remain deferred until the snapshot and historical-membership model is honest enough

Acceptance criteria:

- live-fetched symbol history can be written and reopened through the documented snapshot ownership path without hidden provider refetches
- stored snapshots preserve provider identity, raw bars, corporate actions, and derived normalization inputs explicitly enough for audit
- operator-facing run or audit flows can consume stored snapshot paths without creating CLI-owned market-data truth
- `cargo xtask validate` remains deterministic and network-free while live snapshot capture stays explicitly optional
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## Immediate Next Work

The next implementation work should be Week 31 stored-snapshot inspect and audit flow:

1. add an audit-first reopen and inspect path for persisted snapshots on top of the Week 30 `snapshot.json` plus `daily/*.jsonl` and `actions/*.jsonl` layout
2. surface provider identity, requested window, raw-bar counts, corporate-action counts, and normalization-sensitive fields explicitly for offline inspection
3. keep snapshot reopen and audit behavior in shared/data-layer ownership instead of CLI-local reconstruction
4. keep normalization and resampling derived on reopen rather than persisting a second trusted normalized store
5. keep point-in-time universes explicitly deferred until the snapshot and historical-membership prerequisites are materially closer
6. use the Weeks 29-32 block in `docs/Roadmap.md` as the active checkpoint contract for the next sessions

## Out of Scope For Early Milestones

- intraday execution
- cloud execution
- broad Monte Carlo search
- Pine export
- point-in-time universes
- portfolio breadth before single-symbol trust

## Remaining Open Questions

- how far the Tiingo path should go beyond the first real `validate-live` fetch can be revisited after the post-M6 live-smoke pass lands
- whether derived analysis series ever need their own persisted cache can be revisited if post-M6 data volume or replay cost makes recomputation materially worse
- whether higher-timeframe strategy work needs weekly or monthly analysis open, high, and low semantics remains open
- whether the current semantic integrity fingerprints should later grow into stronger cryptographic digests or signed artifacts remains open
- whether the first strategy fixture and oracle harness needs richer ledger or reason fields once exit-oriented or less pass-through position-management cases land remains open
- whether the new operator-facing run spec should later grow beyond inline or referenced request sources can be revisited once snapshot-backed operator inputs exist
- whether strategy-component labels should remain standardized manifest parameters or eventually graduate into first-class manifest fields can be revisited if later portability or audit work exposes a concrete limitation in the current shared-manifest path
- when point-in-time universe work returns, the repo will still need an honest universe snapshot representation and historical-membership model before it is scheduled
