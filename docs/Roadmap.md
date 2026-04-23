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
- Current roadmap coverage is M1 through M20, with M14 through M18 committed and M19 through M20 explicitly deferred follow-on blocks.
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

### M9 Gate: Snapshot-Backed Operator Runs

- Operators can launch replayable runs from stored snapshots without provider refetches or copied low-level bar payloads.
- Snapshot-backed operator flows keep provider-native types, filesystem details, and snapshot parsing rules out of `trendlab-core`.
- Replay manifests and explain surfaces disclose the stored snapshot source, selected symbol, selected date slice, and resulting run provenance explicitly enough for audit.
- Default validation remains deterministic and network-free by relying on stored snapshots or fixtures.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

### M10 Gate: Shared Operator Orchestration

- CLI and TUI can call the same shared operator library for run execution.
- `trendlab-operator` reuses `trendlab-data` snapshot resolution and `trendlab-artifact` replay writes instead of re-owning those boundaries.
- TUI run execution does not require CLI subprocesses.
- Shared operator outputs remain deterministic and provenance-safe.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

### M11 Gate: TUI Run Workspace

- The TUI can browse and select a stored snapshot.
- The TUI can configure the current trusted snapshot-backed run inputs.
- The TUI can launch a run and transition directly into result inspection.
- Validation and error messages remain explicit and audit-safe.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

### M12 Gate: TUI Operator Lab v1

- The TUI can launch new runs and reopen prior runs from one operator shell.
- The existing inspection shell remains the result viewer after TUI-launched runs.
- The TUI does not introduce duplicate run logic, snapshot parsing, or artifact parsing.
- Default validation remains deterministic and network-free.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

### M13 Gate: TUI Research Audit Surface

- The TUI can reopen saved shared research-report bundles through `trendlab-artifact`.
- The TUI can inspect report-kind summaries without CLI-local reconstruction rules.
- Linked replay-bundle drilldown reuses the existing inspect shell.
- Research execution remains CLI-owned while the TUI extends its audit surface.
- Default validation remains deterministic and network-free.
- Point-in-time universes remain deferred until the snapshot and historical-membership model are honest enough to support them.

### M14 Gate: Live Market Intake And Snapshot Freshness

- Local live snapshot refresh becomes a first-class workflow instead of a narrow manual helper path.
- `trendlab-data` remains the owner of provider fetch, snapshot truth, and freshness metadata.
- Stored snapshots disclose freshness, provenance, and partial-failure state explicitly enough for audit.
- Default validation remains deterministic and network-free while live refresh stays optional.
- Intraday execution semantics remain out of scope.

### M15 Gate: Fresh-Snapshot Research Execution

- Explicit research specs can target a selected or latest stored snapshot without reconstructing low-level replay inputs by hand.
- Once the concrete snapshot is resolved, research execution remains deterministic and replayable.
- Generated replay bundles and shared research reports disclose the resolved snapshot identity and source provenance explicitly enough for audit.
- Research execution remains CLI-owned or background-worker-owned, not TUI-owned.
- Point-in-time universes remain deferred until the prerequisite data model exists.

### M16 Gate: Materialized Leaderboards And Research State

- Refreshed aggregate, walk-forward, bootstrap, and leaderboard outputs can be materialized and reopened as explicit local research state.
- Stale materialized results are detected explicitly when snapshots, specs, or linked artifacts drift.
- Every leaderboard row still drills back to shared replay and research artifacts.
- Comparable-run grouping rules remain explicit and auditable.
- Default validation remains deterministic and network-free.

### M17 Gate: Local Job Orchestration And Background Runs

- Single-machine snapshot refresh, research execution, and leaderboard refresh jobs can be queued, observed, and reopened with explicit status and provenance.
- The local job boundary composes `trendlab-data`, `trendlab-operator`, and `trendlab-artifact` without re-owning those trust boundaries.
- The orchestration model stays local-first and does not require a service or cloud API.
- Research execution remains outside TUI ownership.
- Point-in-time universes remain deferred until the prerequisite data model exists.

### M18 Gate: TUI Monitoring, Live Leaderboards, And Operator Control

- The TUI can monitor snapshot freshness and local job state without becoming the owner of provider fetch or background work.
- The TUI can browse refreshed local leaderboard and research state with audit-first drilldown into shared artifacts.
- Snapshot freshness, job failures, and stale materialized results remain explicit and operator-safe.
- Default validation remains deterministic and network-free.
- Point-in-time universes remain deferred until the prerequisite data model exists.

### M19 Gate: Point-In-Time Universe Foundation

- A universe snapshot representation and historical-membership model exist before PIT research UX expands.
- PIT and non-PIT runs remain explicitly distinguishable in manifests and research outputs.
- PIT support does not weaken deterministic replay or artifact reopenability.

### M20 Gate: Automated Search And Sweep Execution

- Bounded automated search remains artifact-backed and auditable instead of creating a second hidden result path.
- Local queued sweeps can be ranked, reopened, and compared through shared replay and research artifacts.
- Search guardrails remain explicit enough to avoid silently widening into broad opaque full-auto search.

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

## Post-M8 Horizon Note

Week 32 closes M8 honestly without point-in-time universes. The repo now has a truthful snapshot lifecycle across optional live capture, offline reopen and inspection, and a first operator-facing snapshot-backed CLI audit path, while provider-native types remain outside `trendlab-core`.

Week 33 is the planning checkpoint that turns that state into the next concrete milestone block. The next horizon therefore focuses on snapshot-backed operator runs, because that is the remaining operator gap implied by the current open risks and by the existing operator-facing spec work.

## Weeks 33-36: Post-M8 Operator-Source Plan

Week 32 closes M8 with truthful snapshot capture, reopen, inspection, and a first snapshot-backed audit path. The next honest gap is that operators still cannot launch replayable runs from stored snapshots directly; they still fall back to embedded low-level bar requests even though the repo now owns a trustworthy stored-snapshot path. The next block therefore centers on snapshot-backed operator run inputs while keeping point-in-time universes explicitly deferred.

### Week 33

- Objective: post-M8 checkpoint and roadmap reset.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - review the completed M8 snapshot lifecycle against the remaining operator and market-model gaps
  - decide the next milestone block from the actual post-M8 state instead of extending the roadmap speculatively
  - confirm that point-in-time universes remain deferred on the same prerequisite model
- Deliverables/artifacts:
  - updated `docs/Plan.md`, `docs/Roadmap.md`, and `docs/Status.md`
  - frozen next-horizon milestone definition for snapshot-backed operator runs
- Validation/checks:
  - the next milestone starts from current implemented ownership boundaries instead of reopening completed M8 contracts
  - roadmap state remains honest about what is and is not implemented after Week 32
