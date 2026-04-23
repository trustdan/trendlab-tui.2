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

### M9: Snapshot-Backed Operator Runs

Purpose: let operators launch replayable runs from stored snapshots instead of only embedded low-level bar requests, while keeping the core deterministic and provider-agnostic.

Scope:

- operator-facing snapshot source specs layered on top of the existing run-spec path
- snapshot resolution helpers in `trendlab-data`
- first snapshot-backed CLI run flow using stored snapshots
- provenance-safe manifest/source disclosure for snapshot-backed runs

Acceptance criteria:

- operators can launch a run from a stored snapshot path plus explicit symbol and date selection without provider refetches or copy-pasted raw bars
- `trendlab-core` still consumes canonical `DailyBar` inputs and stays free of snapshot, filesystem, and provider-native concerns
- replay manifests disclose snapshot identity, symbol selection, requested slice, and operator source provenance explicitly enough for explain and audit
- `cargo xtask validate` remains deterministic and network-free by using stored snapshots or fixtures only
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M10: Shared Operator Orchestration

Purpose: remove CLI-only ownership of run orchestration without moving truth into the TUI.

Scope:

- shared operator-facing run-spec types and validation
- shared run execution orchestration callable by both CLI and TUI
- replay-bundle output handoff and presentation-neutral run result summaries
- extraction of the current CLI-owned run path into a reusable library boundary

Acceptance criteria:

- CLI and TUI can call the same operator library for run execution
- the shared operator library reuses `trendlab-data` snapshot resolution and `trendlab-artifact` replay writes instead of re-owning those boundaries
- no subprocess requirement exists for TUI run execution
- run outputs remain deterministic and provenance-safe
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M11: TUI Run Workspace

Purpose: let operators configure and launch snapshot-backed runs inside the TUI.

Scope:

- a TUI home or run workspace that can start without a prebuilt replay bundle
- stored-snapshot browsing and summary inspection through shared data-layer helpers
- interactive construction of the current trusted snapshot-backed operator input
- in-session run launch, validation, and failure handling without moving run truth into the TUI

Acceptance criteria:

- the TUI can browse and select a stored snapshot
- the TUI can configure the current trusted run inputs for a snapshot-backed run
- the TUI can launch a run and transition directly into result inspection
- validation and error messages remain explicit and audit-safe
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M12: TUI Operator Lab v1

Purpose: make the TUI usable as the primary operator shell for launching and reopening runs.

Scope:

- prior-run browsing and replay-bundle reopen inside the TUI
- auto-open from successful run launch into the existing inspect shell
- operator-polish and hardening work that keeps the TUI orchestration-first
- deterministic TUI-focused tests for the new run and reopen loop

Acceptance criteria:

- the TUI can launch new runs and reopen prior runs
- the current inspection shell remains the result viewer
- no duplicate TUI-owned run logic, snapshot parsing, or artifact parsing is introduced
- default validation remains deterministic and network-free
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M13: TUI Research Audit Surface

Purpose: make saved research reports first-class shared artifacts in the TUI without moving research execution out of the CLI.

Scope:

- reopen shared `research.json` bundles in the TUI through `trendlab-artifact`
- render audit-first report summaries for aggregate, walk-forward, bootstrap, and leaderboard outputs
- drill down from report rows back into linked replay bundles through the existing inspect shell
- keep broken linked-bundle or invalid report states explicit instead of silently hiding them

Acceptance criteria:

- the TUI can open a saved shared research-report bundle without CLI-local reconstruction
- report summaries preserve kind, baseline context, linked-bundle counts, and provenance-critical fields explicitly enough for audit
- linked replay-bundle drilldown reuses the existing inspect shell instead of introducing a second result viewer
- research execution remains CLI-owned throughout this block
- default validation remains deterministic and network-free
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M14: Live Market Intake And Snapshot Freshness

Purpose: turn live market-data refresh into a first-class local workflow without making provider responses the trust boundary.

Scope:

- refreshable live snapshot capture layered on top of the existing Tiingo and snapshot path
- freshness, provenance, and partial-failure metadata for stored snapshots
- explicit stale, partial, or missing live-data handling in shared/data-layer-owned inspection flows
- no intraday execution semantics

Acceptance criteria:

- operators can refresh stored snapshots locally without hidden provider refetches during later run or research execution
- freshness and provenance are visible enough for audit when a snapshot is reopened or inspected
- stale, partial, and missing live-data states fail explicitly instead of being silently treated as clean snapshots
- `trendlab-data` remains the owner of provider fetch, snapshot truth, and freshness metadata
- `cargo xtask validate` remains deterministic and network-free while live refresh stays optional

### M15: Fresh-Snapshot Research Execution

Purpose: run the existing research flows directly from explicit specs plus a selected or latest stored snapshot instead of manually assembling replay inputs first.

Scope:

- a research execution spec layered on top of the current operator and research boundaries
- selected-snapshot and latest-snapshot targeting for explicit research executions
- deterministic replay-bundle and shared `research.json` generation from refreshed snapshots
- CLI-first or shared-operator-first execution, not TUI-owned execution

Acceptance criteria:

- explicit research specs can target a selected or latest stored snapshot without rebuilding low-level requests by hand
- once a concrete snapshot is resolved, research execution remains deterministic and replayable
- generated replay bundles and research reports disclose the resolved snapshot identity and source provenance explicitly enough for audit
- research execution remains CLI-owned or background-worker-owned throughout this block
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M16: Materialized Leaderboards And Research State

