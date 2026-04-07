Yes — I’d rebuild around Tiingo, with one important caveat: **Tiingo should be the data provider, not the trust boundary**.

For your use case — daily/weekly trend research across a few hundred equities and ETFs — Tiingo is a sensible replacement for Yahoo. Tiingo’s official EOD API covers 65,000+ securities, exposes adjusted OHLC fields, provides separate corporate-action endpoints for splits and dividends, supports weekly/monthly resampling, and has a separate IEX intraday API if you later want finer execution modeling. The practical constraint is pricing and limits: the free plan is capped at 500 unique symbols per month, 50 requests per hour, and 1,000 requests per day, while the individual plan is $30/month, so a 542-symbol universe already pushes you toward the paid tier. ([Tiingo][1])

The good news is that the **shape** of old TrendLab was already right. The core ideas worth preserving are the separation between engine and interface, the terminal-native workflow, the vim-style navigation, the live leaderboards, the charts, and especially the strategy decomposition into **signal generation, position management, execution, and filters**. That decomposition is the right answer to the stickiness problem and to fair comparison more generally.    

My recommendation would be:

## 1. Make “inspectability” a core feature

The new TrendLab should never become a black box. Every run should produce:

* a **run manifest**: data snapshot, provider, date range, universe snapshot, seed, strategy structure, parameters, costs, engine version
* an **event ledger**: one row per bar with the state transitions that actually happened
* a **replayable artifact**: something the CLI and TUI can both reopen exactly

That means a winning leaderboard row is not just “a result”; it is a fully reproducible object you can reopen and audit later. The old system already had the right instinct here with run fingerprints, persistent history, confidence grading, and a CLI/core split. The rebuild should push that much further. 

A ledger row should look something like this:

```text
date | raw O/H/L/C | analysis_close | signal | filter_pass | order_submitted
fill_price | stop_prev | stop_next | shares | cash | equity | reason
```

That single table is the antidote to “I won’t be able to see how it works under the hood.”

## 2. Use Tiingo, but store your own canonical market model

I would not let TrendLab become “whatever Tiingo returns.” I’d store four distinct things locally:

* **raw daily bars**: trade-space OHLCV
* **corporate actions**: splits and cash dividends
* **derived analysis series**: split-adjusted, and optionally total-return
* **universe snapshots**: point-in-time membership files

This matters because **signals, fills, and portfolio accounting should not all live in the same price space**. Tiingo’s adjusted fields are useful, but for fidelity I would still define TrendLab’s own internal conventions:

* **fills** happen on raw tradable prices
* **signals** can use a chosen analysis series
* **equity** can include explicit dividend cashflows or a total-return mode

Also, I would keep **daily bars as the canonical store** and do weekly/monthly resampling inside TrendLab. Tiingo’s resampling support is useful as a cross-check, but the app should own its own calendar semantics and aggregation rules. ([Tiingo][1])

## 3. Rebuild the kernel as an event-driven simulator with explicit bar semantics

The single most important technical decision is to make bar processing explicit and conservative.

For daily data, I’d use an order of operations like this:

1. apply pre-open corporate actions
2. process pending market/stop/limit orders at the current open using a documented gap policy
3. evaluate previously-active stop/limit conditions against the current bar
4. compute end-of-day indicators and signals using information available at that close only
5. queue orders for the next bar
6. update trailing references for **tomorrow**, not retroactively for today

That last point is crucial. On daily bars, you usually do **not** know whether the day’s high happened before the day’s low. So a trailing stop should not be allowed to use today’s high to tighten itself and then also claim it was hit by today’s low. That is false precision. The conservative rule is: **today’s stop level is yesterday’s stop level; today’s high only affects tomorrow’s stop**.

That is the kind of design choice that makes a backtester trustworthy.

## 4. Hardwire the stickiness fix into the architecture

The old analysis already identified the core problem: TrendLab was sometimes comparing **strategy-execution bundles**, not isolated components, and rolling-reference exits could become structurally “sticky.” The rebuild should make that impossible by design.  

I’d formalize three rules:

* **signal generators** may use rolling lookbacks
* **position managers** may only use frozen-at-entry state or since-entry state
* **execution models** may never mutate signal logic

So a breakout can still look back 252 days, but its exit manager must be explicitly one of:

* frozen reference
* since-entry trailing high/low
* ATR trail
* chandelier
* time decay
* fixed stop
* signal reversal

