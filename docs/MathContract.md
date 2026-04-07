# Math Contract

This document defines the default accounting and price-space rules for the rebuild.

## Purpose

Keep the simulator explainable and conservative. If implementation diverges from this document, the document must change in the same milestone.

## Price Spaces

Use three conceptual price spaces:

- raw tradable prices
- analysis series
- total-return series

## Default Rules

- Orders fill against raw tradable prices.
- Signals read from an analysis series.
- Equity is computed from cash plus marked-to-market raw positions.
- Dividends are modeled as explicit cashflows, not silently absorbed into fills.
- Total-return mode is a later overlay, not the default accounting mode.

## Corporate Actions

- Splits are applied before the next market open on the effective date.
- Dividend cashflows are applied according to a documented policy before metrics are finalized for the bar.
- Corporate actions must be visible in the ledger or manifest.
- Historical limitations, including survivorship-biased universe selection when relevant, must be visible in the manifest.

## Costs

The simulator should support:

- commissions
- slippage
- borrow or financing later, not in the first truthful version

Default early behavior can be zero-cost, but the cost model must be explicit in the run manifest.

## Position Scope

The first trustworthy version is:

- single symbol
- long-only
- one position at a time

Broader portfolio behavior is later work.

## Ledger Minimum

Each bar should be able to explain at least:

- date
- raw open, high, low, close
- analysis close
- active position state
- signal output
- filter outcome
- pending order state
- fill price if any
- prior stop
- next stop
- cash
- equity
- reason codes

## Forbidden Shortcuts

- no lookahead from future bars
- no hidden provider adjustments inside the core
- no same-bar use of newly tightened trailing stops
- no silent switch between analysis and tradable price spaces
