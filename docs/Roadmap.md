# TrendLab Roadmap

This file is the master week-by-week delivery roadmap for the current TrendLab rebuild.

It translates the milestone plan into relative-week execution slices for a solo builder working roughly 10-15 hours per week with Codex/Cursor support.

Use this document with:

- `docs/Plan.md` for milestone definitions
- `docs/Workspace.md` for crate boundaries
- `docs/Artifacts.md` for persisted run artifacts
- `docs/MathContract.md`, `docs/BarSemantics.md`, and `docs/Invariants.md` for simulation truth
- `docs/Implement.md` for the session execution loop

## Planning Assumptions

- Weeks are relative, not calendar-bound.
- Builder capacity is 10-15 hours per week.
- Work is done by one human operator with agent assistance.
- Default validation is offline, deterministic, and network-free.
- Current roadmap coverage is M1 through M8.
- The first eight weeks are the most concrete.
- Later weeks are explicit but carry lower confidence when they depend on prior implementation outcomes.

## Confidence Model

- `High`: near-term work anchored to already-locked contracts and workspace boundaries.
- `Medium`: work that depends on earlier implementation results but is still structurally well defined.
- `Low`: later research and statistics work likely to change after implementation feedback.

Every week must carry one confidence label.

## Week Entry Schema

Every week in this roadmap follows the same schema:

- Objective
- Confidence
- Planning status
- Scope
- Deliverables/artifacts
- Validation/checks
- Exit criteria
- Non-goals
- Dependencies/blockers
- Risk note when confidence is not `High`

Planning status values:

- `Fixed`: week is expected to happen substantially as written.
- `Provisional`: week is likely, but details may tighten after prior weeks complete.
- `Contingent`: week should not start until a dependency or milestone gate is satisfied.

## Replan Policy

- Re-baseline after each milestone completion.
- Re-baseline immediately if a week slips by more than 50 percent.
- Re-baseline immediately if a locked contract doc changes materially.
- Keep completed weeks fixed; only reschedule future weeks.
- In weeks marked `Low`, revisit the next four weeks before starting implementation for that block.

## Agent Workflow Contract

- One implementation session equals one scoped slice of one active week.
- Update `docs/Status.md` at milestone-relevant checkpoints and at the end of any week that materially changes the roadmap state.
- Do not run overlapping implementation sessions that edit the same crate or contract area.
- Do not make normal validation depend on live provider access, secrets, or network connectivity.
- If a task changes math, bar semantics, artifacts, or workspace boundaries, update the corresponding contract docs in the same session.
- If a task would change `docs/MathContract.md`, `docs/BarSemantics.md`, or `docs/Invariants.md`, start with a plan-only pass and pause for a human checkpoint before implementation continues.
- Do not blend M1 truthful-kernel work with generalized strategy composition.

## Milestone Gates

These are the "done means done" gates that each milestone must satisfy before the roadmap moves on.

### M1 Gate: Minimal Truthful Kernel

- Workspace exists with `xtask`, `trendlab-core`, `trendlab-artifact`, `trendlab-testkit`, and initial test support.
- One-symbol daily-bar simulation runs from fixtures only.
- Queued market-entry flow and fixed protective stop both work under the documented bar semantics.
- Event ledger and replay bundle are persisted through shared artifact schemas.
- At least one hand-authored oracle scenario exists for the first truthful path.
- `cargo xtask validate` is wired and network-free.
- Golden tests can diff expected versus actual ledger output.

### M2 Gate: Canonical Market Data Layer

- Provider-native types stay outside `trendlab-core`.
- Canonical market types from `trendlab-core` are populated by `trendlab-data`.
- Raw daily bars, corporate actions, and derived analysis series are represented in the data layer.
- Weekly and monthly resampling is implemented and tested.
- Live provider support remains outside normal validation, with any smoke checks exposed through a separate command such as `cargo xtask validate-live`.

### M3 Gate: Componentized Strategy System

- Signals, position managers, execution models, and filters compose independently.
- Close-confirmed breakout and stop-entry breakout are separate execution paths.
- Position managers only use allowed state.
- Execution models do not mutate signal logic.
- New strategy composition preserves replayability and auditability.

### M4 Gate: CLI

- CLI can run, explain, diff, and audit data using shared artifacts.
- CLI output can expose ledger reasoning, warnings, and metric inputs.
- CLI does not become the owner of artifact schemas.

### M5 Gate: TUI Shell

- TUI shell supports keyboard-first navigation and audit-first inspection.
- Results, charts, help, and audit views exist as a coherent shell.
- TUI reopens shared run artifacts without parallel parsing rules.
- Audit views preserve run provenance and warnings.

### M6 Gate: Research Extensions

- Cross-symbol aggregation, walk-forward, bootstrap confidence, and separated leaderboards operate on the stable core.
- Additional research features do not obscure run provenance.
- Point-in-time universes are added only if the prerequisite data model is stable enough to support them honestly.

### M7 Gate: Trust Hardening And Operator Surface

