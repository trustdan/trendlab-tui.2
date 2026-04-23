# Status

## Current State

Week 52 is now complete as the post-M13 checkpoint and roadmap reset. M10 through M13 remain closed honestly from the implemented operator-lab and research-audit state: the repo still has a concrete `trendlab-operator` crate that owns shared operator run-spec resolution and replay-bundle execution, both `trendlab-cli run` and `trendlab-tui` still launch through that shared operator path without a CLI subprocess boundary, and the TUI can inspect both replay bundles and shared research reports with artifact-backed drilldown into replay truth. This planning pass freezes the next concrete horizon as a local-first, near-real-time research lab rather than an intraday or cloud-first system. The committed next milestone block is now M14 through M18: live market intake and snapshot freshness, fresh-snapshot research execution, materialized leaderboards and research state, local background orchestration, and TUI monitoring/live operator surfaces. M19 point-in-time universe foundation and M20 automated search remain planned later blocks rather than prerequisites for the first live local horizon.

## Locked Decisions

- Rust workspace from day one
- ratatui for the TUI
- build order is truthful lab, then CLI, then TUI shell
- Tiingo is the initial provider target behind a provider boundary
- canonical market model types live in `trendlab-core`
- persisted run artifacts live in a shared artifact crate
- default validation is deterministic and network-free
- point-in-time universes are post-v1
- repo validation target is `cargo xtask validate`
- post-M9 run orchestration should move into a shared `trendlab-operator` crate instead of CLI subprocess calls from the TUI
- the first TUI-runner horizon is snapshots-first, single-symbol, daily-bar, and request-template-driven
- research execution stays CLI-owned through the first TUI research-audit block
- live snapshot capture stays outside the first TUI-runner horizon
- the post-M13 horizon is local-first, near-real-time, and explicit-spec-driven rather than intraday or cloud-first
- research execution should remain CLI-owned or background-worker-owned through M14-M18, with the TUI acting as a client and monitor rather than the trust boundary
- point-in-time universes and automated search are planned later blocks, not prerequisites for the first live local horizon

## Completed In This Pass

