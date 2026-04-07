# Plan Feedback Disposition

This file records which outside feedback items are being adopted into the repo plan and which ones are being declined for now.

## Overall Verdict

The feedback was directionally good. It reinforced the core structure already present in the repo:

- truthful kernel before provider breadth or UI work
- shared artifacts and replay as part of the trust model
- deterministic, offline default validation
- audit-first operator surfaces instead of black-box results

The useful parts are now reflected in the planning docs. The parts that would widen early milestones or add premature process have been left out.

## Adopted

### 1. Independent oracle for M1

Accepted in modified form.

The repo will require at least one hand-authored oracle scenario or truth table for the first truthful path. This gives M1 something independent of the Rust implementation and generated golden outputs.

This does not mean Python becomes part of required validation. The default path should stay deterministic, lightweight, and repo-native.

### 2. Tiny audit surface before the CLI milestone

Accepted.

The roadmap now allows a minimal `xtask` inspection utility for replay bundles or ledgers before `trendlab-cli` exists. This keeps the truthful kernel human-inspectable during M1 instead of waiting until M4.

### 3. Separate live-provider smoke lane after M2

Accepted.

Any live provider verification should sit behind a separate command such as `cargo xtask validate-live` or an equivalent smoke path. It must remain outside normal validation.

### 4. Make constitution changes more expensive than normal code changes

Accepted.

Changes to `docs/MathContract.md`, `docs/BarSemantics.md`, and `docs/Invariants.md` now require a plan-only pass and a human checkpoint before implementation continues.

### 5. Turn Week 0 into explicit decisions plus first golden scenarios

Accepted.

Week 0 should close with the open format decisions, frozen initial interfaces, and the first three M1 golden scenarios chosen explicitly enough to guide implementation.

## Declined For Now

### 1. Expand the M3 gate to require non-breakout families before closing it

Not adopted as a gate.

M3 is currently about proving compositional boundaries and keeping breakout execution paths separate. Requiring MA crossover, momentum, or additional position-manager families before closing M3 would widen the milestone instead of tightening it.

Those broader strategy families can be added after the first composition pass if the architecture still needs a stronger generality check.

### 2. Add explicit performance-budget checkpoints in Weeks 6 and 8

Not adopted for now.

Early risk is dominated by semantic drift, accounting errors, and contract violations rather than runtime. Lightweight performance baselines may still be useful once the first runnable kernel and data layer exist, but they are not part of the current Week 0-8 closure criteria.

## Ongoing Caution

The process should stay lighter than the code. Weeks are batching units, not promises, and `docs/Status.md` should stay short and factual.