- Operator-facing run specs map cleanly onto shared and core request types without creating CLI-owned truth.
- Replay and research artifacts fail explicitly on broken links or integrity drift.
- Strategy-layer behavior is covered by deterministic fixture or oracle scenarios in addition to unit-only coverage.
- Live-provider smoke exercises a real provider fetch path while remaining outside normal validation.
- Point-in-time universes remain deferred until the universe snapshot and historical-membership model are honest enough to support them.

### M8 Gate: Snapshot Capture And Data Provenance

- Live-fetched symbol history can be written and reopened through the documented snapshot ownership path without hidden provider refetches.
- Snapshot reopen and audit surfaces preserve provider identity, raw bars, corporate actions, and derived normalization inputs explicitly enough for audit.
- Operator-facing run or audit flows can point at stored snapshots without inventing CLI-owned market-data truth.
- Optional live snapshot capture remains outside `cargo xtask validate`, and default validation stays deterministic and network-free.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

## Week 0: Planning Closure

### Week 0

- Objective: close the remaining planning gaps so M1 can start without unresolved design decisions.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - choose the on-disk encoding for run artifacts
  - choose the cache format for normalized market data
  - freeze the exact M1 crate set and initial interfaces
  - freeze the first fixture strategy
  - freeze the first three M1 golden scenarios
  - freeze the initial `cargo xtask validate` command breakdown
  - freeze the weekly operating loop for implementation sessions
- Deliverables/artifacts:
  - `docs/Roadmap.md`
  - final artifact-encoding decision recorded in `docs/Artifacts.md` or `docs/Status.md`
  - final cache-format decision recorded in `docs/Workspace.md` or `docs/Status.md`
  - clarified M1 interface notes recorded in `docs/Status.md`
  - first three M1 golden scenarios recorded in `docs/Status.md`
- Validation/checks:
  - roadmap references existing contract docs instead of redefining them
  - Week 1 has no unresolved technical decisions left to invent
- Exit criteria:
  - artifact encoding decision chosen
  - normalized cache format chosen
  - first fixture strategy chosen
  - first three M1 golden scenarios chosen
  - `cargo xtask validate` breakdown chosen
  - M1 crate/interface set fully frozen
- Non-goals:
  - creating Cargo crates
  - writing Rust implementation
  - integrating Tiingo
- Dependencies/blockers:
  - current planning docs remain internally consistent

## Weeks 1-8: High-Detail Build Plan

### Week 1

- Objective: scaffold the workspace and create the enforcement points for truthful-kernel work.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - create workspace root `Cargo.toml`
  - create `xtask`
  - create `trendlab-core`
  - create `trendlab-artifact`
  - create `trendlab-testkit`
  - create initial crate wiring and dependency boundaries
  - wire placeholder `cargo xtask validate`
- Deliverables/artifacts:
  - compilable Rust workspace scaffold
  - initial `xtask validate` command
  - empty or placeholder crate skeletons with boundaries matching `docs/Workspace.md`
- Validation/checks:
  - `cargo check --workspace`
  - `cargo xtask validate`
  - verify `trendlab-core` has no dependency on `trendlab-data`, CLI, or TUI crates
  - verify `trendlab-testkit` depends only on `trendlab-core` and `trendlab-artifact`
- Exit criteria:
  - workspace builds
  - validation entrypoint exists and is network-free
  - crate boundaries match the documented plan
- Non-goals:
  - domain logic
  - fixture ingestion
  - simulation behavior
- Dependencies/blockers:
  - Week 0 decisions on crate set and validation breakdown

### Week 2

- Objective: define the core domain surface and first deterministic fixture path.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - define canonical daily-bar and market-model types in `trendlab-core`
  - define run-manifest, ledger, and replay-bundle schemas in `trendlab-artifact`
  - add the first fixture storage path
  - define the first ledger row shape consistent with the contracts
  - define at least one hand-authored oracle scenario independent of the core implementation
  - add testkit support if required for fixture loading and assertions
- Deliverables/artifacts:
  - initial domain types
  - initial artifact types and schema version field
  - first static fixture data set
  - first golden-test harness shell
  - first hand-authored oracle or truth-table scenario for the M1 reference flow
- Validation/checks:
  - unit tests for basic type construction or parsing
  - golden harness can load fixture input and expected output paths, including one hand-authored oracle scenario
  - artifact schemas do not leak into CLI or TUI-only concerns
- Exit criteria:
  - fixture data can be loaded deterministically
  - artifact bundle shape exists
  - one oracle scenario exists independently of the core implementation
  - ledger schema is concrete enough for Week 3 implementation
- Non-goals:
  - order execution
  - provider integration
  - generalized strategy components
- Dependencies/blockers:
  - Week 1 workspace skeleton

### Week 3

- Objective: implement the queued market-entry path and accounting baseline.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - implement initial simulation state
  - implement queued market-entry intent
  - fill at next open using raw tradable prices
  - implement cash, position, and equity transitions for this path
  - emit ledger rows for each bar