No exit should be allowed to “chase” the market using a rolling reference that inherits pre-entry context unless that behavior is explicitly the thing being tested.

I’d also split breakout systems into two distinct families:

* **close-confirmed breakout**: signal on close, fill next open
* **stop-entry breakout**: prior threshold becomes an actual stop order for the next bar

Those are not the same system, and the older analysis was right that a one-size-fits-all next-open fill model can bias breakout entries. 

## 5. Treat survivorship bias as a first-class accuracy issue

The fixed 542-ticker universe is great operationally, but if you replay the same modern list deep into the past, you are still studying survivors. That is a separate fidelity problem from stickiness, and it matters. The old docs make clear that TrendLab was loading a fixed universe on startup, and Tiingo’s search API can distinguish active and delisted assets, which helps, but it is not the same as a true point-in-time constituent history.   ([Tiingo][2])

So I’d support two universe modes:

* **curated modern universe** for quick research and UI flow
* **point-in-time universe** for serious robustness testing

That way the fast lab experience stays intact, but the research-grade mode is honest about history.

## 6. Keep the old statistical layer, then add broader Monte Carlo later

The old TrendLab already had some of the strongest statistical ideas: block-bootstrap confidence intervals, walk-forward validation, cross-symbol ranking, and stickiness diagnostics. Those should absolutely stay. 

Once the deterministic kernel is trusted, then I’d add the broader Monte Carlo stack the earlier analysis pointed toward:

* **parameter sampling**
* **universe sampling**
* **execution noise**
* **path perturbation / block bootstrap scenarios**

But only after the base simulator is auditably correct. Otherwise you just get a more sophisticated way to hide errors. 

## 7. Preserve the UI soul, but add one new first-class surface: Audit

I would absolutely keep the TUI. The original ratatui interface, charts, leaderboards, help system, and vim-style navigation are part of what made TrendLab special. They should survive the rebuild.   

What I would add is an **Audit panel**.

That panel should let you take any leaderboard result and inspect:

* the bar-by-bar event ledger
* the exact entry/exit reasons
* stop levels over time
* blocked signals
* corporate actions applied during the trade
* ambiguous-fill warnings
* metric formulas and inputs

And in the Chart panel, I’d add overlays for:

* entries/exits
* trailing stop lines
* frozen reference lines
* split/dividend markers
* “ambiguous daily bar” badges
* trade duration shading

That gives you visual trust, not just numerical trust.

I’d also keep the CLI as a first-class surface. In the old architecture, the CLI/core split was already one of the best ideas. The new version should let you run:

```bash
trendlab run manifest.yaml
trendlab explain run_id
trendlab diff run_a run_b
trendlab audit data AAPL
```

That way the TUI stays beautiful, but the system is never trapped inside the TUI. 

## 8. What I would ship first

I’d stage the rebuild like this:

**First ship the trustworthy lab**

* Tiingo EOD adapter
* local raw-bar + corporate-action cache
* internal weekly resampler
* deterministic daily-bar kernel
* long-only
* next-open, close-on-signal, and stop-entry execution
* ATR trail, since-entry trail, frozen reference, fixed stop
* event ledger and replay
* CLI + TUI parity

**Then ship the research layer**

* separate leaderboards for signal / position manager / execution
* cross-symbol aggregation
* walk-forward
* block bootstrap
* point-in-time universe mode

**Then ship the “better than before” UX**

* audit panel
* chart overlays
* run diffing
* Pine export with parity vectors
* live data-health and provenance badges

That ordering preserves the original feel while fixing the part you actually distrust: the hidden math.

So my answer is: **yes, Tiingo makes sense**, but the real upgrade is not “Yahoo → Tiingo.” The real upgrade is **opaque backtester → auditable research engine**. The best new TrendLab would still feel like the old one in your hands — fast, terminal-native, vim-fluent, chart-rich — while being much harder to fool under the hood. ([Tiingo][3])    

The next thing I’d do is turn this into a crate-by-crate implementation spec and a test matrix.

[1]: https://www.tiingo.com/documentation/end-of-day?utm_source=chatgpt.com "End-of-Day (EOD) Stock Price API Documentation"
[2]: https://www.tiingo.com/documentation/utilities/search?utm_source=chatgpt.com "Search API Documentation - Tiingo"
[3]: https://www.tiingo.com/pricing?utm_source=chatgpt.com "Pricing"