- established repo-level agent instructions
- wrote the active milestone plan
- wrote the master week-by-week roadmap
- added repo-local Codex and Cursor setup files
- added repo-local Cursor hooks and PowerShell hook scripts
- verified hook behavior with stdin-fed local tests
- proposed the workspace shape
- defined shared run-artifact ownership and manifest truth labels
- narrowed M1 to a truthful order-lifecycle kernel
- wrote the math, bar-semantics, and invariant contracts
- added baseline Cursor rules
- kept global home-directory Codex and Cursor config untouched
- evaluated external roadmap feedback against the repo plan
- adopted the useful feedback items into the roadmap and workflow docs
- declined scope-expanding M3 and early performance-budget changes for now
- chose the on-disk replay-bundle encoding
- chose the normalized market-data cache format
- froze the initial M1 crate set and interface notes
- froze the first fixture-authored M1 reference flow
- froze the first three M1 golden scenarios and initial oracle target
- froze the initial `cargo xtask validate` and `validate-live` shape
- created the workspace root `Cargo.toml` and `cargo xtask` alias
- scaffolded `xtask`, `trendlab-core`, `trendlab-artifact`, and `trendlab-testkit`
- wired `cargo xtask validate` to fmt, clippy, and workspace tests
- added placeholder core, artifact, and testkit boundaries matching the Week 1 plan
- created `fixtures/` and `tests/` roots for upcoming deterministic coverage
- verified `cargo check --workspace`
- verified `cargo xtask validate`
- defined the first concrete M1 domain surface in `trendlab-core`
- defined manifest, bundle, summary, and persisted ledger-row schema shells in `trendlab-artifact`
- implemented fixture loading, golden helpers, and oracle helpers in `trendlab-testkit`
- added the three frozen M1 scenario directories under `fixtures/`
- authored the first hand-written oracle ledger for `m1_intrabar_stop_exit`
- verified deterministic fixture parsing and oracle loading through workspace tests
- implemented the queued market-entry path in `trendlab-core`
- wired cash, position, stop-state carry, and equity transitions for the non-stop Week 3 flow
- emitted concrete per-bar ledger rows from the truthful kernel
- matched the generated hold-path ledger against the frozen `m1_entry_hold_open_position` fixture
- made stop-triggering scenarios fail explicitly instead of producing silent wrong results before Week 4
- implemented fixed protective-stop execution for both intrabar hits and gap-through-open exits
- applied the frozen M1 default gap policy directly in the core engine
- enforced that only a stop carried into the bar can trigger on that bar
- matched the hand-authored `m1_intrabar_stop_exit` oracle ledger
- matched the frozen `m1_gap_through_stop_exit` fixture ledger
- expanded unit coverage for gap-open, intrabar-stop, and same-day stop-sequencing behavior
- re-verified `cargo xtask validate` with the Week 4 stop path enabled
- added JSON and JSONL persistence for `bundle.json`, `manifest.json`, `summary.json`, and `ledger.jsonl`
- added replay-bundle write and load helpers to `trendlab-artifact`
- added fixture-backed bundle helpers to `trendlab-testkit`
- verified that frozen fixture runs round-trip through persisted replay bundles without ledger drift
- added `cargo xtask write-fixture-bundle --scenario ... --output ...`
- added `cargo xtask inspect-ledger <bundle-dir>` for pre-CLI ledger inspection
- manually verified the inspection path against a persisted `m1_intrabar_stop_exit` replay bundle
- re-verified `cargo xtask validate` with the Week 5 persistence path enabled
- added Week 6 request validation for ordered daily bars, valid M1 stop fractions, non-negative costs, and entry-intent dates that actually land on fixture bars
- added replay-bundle reconciliation checks for summary row count, warning count, and terminal cash/equity consistency
- added Week 6 tests for malformed request rejection and inconsistent persisted bundle rejection
- added frozen-ledger reconciliation assertions across generated runs, persisted bundles, and expected fixture ledgers
- manually re-verified fixture bundle writing and ledger inspection for the Week 6 hold and intrabar-stop cases
- closed the M1 gate and moved the active next step to Week 7 canonical market-data work
- added the `trendlab-data` workspace crate and wired it into default workspace validation
- added provider identity, Tiingo-shaped raw daily-bar and corporate-action types, and snapshot-backed stored symbol data
- added Week 7 ingestion validation that keeps provider-native rows outside `trendlab-core` while producing canonical stored raw bars and corporate actions
- added split-adjusted normalization from stored raw bars into `trendlab-core::market::DailyBar`, keeping raw fills and analysis prices in separate spaces
- added deterministic fixture-backed normalization coverage under `fixtures/m2_tiingo_split_adjustment`
- updated the stale immediate-next-work note in `docs/Plan.md` so the planning docs stay aligned with the current milestone state
- re-verified `cargo xtask validate` with `trendlab-data` in the workspace
- extended normalized symbol data with explicit per-date split and cash-dividend effects so dividend cashflows stay inspectable without polluting `analysis_close`
- added internal weekly and monthly resampling from canonical daily bars inside `trendlab-data`
- added deterministic fixture-backed resampling coverage under `fixtures/m2_tiingo_resampling`
- added a minimal provider-adapter smoke boundary for Tiingo inside `trendlab-data`
- replaced the Week 1 `validate-live` placeholder with an honest Tiingo smoke lane that prints its invariants and fails cleanly without `TIINGO_API_TOKEN`
- manually exercised `cargo xtask validate-live --provider tiingo` to confirm the smoke lane stays outside normal validation
- closed the M2 gate and moved the active next step to Week 9 M3 strategy composition
- re-verified `cargo xtask validate` after the Week 8 data-layer and xtask changes
- added Week 9 strategy-layer interfaces in `trendlab-core` for signals, filters, position managers, and execution models
- added a composite strategy evaluation surface that keeps each layer's output explicit without changing the existing M1 reference engine
- added composition tests for pass-through and blocked entry paths, proving execution sees the signal decision by reference and does not rewrite it
- re-verified `cargo xtask validate` after the Week 9 core-interface changes
- added a concrete close-confirmed breakout signal generator in `trendlab-core` that evaluates trailing-window breakouts from `analysis_close`
- added a pass-through filter, keep-position manager, and next-open long execution model so the first breakout path composes from the Week 9 strategy interfaces
- added deterministic strategy tests for strict breakout confirmation, analysis-series usage, next-open queueing, and flat-only execution blocking
- re-verified `cargo xtask validate` after the Week 10 breakout changes
- added a concrete stop-entry breakout signal generator in `trendlab-core` that carries trailing raw-high thresholds into the next bar without requiring close confirmation
- extended pending-order representation so queued market entries and carried stop-entry thresholds remain auditably distinct
- added a stop-entry long execution model that blocks in-position or duplicate entry attempts without mutating the original signal decision
- added deterministic strategy tests for carried-threshold behavior, flat-only stop-entry carrying, and duplicate-entry blocking
- closed the M3 gate and moved the active next step to Week 12 CLI foundation
- re-verified `cargo xtask validate` after the Week 11 stop-entry changes
- added serde-backed `RunRequest` input support in `trendlab-core` so CLI run requests can reuse core request types directly
- created the `trendlab-cli` workspace crate and wired it into default workspace validation
- added `trendlab-cli run --request <path> --output <dir>` to execute the reference flow and persist replay bundles through `trendlab-artifact`
- added `trendlab-cli explain <bundle-dir>` to reopen shared replay bundles and print audit-oriented summary plus ledger reasoning
- added deterministic CLI tests covering bundle-writing run flow, provider validation, and artifact-backed explain output
- closed Week 12 and moved the active next step to Week 13 CLI completion work
- re-verified `cargo xtask validate` after the Week 12 CLI changes
- added shared replay-bundle diff helpers in `trendlab-artifact` so command surfaces can compare manifest, summary, and ledger changes without CLI-owned schema rules
- added canonical daily-bar audit helpers in `trendlab-data` so price-space and bar-shape checks stay in the data layer instead of the CLI
- added `trendlab-cli diff <left-bundle-dir> <right-bundle-dir>` for artifact-backed run comparison
- added `trendlab-cli audit data <bundle-dir>` for replay-bundle data auditing of raw versus analysis price-space differences and basic bar integrity
- tightened `trendlab-cli explain <bundle-dir>` to surface manifest parameters, gap policy, cost inputs, warnings, and stop state alongside ledger reasoning
- added deterministic artifact, data-layer, and CLI coverage for the Week 13 diff and audit flows
- closed the M4 gate and moved the active next step to Week 14 TUI shell work
- re-verified `cargo xtask validate` after the Week 13 CLI completion changes
- created the `trendlab-tui` workspace crate and wired it into default workspace validation
- added a Week 14 ratatui shell that reopens shared replay bundles without TUI-local artifact parsing
- added a keyboard-first focus model with `Results`, `Ledger`, and `Help` panes plus first-pass provenance, warning, data-audit, and ledger-reasoning visibility
- added deterministic TUI state and render coverage for the Week 14 shell
- completed the Week 14 TUI shell slice and moved the active next step to Week 15 audit/chart inspection work
- re-verified `cargo xtask validate` after the Week 14 TUI shell changes
- added a Week 15 chart pane in `trendlab-tui` that plots replay-bundle raw closes, analysis closes, active stops, fills, and the currently selected bar without adding TUI-local artifact parsing
- synced chart and ledger selection so visual price inspection and persisted row reasoning stay on the same keyboard-first cursor
- replaced the focus-swapped detail pane with a persistent audit panel that keeps provenance, warnings, data-audit summary, and selected-row reasoning visible across panes
- updated deterministic TUI coverage for the Week 15 chart and audit layout
- re-verified `cargo xtask validate` after the Week 15 TUI inspection changes
- pivoted the left TUI sidebar from a summary-only list toward an inspect navigator that includes derived trade summaries alongside run-level checkpoints
- synced trade selection from the inspect navigator back into the chart and ledger so the Week 15 shell can move between run-level and trade-level inspection without losing auditability
- closed the M5 gate and moved the active next step to the first Week 16 / M6 re-baseline pass
- re-verified `cargo xtask validate` after the Week 15 navigation-model changes
- re-baselined the first M6 block in `docs/Roadmap.md` so Weeks 16-19 now form a concrete research sequence instead of a generic low-confidence placeholder
- narrowed the immediate next work from "re-baseline M6" to implementing Week 16 deterministic cross-symbol aggregation with explicit drill-down back to existing replay bundles
- updated stale M6 open-question notes in `docs/Plan.md` so they no longer refer to pre-TUI or during-M5 checkpoints
- re-verified `cargo xtask validate` after the Week 16 planning updates
- added `trendlab-cli research aggregate <bundle-dir> <bundle-dir> ...` for the first deterministic curated-multi-symbol aggregation path on top of existing single-symbol replay bundles
- enforced that the first aggregation pass only combines comparable bundles with matching snapshot, provider, engine version, date range, gap policy, reference-flow shape, cost model, and historical limitations
- kept aggregate drill-down explicit by surfacing per-symbol member rows with direct replay-bundle paths instead of inventing a second persisted trust boundary
- added deterministic CLI coverage for aggregate success and mismatch rejection paths
- completed Week 16 and moved the active next step to Week 17 walk-forward orchestration
- re-verified `cargo xtask validate` after the Week 16 aggregation changes
- added `trendlab-cli research walk-forward --train-bars <n> --test-bars <n> [--step-bars <n>] <bundle-dir> ...` on top of the comparable research-bundle path
- implemented deterministic walk-forward split generation with explicit train/test row spans, train/test date spans, and child bundle links for each split
- enforced that walk-forward runs only operate on comparable bundles with matching ledger date sequences before split generation proceeds
- added deterministic CLI coverage for walk-forward split generation and mismatched date-sequence rejection
- completed Week 17 and moved the active next step to Week 18 bootstrap confidence work
- re-verified `cargo xtask validate` after the Week 17 walk-forward changes
- added `trendlab-cli research bootstrap aggregate --samples <n> [--seed <n>] <bundle-dir> ...` and `trendlab-cli research bootstrap walk-forward --samples <n> [--seed <n>] --train-bars <n> --test-bars <n> [--step-bars <n>] <bundle-dir> ...`
- kept the Week 18 bootstrap path CLI-local and auditable by reusing the comparable bundle set, preserving baseline member and split drill-down back to the original replay bundles
- added deterministic seeded bootstrap summaries for aggregate member net-equity changes and walk-forward split test-window average net-equity changes without mutating the baseline aggregate or walk-forward reports
- added deterministic CLI coverage for seeded aggregate bootstrap reproducibility and seeded walk-forward bootstrap split drill-down
- completed Week 18 and moved the active next step to Week 19 separated leaderboard work
- re-verified `cargo xtask validate` after the Week 18 bootstrap changes
- added standardized strategy-component manifest parameters for CLI-produced research bundles so signal, filter, position-manager, and execution-model attribution can be carried without guessing
- added `trendlab-cli research leaderboard <signal|position-manager|execution-model|system> <bundle-dir> ...` on top of a comparable attributed bundle path
- enforced that signal, position-manager, and execution-model leaderboards only rank bundles when the non-target component context stays fixed across the compared set, and that each leaderboard row preserves distinct-symbol drill-down back to the original replay bundles
- added deterministic CLI coverage for signal, execution-model, and combined-system leaderboard ranking plus missing-attribution and mixed-context rejection
- completed Week 19 and moved the active next step to the Week 20 post-block checkpoint
- re-verified `cargo xtask validate` after the Week 19 leaderboard changes
- completed the Week 20 post-block checkpoint and explicitly deferred point-in-time universe work until the repo has an honest universe snapshot and historical membership model
- re-baselined Weeks 21-24 in `docs/Roadmap.md` around shared research-report ownership, reopenability, and provenance hardening rather than premature point-in-time breadth
- updated `docs/Plan.md` so Week 21 shared research-report ownership is now the active next implementation slice
- re-verified `cargo xtask validate` after the Week 20 checkpoint doc updates
- added shared research report types plus `research.json` load/write helpers in `trendlab-artifact` for aggregate, walk-forward, bootstrap, and leaderboard summaries
- rewired `trendlab-cli research aggregate`, `walk-forward`, `bootstrap`, and `leaderboard` to build the shared report models and optionally persist them with `--output <dir>`
- added deterministic artifact and CLI coverage for shared research report round-trip and reopen flows across aggregate, walk-forward, bootstrap, and leaderboard outputs
- updated `docs/Artifacts.md` and `docs/Plan.md` so the repo contract now reflects shared research-report ownership and the Week 22 next slice
- completed Week 21 and moved the active next step to Week 22 normalized research execution and reopen flow
- re-verified `cargo xtask validate` after the Week 21 shared research-report changes
- added `trendlab-cli research explain <report-dir>` so persisted `research.json` outputs reopen through the shared `trendlab-artifact` load path instead of CLI-local reconstruction
- normalized saved research-report rendering so generated `--output` flows and reopened explain flows share the same report view across aggregate, walk-forward, bootstrap, and leaderboard outputs
- added deterministic CLI coverage proving aggregate, walk-forward, bootstrap aggregate, bootstrap walk-forward, and leaderboard reports reopen through the shared research-report path
- updated `docs/Plan.md` so Week 23 provenance and compatibility hardening is now the active next implementation slice
- completed Week 22 and moved the active next step to Week 23 provenance and compatibility hardening
- re-verified `cargo xtask validate` after the Week 22 normalized research execution and reopen changes
- hardened `trendlab-artifact` research-report validation so aggregate totals, walk-forward split structure, bootstrap distribution shape, and leaderboard report invariants reject inconsistent `research.json` payloads on write and load
- hardened `trendlab-cli research explain` so reopened research reports reconcile stored provenance against the linked replay bundles instead of trusting stale paths, stale manifests, or missing strategy-component attribution
- added regression coverage for Week 23 failure paths, including missing aggregate member bundles, malformed leaderboard report payloads, and reopened leaderboard bundles missing required strategy attribution
- updated `docs/Artifacts.md` and `docs/Plan.md` so the repo contract now reflects Week 23 provenance-safe reopen behavior and the Week 24 checkpoint handoff
- completed Week 23 and moved the active next step to Week 24 final M6 checkpoint and roadmap reset
- re-verified `cargo xtask validate` after the Week 23 provenance and compatibility hardening changes
- completed Week 24 and closed the M6 gate without forcing point-in-time universes into the roadmap before the prerequisite data model exists
- explicitly deferred point-in-time universe work beyond the current roadmap until an honest universe snapshot representation and historical-membership model exist
- added the first post-M6 planning horizon in `docs/Plan.md` and `docs/Roadmap.md` around operator-facing run specs, artifact portability and integrity, strategy-oracle hardening, and an honest real-fetch `validate-live` lane
- re-verified `cargo xtask validate` after the Week 24 checkpoint and roadmap-reset updates
- added `trendlab-cli run --spec <path>` as the first operator-facing run-spec path on top of the existing core `RunRequest` boundary
- supported both inline request specs and spec-relative `request_path` specs so operator input can stay self-contained or reference low-level requests explicitly
- kept strategy-component attribution in standardized shared-manifest parameters instead of introducing first-class manifest fields in this pass
- added deterministic CLI coverage for inline spec runs, relative request-path spec runs, and spec-mode rejection of conflicting CLI manifest overrides
- re-verified `cargo xtask validate` after the Week 25 operator-facing run-spec changes
- added semantic manifest, summary, and ledger integrity fingerprints to replay-bundle descriptors and verify them on reopen when present
- normalized shared research-report bundle links relative to the report directory when possible, with absolute fallback when relative normalization is not portable
- moved missing-linked-bundle and linked-bundle-drift rejection into `trendlab-artifact` so shared research-report reopen paths fail before CLI-local provenance checks
- added artifact and CLI regression coverage for replay integrity drift, moved report trees, missing linked bundles, and stale linked-bundle content
- completed Week 26 and moved the active next step to Week 27 strategy fixture and oracle hardening
- re-verified `cargo xtask validate` after the Week 26 portability and integrity changes
- added a narrow strategy replay runner in `trendlab-core` that replays composite strategy decisions into shared ledger rows without introducing a second artifact boundary
- added deterministic disk-backed strategy fixtures for close-confirmed next-open entry, blocked filter rejection, and carried stop-entry duplicate blocking under `fixtures/`
- added the first hand-authored strategy oracle for the carried stop-entry duplicate-block case, including replay-bundle round-trip coverage through `trendlab-testkit`
- verified that strategy fixture bundles carry standardized `strategy.signal_id`, `strategy.filter_id`, `strategy.position_manager_id`, and `strategy.execution_model_id` manifest parameters
- completed Week 27 and moved the active next step to Week 28 live-provider smoke hardening
- re-verified `cargo xtask validate` after the Week 27 strategy fixture and oracle changes
- added a real Tiingo historical-prices fetch path in `trendlab-data` that converts provider rows into stored symbol history before normalization
- extended `cargo xtask validate-live --provider tiingo` so the optional smoke lane now fetches, ingests, normalizes, and resamples real provider data instead of stopping at env validation
- added deterministic live-fetch parsing and conversion tests plus re-verified the no-token failure path for `cargo xtask validate-live --provider tiingo`
- completed Week 28, closed the M7 gate, and moved the active next step to the Week 29 post-M7 snapshot-capture checkpoint
- re-verified `cargo xtask validate` after the Week 28 live-provider smoke changes
- froze the Week 29 snapshot-capture contract around `trendlab-data` ownership instead of `trendlab-artifact` ownership
- chose an inspectable `snapshot.json` plus `daily/*.jsonl` and `actions/*.jsonl` layout for the first persisted live-snapshot slice, with normalization and resampling recomputed on reopen
- chose `xtask` as the first narrow live-snapshot capture entrypoint while later CLI and TUI snapshot flows remain consumers of shared/data-layer helpers
- completed Week 29 and moved the active next step to Week 30 first persisted live-snapshot write path
- re-verified `cargo xtask validate` after the Week 29 checkpoint doc updates
- added persisted snapshot bundle descriptor types plus `snapshot.json` / `daily/*.jsonl` / `actions/*.jsonl` write-load helpers in `trendlab-data`
- added deterministic snapshot round-trip and malformed-layout rejection coverage for the first offline reopen path
- added `cargo xtask capture-live-snapshot --provider tiingo --output <dir>` so the live Tiingo path can fetch, persist, reopen, normalize, and resample one stored snapshot outside `cargo xtask validate`
- completed Week 30 and moved the active next step to Week 31 stored-snapshot inspect and audit flow
- re-verified `cargo xtask validate` after the Week 30 snapshot write/load changes
- added a shared snapshot-inspection report path in `trendlab-data` that derives normalization, resampling, action-effect, and daily-bar audit summaries from reopened stored snapshots
- added deterministic inspection coverage proving stored snapshots surface provider identity, requested window, action counts, normalization inputs, and audit-safe analysis-space differences
- added `cargo xtask inspect-snapshot <snapshot-dir>` so stored snapshots can be reopened and inspected offline without duplicating snapshot parsing rules in operator surfaces
- completed Week 31 and moved the active next step to Week 32 snapshot-backed operator flow
- re-verified `cargo xtask validate` after the Week 31 snapshot inspection changes
- added `trendlab-cli audit snapshot <snapshot-dir>` as the first operator-facing CLI path that targets stored snapshots through shared/data-layer reopen helpers instead of ad hoc live fetches
- added deterministic CLI coverage proving snapshot-backed audit surfaces provider identity, requested window, stored raw-bar counts, corporate-action counts, and normalization inputs without CLI-local snapshot parsing
- completed Week 32, closed the M8 gate honestly without point-in-time universes, and moved the active next step to a post-M8 roadmap reset
- re-verified `cargo xtask validate` after the Week 32 snapshot-backed operator changes
- completed the Week 33 post-M8 checkpoint and re-baselined the next milestone block around snapshot-backed operator runs instead of speculative post-M8 breadth
- added the M9 milestone definition and Weeks 33-36 roadmap slice for snapshot-backed operator sources and runs
- kept point-in-time universes explicitly deferred because the prerequisite universe snapshot and historical-membership model still do not exist
- re-verified `cargo xtask validate` after the Week 33 planning updates
- added `trendlab-data` snapshot run-source resolution helpers that reopen stored snapshots, require explicit symbol and exact start/end bar selection, and derive canonical daily bars without moving snapshot parsing into operator surfaces
- extended `trendlab-cli run --spec <path>` with a first snapshot-backed `snapshot_source` plus `request_template` path on top of the existing operator spec instead of inventing a second operator surface
- kept snapshot-backed replay provenance explicit through standardized manifest parameters for source kind, request/spec origin, snapshot source path, and selected start/end dates while sourcing `snapshot_id` and `provider_identity` from stored snapshots
- added deterministic data-layer and CLI coverage for snapshot-backed run success, missing-symbol rejection, off-bar date rejection, and manifest-override rejection
- re-verified `cargo xtask validate` after the Week 34 source boundary and first snapshot-backed run changes
- hardened snapshot-backed run spec loading so empty `snapshot_source` fields fail explicitly before snapshot reopen is attempted
- tightened `trendlab-cli explain <bundle-dir>` so snapshot-backed runs surface run-source kind, request/spec origin, snapshot source path, and selected snapshot slice as first-class audit lines instead of only burying them in manifest parameters
- added deterministic CLI coverage for snapshot-backed explain provenance and malformed snapshot-source rejection
- completed Week 36, closed the M9 gate honestly, and moved the active next step to the post-M9 checkpoint and roadmap reset
- re-verified `cargo xtask validate` after the Week 36 hardening and M9 close
- re-baselined the post-M9 horizon around a TUI-driven operator lab instead of leaving the roadmap unscheduled
- added M10, M11, and M12 to `docs/Plan.md` for shared operator orchestration, a TUI run workspace, and the first TUI operator-lab loop
- extended `docs/Roadmap.md` with concrete Weeks 37-46 slices covering shared operator extraction, TUI run mode, snapshot selection, run launch, prior-run reopen, and hardening
- added `trendlab-operator` to `docs/Workspace.md` as the shared run-orchestration crate consumed by both CLI and TUI
- locked the post-M9 horizon to shared-library orchestration, snapshots-first inputs, CLI-owned research execution, and no live-capture work inside the first TUI-runner block
- created the `trendlab-operator` workspace crate for shared operator run orchestration
- moved the typed operator run-spec model, snapshot-backed spec validation, request/spec resolution, manifest provenance construction, reference-flow execution, and replay-bundle write path out of CLI-only ownership
- rewired `trendlab-cli run` to call `trendlab-operator::execute_run` while preserving existing request-path, inline-spec, relative request, and snapshot-backed run behavior
- kept stored-snapshot parsing and exact symbol/date slice resolution in `trendlab-data`, and kept replay-bundle schema/write ownership in `trendlab-artifact`
- added direct deterministic operator-crate coverage for replay-bundle writing from a request path
- re-verified `cargo xtask validate` after the Week 38 shared-operator extraction
- added a presentation-neutral `RunExecutionReport` and `RunExecutionProvenance` handoff from `trendlab-operator` so CLI and TUI callers can consume output path, summary values, run-source kind, request/spec source, and snapshot slice provenance without re-parsing manifest parameters
- updated `trendlab-cli run` to format its success output from the shared operator report instead of rebuilding those fields locally
- added direct `trendlab-operator` parity coverage for inline specs, relative request-path specs, and snapshot-backed specs
- re-verified `cargo xtask validate` after the Week 39 operator handoff and parity coverage
- added top-level TUI Home, Inspect, and Help modes while preserving the existing inspect-shell state and render path for loaded replay bundles
- allowed `trendlab-tui` to start without command-line arguments and render a non-empty operator Home shell with no replay bundle loaded
- kept `trendlab-tui open <bundle-dir>` and direct `<bundle-dir>` startup paths opening the existing replay-bundle inspection shell
- added deterministic TUI state and render coverage for no-bundle startup and top-level mode switching
- re-verified `cargo xtask validate` after the Week 40 TUI shell restructure
- added `trendlab-tui --snapshot <snapshot-dir>` startup support for configuring stored snapshot directories in the TUI shell
- added a Home-mode snapshot browser that loads configured snapshot directories through `trendlab-data::snapshot_store::load_snapshot_bundle` and `trendlab-data::inspect::inspect_snapshot_bundle`
- surfaced snapshot id, provider identity, requested window, capture metadata, symbol counts, raw-bar counts, corporate-action counts, resampled counts, and analysis-adjustment summary data in the TUI snapshot summary pane
- kept malformed snapshot selections visible as invalid browser entries with explicit error text instead of adding TUI-local snapshot parsing or failing the whole shell
- added deterministic TUI coverage for valid snapshot summaries, malformed snapshot entries, and combined bundle/snapshot startup arguments
- re-verified `cargo xtask validate` after the Week 41 TUI snapshot browser and summary work
- added `trendlab-data` snapshot run-form option helpers so operator surfaces can consume exact symbol/date choices without reopening snapshot schema rules locally
- added `trendlab-operator::preview_run_spec` so the TUI can validate trusted snapshot-backed run specs without launching a run or writing artifacts
- added a Home-mode run form in `trendlab-tui` for one-symbol selection, exact start/end date selection, request-template field editing, and operator-backed readiness/error display
- added deterministic TUI coverage for a ready snapshot-backed run preview and a blocked invalid run configuration before launch
- added `trendlab-operator::execute_run_spec` so non-CLI callers can execute an in-memory operator spec through the shared operator path without writing temp spec files
- added a deterministic default TUI run output root under `target/tui-runs`, with collision-safe bundle directory suffixing for repeated launches
- added a first Home-mode launch command in `trendlab-tui` that executes through `trendlab-operator`, auto-opens successful bundles in Inspect mode, and keeps failed launches visible in Home mode
- added deterministic TUI coverage for successful in-session launch plus blocked launch attempts that retain explicit error messaging
- added a Home-mode run-history browser in `trendlab-tui` over the documented `target/tui-runs` output root
- kept malformed or drifted prior-run bundle directories visible as invalid history entries with explicit error text instead of hiding them from the operator
- surfaced prior-run summary and provenance preview lines in Home mode before reopen, including run-source kind, request/spec source, snapshot source path, selected snapshot slice, row count, warnings, and ending equity
- added contextual reopen from the Home-mode history list back into the existing inspect shell without adding a second result viewer or TUI-local artifact store
- added deterministic TUI coverage for valid history listing, invalid prior-run visibility, and successful prior-run reopen
- tightened the Home-mode operator footer so the active focus, selected snapshot, next launch target, and selected history target stay visible while navigating snapshots, run configuration, and prior runs
- surfaced snapshot-backed launch context more explicitly in Home-mode preview and validation panes, including output-root visibility for blocked launches and selected snapshot slice visibility for ready launches
- added inspect-header and audit-pane provenance summaries so TUI-launched and TUI-reopened runs keep run-source, request-source, spec-source, snapshot-source, and snapshot-selection context visible in the existing inspect shell
- added deterministic TUI coverage for explicit Home footer context plus preserved snapshot-backed provenance after both in-session launch and history-based reopen
- added deterministic end-to-end TUI coverage for a non-default stored snapshot selection flowing through Home-mode configuration, replay-bundle write, and Inspect-mode reopen
- added explicit TUI rejection coverage for launch attempts with no selected snapshot, an invalid selected snapshot directory, and malformed run-form inputs with zero entry shares
- hardened the Home footer render height so the active snapshot, launch target, and history target remain visible at the covered operator-shell size
- closed Week 46 and recorded that M10, M11, and M12 now complete honestly from the implemented state
- completed the post-M12 checkpoint and froze the next concrete milestone block around TUI research-report reopen and audit rather than broader operator-source expansion
- added M13 to `docs/Plan.md` for shared research-report reopen, audit-first rendering, and linked replay-bundle drilldown inside the TUI
- extended `docs/Roadmap.md` with concrete Weeks 47-51 slices for research-report open-path detection, report-shell rendering, replay drilldown, and hardening
- kept saved research execution CLI-owned and kept point-in-time universes deferred in the new horizon definition
- extended `trendlab-tui` startup/open-path detection so it now distinguishes shared replay bundles from shared research-report bundles using the documented artifact-file layout
- added a first `Research` app mode in `trendlab-tui` that reopens saved `research.json` bundles through `trendlab-artifact` and renders a minimal non-empty report shell
- kept the replay-bundle inspect path intact while exposing report kind, report summary context, and linked replay-bundle counts plus paths in the new research shell
- added deterministic TUI coverage for replay-versus-report startup detection, invalid artifact-directory rejection, and successful research-report startup rendering
- expanded the `trendlab-tui` research mode into a three-pane audit shell with summary, item-list, and detail panes
- rendered kind-specific report summaries and item/detail content for aggregate, walk-forward, bootstrap aggregate, bootstrap walk-forward, and leaderboard research bundles directly from shared report models
- added deterministic TUI coverage for research-shell pane navigation and render behavior across the supported report kinds without adding replay drilldown early
- added linked replay-bundle selection to research-shell detail panes so report members, splits, and leaderboard rows can surface their concrete replay targets explicitly
- reopened selected research-shell replay targets through the shared replay-bundle load path into the existing inspect shell instead of inventing a second result viewer
- added deterministic TUI coverage for successful research-to-replay drilldown and for failed drilldown when a linked replay bundle disappears after report load
- added deterministic startup-to-inspect TUI coverage for report open -> summary shell -> linked replay drilldown -> inspect open
- added deterministic TUI rejection coverage for malformed `research.json` startup bundles in addition to broken linked-bundle provenance
- closed Week 51 and recorded that M13 now completes honestly from the implemented state without pulling research execution into the TUI
- completed the post-M13 checkpoint and re-baselined the next roadmap around a local-first, near-real-time research-lab horizon
- added M14 through M18 to `docs/Plan.md` for live market intake, fresh-snapshot research execution, materialized leaderboard state, local jobs, and TUI monitoring
- extended `docs/Roadmap.md` with concrete Weeks 52-72 slices for the committed next horizon plus optional later Week 73-80 slices for point-in-time universes and automated search
- added `trendlab-jobs` to `docs/Workspace.md` as the planned shared local background-orchestration boundary for snapshot refresh, research execution, and leaderboard refresh jobs