- Deliverables/artifacts:
  - first runnable truthful-kernel slice
  - ledger rows showing pre-entry, entry, hold, and post-trade states
  - tests covering simple entry flows
- Validation/checks:
  - replay of the same fixture produces the same ledger
  - equity and cash reconcile on every row
  - no lookahead across bar boundaries
- Exit criteria:
  - market-entry flow works from a fixture-driven run
  - ledger captures the full lifecycle for this path
- Non-goals:
  - protective stop logic
  - gap-policy variants beyond the chosen default
  - multiple positions or multiple symbols
- Dependencies/blockers:
  - Week 2 domain and artifact shapes

### Week 4

- Objective: implement the fixed protective stop and explicit gap-policy handling.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - implement fixed protective stop behavior
  - encode the chosen default gap policy
  - surface ambiguity or warnings where required by the contracts
  - extend ledger reason codes for stop behavior
- Deliverables/artifacts:
  - protective-stop execution path
  - gap-policy representation in run artifacts
  - fixture cases for stop hits, gap-through scenarios, and non-triggered stops
- Validation/checks:
  - stop handling obeys `docs/BarSemantics.md`
  - same-bar stop tightening does not occur
  - warnings are captured where the contract requires them
- Exit criteria:
  - fixed protective stop works correctly under daily-bar semantics
  - gap behavior is documented and represented in artifacts
- Non-goals:
  - trailing stops
  - breakout entry models
  - provider-backed data
- Dependencies/blockers:
  - Week 3 order lifecycle and ledger output

### Week 5

- Objective: make truthful runs portable and testable through persisted artifacts.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - finish replay-bundle persistence
  - serialize and deserialize manifest and ledger artifacts
  - finalize first golden ledger tests
  - add a minimal replay-bundle inspection utility in `xtask`
  - add schema-version handling for persisted artifacts
- Deliverables/artifacts:
  - persisted run bundle format
  - golden ledger fixtures
  - replay-from-artifact path
  - minimal inspect-ledger or replay-printer utility
- Validation/checks:
  - serialized run can be reopened and replayed without hidden state
  - persisted run can be reopened by the inspection utility without hidden state
  - golden tests diff expected and actual ledger output
  - schema version is explicit
- Exit criteria:
  - replay bundle is portable within the repo
  - artifact load/save works for the M1 kernel path
  - truthful runs can be visually inspected before the CLI milestone
- Non-goals:
  - CLI UX
  - live provider ingestion
  - strategy composition
- Dependencies/blockers:
  - Week 2 artifact schemas
  - Weeks 3-4 simulation behavior

### Week 6

- Objective: harden M1 and close the milestone gate cleanly.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - tighten validation
  - clean up test ergonomics
  - close obvious naming, boundary, and reconciliation issues
  - confirm the M1 gate from this roadmap and `docs/Plan.md`
- Deliverables/artifacts:
  - stable M1 kernel
  - M1 gate checklist marked complete in `docs/Status.md`
  - cleaned validation path
- Validation/checks:
  - `cargo xtask validate`
  - full M1 unit and integration pass
  - confirm all M1 acceptance criteria from `docs/Plan.md`
- Exit criteria:
  - M1 gate is satisfied
  - no known contract violations remain in the truthful kernel
- Non-goals:
  - data providers
  - reusable strategy architecture
  - CLI or TUI work
- Dependencies/blockers:
  - Weeks 1-5 complete

### Week 7

- Objective: begin the canonical market-data layer without weakening the core boundary.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - create `trendlab-data`
  - define provider-facing raw types
  - implement normalization into `trendlab-core` market types
  - add offline fixture-driven data-layer tests
  - start raw-bar and corporate-action storage structure
- Deliverables/artifacts:
  - `trendlab-data` crate
  - normalized market-data pipeline from stored inputs to core types
  - offline tests for normalization behavior
- Validation/checks:
  - `trendlab-core` remains provider-agnostic
  - fixture-backed normalization tests pass
  - no network access required in normal validation
- Exit criteria:
  - data layer exists and produces core market types from offline inputs
- Non-goals:
  - live Tiingo integration
  - weekly/monthly resampling
  - strategy composition
- Dependencies/blockers:
  - M1 gate complete

### Week 8

- Objective: finish M2 essentials around resampling and corporate actions.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - implement weekly and monthly resampling
  - implement corporate-action handling in the data layer
  - ensure derived analysis series rules line up with the contracts
  - define or wire a separate live-provider smoke lane that stays outside normal validation
  - complete M2 validation pass
- Deliverables/artifacts:
  - tested resampler
  - corporate-action ingestion and normalization path
  - separate live-provider smoke command or checklist kept outside normal validation
  - M2 gate checklist in `docs/Status.md`
- Validation/checks:
  - resampling tests on deterministic fixtures
  - corporate-action cases for split and dividend handling
  - confirm any live smoke path is excluded from `cargo xtask validate`
  - confirm M2 acceptance criteria from `docs/Plan.md`
- Exit criteria:
  - M2 gate is satisfied
  - data layer can supply canonical bars and analysis series without provider leakage
