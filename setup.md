Yes — but I’d keep the setup **small and deliberate**.

The most important thing to know up front is that, in your stack, there are really **two separate MCP planes**. Cursor has native MCP support and reads `mcp.json` / `~/.cursor/mcp.json`, while Codex keeps its MCP configuration in `~/.codex/config.toml` or project-scoped `.codex/config.toml`; the Codex CLI and IDE extension share that same config. So if you’re mainly using the **Codex sidebar inside Cursor**, I’d configure **Codex MCP first**, then mirror only the essentials into Cursor’s MCP config if you also expect to use Cursor Agent a lot. ([OpenAI Developers][1])

For this project, I would start with **only two MCP servers**:

1. **OpenAI developer docs MCP**
   This is the one I’d install immediately, because you’re explicitly building around Codex/OpenAI workflows. OpenAI’s docs server is read-only and is meant for pulling official developer documentation into the agent’s context. OpenAI’s docs also show the exact Cursor and Codex setup paths for it. ([OpenAI Developers][2])

2. **One general developer-docs MCP**, not a pile of app integrations
   Codex’s MCP docs use **Context7** as the example server for developer documentation, which makes it a reasonable second choice for Rust/Polars/ratatui/crossterm/library lookups. More importantly, OpenAI’s Codex guidance says to add MCP tools only when they unlock a real workflow and not to wire in everything at once. ([OpenAI Developers][1])

I would **not** start with a Tiingo MCP server. For TrendLab, the risky part is not “can the agent query Tiingo live?” — it’s whether the **local, replayable backtest kernel** is correct. A normal Rust Tiingo adapter plus deterministic fixtures is a better day-one integration than putting live market queries in the agent loop. That fits the old TrendLab lessons: the stickiness problem came from hidden structural coupling in the backtest engine, not from lack of external tools.  

I also would not start by adding browser, Playwright, or lots of workflow servers. Cursor already ships built-in agent tools for browser control and codebase search, so you do not need extra MCP just to browse docs or search your repo. And for a terminal-native Rust app, those tools are lower priority than math and auditability anyway. ([Cursor][3])

The bigger win before coding is to set the **agent defaults** correctly. Codex currently recommends starting with **`gpt-5.4`** for most tasks, with **`gpt-5.4-mini`** as the faster option for lighter work or subagents. The IDE extension also lets you switch models, reasoning effort, and approval modes from inside the editor. ([OpenAI Developers][4])

For a repo like TrendLab, I’d begin with these Codex defaults:

```toml
# ~/.codex/config.toml

model = "gpt-5.4"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
model_reasoning_effort = "high"
personality = "pragmatic"

[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"

[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]
```

That lines up with Codex’s documented config system: `~/.codex/config.toml` for personal defaults, optional project overrides in `.codex/config.toml`, and shared settings across the CLI and IDE extension. OpenAI’s own best-practices guidance also recommends keeping approvals and sandboxing **tight by default** and loosening them only after the workflow proves it needs more autonomy. ([OpenAI Developers][1])

If you also want Cursor Agent to use the same OpenAI docs MCP, the minimal Cursor-side config is:

```json
{
  "mcpServers": {
    "openaiDeveloperDocs": {
      "url": "https://developers.openai.com/mcp"
    }
  }
}
```

OpenAI’s docs explicitly note that Cursor has native MCP support and reads configuration from `mcp.json`, including `~/.cursor/mcp.json`. ([OpenAI Developers][2])

Beyond MCP, the most relevant **Cursor settings** for this project are actually **rules, plan mode, hooks, and sandboxing**.

Cursor’s rules system is meant for persistent agent instructions, and Cursor also supports `AGENTS.md` as part of that policy layer. Cursor’s Plan Mode creates a reviewable implementation plan before code is written. Hooks can run before or after stages of the agent loop and can observe, block, or modify behavior. Cursor’s sandbox/terminal docs also note that network access is blocked by default unless you allow it. ([Cursor][5])

For TrendLab, those settings matter more than extra servers because your two non-negotiables are **backtest trustworthiness** and **preserving the terminal UX**. The old docs make clear that TrendLab’s soul was the keyboard-first ratatui interface with vim navigation, charts, and leaderboards, while the hardest failure mode was the backtesting “stickiness” caused by conflating signal generation, position management, and execution behavior. Your rules should freeze both of those truths into the repo before the first large agent session.    

So I’d create these before you start:

* `AGENTS.md`
* `.cursor/rules/10-backtest-safety.mdc`
* `.cursor/rules/20-ui-philosophy.mdc`
* `.cursor/hooks.json`
* `docs/MATH_CONTRACT.md`
* `docs/BAR_SEMANTICS.md`

And I’d make the core rules say, in effect:

```text
- Always plan before editing backtest-core, execution, or data normalization.
- Never change fill semantics, stop update timing, split/dividend handling, or metric formulas without updating MathContract + golden tests.
- Preserve keyboard-first TUI behavior and vim-style navigation unless explicitly asked to change it.
- Prefer event-ledger visibility, replay artifacts, and explainability over hidden state.
```

That is the right guardrail set for this specific codebase. It encodes the lesson from the old TrendLab: the important thing is not merely “build fast,” but “build something you can still trust after the implementation gets large.”  

One more practical setting: keep Cursor/Codex network access **narrow**. Because Cursor’s sandbox blocks network by default, only allow what you actually need for the first phase — typically package registries and Tiingo if you’re actively implementing the provider. That keeps the agent from quietly depending on random live web state during the phase where you’re trying to make the backtester deterministic. ([Cursor][6])

My actual recommendation, in order:

* Set up **Codex config** first.
* Add **OpenAI docs MCP** immediately.
* Add **Context7** if you want library docs inside the agent.
* Add **repo rules + AGENTS.md + hooks** before any major coding session.
* Keep approvals at **`on-request`** and sandbox at **`workspace-write`**.
* Use **Plan Mode** for every kernel milestone.
* Skip Tiingo/DB/GitHub/issue-tracker MCPs until you feel a real repeated pain.

The clean starting posture is: **two MCPs max, strict rules, strict sandbox, strong audit hooks**. That is the best setup for vibe-coding a mathematically sensitive system without losing the original TrendLab feel. ([OpenAI Developers][7])

Next step: draft the exact starter files for `AGENTS.md`, `.cursor/rules`, `.cursor/hooks.json`, and `.codex/config.toml`.

[1]: https://developers.openai.com/codex/mcp/ "Model Context Protocol – Codex | OpenAI Developers"
[2]: https://developers.openai.com/learn/docs-mcp/ "Docs MCP | OpenAI Developers"
[3]: https://cursor.com/docs/agent/tools/browser?utm_source=chatgpt.com "Browser | Cursor Docs"
[4]: https://developers.openai.com/codex/models/ "Models – Codex | OpenAI Developers"
[5]: https://cursor.com/docs/rules?utm_source=chatgpt.com "Rules | Cursor Docs"
[6]: https://cursor.com/docs/reference/sandbox?utm_source=chatgpt.com "sandbox.json Reference | Cursor Docs"
[7]: https://developers.openai.com/codex/learn/best-practices/ "Best practices – Codex | OpenAI Developers"