- Exit criteria:
  - Weeks 34-36 are concrete enough to implement without another post-M8 planning pass
  - the next milestone gate is explicit
- Non-goals:
  - point-in-time universe implementation
  - speculative scheduling beyond the next concrete milestone block
- Dependencies/blockers:
  - Week 32 complete

### Week 34

- Objective: freeze the first snapshot-backed operator source contract and resolution boundary.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the first operator-facing snapshot source shape on top of the existing run-spec path
  - decide exactly where stored-snapshot symbol and date-slice resolution lives
  - make the first slice explicitly single-symbol and daily so the contract stays honest with the current kernel
- Deliverables/artifacts:
  - clarified operator source contract
  - frozen ownership notes for snapshot resolution and manifest provenance
- Validation/checks:
  - snapshot resolution ownership stays in `trendlab-data` instead of moving into CLI-local parsing
  - the contract keeps provider-native types out of `trendlab-core`
  - `cargo xtask validate`
- Exit criteria:
  - the first snapshot-backed run slice is concrete enough to implement without reopening M8 snapshot ownership
- Non-goals:
  - point-in-time universes
  - intraday or multi-symbol execution
- Dependencies/blockers:
  - Week 33 checkpoint
- Risk note:
  - source-model decisions may pressure run-spec ergonomics and manifest provenance fields if the ownership boundary is not kept narrow.

### Week 35

- Objective: implement the first snapshot-backed CLI run flow on top of stored snapshots.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - reopen a stored snapshot through `trendlab-data`
  - select one explicit symbol and date slice
  - derive canonical daily bars and map them into the existing core run path
  - persist replay bundles with snapshot-backed provenance still visible in the manifest
- Deliverables/artifacts:
  - first snapshot-backed operator run path
  - deterministic tests covering stored-snapshot run success and basic source rejection paths
- Validation/checks:
  - snapshot-backed runs stay offline and deterministic
  - no provider refetch occurs during run execution
  - replay bundles disclose the stored snapshot source they came from
  - `cargo xtask validate`
- Exit criteria:
  - operators can launch one truthful run from a stored snapshot without rebuilding a low-level request by hand
- Non-goals:
  - point-in-time universes
  - broader source-model polymorphism beyond the first stored-snapshot slice
- Dependencies/blockers:
  - Week 34 contract freeze
- Risk note:
  - the first run-source mapping may expose missing manifest/source fields or ambiguity about where symbol/date-slice provenance belongs.

### Week 36

- Objective: harden snapshot-backed run provenance and close M9 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten rejection paths for missing symbols, insufficient date coverage, and malformed stored snapshot sources
  - confirm explain and audit surfaces stay coherent for snapshot-backed runs
  - decide whether M9 closes honestly or whether the remaining operator-source gap needs one more narrow pass
- Deliverables/artifacts:
  - hardened snapshot-backed run provenance behavior
  - M9 checkpoint decision and next-horizon note
- Validation/checks:
  - snapshot-backed runs either reopen and explain honestly or fail with explicit reasons
  - default validation remains deterministic and network-free
  - point-in-time universes remain deferred unless the prerequisite model materially changes
  - `cargo xtask validate`
- Exit criteria:
  - M9 closes honestly with a reusable snapshot-backed run path, or the remaining work is explicitly re-baselined
- Non-goals:
  - point-in-time universe implementation
  - broad live-data health dashboards
- Dependencies/blockers:
  - Weeks 34-35
- Risk note:
  - the right outcome may still be a narrower M9 close if snapshot-backed run provenance exposes more contract pressure than expected.

## Post-M9 Horizon Note

Week 36 closes M9 honestly with a truthful stored-snapshot operator path. The next honest gap is no longer snapshot-backed provenance; it is that the TUI still cannot act as the operator shell for launching those trustworthy runs. The next block therefore centers on a TUI-driven operator lab while preserving the current trust boundaries: `trendlab-core` remains deterministic, `trendlab-data` still owns snapshot reopen and slice resolution, `trendlab-artifact` still owns replay bundles, and research execution stays CLI-owned in this horizon.

## Weeks 37-46: TUI-Driven Operator Lab

Week 36 closes M9 with truthful stored-snapshot operator runs, but the user still cannot launch the TUI without a prebuilt bundle, configure a snapshot-backed run interactively, execute it through the same trusted run path as the CLI, or reopen prior runs from inside the TUI. The next honest block therefore builds a shared operator orchestration layer and then turns the TUI into a first-class client of that path. This block remains snapshots-first, single-symbol, and daily-bar, and it explicitly keeps live capture and research execution outside the first TUI-runner horizon.

### Week 37

- Objective: post-M9 checkpoint and operator-horizon freeze.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - define `trendlab-operator` as the shared operator-orchestration crate
  - freeze shared library orchestration rather than CLI subprocess invocation as the TUI run path
  - freeze stored snapshots as the only v1 TUI run source
  - freeze research execution and live snapshot capture as non-goals for this block
- Deliverables/artifacts:
  - updated `docs/Plan.md`, `docs/Roadmap.md`, `docs/Workspace.md`, and `docs/Status.md`
  - explicit M10-M12 milestone definitions
  - frozen v1 TUI operator input and UX scope
- Validation/checks:
  - the new horizon preserves existing core, data, and artifact ownership boundaries
  - point-in-time universes remain deferred on the same prerequisite model
- Exit criteria:
  - Weeks 38-46 are concrete enough to implement without another planning pass
  - the shared operator boundary and TUI-runner scope are explicit before code changes begin
- Non-goals:
  - point-in-time universe implementation
  - speculative scheduling beyond Week 46
- Dependencies/blockers:
  - Week 36 complete

### Week 38

- Objective: create the shared operator crate and extract the run-spec boundary from CLI-only ownership.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - create `trendlab-operator`
  - move typed operator run-spec models and validation out of `trendlab-cli`
  - move run execution orchestration behind the shared operator boundary while keeping current snapshot-backed parity
- Deliverables/artifacts:
  - `trendlab-operator` crate skeleton plus first shared operator API
  - CLI wired as a thin front-end over the shared operator path
- Validation/checks:
  - CLI run behavior stays aligned with the pre-extraction snapshot-backed path
  - snapshot resolution remains in `trendlab-data`
  - `cargo xtask validate`
- Exit criteria:
  - CLI run execution uses `trendlab-operator`
  - the shared operator boundary is concrete enough for the TUI to consume next
- Non-goals:
  - TUI run UI
  - research execution migration out of CLI ownership
- Dependencies/blockers:
  - Week 37 checkpoint
- Risk note:
  - extraction may reveal CLI-local assumptions about explain summaries or output handoff that need a narrow shared abstraction instead of copy-pasted glue.