## Week 0 Closure Decisions

- artifact bundle encoding:
  - directory bundle with `bundle.json`, `manifest.json`, `summary.json`, and `ledger.jsonl`
  - JSON for bundle, manifest, and summary; JSON Lines for the ledger
- normalized market-data snapshot path:
  - `snapshot.json` plus per-symbol JSON Lines under `daily/` and `actions/`
  - the first persisted slice stores stored raw bars and corporate actions, while normalization and resampling remain derived on reopen
- M1 crate set:
  - `xtask`
  - `trendlab-core`
  - `trendlab-artifact`
  - `trendlab-testkit`
- initial M1 interface notes:
  - `trendlab-core` starts with a pure fixture-driven engine entrypoint for the M1 reference flow
  - `trendlab-artifact` starts with manifest, summary, persisted ledger-row, and bundle-descriptor serde types
  - `trendlab-testkit` starts with fixture loading plus golden/oracle diff helpers
- first M1 reference flow:
  - fixture-authored end-of-day entry-intent stream while flat
  - one share per entry
  - fixed protective stop at 10 percent below entry
  - zero commissions and zero slippage
  - default gap policy from `docs/BarSemantics.md`
- first three M1 golden scenarios:
  1. `m1_entry_hold_open_position`
  2. `m1_intrabar_stop_exit`
  3. `m1_gap_through_stop_exit`
