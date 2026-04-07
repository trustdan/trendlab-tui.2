# Implementation Runbook

Use this loop for milestone work.

## Standard Loop

1. Read the current status and contracts.
2. Restate the active milestone and acceptance criteria.
3. If the work would change `docs/MathContract.md`, `docs/BarSemantics.md`, or `docs/Invariants.md`, stop after planning and get a human checkpoint before implementation.
4. Make only the smallest change set that advances that milestone.
5. Run validation.
6. Repair failures before moving on.
7. Update `docs/Status.md`.

## Scope Rules

- Do not blend milestones in one change unless the dependency is trivial and explicit.
- Do not add UI abstractions to solve core problems.
- Do not let provider or cache details leak into the simulation kernel.
- Do not add search breadth before a replayable truthful kernel exists.
- Do not add generalized strategy composition to M1.
- Treat math, bar-semantics, and invariant changes as constitution-level edits that require a plan-first pass and human review before code continues.

## Validation Target

The repo target is `cargo xtask validate`.

That command should eventually run:

- formatting checks
- lint checks
- unit tests
- integration tests
- golden ledger tests

And it must stay:

- deterministic
- network-free
- runnable without API keys or secrets

Live provider checks should use a separate command such as `cargo xtask validate-live`.

## Status Update Format

When updating `docs/Status.md`, record:

- current milestone
- completed work
- open risks
- next planned step
