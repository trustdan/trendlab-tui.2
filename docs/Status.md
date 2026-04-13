# Status

## Current State

Week 23 provenance and compatibility hardening is complete. The current M6 stack now includes shared reopenable aggregate, walk-forward, bootstrap, and separated leaderboard research reports plus a CLI `research explain` path that revalidates linked replay bundles, rejects malformed or under-attributed reopen inputs explicitly, and keeps point-in-time universes deferred as not yet honest enough to schedule. The next active step is Week 24 final M6 checkpoint and roadmap reset.

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

## Week 0 Closure Decisions

- artifact bundle encoding:
  - directory bundle with `bundle.json`, `manifest.json`, `summary.json`, and `ledger.jsonl`
  - JSON for bundle, manifest, and summary; JSON Lines for the ledger
- normalized market-data cache:
  - `snapshot.json` plus per-symbol Parquet under `daily/` and `actions/`
  - derived analysis series are computed from cached canonical daily data in the first M2 pass
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

## Next Planned Step

Begin Week 24 final M6 checkpoint and roadmap reset:

1. decide whether the current M6 gate is satisfied without point-in-time universes
2. record whether point-in-time universe support remains deferred into a later milestone or becomes the next planning horizon
3. update the backlog and define the next roadmap era from the actual post-Week-23 state

## Setup Verification

- `codex mcp list` from the repo root still reported `No MCP servers configured yet` in this CLI session, so project-local Codex MCP activation remains unverified here.
- `codex mcp list` outside the repo also reported no MCP servers, confirming the global home-directory config was not changed by this setup.
- `.cursor/mcp.json` and `.cursor/hooks.json` both parsed successfully as valid JSON.
- `.cursor/mcp.json` is repo-local and intentionally does not mirror the user's global `MCP_DOCKER` or `dart` servers.
- `.cursor/hooks.json` is repo-local and uses Windows PowerShell hook scripts under `.cursor/hooks/`.
- `beforeShellExecution` denied a synthetic `git reset --hard` payload and allowed a synthetic `git status` payload in stdin-fed tests.
- `afterFileEdit` emitted the expected advisory reminder for `AGENTS.md`, and `stop` emitted the expected `docs/Status.md` reminder when fed matching session state.
- `cargo xtask validate-live --provider tiingo` now prints the Tiingo smoke plan, requires `TIINGO_API_TOKEN`, and fails cleanly without affecting normal validation.

## Open Risks

- the Parquet mapping layer for normalized daily bars and corporate actions may need careful schema choices once `trendlab-data` exists
- the bundle descriptor may need checksums or file hashes later if replay-bundle integrity becomes a practical problem
- the current warning storage is sufficient for the fixture-driven kernel and M2 data layer, but M3 filters and blocked-trade explanations may introduce new warning categories
- higher-timeframe bars currently carry raw OHLC plus period-end `analysis_close`; if later strategy work needs weekly or monthly analysis open/high/low semantics, that contract still needs to be made explicit
- the fixture scenarios are intentionally small; later provider and action edge cases may still want one more data-layer fixture beyond the current split/dividend and resampling cases
- `cargo xtask validate-live` now validates the smoke shape and env prerequisite honestly, but it does not yet execute a real Tiingo HTTP fetch path
- the new strategy interfaces are intentionally narrow; if CLI or later audit surfaces need richer signal metadata or exit directives, that contract may still need one more pass
- strategy composition is now covered by deterministic unit tests, but there is not yet a dedicated fixture/oracle harness for strategy-layer scenarios
- the first CLI run path intentionally uses serialized core `RunRequest` inputs; a higher-level operator-facing run spec may still be needed before the CLI feels complete
- replay-bundle data audit currently inspects persisted per-bar raw and analysis fields; if later TUI audit panels need explicit corporate-action markers inside replay artifacts, that artifact surface may need one more pass
- the current inspect navigator derives per-trade items from the single-position replay ledger; if later M6 work introduces multi-symbol or richer trade grouping needs, that presentation model may need one more pass
- shared research-report bundle links currently preserve the explicit replay-bundle paths supplied to the CLI; if later surfaces need stronger portability or path-normalization rules, that contract may need one more pass
- point-in-time universes are now explicitly deferred; the repo still lacks an honest universe snapshot representation and historical membership path for research runs
- strategy-component attribution currently rides in standardized manifest parameters; Week 23 proves that is sufficient for current reopen and ranking checks, but older-bundle compatibility or richer later audit surfaces may still justify first-class manifest fields
- repo-local `.codex/config.toml` is in place, but this Codex CLI build did not surface the repo-local MCP servers via `codex mcp list`; Cursor/IDE trusted-project behavior still needs manual confirmation