### Week 39

- Objective: finish shared run execution parity and explain handoff.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - return replay-bundle location plus caller-safe summary and provenance from `trendlab-operator`
  - keep CLI `run` and `explain` behavior aligned after the extraction
  - add explicit parity tests around the new shared operator surface
- Deliverables/artifacts:
  - stable shared operator result boundary
  - regression coverage for request-path, inline request, and snapshot-backed runs through the shared operator path
- Validation/checks:
  - CLI no longer owns a divergent run-orchestration branch
  - the TUI can consume the operator path without subprocesses
  - `cargo xtask validate`
- Exit criteria:
  - shared operator outputs are stable enough for TUI launch integration
- Non-goals:
  - TUI run mode implementation
  - broader source-model polymorphism beyond the current trusted snapshot-backed path
- Dependencies/blockers:
  - Week 38 extraction
- Risk note:
  - the explain handoff may expose pressure for a shared summary surface that stays presentation-neutral and does not turn `trendlab-operator` into a second artifact crate.

### Week 40

- Objective: restructure the TUI shell around a run-capable home mode.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add top-level Home or Run, Inspect, and Help app modes
  - preserve the current inspect shell as the post-run destination
  - define task state for in-session run launch success and failure
- Deliverables/artifacts:
  - TUI app shell that starts without a bundle path
  - state and render coverage for the new top-level modes
- Validation/checks:
  - the TUI can render a non-empty operator shell without opening a replay bundle first
  - the existing inspect experience remains intact when a bundle is already open
  - `cargo xtask validate`
- Exit criteria:
  - the TUI can start in run mode and later transition into inspect mode
- Non-goals:
  - snapshot browsing details
  - actual run launch
- Dependencies/blockers:
  - Week 39 shared operator boundary
- Risk note:
  - the shell restructure may pressure current navigation assumptions if inspect-mode state is too tightly coupled to bundle-at-startup behavior.

### Week 41

- Objective: add a snapshot browser and snapshot summary pane to the TUI.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - browse configured snapshot directories from the filesystem
  - inspect the selected snapshot through shared `trendlab-data` helpers only
  - surface provider identity, requested window, symbol list, raw counts, action counts, and normalization-sensitive summary data
- Deliverables/artifacts:
  - TUI snapshot browser
  - snapshot summary pane backed by shared data-layer inspection output
- Validation/checks:
  - the TUI does not introduce local snapshot parsing rules
  - empty and malformed snapshot selections fail with explicit operator-facing messages
  - `cargo xtask validate`
- Exit criteria:
  - the user can choose a stored snapshot inside the TUI and inspect its summary before configuring a run
- Non-goals:
  - live snapshot capture
  - multi-symbol run setup
- Dependencies/blockers:
  - Week 40 run-capable shell
- Risk note:
  - filesystem-facing snapshot browsing may expose a need for a tighter configured-root policy before arbitrary local-path browsing becomes the default UX.

### Week 42

- Objective: add the first interactive run-configuration form.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - select one symbol from the chosen snapshot
  - select exact start and end dates from the available stored bars for that symbol
  - edit the current trusted `request_template` fields only
  - surface validation errors before launch
- Deliverables/artifacts:
  - run form for `snapshot_dir`, `symbol`, `start_date`, `end_date`, and existing request-template fields
  - TUI validation coverage for malformed or incomplete run inputs
- Validation/checks:
  - the form remains single-symbol, daily-bar, and snapshot-backed
  - no live fetch initiation or multi-symbol expansion is added
  - `cargo xtask validate`
- Exit criteria:
  - the TUI can build a valid operator run spec from interactive inputs
- Non-goals:
  - fixture-mode primary UX
  - research command inputs
- Dependencies/blockers:
  - Week 41 snapshot summary flow
- Risk note:
  - mapping the existing request-template surface into interactive controls may expose fields that want clearer operator-facing names without changing the underlying trusted schema.

### Week 43

- Objective: complete the first in-TUI run launch and auto-open loop.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - invoke `trendlab-operator` from the TUI
  - support either explicit output selection or a deterministic default under a documented TUI output root
  - auto-open the resulting replay bundle in the existing inspect shell on success
  - keep failures in run mode with explicit error messaging
- Deliverables/artifacts:
  - first end-to-end TUI launch flow
  - output-root convention for TUI-launched runs
- Validation/checks:
  - TUI launch uses the shared operator path rather than a subprocess
  - successful runs reopen the same replay bundle the CLI would have written
  - `cargo xtask validate`
- Exit criteria:
  - the user can launch one truthful snapshot-backed run from inside the TUI and inspect it immediately
- Non-goals:
  - prior-run browsing
  - research execution from the TUI
- Dependencies/blockers:
  - Week 42 run configuration form
- Risk note:
  - output-root and overwrite behavior may require explicit operator policy to stay audit-safe and predictable across repeated launches.

### Week 44

- Objective: add prior-run browsing and reopen flow inside the TUI.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add a run-history list over the documented TUI output root
  - reopen existing replay bundles from inside the TUI
  - show summary and provenance before opening a selected prior run
- Deliverables/artifacts:
  - TUI prior-run browser
  - reopen flow that reuses the existing inspect shell
- Validation/checks:
  - the TUI reopens shared replay bundles instead of introducing mutable TUI-local run state as a second artifact store
  - the prior-run list handles missing or drifted bundles explicitly
  - `cargo xtask validate`
- Exit criteria:
  - the user can move between launching new runs and reopening prior runs in one TUI session
- Non-goals:
  - arbitrary artifact mutation
  - research report browsing
- Dependencies/blockers:
  - Week 43 launch loop
- Risk note:
  - history browsing may surface a need for a narrow shared bundle-summary helper if the TUI would otherwise duplicate CLI-facing explain formatting.

### Week 45

- Objective: polish the operator workflow with audit-first usability.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten keyboard flow across Home or Run, Inspect, and Help
  - preserve snapshot-backed provenance visibility in inspect mode after TUI-launched runs
  - add explicit empty, invalid-input, and failed-run states
  - make selected source and output location obvious before launch
- Deliverables/artifacts:
  - TUI polish across the new operator loop
  - deterministic state and render coverage for error and empty states
- Validation/checks:
  - operator state remains legible without hiding provenance or validation errors
  - current inspect behavior stays coherent after TUI-launched runs
  - `cargo xtask validate`
- Exit criteria:
  - the TUI-runner flow is coherent enough for real operator use
- Non-goals:
  - broader UX experimentation beyond the current audit-first shell
  - post-v1 source models
- Dependencies/blockers:
  - Weeks 40-44