- Non-goals:
  - live API integration in normal validation
  - CLI or TUI UX
- Dependencies/blockers:
  - Week 7 data layer

## Weeks 9-15: Medium-Detail Build Plan

### Week 9

- Objective: establish the strategy-composition framework without overextending feature breadth.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - define strategy-layer interfaces for signals, position managers, execution models, and filters
  - preserve replay and artifact compatibility
  - define test seams for compositional behavior
- Deliverables/artifacts:
  - core strategy interfaces
  - initial composition tests
- Validation/checks:
  - strategy interfaces do not break M1/M2 flows
  - execution does not mutate signal logic by design
- Exit criteria:
  - strategy layers compose structurally without implementation shortcuts
- Non-goals:
  - full breakout implementations
  - CLI UX
- Dependencies/blockers:
  - M2 gate complete
- Risk note:
  - exact trait and ownership shapes may need adjustment after the first integration pass.

### Week 10

- Objective: implement the close-confirmed breakout family.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - implement close-confirmed breakout signal generation
  - connect it to next-open execution
  - add filters or position-manager hooks only if required by the current contracts
- Deliverables/artifacts:
  - close-confirmed breakout implementation
  - fixtures and tests for entry/hold/exit behavior
- Validation/checks:
  - breakout behavior remains auditably replayable
  - signal generation uses allowed data only
- Exit criteria:
  - close-confirmed breakout family works within the compositional model
- Non-goals:
  - stop-entry breakout
  - CLI or TUI surfaces
- Dependencies/blockers:
  - Week 9 interfaces
- Risk note:
  - if Week 9 exposes interface weaknesses, this week may partly shift into interface cleanup.

### Week 11

- Objective: implement the stop-entry breakout family and close M3.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - implement stop-entry breakout path
  - test separation between signal logic and execution logic
  - confirm position-manager constraints
  - close the M3 gate
- Deliverables/artifacts:
  - stop-entry breakout implementation
  - M3 gate checklist in `docs/Status.md`
- Validation/checks:
  - stop-entry path stays distinct from close-confirmed breakout
  - all M3 acceptance criteria from `docs/Plan.md` pass
- Exit criteria:
  - M3 gate is satisfied
- Non-goals:
  - full search/leaderboard work
  - CLI polish
- Dependencies/blockers:
  - Weeks 9-10
- Risk note:
  - this week is vulnerable to slippage if the first strategy-composition design proves too rigid.

### Week 12

- Objective: establish the CLI surface around shared artifacts and kernel operations.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - create `trendlab-cli`
  - implement `run` and `explain`
  - load shared artifacts through `trendlab-artifact`
  - keep command behavior aligned with the audit-first model
- Deliverables/artifacts:
  - CLI crate
  - working `run` and `explain` commands
- Validation/checks:
  - CLI reuses artifact schemas instead of redefining them
  - CLI commands can run and explain deterministic artifact-backed flows
- Exit criteria:
  - CLI can launch and inspect runs using the existing core/data stack
- Non-goals:
  - full diff and data-audit command set
  - TUI work
- Dependencies/blockers:
  - M3 gate complete
- Risk note:
  - command ergonomics may change after the first real operator usage.

### Week 13

- Objective: finish the core CLI command set and close M4.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - implement `diff`
  - implement `audit data`
  - tighten command UX and error handling
  - close the M4 gate
- Deliverables/artifacts:
  - full planned CLI command set for M4
  - M4 gate checklist in `docs/Status.md`
- Validation/checks:
  - CLI can reopen artifacts, surface warnings, and compare results
  - M4 acceptance criteria from `docs/Plan.md` pass
- Exit criteria:
  - M4 gate is satisfied
- Non-goals:
  - TUI shell
  - research extensions
- Dependencies/blockers:
  - Week 12 CLI foundation
- Risk note:
  - diff and audit UX may reveal artifact fields that need expansion before TUI work begins.

### Week 14

- Objective: create the ratatui shell and restore the keyboard-first workflow.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - create `trendlab-tui`
  - implement app shell, focus model, and navigation
  - add results list and help panel
  - keep artifact loading shared with the CLI
- Deliverables/artifacts:
  - ratatui shell
  - keyboard-first navigation foundation
- Validation/checks:
  - TUI compiles and loads shared artifacts
  - navigation respects the audit-first UI philosophy
- Exit criteria:
  - a minimal shell exists without compromising kernel boundaries
- Non-goals:
  - chart polish
  - full audit panel depth
- Dependencies/blockers:
  - M4 gate complete
- Risk note:
  - navigation and app-state design may need iteration after the first shell pass.

### Week 15

- Objective: add audit-first inspection surfaces and close M5.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add chart view
  - add audit panel
  - ensure warnings, ledger reasoning, and provenance are visible
  - close the M5 gate
- Deliverables/artifacts:
  - chart view
  - audit panel
  - M5 gate checklist in `docs/Status.md`
