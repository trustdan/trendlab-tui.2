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
- raw cache
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

## Week 0 Data Cache Decision

The initial normalized market-data cache format is a snapshot directory with a UTF-8 JSON manifest and per-symbol Parquet files.

Canonical layout:

```text
cache/
  normalized/
    <snapshot-id>/
      snapshot.json
      daily/
        <SYMBOL>.parquet
      actions/
        <SYMBOL>.parquet
```

Rules:

- `snapshot.json` stores provider identity, schema version, snapshot identifier, and generation metadata.
- `daily/<SYMBOL>.parquet` stores canonical normalized daily bars for that snapshot.
- `actions/<SYMBOL>.parquet` stores splits and dividend cashflows for that snapshot.
- Derived analysis series are computed from cached canonical daily data in the first M2 pass rather than persisted as a separate cache.

Rationale:

- Parquet is a better fit than JSON or CSV for medium-scale historical bar storage.
- The JSON snapshot manifest keeps provenance and snapshot identity inspectable.
- Keeping analysis series derived reduces early cache complexity and lowers the risk of multiple partially trusted price-space stores.

## Early Implementation Sequence

1. workspace root and `xtask`
2. `trendlab-core`
3. `trendlab-artifact`
4. `trendlab-testkit`
5. `trendlab-data`
6. `trendlab-cli`
7. `trendlab-tui`
