# Run Artifacts

This document defines the shared persisted artifacts that both the CLI and TUI must be able to load.

## Purpose

Run artifacts are part of the trust model. A result is not just a number on a leaderboard. It is a replayable object with explicit provenance.

## Ownership

Persisted artifact schemas belong in `trendlab-artifact`.

This keeps:

- `trendlab-core` focused on simulation truth
- `trendlab-cli` focused on commands
- `trendlab-tui` focused on presentation

Neither CLI nor TUI should become the sole owner of artifact parsing or versioning, including shared research-report persistence.

## Required Artifact Pieces

Each persisted run should have three conceptual pieces:

- run manifest
- event ledger
- replay bundle

## Run Manifest Minimum

The manifest must be able to identify at least:

- artifact schema version
- engine version
- data snapshot identifier
- provider identity
- symbol or universe selection
- universe mode
- historical limitations
- date range
- strategy or reference-flow definition
- parameters
- cost model
- gap policy
- seed when randomness exists
- warnings raised during the run

When a run is produced from a componentized strategy path rather than a bare reference flow, the manifest should also carry standardized strategy-component labels inside `parameters` so later research surfaces can attribute results without guessing.

Current standardized parameter names:

- `strategy.signal_id`
- `strategy.filter_id`
- `strategy.position_manager_id`
- `strategy.execution_model_id`

## Universe Truth Labels

The manifest must always disclose universe truth explicitly.

Minimum field:

- `universe_mode`: `single_symbol`, `curated_modern`, or `point_in_time`

When the run is not point-in-time accurate, the manifest must also disclose that limitation in `historical_limitations`.

Example values:

- `survivorship_bias`
- `selection_bias`

## Event Ledger Minimum

The persisted ledger should preserve the fields required by `docs/MathContract.md` so a run can be replayed and inspected without hidden state.

## Replay Bundle

The replay bundle is the portable object the CLI and TUI reopen.

It should contain or point to:

- the manifest
- the event ledger
- run summary data
- compatibility metadata

## Week 0 Encoding Decision

The initial on-disk replay-bundle format is a directory-based bundle using UTF-8 text files.

Canonical layout:

```text
<run-dir>/
  bundle.json
  manifest.json
  summary.json
  ledger.jsonl
```

Encoding rules:

- `bundle.json` stores artifact schema version, relative file names, and compatibility metadata.
- `manifest.json` stores the run manifest as JSON.
- `summary.json` stores run-summary data as JSON.
- `ledger.jsonl` stores one persisted ledger row per line as JSON Lines.
- M1 does not compress bundles by default.

Rationale:

- JSON and JSONL keep the trust surface inspectable.
- JSONL allows streaming large ledgers without loading the whole replay into memory.
- A directory bundle is easy for the CLI, TUI, tests, and manual inspection to share without inventing parallel parsing rules.

## Research Report Bundle

Aggregate, walk-forward, bootstrap, and leaderboard summaries are also shared artifacts.

They must stay thin:

- report-level metadata and computed summary fields
- explicit links back to the existing replay bundles they summarize
- no duplicated per-bar ledgers or shadow run manifests

The initial shared research-report path is a directory bundle using one UTF-8 JSON file.

Canonical layout:

```text
<report-dir>/
  research.json
```

Encoding rules:

- `research.json` stores the artifact schema version and one shared `ResearchReport` payload.
- the payload kind is one of `aggregate`, `walk_forward`, `bootstrap_aggregate`, `bootstrap_walk_forward`, or `leaderboard`
- child replay-bundle links preserve the explicit replay-bundle paths used to build the research report
- shared research-report writes and loads validate structural invariants instead of treating `research.json` as trusted opaque text
- reopen surfaces must reject research reports whose linked replay bundles are missing or whose replay provenance no longer reconciles honestly with the stored summary

Rationale:

- research summaries become reopenable without CLI-local reconstruction rules
- report ownership stays in `trendlab-artifact`, so later CLI and TUI surfaces can share the same load path
- the research bundle stays intentionally thin and auditable by linking back to replay truth instead of replacing it

## Versioning Rules

- artifact schema versioning is explicit
- breaking changes require a version bump
- loading older artifacts should be a deliberate compatibility decision, not an accident
- provider-native payloads should not become the canonical replay format