- Risk note:
  - polish work may still reveal one more narrow usability gap in the shared operator result handoff or the TUI's mode model.

### Week 46

- Objective: harden the first TUI operator lab and close M12 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - add deterministic end-to-end coverage for stored snapshot -> TUI-configured run -> replay bundle -> inspect open
  - add rejection coverage for malformed run configs and bad snapshot selections
  - record the checkpoint and close decision for M10-M12 honestly
- Deliverables/artifacts:
  - deterministic end-to-end TUI operator-lab coverage
  - M12 checkpoint and next-horizon note
- Validation/checks:
  - default validation remains deterministic and network-free
  - the TUI can launch and inspect trusted backtests without CLI subprocesses
  - point-in-time universes remain deferred unless the prerequisite model materially changes
  - `cargo xtask validate`
- Exit criteria:
  - M10-M12 close honestly with a TUI that can launch and reopen trusted snapshot-backed runs
- Non-goals:
  - live snapshot capture from the TUI
  - research execution in the TUI
  - multi-symbol or point-in-time operator breadth
- Dependencies/blockers:
  - Weeks 38-45
- Risk note:
  - the honest close may still be a narrower M12 completion if the TUI run loop exposes more shared-boundary pressure than expected.

## Post-M12 Horizon Note

Week 46 closes M10, M11, and M12 honestly from the implemented state. The repo now has a truthful first TUI operator lab: the TUI can start without a bundle, browse stored snapshots, configure a snapshot-backed run, launch through the shared operator path, auto-open the resulting replay bundle in the existing inspect shell, browse prior runs under the documented TUI output root, and reopen those prior runs without introducing duplicate snapshot or artifact parsing rules.

The post-M12 checkpoint resolves the next honest gap as a saved-research audit gap rather than another operator-input expansion. The repo already has shared `research.json` ownership in `trendlab-artifact` and CLI-owned research execution/reopen flows, but the TUI still cannot reopen those saved research reports, inspect their linked-bundle structure, or drill back into the underlying replay bundles from one audit-first shell.

The next block therefore stays narrow:

- saved research execution remains CLI-owned
- the TUI learns to reopen and audit shared research reports, not to generate them
- linked replay-bundle drilldown should reuse the existing inspect shell
- point-in-time universes remain explicitly deferred until the repo has an honest universe snapshot representation and historical-membership model

## Weeks 47-51: TUI Research Audit Surface

Week 46 closes the first TUI operator lab honestly. The next concrete milestone is to make shared research reports first-class audit artifacts in the TUI without moving research execution out of the CLI. This keeps the trust boundaries intact while extending the terminal-native inspection surface around artifacts the repo already owns.

### Week 47

- Objective: post-M12 checkpoint and research-audit horizon freeze.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - review the closed TUI operator lab against the remaining operator, audit, and market-model gaps
  - freeze saved research-report reopen as the next concrete TUI horizon
  - keep research execution CLI-owned for this block
- Deliverables/artifacts:
  - updated `docs/Plan.md`, `docs/Roadmap.md`, and `docs/Status.md`
  - explicit M13 milestone definition for TUI research-report reopen and drilldown
- Validation/checks:
  - the next block reuses existing `trendlab-artifact` report ownership instead of creating CLI-local report truth
  - point-in-time universes remain deferred on the same prerequisite model
- Exit criteria:
  - Weeks 48-51 are concrete enough to implement without another re-baseline pass
  - the repo is explicit that the next horizon is report audit depth, not broader operator-source breadth
- Non-goals:
  - research execution in the TUI
  - speculative scheduling beyond the next concrete milestone block
- Dependencies/blockers:
  - Week 46 complete

### Week 48

- Objective: add a shared research-report open path to the TUI.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - extend TUI startup and open-path logic so it can distinguish replay bundles from shared research-report bundles
  - reopen saved `research.json` bundles only through `trendlab-artifact`
  - keep the existing replay-bundle inspect path intact
- Deliverables/artifacts:
  - TUI startup support for shared research-report bundles
  - deterministic state and load coverage for replay-bundle versus research-report open behavior
- Validation/checks:
  - the TUI does not introduce local research-report parsing rules
  - replay-bundle open behavior remains unchanged
  - `cargo xtask validate`
- Exit criteria:
  - the TUI can load a saved research report as a first-class artifact
- Non-goals:
  - report drilldown rendering
  - research execution in the TUI
- Dependencies/blockers:
  - Week 47 checkpoint
- Risk note:
  - artifact-type auto-detection may reveal a need for a narrow shared helper if replay and research load errors become ambiguous.

### Week 49

- Objective: add the first audit-first research report shell in the TUI.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add a report-focused TUI mode or shell state for loaded research reports
  - render report-kind summaries for aggregate, walk-forward, bootstrap, and leaderboard outputs
  - surface linked-bundle counts, baseline context, and key report metadata without hiding the report kind
- Deliverables/artifacts:
  - TUI research report summary shell
  - deterministic render coverage across the supported report kinds
- Validation/checks:
  - report rendering consumes shared report models rather than CLI-formatted strings
  - broken or invalid report states stay explicit
  - `cargo xtask validate`
- Exit criteria:
  - a saved research report can be inspected coherently in the TUI before any replay-bundle drilldown exists
- Non-goals:
  - replay-bundle drilldown
  - research report mutation
- Dependencies/blockers:
  - Week 48 report open path
- Risk note:
  - the first report shell may expose pressure for presentation-neutral summary helpers if the TUI would otherwise duplicate too much CLI-only formatting.

### Week 50

- Objective: add linked replay-bundle drilldown from research reports back into the existing inspect shell.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - navigate report members, splits, or leaderboard rows to their linked replay bundles
  - reopen those linked bundles in the existing inspect shell
  - keep missing or drifted linked bundles visible with explicit error text instead of silently dropping them
- Deliverables/artifacts:
  - TUI research-to-replay drilldown flow
  - deterministic coverage for successful drilldown and broken-link rejection paths
- Validation/checks:
  - replay-bundle drilldown reuses the current inspect shell rather than inventing a second result viewer
  - linked-bundle reopen still flows through shared artifact ownership
  - `cargo xtask validate`
- Exit criteria:
  - the TUI can move from a research report back into the replay truth it summarizes
- Non-goals:
  - TUI research execution
  - report editing or relinking
- Dependencies/blockers:
  - Week 49 report shell
- Risk note:
  - report-kind-specific row selection may still expose one more shared summary need if linked-bundle rows are too heterogeneous across aggregate, walk-forward, bootstrap, and leaderboard outputs.

### Week 51

- Objective: harden the TUI research audit surface and close M13 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - add deterministic end-to-end coverage for report open -> report summary -> linked replay drilldown -> inspect open
  - add rejection coverage for invalid report bundles and broken linked-bundle provenance
  - record the M13 checkpoint and close decision honestly