- Validation/checks:
  - TUI can inspect a run without losing auditability
  - M5 acceptance criteria from `docs/Plan.md` pass
- Exit criteria:
  - M5 gate is satisfied
- Non-goals:
  - research leaderboards
  - advanced chart overlays beyond what the milestone requires
- Dependencies/blockers:
  - Week 14 TUI shell
- Risk note:
  - chart and audit requirements may uncover artifact fields or CLI behaviors worth normalizing before M6.

## Weeks 16-19: Re-Baselined Research Plan

This is the first post-M5 research block. The shell is now audit-capable, so the next four weeks are re-baselined around research orchestration that preserves drill-down into the existing single-symbol replay bundles.

### Week 16

- Objective: establish the first deterministic cross-symbol aggregation path without obscuring single-symbol run truth.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the curated-modern multi-symbol aggregation contract for the first research runs
  - preserve per-symbol provenance and drill-down back to the existing replay bundles
  - define the first aggregate summary fields and fixture-backed expectations
  - implement the first aggregate result reduction on top of deterministic single-symbol runs
- Deliverables/artifacts:
  - aggregation model
  - deterministic multi-symbol fixture set
  - tests for aggregate result reduction and drill-down integrity
- Validation/checks:
  - aggregate outputs still preserve drill-down to per-symbol run provenance
  - aggregate summaries do not replace or mutate the underlying replay bundles
  - `cargo xtask validate`
- Exit criteria:
  - the first multi-symbol aggregation path exists and is auditable
  - aggregation rules are stable enough for later research work
- Non-goals:
  - walk-forward validation
  - bootstrap confidence
  - point-in-time universes
  - leaderboard UX
- Dependencies/blockers:
  - M5 gate complete
- Risk note:
  - aggregate research summaries may pressure artifact ownership, so this week should stay on top of the existing replay-bundle truth model instead of inventing opaque aggregate artifacts.

### Week 17

- Objective: implement deterministic walk-forward orchestration on top of the first aggregation path.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - define walk-forward windowing rules
  - implement deterministic split generation for curated multi-symbol research runs
  - surface split metadata and child-run links in research summaries where needed
- Deliverables/artifacts:
  - walk-forward execution path
  - walk-forward test coverage
- Validation/checks:
  - repeated runs produce identical train/test partitions
  - split summaries preserve drill-down into symbol-level run provenance
  - `cargo xtask validate`
- Exit criteria:
  - walk-forward validation works as a reproducible research mode
- Non-goals:
  - bootstrap confidence
  - universe history
- Dependencies/blockers:
  - Week 16 aggregation model
- Risk note:
  - data-volume and artifact-volume growth may change how this should be represented.

### Week 18

- Objective: add seeded bootstrap confidence without weakening interpretability.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - implement bootstrap sampling path
  - define reproducible randomness and seed handling over research-run outputs
  - surface confidence outputs without obscuring baseline aggregate and walk-forward truth
- Deliverables/artifacts:
  - bootstrap confidence module
  - artifact/reporting updates if needed
- Validation/checks:
  - seeded runs are reproducible
  - baseline deterministic runs remain unchanged
  - bootstrap outputs still drill back into the baseline research run they summarize
  - `cargo xtask validate`
- Exit criteria:
  - bootstrap confidence is available and reproducible
- Non-goals:
  - parameter sweeps
  - execution-noise Monte Carlo
- Dependencies/blockers:
  - Week 17 walk-forward support
- Risk note:
  - once randomness enters, artifact and reporting complexity may increase materially.

### Week 19

- Objective: separate leaderboards by layer so ideas remain comparable across the new research stack.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - define leaderboard views for signals, position managers, execution models, and combined systems
  - preserve auditability back to aggregate research runs and individual replay bundles
  - tighten component-attribution expectations where the current strategy metadata is too thin
- Deliverables/artifacts:
  - separated leaderboard model
  - tests ensuring attribution remains correct
- Validation/checks:
  - ranking outputs can still be traced back to specific componentized runs
  - leaderboard rows can still drill down through research summaries into per-symbol replay truth
  - `cargo xtask validate`
- Exit criteria:
  - layer-specific ranking works without collapsing ideas back into opaque bundles
- Non-goals:
  - UI-heavy leaderboard polish
  - point-in-time universes unless already unblocked
- Dependencies/blockers:
  - Weeks 16-18
- Risk note:
  - this work depends heavily on how componentized runs and aggregation artifacts feel in practice.

## Weeks 20-24: Post-Checkpoint Research Plan

The Week 20 checkpoint concludes that point-in-time universe support is not honestly ready to start. The current research stack works, but aggregate, walk-forward, bootstrap, and leaderboard summaries still live as CLI-local report shapes layered over single-symbol replay bundles. The next block therefore shifts toward shared research-report ownership, reopenability, and provenance hardening before point-in-time universe work is scheduled again.

### Week 20

- Objective: post-block M6 checkpoint.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - review Weeks 16-19 outcomes
  - rebalance the next four weeks
  - decide whether point-in-time universe support is genuinely ready
  - decide whether any M6 items should be deferred into backlog