- first oracle target:
  - `m1_intrabar_stop_exit` is the first hand-authored oracle scenario
- initial validation shape:
  - `cargo xtask validate` runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`
  - `cargo xtask validate-live --provider tiingo` is reserved for M2 smoke checks and must never be called by `cargo xtask validate`

## M1 Gate Checklist

- complete: workspace exists with `xtask`, `trendlab-core`, `trendlab-artifact`, `trendlab-testkit`, and initial test support
- complete: one-symbol daily-bar simulation runs from fixtures only
- complete: queued market-entry flow and fixed protective stop both work under the documented bar semantics
- complete: event ledger and replay bundle are persisted through shared artifact schemas
- complete: at least one hand-authored oracle scenario exists for the first truthful path
- complete: `cargo xtask validate` is wired and remains deterministic and network-free
- complete: golden tests can diff expected versus actual ledger output

## M2 Gate Checklist

- complete: provider-native types stay outside `trendlab-core`
- complete: canonical market types from `trendlab-core` are populated by `trendlab-data`
- complete: raw daily bars, explicit corporate-action effects, and derived analysis-close series are represented in the data layer
- complete: weekly and monthly resampling is implemented and tested on deterministic fixtures
- complete: the Tiingo live-provider smoke lane exists behind `cargo xtask validate-live` and remains outside `cargo xtask validate`

## M3 Gate Checklist

- complete: signals, filters, position managers, and execution models compose independently in `trendlab-core`
- complete: close-confirmed next-open and stop-entry breakout families are represented as separate execution paths
- complete: position-manager behavior remains isolated from signal and execution logic in the current strategy surface
- complete: execution models consume signal decisions without rewriting the original signal output
- complete: strategy evaluations keep signal, filter, position, and execution decisions explicit and deterministic under unit coverage

## M4 Gate Checklist

- complete: CLI can run, explain, diff, and audit data using shared replay bundles
- complete: CLI output can surface ledger reasoning, warnings, and metric inputs without inventing command-local schema rules
- complete: replay-bundle diffing lives in `trendlab-artifact`, and data-audit logic lives in `trendlab-data`
- complete: deterministic validation covers the full M4 command set and shared-artifact reopen flows

## M5 Gate Checklist

- complete: TUI shell supports keyboard-first navigation and audit-first inspection
- complete: results, charts, help, and audit views exist as one coherent shell
- complete: `trendlab-tui` reopens shared run artifacts without parallel parsing rules
- complete: audit views preserve run provenance and warnings while moving across panes

## M6 Gate Checklist

- complete: cross-symbol aggregation, walk-forward validation, bootstrap confidence, and separated leaderboards operate on top of the stable deterministic core
- complete: shared `research.json` report ownership plus `research explain` preserve drill-down from research summaries back to replay-bundle truth
- complete: malformed, stale, or under-attributed research inputs now reject explicitly instead of reopening or ranking dishonestly
- complete: M6 closes honestly without point-in-time universes because the milestone only schedules them when the prerequisite universe snapshot and historical-membership model are stable enough

## M7 Gate Checklist

- complete: operator-facing run specs map cleanly onto shared and core request types without creating CLI-owned truth
- complete: replay and research artifacts fail explicitly on broken links or integrity drift
- complete: strategy-layer behavior is covered by deterministic fixture or oracle scenarios in addition to unit-only coverage
- complete: `cargo xtask validate-live --provider tiingo` exercises a real provider fetch path while remaining outside `cargo xtask validate`
- complete: point-in-time universes remain deferred until the universe snapshot and historical-membership model are honest enough to support them

## M8 Gate Checklist

- complete: live-fetched symbol history can be written and reopened through the documented snapshot ownership path without hidden provider refetches
- complete: stored snapshots preserve provider identity, raw bars, corporate actions, and derived normalization inputs explicitly enough for audit
- complete: operator-facing audit flows can consume stored snapshot paths without creating CLI-owned market-data truth
- complete: `cargo xtask validate` remains deterministic and network-free while live snapshot capture stays explicitly optional
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## M9 Gate Checklist

- complete: operators can launch replayable runs from stored snapshots without provider refetches or copied low-level bar payloads
- complete: snapshot-backed operator flows keep provider-native types, filesystem details, and snapshot parsing rules out of `trendlab-core`
- complete: replay manifests and explain surfaces disclose the stored snapshot source, selected symbol, selected date slice, and resulting run provenance explicitly enough for audit
- complete: default validation remains deterministic and network-free by relying on stored snapshots or fixtures
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## M10 Gate Checklist

- complete: CLI and TUI call the same shared operator library for run execution
- complete: `trendlab-operator` reuses `trendlab-data` snapshot resolution and `trendlab-artifact` replay writes instead of re-owning those boundaries
- complete: TUI run execution does not require CLI subprocess invocation
- complete: shared operator outputs remain deterministic and provenance-safe
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## M11 Gate Checklist

- complete: the TUI can browse and select a stored snapshot
- complete: the TUI can configure the current trusted snapshot-backed run inputs interactively
- complete: the TUI can launch a run and transition directly into result inspection
- complete: validation and error messages remain explicit and audit-safe
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## M12 Gate Checklist

- complete: the TUI can launch new runs and reopen prior runs from one operator shell
- complete: the existing inspection shell remains the result viewer after TUI-launched runs
- complete: the TUI does not introduce duplicate run logic, snapshot parsing, or artifact parsing
- complete: default validation remains deterministic and network-free
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## M13 Gate Checklist

- complete: the TUI can reopen saved shared research-report bundles through `trendlab-artifact`
- complete: the TUI can inspect report-kind summaries without CLI-local reconstruction rules
- complete: linked replay-bundle drilldown reuses the existing inspect shell
- complete: research execution remains CLI-owned while the TUI extends its audit surface
- complete: default validation remains deterministic and network-free
- complete: point-in-time universes remain deferred until the repo has an honest universe snapshot representation plus historical-membership model

## Next Planned Step

Begin Week 53 and start M14 live market intake and snapshot freshness:

1. freeze the shared live snapshot refresh request shape, freshness metadata, and stale or partial failure semantics
2. keep `trendlab-data` as the owner of provider fetch, snapshot truth, and freshness inspection
3. keep research execution CLI-owned or background-worker-owned through M14 so the TUI remains a client rather than an execution boundary
4. keep point-in-time universes and automated search deferred into M19 and M20 unless a later re-baseline deliberately pulls them forward

## Setup Verification

- `codex mcp list` from the repo root still reported `No MCP servers configured yet` in this CLI session, so project-local Codex MCP activation remains unverified here.
- `codex mcp list` outside the repo also reported no MCP servers, confirming the global home-directory config was not changed by this setup.
- `.cursor/mcp.json` and `.cursor/hooks.json` both parsed successfully as valid JSON.
- `.cursor/mcp.json` is repo-local and intentionally does not mirror the user's global `MCP_DOCKER` or `dart` servers.
- `.cursor/hooks.json` is repo-local and uses Windows PowerShell hook scripts under `.cursor/hooks/`.
- `beforeShellExecution` denied a synthetic `git reset --hard` payload and allowed a synthetic `git status` payload in stdin-fed tests.
- `afterFileEdit` emitted the expected advisory reminder for `AGENTS.md`, and `stop` emitted the expected `docs/Status.md` reminder when fed matching session state.
- `cargo xtask validate-live --provider tiingo` now prints the Tiingo smoke plan, fails cleanly without `TIINGO_API_TOKEN`, and when configured exercises a real fetch plus ingest/normalize/resample pipeline without affecting normal validation.

## Open Risks

- the first live-snapshot slice now favors inspectable JSON and JSONL over a columnar format; if capture volume grows materially, later storage-efficiency work may still justify a more compact encoding
- the current replay and research integrity fingerprints are aimed at accidental drift and stale-content detection; later real-world trust needs may still justify stronger cryptographic digests or signatures
- the new strategy replay runner is intentionally narrow and currently centered on entry-oriented strategy flows; later exit-oriented execution or richer position-management scenarios may still justify another pass
- the current warning storage is sufficient for the fixture-driven kernel and M2 data layer, but M3 filters and blocked-trade explanations may introduce new warning categories
- higher-timeframe bars currently carry raw OHLC plus period-end `analysis_close`; if later strategy work needs weekly or monthly analysis open/high/low semantics, that contract still needs to be made explicit
- the fixture scenarios are intentionally small; later provider and action edge cases may still want one more data-layer fixture beyond the current split/dividend and resampling cases
- the real `cargo xtask validate-live` path now depends on Tiingo endpoint health, token validity, and external response shape, so smoke failures may reflect provider-side variability instead of local regressions
- the current snapshot-backed operator path is still intentionally single-symbol, daily-bar, and template-driven; any broader source polymorphism should be treated as a new planning horizon instead of being smuggled into the closed M9 slice
- the first snapshot-backed run-source contract currently uses `snapshot_source` plus an inline `request_template`; later operator ergonomics may still want a portable `request_template_path` or a richer higher-level strategy input without copying low-level bars
- the new strategy interfaces are intentionally narrow; if CLI or later audit surfaces need richer signal metadata or exit directives, that contract may still need one more pass
- strategy composition is now covered by deterministic unit tests, but there is not yet a dedicated fixture/oracle harness for strategy-layer scenarios
- the operator-facing run spec can now source bars from stored snapshots, but later live-provider work may still want a richer source model than inline/reference-flow request templates alone
- shared research-report ownership exists in `trendlab-artifact`, but report rendering is still primarily CLI-shaped; the new TUI research-audit block may still expose a need for presentation-neutral summary helpers instead of duplicated report formatting logic
- replay-bundle data audit currently inspects persisted per-bar raw and analysis fields; if later TUI audit panels need explicit corporate-action markers inside replay artifacts, that artifact surface may need one more pass
- the post-M13 local-first horizon assumes latest-snapshot targeting can be made provenance-safe, but the exact resolution moment still needs to be frozen before M15 implementation starts
- the planned `trendlab-jobs` boundary is intentionally local-only; later service or cloud execution work may still want a different persistence or API shape
- materialized leaderboard identity and stale-result detection are now part of the committed next horizon, but the exact boundary between shared artifact metadata and local job or state records still needs to be frozen during M16 planning
- the current inspect navigator derives per-trade items from the single-position replay ledger; if later M6 work introduces multi-symbol or richer trade grouping needs, that presentation model may need one more pass
- legacy replay bundles or research reports written before the Week 26 integrity metadata still reopen through compatibility paths, but they do not get the same drift checks until they are rewritten
- the first strategy fixture harness now exists, but later strategy families may still expose a need for richer row-level reason codes or pending-order provenance than the current ledger shape carries
- point-in-time universes are now explicitly deferred beyond the current roadmap; the repo still lacks an honest universe snapshot representation and historical-membership path for research runs
- strategy-component attribution still rides in standardized manifest parameters; Week 25 keeps that shared-manifest path in place, but older-bundle compatibility or richer later audit surfaces may still justify first-class manifest fields
- repo-local `.codex/config.toml` is in place, but this Codex CLI build did not surface the repo-local MCP servers via `codex mcp list`; Cursor/IDE trusted-project behavior still needs manual confirmation