- Deliverables/artifacts:
  - deterministic end-to-end TUI research-audit coverage
  - M13 checkpoint and next-horizon note
- Validation/checks:
  - the TUI can audit saved research reports without taking over research execution
  - default validation remains deterministic and network-free
  - point-in-time universes remain deferred unless the prerequisite model materially changes
  - `cargo xtask validate`
- Exit criteria:
  - M13 closes honestly with a TUI that can reopen shared research reports and drill back into replay truth
- Non-goals:
  - research execution in the TUI
  - point-in-time or multi-symbol operator breadth
- Dependencies/blockers:
  - Weeks 48-50
- Risk note:
  - the honest close may still be a narrower M13 completion if report-shell rendering exposes more pressure for shared summary helpers than expected.

## Post-M13 Horizon Note

Week 51 closes M13 honestly and completes the first offline operator and research-audit horizon. The repo now has deterministic simulation truth, stored snapshot ownership, CLI-owned research execution, shared replay and research artifacts, and a TUI that can inspect both runs and saved research outputs with drilldown back into replay truth.

The next honest gap is no longer core trust or basic research breadth. The next gap is that the repo still treats live data refresh as a narrow helper path rather than the front door to a local near-real-time research loop. The post-M13 roadmap therefore stays local-first and explicit-spec-driven:

- live market-data refresh becomes first-class without turning provider responses into the trust boundary
- research execution stays CLI-owned or background-worker-owned rather than moving into the TUI
- materialized leaderboard and research state become explicit local artifacts instead of ephemeral command output only
- local background jobs are planned before any cloud or service-first architecture
- point-in-time universes and automated search remain later blocks rather than prerequisites for the first live local horizon

## Weeks 52-72: Local-First Near-Real-Time Research Lab

Weeks 52-72 are the committed next horizon after M13. They move the repo from a trustworthy offline lab into a local-first near-real-time workflow with refreshable snapshots, explicit research specs over fresh data, materialized leaderboard state, local background jobs, and TUI monitoring surfaces. This horizon explicitly does not add intraday execution semantics or a required service/API layer.

### Week 52

- Objective: complete the post-M13 checkpoint and freeze the local-first live-research horizon.
- Confidence: `High`
- Planning status: `Fixed`
- Scope:
  - review the closed M13 state against the remaining live-data, orchestration, and leaderboard gaps
  - freeze the next committed milestone sequence as M14 through M18
  - freeze CLI/background-worker ownership of research execution through this horizon
  - freeze `trendlab-jobs` as the planned shared local job boundary
- Deliverables/artifacts:
  - updated `docs/Plan.md`, `docs/Roadmap.md`, `docs/Status.md`, and `docs/Workspace.md`
  - explicit local-first, near-real-time scope statement for the next horizon
- Validation/checks:
  - the next block preserves existing `trendlab-core`, `trendlab-data`, `trendlab-artifact`, `trendlab-operator`, and TUI ownership boundaries
  - point-in-time universes and automated search remain later blocks instead of hidden prerequisites
- Exit criteria:
  - Weeks 53-72 are concrete enough to implement without another roadmap reset
  - the post-M13 target is explicit enough that later implementation does not have to invent architecture
- Non-goals:
  - code changes beyond the planning docs
  - intraday execution planning
- Dependencies/blockers:
  - Week 51 complete

## Weeks 53-56: Live Market Intake And Snapshot Freshness

### Week 53

- Objective: freeze the live snapshot refresh contract and freshness model.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the first shared live snapshot refresh request shape
  - decide the freshness, capture-result, and partial-failure fields that stored snapshots must disclose
  - define how stale or missing live data is surfaced in shared inspection flows
- Deliverables/artifacts:
  - clarified live refresh contract in planning and workspace docs
  - frozen freshness/provenance vocabulary for later CLI and TUI surfaces
- Validation/checks:
  - provider fetch and snapshot truth stay in `trendlab-data`
  - no intraday or streaming execution semantics are introduced
- Exit criteria:
  - M14 contracts are concrete enough to implement without inventing snapshot freshness semantics later
- Non-goals:
  - leaderboard refresh
  - research execution from fresh snapshots
- Dependencies/blockers:
  - Week 52 checkpoint
- Risk note:
  - freshness semantics may still expose pressure for stronger capture-result typing if partial-success cases are more common than expected.

### Week 54

- Objective: implement the first refreshable live snapshot workflow.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - extend the current live-capture path from a narrow helper into a first-class local refresh workflow
  - persist freshness and capture-result metadata with stored snapshots
  - preserve inspectable raw bars and corporate actions as the stored truth
- Deliverables/artifacts:
  - first live snapshot refresh flow
  - deterministic coverage for freshness metadata and refresh failure paths
- Validation/checks:
  - refreshed snapshots reopen through the same shared/data-layer path as existing stored snapshots
  - default validation remains deterministic and network-free
  - `cargo xtask validate`
- Exit criteria:
  - a local operator can refresh a stored snapshot and reopen it without hidden provider dependence later
- Non-goals:
  - background jobs
  - latest-snapshot research execution
- Dependencies/blockers:
  - Week 53 contract freeze
- Risk note:
  - refresh semantics may expose a need for snapshot retention or overwrite policy before repeated live refreshes become routine.

### Week 55

- Objective: add freshness-aware inspect and audit surfaces for refreshed snapshots.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - expose freshness, capture result, and stale-state summary through shared inspection helpers
  - add operator-facing snapshot audit surfaces that make partial or stale data explicit
  - keep snapshot inspection shared/data-layer-owned rather than CLI-local reconstruction
- Deliverables/artifacts:
  - refreshed snapshot inspection and audit outputs
  - deterministic coverage for stale, partial, and missing-data visibility
- Validation/checks:
  - inspection surfaces do not hide stale or partial captures behind normal snapshot summaries
  - `cargo xtask validate`
- Exit criteria:
  - refreshed snapshots are auditable enough for later near-real-time research execution
- Non-goals:
  - research spec execution
  - TUI live monitoring
- Dependencies/blockers:
  - Week 54 refresh flow
- Risk note:
  - freshness UX may expose pressure for a small shared summary helper if CLI and TUI would otherwise duplicate status wording.

### Week 56

- Objective: harden live snapshot refresh behavior and close M14 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten failure paths for stale, partial, missing, and incompatible live refresh results
  - confirm the M14 close decision from the implemented state
  - keep intraday execution explicitly out of the block
- Deliverables/artifacts:
  - hardened snapshot refresh behavior
  - M14 checkpoint and next-horizon note
