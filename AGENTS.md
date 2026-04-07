# TrendLab Agent Guide

This repository is the planning-stage rebuild of TrendLab in Rust with a ratatui interface.

## Mission

Build an auditable trend-research engine that is trustworthy before it is feature-rich.

## Current Build Order

1. Truthful lab
2. CLI
3. TUI shell
4. Research-grade extensions

Do not skip ahead just because a later surface feels more visible.

## Required Reading

Read these before starting substantive work:

1. `docs/Status.md`
2. `docs/Plan.md`
3. `docs/Roadmap.md`
4. `docs/Workspace.md`
5. `docs/Artifacts.md`
6. `docs/MathContract.md`
7. `docs/BarSemantics.md`
8. `docs/Invariants.md`

For planning or session setup work, also read:

1. `docs/Prompt.md`
2. `docs/Implement.md`
3. `docs/background.md`
4. `docs/vibe-coding-start-point.md`

## Non-Negotiable Constraints

- Rust workspace from day one.
- `trendlab-core` stays deterministic and free of network, provider, and UI concerns.
- `trendlab-core` owns the canonical market model types consumed by simulation.
- Market-data providers are adapters, not the trust boundary.
- Persisted run artifacts are shared schema types, not CLI-owned types.
- Signals, position management, execution, and filters stay separate.
- Daily-bar semantics are explicit and conservative.
- Fills use raw tradable prices.
- Signals may use an analysis series.
- Same-bar stop tightening is not allowed unless the contract is explicitly changed.
- Replayable ledgers are required for trust.
- `cargo xtask validate` must stay deterministic and network-free.

## Working Rules

- Plan before code for any non-trivial change.
- Keep diffs scoped to the active milestone.
- If you change math, bar semantics, or invariants, update the contract docs in the same change.
- If you propose changing `docs/MathContract.md`, `docs/BarSemantics.md`, or `docs/Invariants.md`, start with a plan-only pass and get a human checkpoint before implementation continues.
- Prefer adding tests and fixtures before adding breadth.
- Do not introduce TUI-first abstractions into the kernel.
- Do not let default validation depend on live providers, secrets, or network access.
- Do not add point-in-time universe work in the first trustworthy version.

## Expected Workspace Shape

The planned workspace is documented in `docs/Workspace.md`. Until the crates exist, keep names and ownership aligned with that document.

## Validation

The repo-wide validation entrypoint will be `cargo xtask validate`.

Once the workspace exists, that command should become the default post-change check. Until then, validate by keeping the planning docs mutually consistent and updating `docs/Status.md`.

## Repo-Local Setup

This repository has project-local agent setup.

- Codex repo-local overrides live in `.codex/config.toml`.
- Cursor repo-local MCP config lives in `.cursor/mcp.json`.
- Intended MCP servers for this repo are `openaiDeveloperDocs` and `context7`.
- Cursor repo-local hooks live in `.cursor/hooks.json`.
- Hooks hard-block destructive shell commands and provide advisory reminders for protected core/data and repo-guidance edits.

## Completion Discipline

After each milestone-level change:

1. Update `docs/Status.md`.
2. Note any unresolved ambiguity in contracts or architecture.
3. Avoid moving into the next milestone unless the current acceptance criteria are met.
