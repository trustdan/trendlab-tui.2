#![forbid(unsafe_code)]

pub mod accounting {
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct CashSummary {
        pub cash: f64,
        pub equity: f64,
    }

    #[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct CostModel {
        pub commission_per_fill: f64,
        pub slippage_per_share: f64,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct PositionState {
        pub shares: u32,
        pub entry_price: Option<f64>,
        pub active_stop: Option<f64>,
    }
}

pub mod market {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DailyBar {
        pub date: String,
        pub raw_open: f64,
        pub raw_high: f64,
        pub raw_low: f64,
        pub raw_close: f64,
        pub analysis_close: f64,
    }
}

pub mod orders {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum GapPolicy {
        #[serde(rename = "m1_default")]
        M1Default,
    }

    impl GapPolicy {
        pub const M1_DEFAULT: &'static str = "m1_default";

        pub fn as_str(self) -> &'static str {
            match self {
                Self::M1Default => Self::M1_DEFAULT,
            }
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value.trim() {
                Self::M1_DEFAULT => Some(Self::M1Default),
                _ => None,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum OrderIntent {
        QueueMarketEntry,
        CarryStopEntry,
    }

    impl OrderIntent {
        pub const QUEUE_MARKET_ENTRY: &'static str = "queue_market_entry";
        pub const CARRY_STOP_ENTRY: &'static str = "carry_stop_entry";

        pub fn as_str(self) -> &'static str {
            match self {
                Self::QueueMarketEntry => Self::QUEUE_MARKET_ENTRY,
                Self::CarryStopEntry => Self::CARRY_STOP_ENTRY,
            }
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value.trim() {
                Self::QUEUE_MARKET_ENTRY => Some(Self::QueueMarketEntry),
                Self::CARRY_STOP_ENTRY => Some(Self::CarryStopEntry),
                _ => None,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct EntryIntent {
        pub signal_date: String,
        pub intent: OrderIntent,
        pub shares: u32,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct PendingOrder {
        pub intent: OrderIntent,
        pub shares: u32,
        pub stop_price: Option<f64>,
    }

    impl PendingOrder {
        pub fn describe_state(&self) -> String {
            match self.stop_price {
                Some(stop_price) => {
                    format!("{}:{}@{}", self.intent.as_str(), self.shares, stop_price)
                }
                None => format!("{}:{}", self.intent.as_str(), self.shares),
            }
        }
    }
}

pub mod ledger {
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct LedgerRow {
        pub date: String,
        pub raw_open: f64,
        pub raw_high: f64,
        pub raw_low: f64,
        pub raw_close: f64,
        pub analysis_close: f64,
        pub position_shares: u32,
        pub signal_output: String,
        pub filter_outcome: String,
        pub pending_order_state: String,
        pub fill_price: Option<f64>,
        pub prior_stop: Option<f64>,
        pub next_stop: Option<f64>,
        pub cash: f64,
        pub equity: f64,
        pub reason_codes: Vec<String>,
    }
}

pub mod strategy {
    use crate::accounting::PositionState;
    use crate::market::DailyBar;
    use crate::orders::PendingOrder;

    #[derive(Clone, Debug, PartialEq)]
    pub struct StrategyContext<'a> {
        pub symbol: &'a str,
        pub completed_bars: &'a [DailyBar],
        pub current_bar: &'a DailyBar,
        pub position: &'a PositionState,
        pub pending_order: Option<&'a PendingOrder>,
    }

    impl<'a> StrategyContext<'a> {
        pub fn current_date(&self) -> &str {
            self.current_bar.date.as_str()
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum SignalDecision {
        None,
        EnterLong {
            signal_id: String,
            entry_reference: Option<f64>,
        },
        ExitLong {
            signal_id: String,
        },
    }

    impl SignalDecision {
        pub fn signal_id(&self) -> Option<&str> {
            match self {
                Self::None => None,
                Self::EnterLong { signal_id, .. } | Self::ExitLong { signal_id } => {
                    Some(signal_id.as_str())
                }
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum FilterDecision {
        Pass,
        Block { reason_code: String },
    }

    impl FilterDecision {
        pub fn allows_signal(&self) -> bool {
            matches!(self, Self::Pass)
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum PositionDecision {
        Keep,
        SetProtectiveStop {
            stop_price: f64,
            reason_code: String,
        },
        Exit {
            reason_code: String,
        },
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum ExecutionDecision {
        None,
        QueueMarketEntry {
            shares: u32,
            signal_id: String,
        },
        CarryStopEntry {
            stop_price: f64,
            shares: u32,
            signal_id: String,
        },
        QueueMarketExit {
            shares: u32,
            signal_id: String,
        },
        Blocked {
            reason_code: String,
        },
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct StrategyEvaluation {
        pub signal: SignalDecision,
        pub filter: FilterDecision,
        pub position: PositionDecision,
        pub execution: ExecutionDecision,
    }

    pub trait SignalGenerator {
        fn evaluate(&self, context: &StrategyContext<'_>) -> SignalDecision;
    }

    pub trait SignalFilter {
        fn evaluate(
            &self,
            context: &StrategyContext<'_>,
            signal: &SignalDecision,
        ) -> FilterDecision;
    }

    pub trait PositionManager {
        fn evaluate(&self, context: &StrategyContext<'_>) -> PositionDecision;
    }

    pub trait ExecutionModel {
        fn evaluate(
            &self,
            context: &StrategyContext<'_>,
            signal: &SignalDecision,
            filter: &FilterDecision,
            position: &PositionDecision,
        ) -> ExecutionDecision;
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct CompositeStrategy<S, F, P, E> {
        signal: S,
        filter: F,
        position: P,
        execution: E,
    }

    impl<S, F, P, E> CompositeStrategy<S, F, P, E> {
        pub fn new(signal: S, filter: F, position: P, execution: E) -> Self {
            Self {
                signal,
                filter,
                position,
                execution,
            }
        }
    }

    impl<S, F, P, E> CompositeStrategy<S, F, P, E>
    where
        S: SignalGenerator,
        F: SignalFilter,
        P: PositionManager,
        E: ExecutionModel,
    {
        pub fn evaluate(&self, context: &StrategyContext<'_>) -> StrategyEvaluation {
            let signal = self.signal.evaluate(context);
            let filter = self.filter.evaluate(context, &signal);
            let position = self.position.evaluate(context);
            let execution = self
                .execution
                .evaluate(context, &signal, &filter, &position);

            StrategyEvaluation {
                signal,
                filter,
                position,
                execution,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct PassFilter;

    impl SignalFilter for PassFilter {
        fn evaluate(
            &self,
            _context: &StrategyContext<'_>,
            _signal: &SignalDecision,
        ) -> FilterDecision {
            FilterDecision::Pass
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct KeepPositionManager;

    impl PositionManager for KeepPositionManager {
        fn evaluate(&self, _context: &StrategyContext<'_>) -> PositionDecision {
            PositionDecision::Keep
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct CloseConfirmedBreakoutSignal {
        lookback_bars: usize,
        signal_id: String,
    }

    impl CloseConfirmedBreakoutSignal {
        pub fn new(lookback_bars: usize) -> Self {
            Self::with_signal_id(lookback_bars, "close_confirmed_breakout")
        }

        pub fn with_signal_id(lookback_bars: usize, signal_id: impl Into<String>) -> Self {
            Self {
                lookback_bars,
                signal_id: signal_id.into(),
            }
        }
    }

    impl SignalGenerator for CloseConfirmedBreakoutSignal {
        fn evaluate(&self, context: &StrategyContext<'_>) -> SignalDecision {
            if self.lookback_bars == 0 || context.completed_bars.len() < self.lookback_bars {
                return SignalDecision::None;
            }

            let trailing_window = &context.completed_bars
                [context.completed_bars.len() - self.lookback_bars..context.completed_bars.len()];
            let breakout_level = trailing_window
                .iter()
                .map(|bar| bar.analysis_close)
                .fold(f64::NEG_INFINITY, f64::max);

            if context.current_bar.analysis_close > breakout_level {
                SignalDecision::EnterLong {
                    signal_id: self.signal_id.clone(),
                    entry_reference: Some(breakout_level),
                }
            } else {
                SignalDecision::None
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct StopEntryBreakoutSignal {
        lookback_bars: usize,
        signal_id: String,
    }

    impl StopEntryBreakoutSignal {
        pub fn new(lookback_bars: usize) -> Self {
            Self::with_signal_id(lookback_bars, "stop_entry_breakout")
        }

        pub fn with_signal_id(lookback_bars: usize, signal_id: impl Into<String>) -> Self {
            Self {
                lookback_bars,
                signal_id: signal_id.into(),
            }
        }
    }

    impl SignalGenerator for StopEntryBreakoutSignal {
        fn evaluate(&self, context: &StrategyContext<'_>) -> SignalDecision {
            if self.lookback_bars == 0 || context.completed_bars.len() + 1 < self.lookback_bars {
                return SignalDecision::None;
            }

            let trailing_window_start = context.completed_bars.len() + 1 - self.lookback_bars;
            let breakout_level = context.completed_bars[trailing_window_start..]
                .iter()
                .map(|bar| bar.raw_high)
                .chain(std::iter::once(context.current_bar.raw_high))
                .fold(f64::NEG_INFINITY, f64::max);

            SignalDecision::EnterLong {
                signal_id: self.signal_id.clone(),
                entry_reference: Some(breakout_level),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct NextOpenLongExecution {
        shares: u32,
    }

    impl NextOpenLongExecution {
        pub fn new(shares: u32) -> Self {
            Self { shares }
        }
    }

    impl ExecutionModel for NextOpenLongExecution {
        fn evaluate(
            &self,
            context: &StrategyContext<'_>,
            signal: &SignalDecision,
            filter: &FilterDecision,
            _position: &PositionDecision,
        ) -> ExecutionDecision {
            if let FilterDecision::Block { reason_code } = filter {
                return ExecutionDecision::Blocked {
                    reason_code: reason_code.clone(),
                };
            }

            match signal {
                SignalDecision::EnterLong { signal_id, .. } => {
                    if context.position.shares != 0 {
                        ExecutionDecision::Blocked {
                            reason_code: "position_already_open".to_string(),
                        }
                    } else if context.pending_order.is_some() {
                        ExecutionDecision::Blocked {
                            reason_code: "pending_order_already_active".to_string(),
                        }
                    } else if self.shares == 0 {
                        ExecutionDecision::None
                    } else {
                        ExecutionDecision::QueueMarketEntry {
                            shares: self.shares,
                            signal_id: signal_id.clone(),
                        }
                    }
                }
                _ => ExecutionDecision::None,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct StopEntryLongExecution {
        shares: u32,
    }

    impl StopEntryLongExecution {
        pub fn new(shares: u32) -> Self {
            Self { shares }
        }
    }

    impl ExecutionModel for StopEntryLongExecution {
        fn evaluate(
            &self,
            context: &StrategyContext<'_>,
            signal: &SignalDecision,
            filter: &FilterDecision,
            _position: &PositionDecision,
        ) -> ExecutionDecision {
            if let FilterDecision::Block { reason_code } = filter {
                return ExecutionDecision::Blocked {
                    reason_code: reason_code.clone(),
                };
            }

            match signal {
                SignalDecision::EnterLong {
                    signal_id,
                    entry_reference: Some(stop_price),
                } => {
                    if context.position.shares != 0 {
                        ExecutionDecision::Blocked {
                            reason_code: "position_already_open".to_string(),
                        }
                    } else if self.shares == 0 {
                        ExecutionDecision::None
                    } else if let Some(order) = context.pending_order {
                        if order.intent == crate::orders::OrderIntent::CarryStopEntry
                            && order.shares == self.shares
                            && order.stop_price == Some(*stop_price)
                        {
                            ExecutionDecision::Blocked {
                                reason_code: "pending_order_already_active".to_string(),
                            }
                        } else if order.intent == crate::orders::OrderIntent::CarryStopEntry {
                            ExecutionDecision::CarryStopEntry {
                                stop_price: *stop_price,
                                shares: self.shares,
                                signal_id: signal_id.clone(),
                            }
                        } else {
                            ExecutionDecision::Blocked {
                                reason_code: "pending_order_already_active".to_string(),
                            }
                        }
                    } else {
                        ExecutionDecision::CarryStopEntry {
                            stop_price: *stop_price,
                            shares: self.shares,
                            signal_id: signal_id.clone(),
                        }
                    }
                }
                SignalDecision::EnterLong {
                    entry_reference: None,
                    ..
                } => ExecutionDecision::Blocked {
                    reason_code: "stop_entry_requires_reference".to_string(),
                },
                _ => ExecutionDecision::None,
            }
        }
    }
}

pub mod engine {
    use std::collections::{BTreeMap, BTreeSet};
    use std::error::Error;
    use std::fmt::{Display, Formatter};

    use crate::accounting::{CashSummary, CostModel, PositionState};
    use crate::ledger::LedgerRow;
    use crate::market::DailyBar;
    use crate::orders::{EntryIntent, GapPolicy, OrderIntent, PendingOrder};
    use crate::strategy::{
        CompositeStrategy, ExecutionDecision, FilterDecision, PositionDecision, SignalDecision,
        SignalFilter, SignalGenerator, StrategyContext,
    };

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct ReferenceFlowSpec {
        pub initial_cash: f64,
        pub entry_shares: u32,
        pub protective_stop_fraction: f64,
        pub cost_model: CostModel,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct RunRequest {
        pub symbol: String,
        pub bars: Vec<DailyBar>,
        pub entry_intents: Vec<EntryIntent>,
        pub reference_flow: ReferenceFlowSpec,
        pub gap_policy: GapPolicy,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct StrategyRunRequest {
        pub symbol: String,
        pub bars: Vec<DailyBar>,
        pub reference_flow: ReferenceFlowSpec,
        pub gap_policy: GapPolicy,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct RunResult {
        pub ledger: Vec<LedgerRow>,
        pub cash: CashSummary,
        pub position: PositionState,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SimulationError {
        message: String,
    }

    impl SimulationError {
        pub fn not_implemented() -> Self {
            Self {
                message: "the truthful kernel is scaffolded but not implemented yet".to_string(),
            }
        }

        pub fn invalid_request(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl Display for SimulationError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.message)
        }
    }

    impl Error for SimulationError {}

    pub fn run_reference_flow(request: &RunRequest) -> Result<RunResult, SimulationError> {
        validate_request(request)?;

        let intents_by_date = index_entry_intents(&request.entry_intents)?;

        let mut ledger = Vec::with_capacity(request.bars.len());
        let mut cash = request.reference_flow.initial_cash;
        let mut position = PositionState::default();
        let mut pending_order: Option<PendingOrder> = None;

        for (index, bar) in request.bars.iter().enumerate() {
            let is_last_bar = index + 1 == request.bars.len();
            let mut reason_codes = Vec::new();
            let mut signal_output = "none".to_string();
            let mut filter_outcome = "not_checked".to_string();
            let mut fill_price = None;
            let prior_stop = position.active_stop;

            if let Some(order) = pending_order.take() {
                if position.shares != 0 {
                    return Err(SimulationError::invalid_request(
                        "queued market entry cannot fill while a position is already open",
                    ));
                }

                if order.intent != OrderIntent::QueueMarketEntry {
                    return Err(SimulationError::not_implemented());
                }

                let execution_price =
                    apply_buy_costs(bar.raw_open, &request.reference_flow.cost_model);
                let total_cash_out = total_buy_cash_out(
                    execution_price,
                    order.shares,
                    &request.reference_flow.cost_model,
                );

                if total_cash_out > cash {
                    return Err(SimulationError::invalid_request(
                        "queued market entry requires more cash than is available",
                    ));
                }

                cash -= total_cash_out;
                position.shares = order.shares;
                position.entry_price = Some(execution_price);
                position.active_stop = Some(round_price(
                    execution_price * (1.0 - request.reference_flow.protective_stop_fraction),
                ));
                fill_price = Some(execution_price);
                reason_codes.push("entry_filled_at_open".to_string());
                reason_codes.push("protective_stop_set".to_string());
            }

            if let Some(active_stop) = prior_stop {
                if position.shares == 0 {
                    return Err(SimulationError::invalid_request(
                        "active stop requires an open position",
                    ));
                }

                if let Some((execution_price, reason_code)) = resolve_stop_fill(
                    request.gap_policy,
                    active_stop,
                    bar,
                    &request.reference_flow.cost_model,
                ) {
                    cash += total_sell_cash_in(
                        execution_price,
                        position.shares,
                        &request.reference_flow.cost_model,
                    );
                    position = PositionState::default();
                    fill_price = Some(execution_price);
                    reason_codes.push(reason_code.to_string());
                }
            }

            if pending_order.is_none()
                && let Some(intent) = intents_by_date.get(bar.date.as_str())
            {
                if position.shares != 0 {
                    return Err(SimulationError::invalid_request(
                        "entry intents while a position is open are not supported in M1",
                    ));
                }

                if intent.shares != request.reference_flow.entry_shares {
                    return Err(SimulationError::invalid_request(
                        "entry-intent share count must match the reference flow entry_shares",
                    ));
                }

                signal_output = intent.intent.as_str().to_string();
                filter_outcome = "pass".to_string();
                pending_order = Some(PendingOrder {
                    intent: intent.intent,
                    shares: intent.shares,
                    stop_price: None,
                });
                reason_codes.push("entry_intent_queued".to_string());
            }

            if position.shares != 0 && reason_codes.is_empty() {
                reason_codes.push("hold_position".to_string());
            }

            if is_last_bar && position.shares != 0 {
                reason_codes.push("open_position_at_end".to_string());
            }

            let pending_order_state = pending_order
                .as_ref()
                .map(PendingOrder::describe_state)
                .unwrap_or_else(|| "none".to_string());
            let equity = round_price(cash + mark_to_market(bar.raw_close, position.shares));

            ledger.push(LedgerRow {
                date: bar.date.clone(),
                raw_open: bar.raw_open,
                raw_high: bar.raw_high,
                raw_low: bar.raw_low,
                raw_close: bar.raw_close,
                analysis_close: bar.analysis_close,
                position_shares: position.shares,
                signal_output,
                filter_outcome,
                pending_order_state,
                fill_price,
                prior_stop,
                next_stop: position.active_stop,
                cash: round_price(cash),
                equity,
                reason_codes,
            });
        }

        Ok(RunResult {
            ledger,
            cash: CashSummary {
                cash: round_price(cash),
                equity: round_price(
                    cash + mark_to_market(
                        request
                            .bars
                            .last()
                            .expect("validated non-empty bar sequence")
                            .raw_close,
                        position.shares,
                    ),
                ),
            },
            position,
        })
    }

    pub fn run_strategy_flow<S, F, P, E>(
        request: &StrategyRunRequest,
        strategy: &CompositeStrategy<S, F, P, E>,
    ) -> Result<RunResult, SimulationError>
    where
        S: SignalGenerator,
        F: SignalFilter,
        P: crate::strategy::PositionManager,
        E: crate::strategy::ExecutionModel,
    {
        validate_strategy_request(request)?;

        let mut ledger = Vec::with_capacity(request.bars.len());
        let mut cash = request.reference_flow.initial_cash;
        let mut position = PositionState::default();
        let mut pending_order: Option<PendingOrder> = None;

        for (index, bar) in request.bars.iter().enumerate() {
            let is_last_bar = index + 1 == request.bars.len();
            let mut reason_codes = Vec::new();
            let mut filter_outcome = "not_checked".to_string();
            let mut fill_price = None;
            let prior_stop = position.active_stop;
            let mut next_pending_order = pending_order.take();

            if let Some(order) = next_pending_order.take() {
                if position.shares != 0 {
                    return Err(SimulationError::invalid_request(
                        "pending strategy entry order cannot fill while a position is already open",
                    ));
                }

                match order.intent {
                    OrderIntent::QueueMarketEntry => {
                        let execution_price =
                            apply_buy_costs(bar.raw_open, &request.reference_flow.cost_model);
                        let total_cash_out = total_buy_cash_out(
                            execution_price,
                            order.shares,
                            &request.reference_flow.cost_model,
                        );

                        if total_cash_out > cash {
                            return Err(SimulationError::invalid_request(
                                "queued market entry requires more cash than is available",
                            ));
                        }

                        cash -= total_cash_out;
                        position.shares = order.shares;
                        position.entry_price = Some(execution_price);
                        fill_price = Some(execution_price);
                        reason_codes.push("entry_filled_at_open".to_string());
                    }
                    OrderIntent::CarryStopEntry => {
                        let stop_price = order.stop_price.ok_or_else(|| {
                            SimulationError::invalid_request(
                                "carried stop-entry orders require a stop_price",
                            )
                        })?;

                        if let Some((execution_price, reason_code)) = resolve_stop_entry_fill(
                            request.gap_policy,
                            stop_price,
                            bar,
                            &request.reference_flow.cost_model,
                        ) {
                            let total_cash_out = total_buy_cash_out(
                                execution_price,
                                order.shares,
                                &request.reference_flow.cost_model,
                            );

                            if total_cash_out > cash {
                                return Err(SimulationError::invalid_request(
                                    "stop-entry order requires more cash than is available",
                                ));
                            }

                            cash -= total_cash_out;
                            position.shares = order.shares;
                            position.entry_price = Some(execution_price);
                            fill_price = Some(execution_price);
                            reason_codes.push(reason_code.to_string());
                        } else {
                            reason_codes.push("pending_order_carried".to_string());
                            next_pending_order = Some(order);
                        }
                    }
                }
            }

            if let Some(active_stop) = prior_stop {
                if position.shares == 0 {
                    return Err(SimulationError::invalid_request(
                        "active stop requires an open position",
                    ));
                }

                if let Some((execution_price, reason_code)) = resolve_stop_fill(
                    request.gap_policy,
                    active_stop,
                    bar,
                    &request.reference_flow.cost_model,
                ) {
                    cash += total_sell_cash_in(
                        execution_price,
                        position.shares,
                        &request.reference_flow.cost_model,
                    );
                    position = PositionState::default();
                    next_pending_order = None;
                    fill_price = Some(execution_price);
                    reason_codes.push(reason_code.to_string());
                }
            }

            let evaluation = strategy.evaluate(&StrategyContext {
                symbol: request.symbol.as_str(),
                completed_bars: &request.bars[..index],
                current_bar: bar,
                position: &position,
                pending_order: next_pending_order.as_ref(),
            });

            let signal_output = describe_signal_output(&evaluation.signal);
            if !matches!(evaluation.signal, SignalDecision::None) {
                filter_outcome = describe_filter_outcome(&evaluation.filter);
            }

            match evaluation.position {
                PositionDecision::Keep => {}
                PositionDecision::SetProtectiveStop {
                    stop_price,
                    reason_code,
                } => {
                    if position.shares == 0 {
                        return Err(SimulationError::invalid_request(
                            "protective stop decisions require an open position",
                        ));
                    }

                    position.active_stop = Some(round_price(stop_price));
                    reason_codes.push(reason_code);
                }
                PositionDecision::Exit { .. } => {
                    return Err(SimulationError::not_implemented());
                }
            }

            match evaluation.execution {
                ExecutionDecision::None => {}
                ExecutionDecision::QueueMarketEntry { shares, .. } => {
                    next_pending_order = Some(PendingOrder {
                        intent: OrderIntent::QueueMarketEntry,
                        shares,
                        stop_price: None,
                    });
                    reason_codes.push("entry_queued_from_strategy".to_string());
                }
                ExecutionDecision::CarryStopEntry {
                    stop_price, shares, ..
                } => {
                    next_pending_order = Some(PendingOrder {
                        intent: OrderIntent::CarryStopEntry,
                        shares,
                        stop_price: Some(round_price(stop_price)),
                    });
                    reason_codes.push("stop_entry_order_carried".to_string());
                }
                ExecutionDecision::QueueMarketExit { .. } => {
                    return Err(SimulationError::not_implemented());
                }
                ExecutionDecision::Blocked { reason_code } => {
                    reason_codes.push(reason_code);
                }
            }

            if position.shares != 0 && reason_codes.is_empty() {
                reason_codes.push("hold_position".to_string());
            }

            if is_last_bar && position.shares != 0 {
                reason_codes.push("open_position_at_end".to_string());
            }

            if is_last_bar && next_pending_order.is_some() {
                reason_codes.push("pending_order_at_end".to_string());
            }

            let pending_order_state = next_pending_order
                .as_ref()
                .map(PendingOrder::describe_state)
                .unwrap_or_else(|| "none".to_string());
            let equity = round_price(cash + mark_to_market(bar.raw_close, position.shares));

            ledger.push(LedgerRow {
                date: bar.date.clone(),
                raw_open: bar.raw_open,
                raw_high: bar.raw_high,
                raw_low: bar.raw_low,
                raw_close: bar.raw_close,
                analysis_close: bar.analysis_close,
                position_shares: position.shares,
                signal_output,
                filter_outcome,
                pending_order_state,
                fill_price,
                prior_stop,
                next_stop: position.active_stop,
                cash: round_price(cash),
                equity,
                reason_codes,
            });

            pending_order = next_pending_order;
        }

        Ok(RunResult {
            ledger,
            cash: CashSummary {
                cash: round_price(cash),
                equity: round_price(
                    cash + mark_to_market(
                        request
                            .bars
                            .last()
                            .expect("validated non-empty bar sequence")
                            .raw_close,
                        position.shares,
                    ),
                ),
            },
            position,
        })
    }

    fn validate_request(request: &RunRequest) -> Result<(), SimulationError> {
        if request.symbol.trim().is_empty() {
            return Err(SimulationError::invalid_request(
                "run requests must include a non-empty symbol",
            ));
        }

        validate_reference_flow_spec(&request.reference_flow)?;
        let bar_dates = validate_bars(&request.bars)?;
        validate_entry_intent_inputs(
            &request.entry_intents,
            &bar_dates,
            request.reference_flow.entry_shares,
        )?;

        Ok(())
    }

    fn validate_strategy_request(request: &StrategyRunRequest) -> Result<(), SimulationError> {
        if request.symbol.trim().is_empty() {
            return Err(SimulationError::invalid_request(
                "run requests must include a non-empty symbol",
            ));
        }

        validate_reference_flow_spec(&request.reference_flow)?;
        validate_bars(&request.bars)?;

        Ok(())
    }

    fn validate_reference_flow_spec(
        reference_flow: &ReferenceFlowSpec,
    ) -> Result<(), SimulationError> {
        if reference_flow.entry_shares == 0 {
            return Err(SimulationError::invalid_request(
                "reference flow entry_shares must be greater than zero",
            ));
        }

        if reference_flow.protective_stop_fraction <= 0.0
            || reference_flow.protective_stop_fraction >= 1.0
        {
            return Err(SimulationError::invalid_request(
                "protective_stop_fraction must be greater than zero and less than one",
            ));
        }

        if reference_flow.cost_model.commission_per_fill < 0.0 {
            return Err(SimulationError::invalid_request(
                "commission_per_fill must not be negative",
            ));
        }

        if reference_flow.cost_model.slippage_per_share < 0.0 {
            return Err(SimulationError::invalid_request(
                "slippage_per_share must not be negative",
            ));
        }

        Ok(())
    }

    fn index_entry_intents(
        intents: &[EntryIntent],
    ) -> Result<BTreeMap<&str, &EntryIntent>, SimulationError> {
        let mut indexed = BTreeMap::new();

        for intent in intents {
            if indexed
                .insert(intent.signal_date.as_str(), intent)
                .is_some()
            {
                return Err(SimulationError::invalid_request(
                    "duplicate entry-intent signal_date values are not supported",
                ));
            }
        }

        Ok(indexed)
    }

    fn validate_bars(bars: &[DailyBar]) -> Result<BTreeSet<&str>, SimulationError> {
        if bars.is_empty() {
            return Err(SimulationError::invalid_request(
                "run requests must include at least one daily bar",
            ));
        }

        let mut dates = BTreeSet::new();
        let mut previous_date: Option<&str> = None;

        for bar in bars {
            if bar.date.trim().is_empty() {
                return Err(SimulationError::invalid_request(
                    "daily bars must include a non-empty date",
                ));
            }

            if let Some(previous_date) = previous_date
                && bar.date.as_str() <= previous_date
            {
                return Err(SimulationError::invalid_request(
                    "daily bars must be in strictly increasing date order with unique dates",
                ));
            }

            validate_bar_prices(bar)?;
            dates.insert(bar.date.as_str());
            previous_date = Some(bar.date.as_str());
        }

        Ok(dates)
    }

    fn validate_bar_prices(bar: &DailyBar) -> Result<(), SimulationError> {
        if [
            bar.raw_open,
            bar.raw_high,
            bar.raw_low,
            bar.raw_close,
            bar.analysis_close,
        ]
        .into_iter()
        .any(|value| value <= 0.0)
        {
            return Err(SimulationError::invalid_request(
                "daily bars must use positive raw and analysis prices",
            ));
        }

        let max_price = bar.raw_open.max(bar.raw_close).max(bar.raw_low);
        if bar.raw_high < max_price {
            return Err(SimulationError::invalid_request(
                "daily bars must satisfy raw_high >= max(raw_open, raw_close, raw_low)",
            ));
        }

        let min_price = bar.raw_open.min(bar.raw_close).min(bar.raw_high);
        if bar.raw_low > min_price {
            return Err(SimulationError::invalid_request(
                "daily bars must satisfy raw_low <= min(raw_open, raw_close, raw_high)",
            ));
        }

        Ok(())
    }

    fn validate_entry_intent_inputs(
        entry_intents: &[EntryIntent],
        bar_dates: &BTreeSet<&str>,
        expected_shares: u32,
    ) -> Result<(), SimulationError> {
        for intent in entry_intents {
            if intent.signal_date.trim().is_empty() {
                return Err(SimulationError::invalid_request(
                    "entry intents must include a non-empty signal_date",
                ));
            }

            if intent.shares == 0 {
                return Err(SimulationError::invalid_request(
                    "entry intents must request at least one share",
                ));
            }

            if intent.shares != expected_shares {
                return Err(SimulationError::invalid_request(
                    "entry-intent share count must match the reference flow entry_shares",
                ));
            }

            if !bar_dates.contains(intent.signal_date.as_str()) {
                return Err(SimulationError::invalid_request(format!(
                    "entry-intent signal_date `{}` does not match any bar date",
                    intent.signal_date
                )));
            }
        }

        Ok(())
    }

    fn apply_buy_costs(price: f64, cost_model: &CostModel) -> f64 {
        round_price(price + cost_model.slippage_per_share)
    }

    fn apply_sell_costs(price: f64, cost_model: &CostModel) -> f64 {
        round_price(price - cost_model.slippage_per_share)
    }

    fn total_buy_cash_out(price: f64, shares: u32, cost_model: &CostModel) -> f64 {
        round_price(price * f64::from(shares) + cost_model.commission_per_fill)
    }

    fn total_sell_cash_in(price: f64, shares: u32, cost_model: &CostModel) -> f64 {
        round_price(price * f64::from(shares) - cost_model.commission_per_fill)
    }

    fn resolve_stop_fill(
        gap_policy: GapPolicy,
        active_stop: f64,
        bar: &DailyBar,
        cost_model: &CostModel,
    ) -> Option<(f64, &'static str)> {
        match gap_policy {
            GapPolicy::M1Default => {
                if bar.raw_open <= active_stop {
                    Some((
                        apply_sell_costs(bar.raw_open, cost_model),
                        "protective_stop_hit_gap_open",
                    ))
                } else if bar.raw_low <= active_stop {
                    Some((
                        apply_sell_costs(active_stop, cost_model),
                        "protective_stop_hit_intrabar",
                    ))
                } else {
                    None
                }
            }
        }
    }

    fn resolve_stop_entry_fill(
        gap_policy: GapPolicy,
        entry_stop: f64,
        bar: &DailyBar,
        cost_model: &CostModel,
    ) -> Option<(f64, &'static str)> {
        match gap_policy {
            GapPolicy::M1Default => {
                if bar.raw_open >= entry_stop {
                    Some((
                        apply_buy_costs(bar.raw_open, cost_model),
                        "stop_entry_triggered_gap_open",
                    ))
                } else if bar.raw_high >= entry_stop {
                    Some((
                        apply_buy_costs(entry_stop, cost_model),
                        "stop_entry_triggered_intrabar",
                    ))
                } else {
                    None
                }
            }
        }
    }

    fn describe_signal_output(signal: &SignalDecision) -> String {
        match signal {
            SignalDecision::None => "none".to_string(),
            SignalDecision::EnterLong { signal_id, .. }
            | SignalDecision::ExitLong { signal_id } => signal_id.clone(),
        }
    }

    fn describe_filter_outcome(filter: &FilterDecision) -> String {
        match filter {
            FilterDecision::Pass => "pass".to_string(),
            FilterDecision::Block { reason_code } => format!("block:{reason_code}"),
        }
    }

    fn mark_to_market(raw_close: f64, shares: u32) -> f64 {
        round_price(raw_close * f64::from(shares))
    }

    fn round_price(value: f64) -> f64 {
        (value * 10_000.0).round() / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::CostModel;
    use crate::accounting::PositionState;
    use crate::engine::{ReferenceFlowSpec, RunRequest, run_reference_flow};
    use crate::market::DailyBar;
    use crate::orders::{EntryIntent, GapPolicy, OrderIntent, PendingOrder};
    use crate::strategy::{
        CloseConfirmedBreakoutSignal, CompositeStrategy, ExecutionDecision, FilterDecision,
        KeepPositionManager, NextOpenLongExecution, PassFilter, PositionDecision, SignalGenerator,
        StopEntryBreakoutSignal, StopEntryLongExecution, StrategyContext,
    };

    #[test]
    fn gap_policy_round_trips() {
        assert_eq!(
            GapPolicy::parse(GapPolicy::M1_DEFAULT),
            Some(GapPolicy::M1Default)
        );
        assert_eq!(GapPolicy::M1Default.as_str(), GapPolicy::M1_DEFAULT);
    }

    #[test]
    fn order_intent_round_trips() {
        assert_eq!(
            OrderIntent::parse(OrderIntent::QUEUE_MARKET_ENTRY),
            Some(OrderIntent::QueueMarketEntry)
        );
        assert_eq!(
            OrderIntent::QueueMarketEntry.as_str(),
            OrderIntent::QUEUE_MARKET_ENTRY
        );
        assert_eq!(
            OrderIntent::parse(OrderIntent::CARRY_STOP_ENTRY),
            Some(OrderIntent::CarryStopEntry)
        );
        assert_eq!(
            OrderIntent::CarryStopEntry.as_str(),
            OrderIntent::CARRY_STOP_ENTRY
        );
    }

    #[test]
    fn queued_entry_fills_next_open_and_marks_to_market() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![
                DailyBar {
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 99.0,
                    raw_close: 100.5,
                    analysis_close: 100.5,
                },
                DailyBar {
                    date: "2025-01-03".to_string(),
                    raw_open: 102.0,
                    raw_high: 104.0,
                    raw_low: 101.5,
                    raw_close: 103.0,
                    analysis_close: 103.0,
                },
            ],
            entry_intents: vec![EntryIntent {
                signal_date: "2025-01-02".to_string(),
                intent: OrderIntent::QueueMarketEntry,
                shares: 1,
            }],
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let result = run_reference_flow(&request).expect("week 3 path should be supported");

        assert_eq!(result.ledger.len(), 2);
        assert_eq!(result.ledger[0].pending_order_state, "queue_market_entry:1");
        assert_eq!(result.ledger[0].reason_codes, vec!["entry_intent_queued"]);
        assert_eq!(result.ledger[1].fill_price, Some(102.0));
        assert_eq!(result.ledger[1].cash, 898.0);
        assert_eq!(result.ledger[1].equity, 1001.0);
        assert_eq!(result.ledger[1].next_stop, Some(91.8));
    }

    #[test]
    fn intrabar_protective_stop_fills_at_stop_price() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![
                DailyBar {
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 99.0,
                    raw_close: 100.5,
                    analysis_close: 100.5,
                },
                DailyBar {
                    date: "2025-01-03".to_string(),
                    raw_open: 102.0,
                    raw_high: 104.0,
                    raw_low: 101.0,
                    raw_close: 103.5,
                    analysis_close: 103.5,
                },
                DailyBar {
                    date: "2025-01-06".to_string(),
                    raw_open: 103.0,
                    raw_high: 103.5,
                    raw_low: 91.0,
                    raw_close: 92.0,
                    analysis_close: 92.0,
                },
            ],
            entry_intents: vec![EntryIntent {
                signal_date: "2025-01-02".to_string(),
                intent: OrderIntent::QueueMarketEntry,
                shares: 1,
            }],
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let result = run_reference_flow(&request).expect("intrabar stop should be supported");

        assert_eq!(result.ledger[2].fill_price, Some(91.8));
        assert_eq!(result.ledger[2].prior_stop, Some(91.8));
        assert_eq!(result.ledger[2].next_stop, None);
        assert_eq!(result.ledger[2].position_shares, 0);
        assert_eq!(result.ledger[2].cash, 989.8);
        assert_eq!(result.ledger[2].equity, 989.8);
        assert_eq!(
            result.ledger[2].reason_codes,
            vec!["protective_stop_hit_intrabar"]
        );
    }

    #[test]
    fn gap_open_protective_stop_fills_at_open() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![
                DailyBar {
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 99.0,
                    raw_close: 100.5,
                    analysis_close: 100.5,
                },
                DailyBar {
                    date: "2025-01-03".to_string(),
                    raw_open: 102.0,
                    raw_high: 104.0,
                    raw_low: 101.0,
                    raw_close: 103.5,
                    analysis_close: 103.5,
                },
                DailyBar {
                    date: "2025-01-06".to_string(),
                    raw_open: 90.0,
                    raw_high: 91.0,
                    raw_low: 89.0,
                    raw_close: 90.5,
                    analysis_close: 90.5,
                },
            ],
            entry_intents: vec![EntryIntent {
                signal_date: "2025-01-02".to_string(),
                intent: OrderIntent::QueueMarketEntry,
                shares: 1,
            }],
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let result = run_reference_flow(&request).expect("gap-open stop should be supported");

        assert_eq!(result.ledger[2].fill_price, Some(90.0));
        assert_eq!(result.ledger[2].prior_stop, Some(91.8));
        assert_eq!(result.ledger[2].next_stop, None);
        assert_eq!(result.ledger[2].position_shares, 0);
        assert_eq!(result.ledger[2].cash, 988.0);
        assert_eq!(result.ledger[2].equity, 988.0);
        assert_eq!(
            result.ledger[2].reason_codes,
            vec!["protective_stop_hit_gap_open"]
        );
    }

    #[test]
    fn stop_set_by_today_entry_cannot_trigger_on_same_bar() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![
                DailyBar {
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 99.0,
                    raw_close: 100.5,
                    analysis_close: 100.5,
                },
                DailyBar {
                    date: "2025-01-03".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 89.0,
                    raw_close: 90.0,
                    analysis_close: 90.0,
                },
            ],
            entry_intents: vec![EntryIntent {
                signal_date: "2025-01-02".to_string(),
                intent: OrderIntent::QueueMarketEntry,
                shares: 1,
            }],
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let result =
            run_reference_flow(&request).expect("newly set stop should not trigger on entry bar");

        assert_eq!(result.ledger[1].position_shares, 1);
        assert_eq!(result.ledger[1].fill_price, Some(100.0));
        assert_eq!(result.ledger[1].prior_stop, None);
        assert_eq!(result.ledger[1].next_stop, Some(90.0));
        assert_eq!(
            result.ledger[1].reason_codes,
            vec![
                "entry_filled_at_open".to_string(),
                "protective_stop_set".to_string(),
                "open_position_at_end".to_string(),
            ]
        );
    }

    #[test]
    fn bars_must_be_strictly_increasing() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![
                DailyBar {
                    date: "2025-01-03".to_string(),
                    raw_open: 102.0,
                    raw_high: 104.0,
                    raw_low: 101.0,
                    raw_close: 103.5,
                    analysis_close: 103.5,
                },
                DailyBar {
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 101.0,
                    raw_low: 99.0,
                    raw_close: 100.5,
                    analysis_close: 100.5,
                },
            ],
            entry_intents: Vec::new(),
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let error = run_reference_flow(&request).unwrap_err();

        assert_eq!(
            error.to_string(),
            "daily bars must be in strictly increasing date order with unique dates"
        );
    }

    #[test]
    fn entry_intents_must_land_on_known_bar_dates() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.5,
                analysis_close: 100.5,
            }],
            entry_intents: vec![EntryIntent {
                signal_date: "2025-01-03".to_string(),
                intent: OrderIntent::QueueMarketEntry,
                shares: 1,
            }],
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 0.10,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let error = run_reference_flow(&request).unwrap_err();

        assert_eq!(
            error.to_string(),
            "entry-intent signal_date `2025-01-03` does not match any bar date"
        );
    }

    #[test]
    fn protective_stop_fraction_must_stay_between_zero_and_one() {
        let request = RunRequest {
            symbol: "TEST".to_string(),
            bars: vec![DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.5,
                analysis_close: 100.5,
            }],
            entry_intents: Vec::new(),
            reference_flow: ReferenceFlowSpec {
                initial_cash: 1000.0,
                entry_shares: 1,
                protective_stop_fraction: 1.0,
                cost_model: CostModel::default(),
            },
            gap_policy: GapPolicy::M1Default,
        };

        let error = run_reference_flow(&request).unwrap_err();

        assert_eq!(
            error.to_string(),
            "protective_stop_fraction must be greater than zero and less than one"
        );
    }

    #[test]
    fn strategy_context_exposes_current_date_without_lookahead_state() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let context = StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..1],
            current_bar: &bars[1],
            position: &position,
            pending_order: None,
        };

        assert_eq!(context.current_date(), "2025-01-03");
        assert_eq!(context.completed_bars.len(), 1);
        assert_eq!(context.completed_bars[0].date, "2025-01-02");
    }

    #[test]
    fn close_confirmed_breakout_requires_strict_break_of_trailing_analysis_close() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let signal = CloseConfirmedBreakoutSignal::new(3);
        let context = StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..3],
            current_bar: &bars[3],
            position: &position,
            pending_order: None,
        };

        let evaluation = signal.evaluate(&context);

        assert_eq!(
            evaluation,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "close_confirmed_breakout".to_string(),
                entry_reference: Some(102.0),
            }
        );
    }

    #[test]
    fn close_confirmed_breakout_uses_analysis_series_not_raw_close() {
        let bars = [
            DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.0,
                analysis_close: 100.0,
            },
            DailyBar {
                date: "2025-01-03".to_string(),
                raw_open: 90.0,
                raw_high: 91.0,
                raw_low: 89.0,
                raw_close: 90.0,
                analysis_close: 101.0,
            },
        ];
        let position = PositionState::default();
        let signal = CloseConfirmedBreakoutSignal::new(1);

        let evaluation = signal.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..1],
            current_bar: &bars[1],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "close_confirmed_breakout".to_string(),
                entry_reference: Some(100.0),
            }
        );
    }

    #[test]
    fn stop_entry_breakout_carries_trailing_raw_high_without_close_confirmation() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let signal = StopEntryBreakoutSignal::new(3);
        let evaluation = signal.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..2],
            current_bar: &bars[2],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "stop_entry_breakout".to_string(),
                entry_reference: Some(103.5),
            }
        );
    }

    #[test]
    fn composite_strategy_queues_next_open_entry_for_close_confirmed_breakout() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let strategy = CompositeStrategy::new(
            CloseConfirmedBreakoutSignal::new(3),
            PassFilter,
            KeepPositionManager,
            NextOpenLongExecution::new(1),
        );
        let evaluation = strategy.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..3],
            current_bar: &bars[3],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation.signal,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "close_confirmed_breakout".to_string(),
                entry_reference: Some(102.0),
            }
        );
        assert_eq!(evaluation.filter, FilterDecision::Pass);
        assert_eq!(evaluation.position, PositionDecision::Keep);
        assert_eq!(
            evaluation.execution,
            ExecutionDecision::QueueMarketEntry {
                shares: 1,
                signal_id: "close_confirmed_breakout".to_string(),
            }
        );
    }

    #[test]
    fn composite_strategy_carries_stop_entry_threshold_when_flat() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let strategy = CompositeStrategy::new(
            StopEntryBreakoutSignal::new(3),
            PassFilter,
            KeepPositionManager,
            StopEntryLongExecution::new(1),
        );
        let evaluation = strategy.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..2],
            current_bar: &bars[2],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation.signal,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "stop_entry_breakout".to_string(),
                entry_reference: Some(103.5),
            }
        );
        assert_eq!(evaluation.filter, FilterDecision::Pass);
        assert_eq!(evaluation.position, PositionDecision::Keep);
        assert_eq!(
            evaluation.execution,
            ExecutionDecision::CarryStopEntry {
                stop_price: 103.5,
                shares: 1,
                signal_id: "stop_entry_breakout".to_string(),
            }
        );
    }

    #[test]
    fn next_open_execution_blocks_when_position_is_already_open() {
        let bars = sample_breakout_bars();
        let position = PositionState {
            shares: 1,
            entry_price: Some(100.0),
            active_stop: Some(90.0),
        };
        let strategy = CompositeStrategy::new(
            CloseConfirmedBreakoutSignal::new(3),
            PassFilter,
            KeepPositionManager,
            NextOpenLongExecution::new(1),
        );
        let evaluation = strategy.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..3],
            current_bar: &bars[3],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation.signal,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "close_confirmed_breakout".to_string(),
                entry_reference: Some(102.0),
            }
        );
        assert_eq!(evaluation.filter, FilterDecision::Pass);
        assert_eq!(evaluation.position, PositionDecision::Keep);
        assert_eq!(
            evaluation.execution,
            ExecutionDecision::Blocked {
                reason_code: "position_already_open".to_string(),
            }
        );
    }