- Validation/checks:
  - local live snapshot refresh is first-class without weakening default deterministic validation
  - `cargo xtask validate`
- Exit criteria:
  - M14 closes honestly with refreshable, auditable stored snapshots
- Non-goals:
  - research execution from fresh snapshots
  - local job orchestration
- Dependencies/blockers:
  - Weeks 53-55
- Risk note:
  - the honest close may still be a narrower M14 completion if refresh retention and overwrite policy need one more explicit pass.

## Weeks 57-60: Fresh-Snapshot Research Execution

### Week 57

- Objective: freeze the explicit research execution spec over selected or latest snapshots.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the first research execution spec shape over aggregate, walk-forward, bootstrap, and leaderboard flows
  - decide how selected versus latest snapshot targeting resolves into a concrete snapshot identity
  - keep execution ownership in the CLI or shared background orchestration path rather than the TUI
- Deliverables/artifacts:
  - frozen research execution spec contract
  - explicit provenance rules for latest-snapshot resolution
- Validation/checks:
  - once resolved, the concrete snapshot identity remains visible and audit-safe
  - `trendlab-core` stays free of provider, freshness, and selector semantics
- Exit criteria:
  - M15 execution inputs are concrete enough to implement without inventing selector precedence later
- Non-goals:
  - materialized leaderboard state
  - background job queueing
- Dependencies/blockers:
  - M14 close
- Risk note:
  - latest-snapshot resolution timing may still need one more explicit rule if queue-time and run-time semantics diverge.

### Week 58

- Objective: implement the first fresh-snapshot research execution path.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - execute explicit research specs directly from a resolved fresh snapshot
  - generate replay bundles and shared research reports through the existing ownership boundaries
  - keep aggregate and leaderboard flows working from explicit specs first
- Deliverables/artifacts:
  - first fresh-snapshot research execution flow
  - deterministic coverage for selected-snapshot and latest-snapshot resolution
- Validation/checks:
  - once a snapshot is resolved, research execution remains deterministic
  - generated artifacts disclose resolved snapshot provenance explicitly enough for audit
  - `cargo xtask validate`
- Exit criteria:
  - operators can rerun explicit research directly from fresh snapshot state without manually assembling replay-bundle sets first
- Non-goals:
  - local background execution
  - TUI research execution
- Dependencies/blockers:
  - Week 57 contract freeze
- Risk note:
  - the first path may expose pressure for a shared research-summary helper if explicit-spec execution and reopen formatting diverge.

### Week 59

- Objective: complete fresh-snapshot research execution across the current research families.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - extend the fresh-snapshot path across walk-forward and bootstrap in addition to aggregate and leaderboard
  - keep replay-bundle drilldown explicit through shared artifacts
  - preserve comparable-run rules and existing attribution requirements
- Deliverables/artifacts:
  - full current-family research execution from fresh snapshots
  - deterministic coverage for mismatched or stale-input rejection paths
- Validation/checks:
  - research execution still rejects dishonest comparisons explicitly
  - `cargo xtask validate`
- Exit criteria:
  - the current research families can be rerun from fresh snapshots without degrading replayability or auditability
- Non-goals:
  - materialized leaderboard state
  - queued execution
- Dependencies/blockers:
  - Week 58 execution path
- Risk note:
  - extending the path across every current research family may still expose one more shared execution-summary need.

### Week 60

- Objective: harden fresh-snapshot research execution and close M15 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten failure paths for stale selectors, missing latest snapshots, and incompatible explicit research specs
  - confirm the M15 close decision from the implemented state
- Deliverables/artifacts:
  - hardened fresh-snapshot research execution behavior
  - M15 checkpoint and next-horizon note
- Validation/checks:
  - fresh-snapshot research execution stays deterministic once snapshot identity is resolved
  - `cargo xtask validate`
- Exit criteria:
  - M15 closes honestly with explicit research specs that run directly from fresh snapshots
- Non-goals:
  - materialized leaderboard state
  - background queueing
- Dependencies/blockers:
  - Weeks 57-59
- Risk note:
  - the honest close may still be narrower if latest-snapshot selector semantics need one more pass.

## Weeks 61-64: Materialized Leaderboards And Research State

### Week 61

- Objective: freeze the materialized research-state identity model.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define materialized leaderboard or report identity over snapshots, explicit specs, and linked artifacts
  - decide the stale-result detection rules when inputs drift
  - decide which pieces live in shared artifact metadata versus a local state layer
- Deliverables/artifacts:
  - frozen materialized-state identity contract
  - explicit stale-result vocabulary for later monitoring surfaces
- Validation/checks:
  - materialized state still drills back to shared replay and research artifacts
  - `cargo xtask validate`
- Exit criteria:
  - M16 identity and stale-result rules are concrete enough to implement without later rework
- Non-goals:
  - background jobs
  - TUI live monitoring
- Dependencies/blockers:
  - M15 close
- Risk note:
  - stale-result identity may still expose pressure for stronger digest or linkage rules than the current integrity metadata provides.

### Week 62

- Objective: implement materialized research-state writes and reopen flows.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - persist refreshed local research and leaderboard state explicitly
  - add local list and reopen flows over the materialized state
  - keep artifact ownership in shared crates instead of command-local stores
- Deliverables/artifacts:
  - first materialized research-state flow
  - deterministic coverage for reopen and stale-state detection
- Validation/checks:
  - materialized state can be reopened without reconstructing execution inputs heuristically
  - `cargo xtask validate`
- Exit criteria:
  - refreshed research state exists as explicit local state rather than transient command output only
- Non-goals:
  - background queueing
  - TUI monitoring
- Dependencies/blockers:
  - Week 61 contract freeze
- Risk note:
  - materialized-state writes may expose a need for retention or garbage-collection policy before repeated refresh becomes routine.

### Week 63

- Objective: complete materialized leaderboard refresh and stale-result visibility.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - add materialized leaderboard refresh on top of the explicit research-state path
  - surface stale, drifted, or superseded results explicitly
  - preserve audit drilldown from leaderboard rows back to replay and research truth
- Deliverables/artifacts:
  - materialized leaderboard refresh flow
  - deterministic coverage for stale and superseded leaderboard-state paths
- Validation/checks:
  - leaderboard rows still reopen through shared artifacts instead of a second summary store
  - `cargo xtask validate`
- Exit criteria:
  - refreshed local leaderboards behave as explicit, auditable research state
- Non-goals:
  - local jobs
  - TUI monitoring
- Dependencies/blockers:
  - Week 62 materialized-state flow
- Risk note:
  - materialized leaderboard identity may still expose one more grouping or supersession rule before hardening.

### Week 64