- Deliverables/artifacts:
  - updated future-week roadmap
  - explicit decision on point-in-time readiness
- Validation/checks:
  - roadmap remains honest about uncertainty and implementation feedback
- Exit criteria:
  - Weeks 21-24 are re-baselined from actual progress
- Non-goals:
  - major implementation breadth
- Dependencies/blockers:
  - Weeks 16-19 complete or intentionally re-scoped
- Risk note:
  - this checkpoint exists specifically because the prior four weeks are expected to surface new information.

### Week 21

- Objective: move research-report ownership out of CLI-local formatting without weakening drill-down to replay truth.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the shared ownership location for research-run report shapes
  - persist or otherwise reopen aggregate, walk-forward, bootstrap, and leaderboard summaries through one shared path
  - keep research summaries thin and auditable by linking back to existing replay bundles instead of replacing them
- Deliverables/artifacts:
  - shared research-report model
  - first reopenable research-summary path
  - tests covering shared reopen and drill-down integrity
- Validation/checks:
  - research summaries can be reopened without CLI-local reconstruction rules
  - child replay-bundle links remain explicit and deterministic
  - `cargo xtask validate`
- Exit criteria:
  - research summaries no longer exist only as transient CLI text output
- Non-goals:
  - point-in-time universe membership
  - new statistics beyond the existing Weeks 16-19 stack
- Dependencies/blockers:
  - Week 20 checkpoint
- Risk note:
  - this week may pressure crate ownership, because shared research summaries may belong in `trendlab-artifact` or may justify a dedicated research crate later.

### Week 22

- Objective: normalize the research execution and reopen flow around the shared report path.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - unify aggregate, walk-forward, bootstrap, and leaderboard command flows around the shared report shape
  - normalize seed, component-attribution, and child-run-link handling across research modes
  - add a coherent research explain or inspect path where needed
- Deliverables/artifacts:
  - coherent research execution/reopen flow
  - normalized research reporting surface
- Validation/checks:
  - each research mode reopens through the shared path
  - no research report loses drill-down to replay truth
  - `cargo xtask validate`
- Exit criteria:
  - Weeks 16-19 research features operate as one coherent report lifecycle instead of parallel ad hoc outputs
- Non-goals:
  - point-in-time universe membership
  - new ranking families or fresh statistical breadth
- Dependencies/blockers:
  - Week 21 shared report ownership
- Risk note:
  - if shared research reporting exposes missing provenance fields, some manifest or artifact tightening may be needed before this flow feels stable.

### Week 23

- Objective: harden provenance, attribution, and compatibility across the normalized research stack.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten handling for unlabeled or partially labeled bundles used by leaderboard and research reopen flows
  - finalize warnings and rejection paths for research inputs that cannot be ranked or reopened honestly
  - close remaining integration gaps across aggregate, walk-forward, bootstrap, and leaderboard reporting
- Deliverables/artifacts:
  - hardened research provenance behavior
  - expanded regression coverage for research compatibility and failure paths
- Validation/checks:
  - malformed or under-attributed research inputs fail explicitly
  - reopened research outputs remain deterministic and provenance-safe
  - `cargo xtask validate`
- Exit criteria:
  - research inputs either rank and reopen honestly or reject with explicit reasons
- Non-goals:
  - point-in-time universe membership
  - new UI-heavy research polish
- Dependencies/blockers:
  - Weeks 21-22
- Risk note:
  - older bundles may need one more explicit compatibility posture if their manifests lack enough attribution for the hardened research path.

### Week 24

- Objective: final M6 checkpoint and roadmap reset after the research ownership and hardening pass.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - decide whether the current M6 gate is satisfied without point-in-time universes
  - record whether point-in-time universe support remains deferred into a later milestone or becomes the next planning horizon
  - update backlog and define the next roadmap era from the actual post-Week-23 state
- Deliverables/artifacts:
  - M6 gate decision in `docs/Status.md`
  - updated backlog and next-era planning note
- Validation/checks:
  - confirm the completed research stack still respects the trust model and preserves drill-down to replay truth
- Exit criteria:
  - the next planning horizon is explicit, with point-in-time universes either honestly deferred or newly scheduled from a stable prerequisite model
- Non-goals:
  - forcing point-in-time universes into the roadmap without the needed data model
- Dependencies/blockers:
  - Weeks 20-23
- Risk note:
  - the correct outcome may still be a narrower but honest M6 close, with point-in-time universes explicitly pushed beyond the current milestone block.

## Weeks 25-28: Post-M6 Trust-Hardening Plan

Week 24 concludes that M6 is honestly complete without point-in-time universes. The current research stack already satisfies the milestone gate because aggregate, walk-forward, bootstrap, and separated leaderboard reporting now reopen through shared artifact ownership and preserve drill-down back to replay truth. The next block therefore focuses on remaining operator, portability, and validation gaps before any new market-model breadth is scheduled.

### Week 25