Purpose: turn generated research outputs into refreshable materialized state for local near-real-time leaderboard and report workflows.

Scope:

- a materialized leaderboard or report identity model over the existing shared artifact path
- refreshed aggregate, walk-forward, bootstrap, and leaderboard outputs
- stale-result detection when snapshots, specs, or linked replay inputs change
- reopen, list, and explain surfaces over the materialized research state

Acceptance criteria:

- refreshed research and leaderboard state can be listed, reopened, and explained without reconstructing the inputs heuristically
- stale materialized results are detected explicitly instead of being shown as fresh
- every leaderboard row still drills back to replay and research artifacts through shared ownership paths
- comparable-run grouping rules remain explicit and auditable
- default validation remains deterministic and network-free

### M17: Local Job Orchestration And Background Runs

Purpose: add single-machine background orchestration for refresh and research work without jumping to a service-first or cloud-first design.

Scope:

- a local job request, status, and provenance model
- background snapshot refresh jobs
- background research execution jobs
- background leaderboard or materialized-state refresh jobs
- local-only persistence and failure semantics for queued and completed work

Acceptance criteria:

- local refresh and research jobs can be queued, observed, and reopened with explicit status and provenance
- the orchestration model stays local-first and does not require a service or network API boundary
- `trendlab-data`, `trendlab-operator`, and `trendlab-artifact` keep their current ownership boundaries while background work composes them
- research execution remains outside TUI ownership
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M18: TUI Monitoring, Live Leaderboards, And Operator Control

Purpose: expose fresh snapshot state, background job state, and refreshed local leaderboards in the TUI without moving trust or execution ownership into the TUI.

Scope:

- TUI monitoring for snapshot freshness and live-ingest status
- TUI monitoring for queued, running, failed, and completed local jobs
- refreshed leaderboard and research browsing over materialized local state
- drilldown from live leaderboard state back into shared replay and research artifacts

Acceptance criteria:

- the TUI can monitor fresh local research state without becoming the owner of provider fetch, background jobs, or research execution
- refreshed leaderboard views preserve audit-first drilldown into shared artifacts
- snapshot freshness and job failures remain explicit and operator-safe
- default validation remains deterministic and network-free
- point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

### M19: Point-In-Time Universe Foundation

Purpose: add the honest data-model foundation for point-in-time research once the local live-research loop is stable.

Scope:

- a universe snapshot representation
- a historical-membership model
- point-in-time-aware provenance and operator disclosures
- point-in-time research constraints before broader PIT UX or search expansion

Acceptance criteria:

- point-in-time research uses an explicit universe snapshot and historical-membership model instead of inferred membership
- non-point-in-time and point-in-time runs remain explicitly distinguishable in manifests and research outputs
- point-in-time work does not weaken deterministic replay or artifact reopenability

### M20: Automated Search And Sweep Execution

Purpose: add bounded automated search over explicit strategy packs after the live local research loop and its trust surfaces are stable.

Scope:

- bounded search-pack definitions over explicit strategies or parameter sets
- local queued sweep execution on top of the existing replay and research ownership boundaries
- materialized rankings and drilldown over search results
- explicit guardrails against unbounded or opaque full-auto search

Acceptance criteria:

- automated search remains artifact-backed and auditable instead of becoming a second hidden result path
- bounded sweeps can be queued, ranked, reopened, and compared through shared replay and research artifacts
- broader full-auto search remains out of scope unless a later re-baseline expands it explicitly

## Immediate Next Work

The next implementation work should begin Week 52 and then move into the new M14 through M18 horizon:

1. freeze the post-M13 horizon around local-first live market intake, fresh-snapshot research execution, materialized leaderboard state, local background jobs, and TUI monitoring
2. begin Week 53 M14 work by defining the live snapshot refresh contract, freshness metadata, and stale or partial failure semantics
3. keep research execution CLI-owned or background-worker-owned through M14-M18 so the TUI remains a client and monitor rather than the trust boundary
4. keep point-in-time universes and automated search deferred into M19 and M20 unless a later re-baseline deliberately pulls them forward

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
- how far the first snapshot-backed operator run path should go beyond single-symbol daily selection can be revisited after the initial stored-snapshot run flow lands
- whether `trendlab-operator` later needs a shared explain-summary surface beyond run execution handoff can be revisited once both CLI and TUI consume the same operator path
- whether the first TUI operator workspace needs saved presets beyond direct run-history reopen can be revisited after the Week 46 hardening pass lands
- whether shared research-report summary helpers should live in `trendlab-artifact` or whether the TUI should render directly from shared report models can be revisited once the first TUI research-report reopen slice lands
- whether the TUI should later absorb CLI-owned research execution can be revisited only after the first shared research-report reopen and drilldown block closes honestly
- whether latest-snapshot selection should resolve at queue time or execution time needs to be frozen during M15 so reruns stay provenance-safe
- whether materialized leaderboard identity should live in shared artifact metadata or in a local job or state layer needs to be frozen before M16 implementation starts
- whether the planned local background job state should live in a new shared crate or remain a narrower extension of the existing operator boundary should be frozen during the Week 52 checkpoint
- whether strategy-component labels should remain standardized manifest parameters or eventually graduate into first-class manifest fields can be revisited if later portability or audit work exposes a concrete limitation in the current shared-manifest path
- when point-in-time universe work returns, the repo will still need an honest universe snapshot representation and historical-membership model before it is scheduled