- Objective: harden materialized research state and close M16 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten stale-result, missing-artifact, and superseded-state rejection paths
  - confirm the M16 close decision from the implemented state
- Deliverables/artifacts:
  - hardened materialized research-state behavior
  - M16 checkpoint and next-horizon note
- Validation/checks:
  - refreshed local leaderboard and research state remain explicit, reopenable, and auditable
  - `cargo xtask validate`
- Exit criteria:
  - M16 closes honestly with explicit local materialized leaderboard and research state
- Non-goals:
  - local jobs
  - TUI monitoring
- Dependencies/blockers:
  - Weeks 61-63
- Risk note:
  - the honest close may still be narrower if stale-result identity proves more coupled to job state than expected.

## Weeks 65-68: Local Job Orchestration And Background Runs

### Week 65

- Objective: freeze the local background job model and boundary.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define single-machine job requests, statuses, progress reporting, and provenance
  - freeze `trendlab-jobs` as the shared local orchestration boundary
  - define the first local persistence shape for queued and completed jobs
- Deliverables/artifacts:
  - frozen local job model
  - explicit ownership notes for `trendlab-jobs`
- Validation/checks:
  - background jobs compose `trendlab-data`, `trendlab-operator`, and `trendlab-artifact` rather than re-owning them
  - no service/API layer becomes required
- Exit criteria:
  - M17 job semantics are concrete enough to implement without architecture churn
- Non-goals:
  - TUI live monitoring
  - cloud execution
- Dependencies/blockers:
  - M16 close
- Risk note:
  - local persistence may still expose a need for stronger resume or cancellation semantics once longer-running jobs exist.

### Week 66

- Objective: implement background snapshot refresh jobs.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - queue and run local snapshot refresh jobs through the shared local job boundary
  - persist job status and provenance for refresh runs
  - keep actual provider and snapshot work inside `trendlab-data`
- Deliverables/artifacts:
  - first local snapshot refresh jobs
  - deterministic coverage for queued, failed, and completed refresh status paths
- Validation/checks:
  - snapshot refresh jobs do not create a second snapshot ownership boundary
  - `cargo xtask validate`
- Exit criteria:
  - operators can queue and inspect local snapshot refresh work without manual polling loops
- Non-goals:
  - background research execution
  - TUI monitoring
- Dependencies/blockers:
  - Week 65 job model
- Risk note:
  - job resume and duplicate-scheduling policy may still need one more explicit rule if refresh frequency grows.

### Week 67

- Objective: implement background research and leaderboard refresh jobs.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - queue fresh-snapshot research execution and materialized leaderboard refresh jobs
  - persist status and provenance across queued and completed research work
  - keep research execution owned by shared operator and artifact boundaries
- Deliverables/artifacts:
  - first background research and leaderboard refresh jobs
  - deterministic coverage for queued, failed, and completed research job paths
- Validation/checks:
  - background jobs still produce the same shared replay and research artifacts as foreground execution
  - `cargo xtask validate`
- Exit criteria:
  - the local near-real-time loop can refresh snapshots, run research, and refresh leaderboards without manual foreground chaining
- Non-goals:
  - TUI live monitoring
  - service/API orchestration
- Dependencies/blockers:
  - Week 66 snapshot refresh jobs
- Risk note:
  - background execution may still expose pressure for richer progress or cancellation semantics before TUI monitoring lands.

### Week 68

- Objective: harden the local background job model and close M17 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten failure, retry, and provenance paths for local jobs
  - confirm the M17 close decision from the implemented state
- Deliverables/artifacts:
  - hardened local job orchestration behavior
  - M17 checkpoint and next-horizon note
- Validation/checks:
  - local jobs remain single-machine, explicit, and audit-safe without requiring a service layer
  - `cargo xtask validate`
- Exit criteria:
  - M17 closes honestly with usable local background refresh and research orchestration
- Non-goals:
  - cloud execution
  - TUI job control beyond later monitoring surfaces
- Dependencies/blockers:
  - Weeks 65-67
- Risk note:
  - the honest close may still be narrower if local job persistence needs one more pass before it is stable enough for TUI monitoring.

## Weeks 69-72: TUI Monitoring, Live Leaderboards, And Operator Control

### Week 69

- Objective: freeze the TUI live-monitoring horizon.
- Confidence: `Medium`
- Planning status: `Fixed`
- Scope:
  - define the TUI surfaces for snapshot freshness, local jobs, and refreshed leaderboard state
  - preserve the existing inspect and research audit surfaces as drilldown destinations
  - keep execution ownership out of the TUI
- Deliverables/artifacts:
  - frozen TUI monitoring scope for the live local horizon
  - explicit non-goals for research execution in the TUI
- Validation/checks:
  - the TUI remains a client and monitor rather than a new trust boundary
  - `cargo xtask validate`
- Exit criteria:
  - M18 UX scope is concrete enough to implement without turning the TUI into a second orchestration engine
- Non-goals:
  - TUI-owned research execution
  - intraday dashboards
- Dependencies/blockers:
  - M17 close
- Risk note:
  - the first monitoring shell may still expose one more shared summary need if job and freshness state are too heterogeneous.

### Week 70

- Objective: implement snapshot freshness and local job monitoring in the TUI.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - surface snapshot freshness and capture-result state in the TUI
  - surface queued, running, failed, and completed local job state in the TUI
  - keep drilldown explicit into stored snapshots and shared artifacts
- Deliverables/artifacts:
  - TUI monitoring for snapshot freshness and local jobs
  - deterministic TUI coverage for live-status visibility and failure states
- Validation/checks:
  - TUI monitoring does not add TUI-local snapshot or job truth
  - `cargo xtask validate`
- Exit criteria:
  - operators can monitor the local refresh loop in the TUI without leaving the audit-first shell
- Non-goals:
  - refreshed leaderboard browsing
  - TUI-owned job execution
- Dependencies/blockers:
  - Week 69 scope freeze
- Risk note:
  - monitoring UX may still expose a need for aggregation or filtering once multiple local jobs exist.

### Week 71

- Objective: add refreshed leaderboard browsing and drilldown to the TUI.
- Confidence: `Medium`
- Planning status: `Provisional`
- Scope:
  - browse refreshed local leaderboard and research state in the TUI
  - drill back into materialized research state, saved reports, and replay bundles
  - keep stale or superseded state explicit
- Deliverables/artifacts:
  - TUI refreshed leaderboard browser
  - deterministic TUI coverage for stale, superseded, and successful drilldown states
- Validation/checks:
  - refreshed leaderboard views remain audit-first and artifact-backed
  - `cargo xtask validate`
- Exit criteria:
  - the TUI can monitor and inspect the local near-real-time research state end to end