- Objective: add a higher-level operator-facing run spec without moving truth ownership out of shared and core types.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the first operator-facing run-spec shape on top of the existing core request surface
  - decide whether strategy-component attribution remains standardized manifest parameters or becomes first-class manifest fields
  - keep universe-truth and cost or gap provenance explicit in the resulting run path
- Deliverables/artifacts:
  - operator-facing run-spec model
  - mapping path into the existing core and shared run-request surface
  - tests covering manifest and provenance preservation
- Validation/checks:
  - run-spec loading does not create CLI-owned truth about artifacts or strategy attribution
  - the same operator spec produces the same manifest and replay bundle deterministically
  - `cargo xtask validate`
- Exit criteria:
  - the CLI no longer relies on raw serialized core `RunRequest` inputs as the only operator-facing run surface
- Non-goals:
  - point-in-time universe membership
  - new strategy families
- Dependencies/blockers:
  - Week 24 checkpoint
- Risk note:
  - run-spec ergonomics may pressure crate ownership if shared manifest and request concerns are not kept cleanly separated.

### Week 26

- Objective: harden replay-bundle and research-report portability plus integrity signaling.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - decide how shared artifact links normalize across move, copy, and reopen flows
  - add explicit integrity metadata where path validation alone is not enough
  - tighten older-bundle compatibility posture where the current metadata is too thin
- Deliverables/artifacts:
  - clarified portability rules
  - replay and research bundle integrity metadata or checks
  - tests covering stale-path and stale-content rejection
- Validation/checks:
  - moved or copied artifacts either reopen honestly or fail with explicit reasons
  - integrity drift is surfaced without inventing parallel CLI or TUI parsing rules
  - `cargo xtask validate`
- Exit criteria:
  - shared artifact reopen behavior is portable enough to support future CLI and TUI workflows without silent trust loss
- Non-goals:
  - new research statistics
  - point-in-time universe membership
- Dependencies/blockers:
  - Week 25 operator-facing spec boundary
- Risk note:
  - portability decisions may force an explicit compatibility story for already-written bundles and reports.

### Week 27

- Objective: add audit-grade strategy fixtures and oracles so the compositional system is not validated only by unit tests.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - define the first deterministic fixture-backed strategy scenarios spanning signals, filters, execution blocking, and position management
  - add hand-authored oracle coverage where unit tests are not enough to explain the expected ledger path
  - tighten explain surfaces only if the new scenarios expose missing reason metadata
- Deliverables/artifacts:
  - strategy-layer fixtures
  - oracle-backed strategy regressions
  - any narrowly-scoped explainability additions required by the new fixtures
- Validation/checks:
  - strategy-layer scenarios replay through persisted artifacts without hidden state
  - blocked-trade and execution-path reasoning remains explicit
  - `cargo xtask validate`
- Exit criteria:
  - strategy composition is covered by deterministic audit-grade regression paths in addition to unit-only tests
- Non-goals:
  - new entry or exit families
  - TUI-polish-only work
- Dependencies/blockers:
  - Weeks 25-26 if new manifest or artifact surfaces are needed
- Risk note:
  - strategy fixtures may expose missing ledger fields or attribution details that were invisible in the current unit-test-only path.

### Week 28

- Objective: make the live-provider smoke lane execute a real provider fetch and re-check the next planning horizon.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - turn the current Tiingo smoke-plan and config-validation lane into a minimal real-fetch smoke path
  - keep networked validation explicitly outside `cargo xtask validate`
  - re-check whether point-in-time prerequisites are any closer or remain backlog-only
- Deliverables/artifacts:
  - real-fetch `validate-live` smoke path
  - updated future-week roadmap checkpoint
- Validation/checks:
  - `cargo xtask validate-live --provider tiingo` fails cleanly without `TIINGO_API_TOKEN` and verifies a real provider response when configured
  - `cargo xtask validate` remains deterministic and network-free
- Exit criteria:
  - live-provider smoke verifies the actual provider boundary honestly
  - the next planning horizon is explicit after the first post-M6 hardening block
- Non-goals:
  - point-in-time universe implementation
  - default validation that depends on network access
- Dependencies/blockers:
  - Weeks 25-27
- Risk note:
  - a real provider smoke path introduces external API variability, so the scope must stay narrow and explicitly optional.

## Weeks 29-32: Post-M7 Snapshot-Capture Plan

Week 28 closes M7 honestly, but the repo still lacks a persisted live-data snapshot path that operators can reopen and audit without going back to the provider. Week 29 freezes the first slice around `trendlab-data` ownership, an initial `xtask` capture entrypoint, and an inspectable `snapshot.json` plus `daily/*.jsonl` and `actions/*.jsonl` layout that persists stored raw bars and corporate actions while recomputing normalization on reopen. The next block therefore shifts from proving the live boundary exists to making fetched data portable and reusable while keeping point-in-time universes explicitly deferred.

### Week 29