    #[test]
    fn stop_entry_execution_blocks_when_position_is_already_open() {
        let bars = sample_breakout_bars();
        let position = PositionState {
            shares: 1,
            entry_price: Some(100.0),
            active_stop: Some(90.0),
        };
        let strategy = CompositeStrategy::new(
            StopEntryBreakoutSignal::new(3),
            PassFilter,
            KeepPositionManager,
            StopEntryLongExecution::new(1),
        );
        let evaluation = strategy.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..2],
            current_bar: &bars[2],
            position: &position,
            pending_order: None,
        });

        assert_eq!(
            evaluation.signal,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "stop_entry_breakout".to_string(),
                entry_reference: Some(103.5),
            }
        );
        assert_eq!(evaluation.filter, FilterDecision::Pass);
        assert_eq!(evaluation.position, PositionDecision::Keep);
        assert_eq!(
            evaluation.execution,
            ExecutionDecision::Blocked {
                reason_code: "position_already_open".to_string(),
            }
        );
    }

    #[test]
    fn stop_entry_execution_blocks_duplicate_pending_threshold() {
        let bars = sample_breakout_bars();
        let position = PositionState::default();
        let pending_order = PendingOrder {
            intent: OrderIntent::CarryStopEntry,
            shares: 1,
            stop_price: Some(103.5),
        };
        let strategy = CompositeStrategy::new(
            StopEntryBreakoutSignal::new(3),
            PassFilter,
            KeepPositionManager,
            StopEntryLongExecution::new(1),
        );
        let evaluation = strategy.evaluate(&StrategyContext {
            symbol: "TEST",
            completed_bars: &bars[..2],
            current_bar: &bars[2],
            position: &position,
            pending_order: Some(&pending_order),
        });

        assert_eq!(
            evaluation.signal,
            crate::strategy::SignalDecision::EnterLong {
                signal_id: "stop_entry_breakout".to_string(),
                entry_reference: Some(103.5),
            }
        );
        assert_eq!(evaluation.filter, FilterDecision::Pass);
        assert_eq!(evaluation.position, PositionDecision::Keep);
        assert_eq!(
            evaluation.execution,
            ExecutionDecision::Blocked {
                reason_code: "pending_order_already_active".to_string(),
            }
        );
    }

    fn sample_breakout_bars() -> Vec<DailyBar> {
        vec![
            DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.5,
                analysis_close: 100.5,
            },
            DailyBar {
                date: "2025-01-03".to_string(),
                raw_open: 101.0,
                raw_high: 103.0,
                raw_low: 100.0,
                raw_close: 101.0,
                analysis_close: 101.0,
            },
            DailyBar {
                date: "2025-01-06".to_string(),
                raw_open: 101.0,
                raw_high: 103.5,
                raw_low: 100.5,
                raw_close: 102.0,
                analysis_close: 102.0,
            },
            DailyBar {
                date: "2025-01-07".to_string(),
                raw_open: 102.5,
                raw_high: 104.5,
                raw_low: 102.0,
                raw_close: 103.0,
                analysis_close: 103.0,
            },
        ]
    }
}
