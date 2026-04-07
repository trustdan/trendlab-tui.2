# Bar Semantics

This document defines the default daily-bar processing order.

## Purpose

Daily bars do not reveal intraday ordering between the high and low. The simulator must prefer conservative rules over false precision.

## Canonical Daily Processing Order

For each trading day:

1. apply pre-open corporate actions
2. process pending marketable orders at the current open using the active gap policy
3. evaluate previously active stop or limit conditions using the current bar
4. compute end-of-day indicators and signals using information available at the close only
5. queue new orders for the next bar
6. update trailing references for tomorrow, not retroactively for today

## Core Implications

- Today's stop level is yesterday's stop level.
- Today's high can affect tomorrow's trailing stop.
- The current day's low cannot claim to hit a stop that was tightened by the current day's high.

## M1 Reference Flow

The first trustworthy kernel does not need a full strategy layer.

M1 should support only:

- a queued market-entry intent that fills at the next open
- a fixed protective stop evaluated under the default gap policy

This keeps the kernel focused on truthful order lifecycle, replay, and accounting.

## Entry Models To Support After M1

- close-confirmed signal, fill next open
- stop-entry based on a threshold carried into the next bar

These are distinct systems and must not be collapsed into one execution rule.

## Gap Policy

Gap behavior must be explicit. Early implementation should choose one documented default and expose it in the run manifest.

Examples:

- market order fills at open
- stop order triggered through the open fills at open
- intrabar stop fills at the stop price or a conservative alternative defined by the policy

## Initial M1 Default Gap Policy

The default M1 gap policy is:

- queued market entries fill at the current open
- a protective stop that is already active before the bar and is crossed through the open fills at the open
- if the open does not cross the active stop and the bar later trades through it, the stop fills at the stop price

This is the default gap policy that M1 fixtures, artifacts, and golden scenarios should assume unless the contract is explicitly revised.

## Ambiguity Handling

When a daily bar leaves multiple intrabar paths plausible:

- prefer the conservative interpretation
- surface ambiguity in the ledger or warnings
- do not infer intraday sequencing that the data does not contain

## Weekly And Monthly Bars

Weekly and monthly series are derived internally from canonical daily data. Resampling is a data-layer responsibility, not a core simulation shortcut.