- Objective: checkpoint the post-M7 data-capture direction and freeze the first persisted snapshot slice.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - review the Week 28 real-fetch smoke outcome and confirm the next ownership boundary belongs in `trendlab-data`
  - decide the first persisted snapshot write surface and where operator entrypoints should live
  - re-confirm that point-in-time universes remain blocked on the same prerequisite model
- Deliverables/artifacts:
  - updated snapshot-capture checkpoint in `docs/Plan.md`, `docs/Roadmap.md`, and `docs/Status.md`
  - frozen first-slice contract for persisted snapshot capture and reopen flow
- Validation/checks:
  - the next live-data step stays outside `cargo xtask validate`
  - snapshot ownership remains in shared/data-layer code instead of CLI-local truth
- Exit criteria:
  - the first persisted snapshot slice is concrete enough to implement without reopening the post-M7 planning questions
- Non-goals:
  - point-in-time universe membership
  - broad live-data orchestration beyond one truthful snapshot path
- Dependencies/blockers:
  - Week 28 complete

### Week 30

- Objective: persist the first live-fetched symbol-history snapshot through the documented ownership path.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add the first snapshot write and load helpers in `trendlab-data` on top of the existing Tiingo live fetch boundary
  - persist provider identity, requested date window, stored raw bars, and stored corporate actions in the frozen `snapshot.json` plus `daily/*.jsonl` and `actions/*.jsonl` layout
  - expose a narrow `cargo xtask` capture entrypoint for optional live snapshot capture
- Deliverables/artifacts:
  - persisted live-snapshot write path
  - deterministic tests covering snapshot write and reopen compatibility on synthetic or fixture-backed stored symbol data
- Validation/checks:
  - captured snapshots reopen without a second provider call
  - persisted snapshot data preserves the same provider and action truth used during capture
  - `cargo xtask validate`
- Exit criteria:
  - one truthful live-snapshot capture path exists and can be reopened offline
- Non-goals:
  - point-in-time universes
  - multi-provider breadth
- Dependencies/blockers:
  - Week 29 checkpoint
- Risk note:
  - the documented cache layout may pressure dependency or schema choices once the first persisted write path is real instead of aspirational.

### Week 31

- Objective: make stored snapshots inspectable and auditable without refetching live data.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add reopen helpers and an audit-first inspect path for persisted snapshots on top of the Week 30 snapshot format
  - surface provider identity, action counts, and normalization-sensitive fields explicitly
  - keep reopen behavior shared or data-layer-owned instead of duplicating snapshot parsing in the CLI or TUI
- Deliverables/artifacts:
  - snapshot reopen path
  - `xtask` or data-layer-backed inspect coverage for stored snapshots
- Validation/checks:
  - snapshot inspection works offline after capture
  - audit output still traces back to stored raw bars and corporate actions explicitly
  - `cargo xtask validate`
- Exit criteria:
  - stored snapshots can be reopened and inspected honestly without a provider round-trip
- Non-goals:
  - point-in-time universes
  - UI-heavy live-data polish
- Dependencies/blockers:
  - Week 30 snapshot capture
- Risk note:
  - snapshot inspection may expose missing provenance fields or an inconvenient on-disk layout that needs one more normalization pass.

### Week 32

- Objective: connect the first stored-snapshot path back into operator workflows and close the M8 checkpoint honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - allow an operator-facing CLI run or audit path to target stored snapshots explicitly where that improves honesty over ad hoc live fetches
  - confirm the stored-snapshot flow still keeps provider-native types out of `trendlab-core`
  - decide whether M8 is complete without point-in-time universes and record the next horizon
- Deliverables/artifacts:
  - snapshot-backed operator flow
  - M8 checkpoint decision and next-horizon note
- Validation/checks:
  - stored snapshots can feed operator workflows without creating CLI-owned market-data truth
  - default validation remains deterministic and network-free
  - point-in-time universes are still deferred unless the prerequisite model materially changes
- Exit criteria:
  - M8 closes honestly with a reusable snapshot path, or the remaining work is explicitly re-baselined
- Non-goals:
  - point-in-time universe implementation
  - broad live-data health dashboards
- Dependencies/blockers:
  - Weeks 29-31
- Risk note:
  - the right outcome may still be a narrower M8 close if snapshot-backed operator flows expose more contract work than expected.

## Backlog After the Roadmap

These items are intentionally outside the current planned sequence unless a later re-baseline pulls them forward:

- intraday execution modeling
- Pine export and parity vectors
- broader Monte Carlo and execution-noise search
- cloud execution workflows
- portfolio breadth beyond what is needed for the research layer
- point-in-time universe implementation before an honest universe snapshot representation and historical-membership model exist
- live data-health and provenance badges beyond the minimum audit surfaces
- advanced chart overlays beyond the first audit-capable TUI shell

## Review Checklist

Use this checklist whenever the roadmap is revised:

- every milestone from M1 through M8 has weekly slices
- no week asks the implementer to invent acceptance criteria
- Week 0 owns the current open design decisions and initial golden-scenario choices
- every week includes explicit validation and exit criteria
- later weeks show uncertainty honestly
- the roadmap points back to the contract docs instead of silently replacing them
