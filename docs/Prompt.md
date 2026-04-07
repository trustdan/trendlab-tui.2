# Default Session Prompt

Use this as the baseline prompt for future agent sessions.

```text
Read AGENTS.md, docs/Status.md, docs/Plan.md, docs/Roadmap.md, docs/Workspace.md, and any contract docs relevant to the task.

Do not start coding until you:
1. summarize the active constraints
2. identify the current milestone
3. list the files you expect to touch
4. list the validation you will run
5. call out any ambiguity in math, bar semantics, artifacts, or architecture
6. if the task would change `docs/MathContract.md`, `docs/BarSemantics.md`, or `docs/Invariants.md`, stop after the plan and wait for explicit human approval before coding

Then implement only the approved milestone slice.
Keep diffs scoped.
Update docs/Status.md before stopping.
```

## Task-Specific Additions

Add these when relevant:

- For kernel work: read `docs/MathContract.md`, `docs/BarSemantics.md`, and `docs/Invariants.md`.
- For artifact work: read `docs/Artifacts.md`.
- For workspace work: read `docs/Workspace.md`.
- For schedule-sensitive work: read `docs/Roadmap.md`.
- For TUI work: preserve audit-first navigation and keep the kernel untouched.
- For constitution changes: treat `docs/MathContract.md`, `docs/BarSemantics.md`, and `docs/Invariants.md` as plan-first, human-reviewed documents.
