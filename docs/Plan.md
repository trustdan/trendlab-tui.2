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

## Immediate Next Work

The next implementation work should be Week 24 final M6 checkpoint and roadmap reset:

1. decide whether the current M6 gate is satisfied without point-in-time universes
2. record whether point-in-time universe support remains deferred into a later milestone or becomes the next planning horizon
3. update backlog and define the next roadmap era from the actual post-Week-23 state
4. use the re-baselined Weeks 21-24 block in `docs/Roadmap.md` as the active checkpoint contract for the next sessions

## Out of Scope For Early Milestones

- intraday execution
- cloud execution
- broad Monte Carlo search
- Pine export
- point-in-time universes
- portfolio breadth before single-symbol trust

## Remaining Open Questions

- how far the Tiingo path should go beyond the current smoke-plan/config-validation boundary can be revisited once M6 research-run data volume makes a stronger live-provider path worth the complexity
- whether derived analysis series ever need their own persisted cache can be revisited after the first M2 implementation pass
- whether the initial strategy interfaces need richer signal metadata or exit directives should be revisited during early M6 aggregation and leaderboard work
- whether strategy-layer scenarios want a dedicated fixture/oracle harness in addition to the current unit coverage should be revisited during early M6 research-work hardening
- whether shared research-report bundle links should normalize to relative paths or a stricter portability rule can be revisited once more than the CLI reopens them regularly
- whether the first shared `research.json` encoding needs richer compatibility metadata should be revisited now that the Week 23 reopen hardening pass is in place
- whether strategy-component labels should remain standardized manifest parameters or graduate into first-class manifest fields can be revisited after the Week 24 checkpoint fixes the next planning horizon
- whether the low-level JSON `RunRequest` input used by the CLI should become a higher-level operator-facing run spec can be revisited once the first research orchestration path is concrete