- Non-goals:
  - TUI-owned research execution
  - point-in-time or automated search expansion
- Dependencies/blockers:
  - Week 70 monitoring surfaces
- Risk note:
  - refreshed leaderboard UX may still reveal one more shared state-summary requirement before hardening.

### Week 72

- Objective: harden the TUI live-monitoring surface and close M18 honestly.
- Confidence: `Low`
- Planning status: `Provisional`
- Scope:
  - tighten stale-state, failed-job, and missing-artifact visibility in the TUI
  - confirm the M18 close decision from the implemented state
- Deliverables/artifacts:
  - hardened TUI live-monitoring and refreshed leaderboard behavior
  - M18 checkpoint and next-horizon note
- Validation/checks:
  - the TUI can monitor the local near-real-time research loop without becoming the owner of fetch, jobs, or execution
  - `cargo xtask validate`
- Exit criteria:
  - M18 closes honestly with audit-first TUI monitoring over fresh local research state
- Non-goals:
  - TUI-owned research execution
  - intraday monitoring or cloud control
- Dependencies/blockers:
  - Weeks 69-71
- Risk note:
  - the honest close may still be narrower if refreshed leaderboard browsing and job monitoring need one more shared summary pass.

## Weeks 73-80: Optional Later Blocks

Weeks 73-80 are planned later blocks rather than committed follow-on work. They should not start until the M14 through M18 local-first live-research horizon closes honestly and the repo re-baselines again.

## Weeks 73-76: Point-In-Time Universe Foundation

### Week 73

- Objective: freeze the point-in-time universe contract and prerequisites.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - define the first honest universe snapshot representation
  - define the historical-membership model and provenance requirements
  - decide the first PIT-versus-non-PIT disclosure rules
- Deliverables/artifacts:
  - frozen PIT universe contract
- Validation/checks:
  - PIT work starts from an explicit data model instead of inferred membership
- Exit criteria:
  - M19 is concrete enough to implement honestly
- Non-goals:
  - automated search
- Dependencies/blockers:
  - M18 close

### Week 74

- Objective: implement the first universe snapshot and historical-membership store.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - persist point-in-time universe snapshots and historical membership
  - keep provider and universe truth shared/data-layer-owned
- Deliverables/artifacts:
  - first PIT universe storage path
- Validation/checks:
  - PIT universe truth remains explicit and reopenable
- Exit criteria:
  - stored PIT universe state exists
- Non-goals:
  - TUI PIT UX
- Dependencies/blockers:
  - Week 73 contract freeze

### Week 75

- Objective: add PIT-aware research and operator constraints.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - add PIT-aware provenance, validation, and research constraints
  - keep PIT and non-PIT flows explicitly distinguishable
- Deliverables/artifacts:
  - PIT-aware execution and research constraints
- Validation/checks:
  - PIT research cannot silently fall back to non-PIT assumptions
- Exit criteria:
  - PIT-aware flows are honest enough for hardening
- Non-goals:
  - automated search
- Dependencies/blockers:
  - Week 74 PIT storage

### Week 76

- Objective: harden PIT universe foundation and close M19 honestly.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - tighten PIT provenance and incompatibility failures
  - confirm the M19 close decision from the implemented state
- Deliverables/artifacts:
  - hardened PIT universe foundation
  - M19 checkpoint and next-horizon note
- Validation/checks:
  - PIT and non-PIT modes remain explicit and audit-safe
- Exit criteria:
  - M19 closes honestly with a usable PIT foundation
- Non-goals:
  - automated search
- Dependencies/blockers:
  - Weeks 73-75

## Weeks 77-80: Automated Search And Sweep Execution

### Week 77

- Objective: freeze the bounded automated-search model.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - define bounded search packs over explicit strategies or parameter sets
  - define search guardrails and queue semantics
  - keep broad opaque full-auto search out of scope
- Deliverables/artifacts:
  - frozen search-pack contract
- Validation/checks:
  - search remains explicit and auditable
- Exit criteria:
  - M20 is concrete enough to implement honestly
- Non-goals:
  - cloud search execution
- Dependencies/blockers:
  - M18 close, and optionally M19 if PIT-aware search is desired

### Week 78

- Objective: implement local queued sweep execution for bounded search packs.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - execute bounded search packs through the local job and artifact boundaries
  - preserve replay and research artifacts for every candidate
- Deliverables/artifacts:
  - first local bounded search execution path
- Validation/checks:
  - automated search does not create a second hidden result path
- Exit criteria:
  - bounded sweeps can run locally and persist audit-safe artifacts
- Non-goals:
  - broad full-auto search
- Dependencies/blockers:
  - Week 77 contract freeze

### Week 79

- Objective: add materialized rankings and drilldown for search results.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - materialize bounded search rankings
  - drill from ranked search results back into replay and research artifacts
  - keep guardrails explicit
- Deliverables/artifacts:
  - materialized bounded search rankings
- Validation/checks:
  - ranked search results remain artifact-backed and explainable
- Exit criteria:
  - bounded search results can be reopened and compared honestly
- Non-goals:
  - cloud search orchestration
- Dependencies/blockers:
  - Week 78 execution path

### Week 80

- Objective: harden bounded automated search and close M20 honestly.
- Confidence: `Low`
- Planning status: `Contingent`
- Scope:
  - tighten guardrails, stale-state handling, and provenance visibility for search results
  - confirm the M20 close decision from the implemented state
- Deliverables/artifacts:
  - hardened bounded search workflow
  - M20 checkpoint and next-horizon note
- Validation/checks:
  - bounded search remains local-first, explicit, and artifact-backed
- Exit criteria:
  - M20 closes honestly with bounded automated sweeps
- Non-goals:
  - broad opaque full-auto search
- Dependencies/blockers:
  - Weeks 77-79

## Backlog After the Roadmap

These items are intentionally outside the current planned sequence unless a later re-baseline pulls them forward:

- intraday execution modeling
- Pine export and parity vectors
- cloud or service-first execution workflows beyond the planned local job horizon
- portfolio breadth beyond what is needed for the research layer
- live data-health and provenance badges beyond the committed freshness and local monitoring surfaces
- advanced chart overlays beyond the current audit-capable TUI shell and the planned refreshed leaderboard surfaces

## Review Checklist

Use this checklist whenever the roadmap is revised:

- every committed milestone from M1 through M18 has weekly slices, and any later optional blocks are marked explicitly as contingent
- no week asks the implementer to invent acceptance criteria
- Week 0 owns the initial design decisions and golden-scenario choices, and later planning checkpoints freeze the next horizon before implementation resumes
- every week includes explicit validation and exit criteria
- later weeks show uncertainty honestly
- the roadmap points back to the contract docs instead of silently replacing them
