# Workspace Plan

This file describes the intended Rust workspace before the crates are created.

## Proposed Layout

```text
trendlab-tui.2/
  AGENTS.md
  Cargo.toml
  crates/
    trendlab-core/
    trendlab-artifact/
    trendlab-data/
    trendlab-cli/
    trendlab-tui/
    trendlab-testkit/
  xtask/
  docs/
  fixtures/
  tests/
```

## Crate Responsibilities

### `trendlab-core`

Owns:

- canonical market model types consumed by simulation
- domain types
- simulation state
- order and fill processing
- ledger generation
- replay
- run-local metrics that are inseparable from simulation truth

Must not own:

- HTTP clients
- provider-specific types
- filesystem caching policy
- ratatui code

### `trendlab-data`

Owns:

- provider-specific raw types
- provider trait implementations
- Tiingo adapter
- persisted market-data snapshots and their schema/versioning
- corporate actions ingestion
- normalization into `trendlab-core` market types
- resampling

### `trendlab-artifact`

Owns:

- run manifest schema
- persisted ledger schema
- replay bundle schema and versioning
- compatibility helpers for loading older artifacts
- artifact serialization boundaries shared by CLI and TUI

### `trendlab-cli`

Owns:

- command-line entrypoints
- explain and diff surfaces
- user-facing command UX

### `trendlab-tui`

Owns:

- ratatui application shell
- navigation
- charts
- audit panels
- presentation state

### `trendlab-testkit`

Owns:

- golden fixture helpers
- reusable ledger assertions
- synthetic market-data builders

### `xtask`

Owns:

- repo validation commands
- fixture generation helpers if needed
- lightweight ledger and replay inspection utilities
- separate live-provider smoke checks that remain outside normal validation
- repeatable local developer workflows

## Dependency Direction

Use this dependency direction and keep it one-way:

`trendlab-data` -> `trendlab-core`

`trendlab-artifact` -> `trendlab-core`

`trendlab-cli` -> `trendlab-core`, `trendlab-artifact`, `trendlab-data`

`trendlab-tui` -> `trendlab-core`, `trendlab-artifact`, `trendlab-data`

`trendlab-testkit` -> `trendlab-core`, `trendlab-artifact`

`xtask` -> workspace crates as needed for validation and tooling

`trendlab-core` should not depend on other workspace crates.

## Directory Intent

- `fixtures/` stores hand-authored or generated inputs for deterministic tests.
- `tests/` stores integration coverage that spans crates.
- `docs/` stores contracts, plans, and operating notes.

## Validation Rule

`cargo xtask validate` is the default repo check and must stay:

- deterministic
- network-free
- safe to run without secrets

Live provider checks should live behind a separate command such as `cargo xtask validate-live` and must not be required for normal milestone completion.

## Week 29 Snapshot Capture Decision

The first persisted live-snapshot slice is owned by `trendlab-data`, not by `trendlab-artifact`.

The first truthful on-disk snapshot format is a snapshot directory with a UTF-8 JSON manifest and per-symbol JSON Lines files.

Canonical layout:

```text
cache/
  snapshots/
    <snapshot-id>/
      snapshot.json
      daily/
        <SYMBOL>.jsonl
      actions/
        <SYMBOL>.jsonl
```

Rules:

- `snapshot.json` stores schema version, snapshot identifier, provider identity, symbol list, requested date window, capture metadata, and compatibility metadata for the snapshot bundle.
- `daily/<SYMBOL>.jsonl` stores the persisted stored raw daily bars for that symbol.
- `actions/<SYMBOL>.jsonl` stores the persisted stored corporate actions for that symbol.
- the first Week 30 capture path may only write one symbol, but the layout stays per-symbol so later curated multi-symbol capture does not need a second on-disk shape.
- normalization into canonical `trendlab-core` daily bars, split effects, and resampled higher-timeframe bars remains derived on reopen rather than persisted as a second trusted store.
- provider-native HTTP payloads are not the canonical persisted snapshot format.

Rationale:

- UTF-8 JSON plus JSONL keeps the first snapshot path directly inspectable during the trust-hardening phase.
- Persisting stored raw bars and corporate actions matches the existing `trendlab-data` ownership boundary and avoids inventing a second partially trusted normalized cache too early.
- Keeping normalization derived on reopen reduces early cache complexity and lowers the risk of multiple partially trusted price-space stores.
- Columnar or compressed snapshot encodings can be revisited later if the truthful reopen path proves stable and data volume makes them necessary.

## Week 29 Operator Boundary Decision

- the first live-snapshot capture entrypoint should live in `xtask` as an optional operator/developer task while the capture path is still narrow and explicitly outside normal validation
- snapshot reopen, load, and audit helpers should live in `trendlab-data`
- later CLI or TUI snapshot workflows should consume those shared/data-layer helpers instead of owning snapshot parsing rules

## Early Implementation Sequence

1. workspace root and `xtask`
2. `trendlab-core`
3. `trendlab-artifact`
4. `trendlab-testkit`
5. `trendlab-data`
6. `trendlab-cli`
7. `trendlab-tui`
