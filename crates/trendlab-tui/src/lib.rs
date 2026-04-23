#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::Marker;
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph, Wrap,
};
use trendlab_artifact::{
    BUNDLE_FILE_NAME, PersistedLedgerRow, RESEARCH_REPORT_FILE_NAME, ReplayBundle, ResearchReport,
    load_replay_bundle, load_research_report_bundle,
};
use trendlab_core::accounting::CostModel;
use trendlab_core::engine::ReferenceFlowSpec;
use trendlab_core::orders::{EntryIntent, GapPolicy, OrderIntent};
use trendlab_data::audit::{DataAuditReport, audit_daily_bars};
use trendlab_data::inspect::{SnapshotInspectionReport, inspect_snapshot_bundle};
use trendlab_data::run_source::{
    SnapshotRunFormOptions, SnapshotRunSymbolOptions, snapshot_run_form_options,
};
use trendlab_data::snapshot_store::load_snapshot_bundle;
use trendlab_operator::{
    OperatorRunManifestSpec, OperatorRunRequestTemplate, OperatorRunSpec,
    OperatorSnapshotSourceSpec, RUN_REQUEST_SOURCE_PARAMETER, RUN_SOURCE_KIND_PARAMETER,
    RUN_SPEC_SOURCE_PARAMETER, RunSpecPreview, SNAPSHOT_SELECTION_END_PARAMETER,
    SNAPSHOT_SELECTION_START_PARAMETER, SNAPSHOT_SOURCE_PATH_PARAMETER, execute_run_spec,
    preview_run_spec,
};

const APP_TITLE: &str = "TrendLab TUI";
const USAGE: &str =
    "usage: trendlab-tui [open <artifact-dir>|<artifact-dir>] [--snapshot <snapshot-dir> ...]";
const DEFAULT_INITIAL_CASH: f64 = 1000.0;
const DEFAULT_ENTRY_SHARES: u32 = 1;
const DEFAULT_PROTECTIVE_STOP_FRACTION: f64 = 0.10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiError {
    message: String,
}

impl TuiError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn io(action: &str, err: &std::io::Error) -> Self {
        Self::invalid(format!("{action}: {err}"))
    }
}

impl Display for TuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TuiError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppCommand {
    NextFocus,
    PreviousFocus,
    AdjustPrevious,
    AdjustNext,
    LaunchRun,
    MoveUp,
    MoveDown,
    ShowHome,
    ShowInspect,
    ShowHelp,
    ToggleHelp,
    Quit,
}

impl AppCommand {
    fn from_key_event(key: &KeyEvent) -> Option<Self> {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return None;
        }

        match key.code {
            KeyCode::Tab | KeyCode::Char('l') => Some(Self::NextFocus),
            KeyCode::BackTab | KeyCode::Char('h') => Some(Self::PreviousFocus),
            KeyCode::Left | KeyCode::Char('[') => Some(Self::AdjustPrevious),
            KeyCode::Right | KeyCode::Char(']') => Some(Self::AdjustNext),
            KeyCode::Enter | KeyCode::Char('r') => Some(Self::LaunchRun),
            KeyCode::Up | KeyCode::Char('k') => Some(Self::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Self::MoveDown),
            KeyCode::Char('1') => Some(Self::ShowHome),
            KeyCode::Char('2') => Some(Self::ShowInspect),
            KeyCode::Char('3') => Some(Self::ShowHelp),
            KeyCode::Char('?') => Some(Self::ToggleHelp),
            KeyCode::Esc | KeyCode::Char('q') => Some(Self::Quit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMode {
    Home,
    Inspect,
    Research,
    Help,
}

impl AppMode {
    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Inspect => "Inspect",
            Self::Research => "Research",
            Self::Help => "Help",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusPane {
    Results,
    Chart,
    Ledger,
    Help,
}

impl FocusPane {
    fn next(self) -> Self {
        match self {
            Self::Results => Self::Chart,
            Self::Chart => Self::Ledger,
            Self::Ledger => Self::Help,
            Self::Help => Self::Results,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Results => Self::Help,
            Self::Chart => Self::Results,
            Self::Ledger => Self::Chart,
            Self::Help => Self::Ledger,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Results => "Results",
            Self::Chart => "Chart",
            Self::Ledger => "Ledger",
            Self::Help => "Help",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResearchFocusPane {
    Summary,
    Items,
    Detail,
}

impl ResearchFocusPane {
    fn next(self) -> Self {
        match self {
            Self::Summary => Self::Items,
            Self::Items => Self::Detail,
            Self::Detail => Self::Summary,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Summary => Self::Detail,
            Self::Items => Self::Summary,
            Self::Detail => Self::Items,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Items => "Items",
            Self::Detail => "Detail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HomeFocusPane {
    Snapshots,
    RunForm,
    History,
}

impl HomeFocusPane {
    fn next(self) -> Self {
        match self {
            Self::Snapshots => Self::RunForm,
            Self::RunForm => Self::History,
            Self::History => Self::Snapshots,
        }
    }

    fn previous(self) -> Self {
        self.next()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunFormField {
    Symbol,
    StartDate,
    EndDate,
    SignalDate,
    InitialCash,
    EntryShares,
    ProtectiveStopFraction,
    CommissionPerFill,
    SlippagePerShare,
    GapPolicy,
}

impl RunFormField {
    const ALL: [Self; 10] = [
        Self::Symbol,
        Self::StartDate,
        Self::EndDate,
        Self::SignalDate,
        Self::InitialCash,
        Self::EntryShares,
        Self::ProtectiveStopFraction,
        Self::CommissionPerFill,
        Self::SlippagePerShare,
        Self::GapPolicy,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::StartDate => "start_date",
            Self::EndDate => "end_date",
            Self::SignalDate => "signal_date",
            Self::InitialCash => "initial_cash",
            Self::EntryShares => "entry_shares",
            Self::ProtectiveStopFraction => "protective_stop_fraction",
            Self::CommissionPerFill => "commission_per_fill",
            Self::SlippagePerShare => "slippage_per_share",
            Self::GapPolicy => "gap_policy",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultItem {
    label: String,
    value: String,
    detail_title: String,
    detail_lines: Vec<String>,
    anchor_ledger_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
struct TradeSummary {
    sequence: usize,
    entry_row_index: usize,
    entry_date: String,
    entry_price: f64,
    entry_equity: f64,
    entry_stop: Option<f64>,
    entry_reasons: Vec<String>,
    exit_row_index: Option<usize>,
    exit_date: Option<String>,
    exit_price: Option<f64>,
    exit_equity: Option<f64>,
    exit_reasons: Vec<String>,
    mark_date: String,
    mark_price: f64,
    bars_held: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct InspectApp {
    bundle_path: PathBuf,
    bundle: ReplayBundle,
    audit_report: DataAuditReport,
    results: Vec<ResultItem>,
    focus: FocusPane,
    selected_result: usize,
    selected_ledger: usize,
    help_expanded: bool,
    should_quit: bool,
}

impl InspectApp {
    fn from_bundle(bundle_path: PathBuf, bundle: ReplayBundle) -> Self {
        let bars = bundle
            .ledger
            .iter()
            .map(PersistedLedgerRow::market_bar)
            .collect::<Vec<_>>();
        let audit_report = audit_daily_bars(&bars);
        let results = build_result_items(&bundle_path, &bundle, &audit_report);

        Self {
            bundle_path,
            bundle,
            audit_report,
            results,
            focus: FocusPane::Results,
            selected_result: 0,
            selected_ledger: 0,
            help_expanded: true,
            should_quit: false,
        }
    }

    fn apply(&mut self, command: AppCommand) {
        match command {
            AppCommand::NextFocus => self.focus = self.focus.next(),
            AppCommand::PreviousFocus => self.focus = self.focus.previous(),
            AppCommand::MoveUp => match self.focus {
                FocusPane::Results => self.move_result_selection_up(),
                FocusPane::Chart | FocusPane::Ledger => {
                    cycle_selection_up(&mut self.selected_ledger, self.bundle.ledger.len())
                }
                FocusPane::Help => {}
            },
            AppCommand::MoveDown => match self.focus {
                FocusPane::Results => self.move_result_selection_down(),
                FocusPane::Ledger => {
                    cycle_selection_down(&mut self.selected_ledger, self.bundle.ledger.len())
                }
                FocusPane::Chart => {
                    cycle_selection_down(&mut self.selected_ledger, self.bundle.ledger.len())
                }
                FocusPane::Help => {}
            },
            AppCommand::AdjustPrevious | AppCommand::AdjustNext | AppCommand::LaunchRun => {}
            AppCommand::ShowHome | AppCommand::ShowInspect | AppCommand::ShowHelp => {}
            AppCommand::ToggleHelp => self.help_expanded = !self.help_expanded,
            AppCommand::Quit => self.should_quit = true,
        }
    }

    fn selected_result(&self) -> Option<&ResultItem> {
        self.results.get(self.selected_result)
    }

    fn selected_result_index(&self) -> Option<usize> {
        (!self.results.is_empty()).then_some(self.selected_result)
    }

    fn selected_result_anchor_ledger_index(&self) -> Option<usize> {
        self.selected_result()
            .and_then(|item| item.anchor_ledger_index)
            .filter(|index| *index < self.bundle.ledger.len())
    }

    fn selected_ledger_row(&self) -> Option<&PersistedLedgerRow> {
        self.bundle.ledger.get(self.selected_ledger)
    }

    fn selected_ledger_index(&self) -> Option<usize> {
        (!self.bundle.ledger.is_empty()).then_some(self.selected_ledger)
    }

    fn move_result_selection_up(&mut self) {
        cycle_selection_up(&mut self.selected_result, self.results.len());
        self.sync_ledger_selection_from_result();
    }

    fn move_result_selection_down(&mut self) {
        cycle_selection_down(&mut self.selected_result, self.results.len());
        self.sync_ledger_selection_from_result();
    }

    fn sync_ledger_selection_from_result(&mut self) {
        if let Some(index) = self.selected_result_anchor_ledger_index() {
            self.selected_ledger = index;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResearchApp {
    report_path: PathBuf,
    report: ResearchReport,
    items: Vec<ResearchItem>,
    focus: ResearchFocusPane,
    selected_item: usize,
    selected_link: usize,
}

impl ResearchApp {
    fn from_report(report_path: PathBuf, report: ResearchReport) -> Self {
        Self {
            report_path,
            items: build_research_items(&report),
            report,
            focus: ResearchFocusPane::Summary,
            selected_item: 0,
            selected_link: 0,
        }
    }

    fn apply(&mut self, command: AppCommand) {
        match command {
            AppCommand::NextFocus => self.focus = self.focus.next(),
            AppCommand::PreviousFocus => self.focus = self.focus.previous(),
            AppCommand::MoveUp if self.focus == ResearchFocusPane::Items => self.move_item_up(),
            AppCommand::MoveDown if self.focus == ResearchFocusPane::Items => self.move_item_down(),
            AppCommand::MoveUp if self.focus == ResearchFocusPane::Detail => self.move_link_up(),
            AppCommand::MoveDown if self.focus == ResearchFocusPane::Detail => {
                self.move_link_down()
            }
            AppCommand::AdjustPrevious
            | AppCommand::AdjustNext
            | AppCommand::LaunchRun
            | AppCommand::MoveUp
            | AppCommand::MoveDown
            | AppCommand::ShowHome
            | AppCommand::ShowInspect
            | AppCommand::ShowHelp
            | AppCommand::ToggleHelp
            | AppCommand::Quit => {}
        }
    }

    fn selected_item(&self) -> Option<&ResearchItem> {
        self.items.get(self.selected_item)
    }

    fn selected_item_index(&self) -> Option<usize> {
        (!self.items.is_empty()).then_some(self.selected_item)
    }

    fn selected_link_path(&self) -> Option<&PathBuf> {
        self.selected_item()?
            .linked_bundle_paths
            .get(self.selected_link)
    }

    fn selected_link_index(&self) -> Option<usize> {
        (!self.selected_link_paths().is_empty()).then_some(self.selected_link)
    }

    fn selected_link_paths(&self) -> &[PathBuf] {
        self.selected_item()
            .map(|item| item.linked_bundle_paths.as_slice())
            .unwrap_or(&[])
    }

    fn move_item_up(&mut self) {
        cycle_selection_up(&mut self.selected_item, self.items.len());
        self.selected_link = 0;
        self.clamp_selected_link();
    }

    fn move_item_down(&mut self) {
        cycle_selection_down(&mut self.selected_item, self.items.len());
        self.selected_link = 0;
        self.clamp_selected_link();
    }

    fn move_link_up(&mut self) {
        let link_count = self.selected_link_paths().len();
        cycle_selection_up(&mut self.selected_link, link_count);
    }

    fn move_link_down(&mut self) {
        let link_count = self.selected_link_paths().len();
        cycle_selection_down(&mut self.selected_link, link_count);
    }

    fn clamp_selected_link(&mut self) {
        let link_count = self.selected_link_paths().len();
        if link_count == 0 {
            self.selected_link = 0;
        } else {
            self.selected_link = self.selected_link.min(link_count - 1);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResearchItem {
    label: String,
    value: String,
    detail_title: String,
    detail_lines: Vec<String>,
    linked_bundle_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
struct SnapshotBrowser {
    entries: Vec<SnapshotBrowserEntry>,
    selected_snapshot: usize,
}

impl SnapshotBrowser {
    fn from_paths(snapshot_paths: Vec<PathBuf>) -> Self {
        Self {
            entries: snapshot_paths
                .into_iter()
                .map(load_snapshot_entry)
                .collect(),
            selected_snapshot: 0,
        }
    }

    fn selected(&self) -> Option<&SnapshotBrowserEntry> {
        self.entries.get(self.selected_snapshot)
    }

    fn move_up(&mut self) {
        cycle_selection_up(&mut self.selected_snapshot, self.entries.len());
    }

    fn move_down(&mut self) {
        cycle_selection_down(&mut self.selected_snapshot, self.entries.len());
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SnapshotBrowserEntry {
    path: PathBuf,
    state: SnapshotLoadState,
}

#[derive(Clone, Debug, PartialEq)]
enum SnapshotLoadState {
    Loaded(Box<LoadedSnapshot>),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
struct LoadedSnapshot {
    report: SnapshotInspectionReport,
    run_form_options: SnapshotRunFormOptions,
}

#[derive(Clone, Debug, PartialEq)]
struct RunHistoryBrowser {
    root: PathBuf,
    entries: Vec<RunHistoryEntry>,
    selected_run: usize,
}

impl RunHistoryBrowser {
    fn load(root: PathBuf) -> Self {
        let mut entries = load_history_entries(&root);
        entries.sort_by(|left, right| right.path.cmp(&left.path));

        Self {
            root,
            entries,
            selected_run: 0,
        }
    }

    fn selected(&self) -> Option<&RunHistoryEntry> {
        self.entries.get(self.selected_run)
    }

    fn move_up(&mut self) {
        cycle_selection_up(&mut self.selected_run, self.entries.len());
    }

    fn move_down(&mut self) {
        cycle_selection_down(&mut self.selected_run, self.entries.len());
    }

    fn refresh(&mut self) {
        let selected_path = self.selected().map(|entry| entry.path.clone());
        *self = Self::load(self.root.clone());

        if let Some(selected_path) = selected_path
            && let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.path == selected_path)
        {
            self.selected_run = index;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RunHistoryEntry {
    path: PathBuf,
    state: RunHistoryState,
}

#[derive(Clone, Debug, PartialEq)]
enum RunHistoryState {
    Loaded(Box<RunHistoryPreview>),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
struct RunHistoryPreview {
    snapshot_id: String,
    provider_identity: String,
    symbol: String,
    start_date: String,
    end_date: String,
    row_count: usize,
    warning_count: usize,
    ending_equity: f64,
    run_source_kind: String,
    request_source: String,
    spec_source: String,
    snapshot_source_path: String,
    snapshot_selection: String,
}

#[derive(Clone, Debug, PartialEq)]
struct RunFormDraft {
    selected_field: usize,
    symbol_index: usize,
    start_date_index: usize,
    end_date_index: usize,
    signal_date_index: usize,
    initial_cash: f64,
    entry_shares: u32,
    protective_stop_fraction: f64,
    commission_per_fill: f64,
    slippage_per_share: f64,
}

impl Default for RunFormDraft {
    fn default() -> Self {
        Self {
            selected_field: 0,
            symbol_index: 0,
            start_date_index: 0,
            end_date_index: 0,
            signal_date_index: 0,
            initial_cash: DEFAULT_INITIAL_CASH,
            entry_shares: DEFAULT_ENTRY_SHARES,
            protective_stop_fraction: DEFAULT_PROTECTIVE_STOP_FRACTION,
            commission_per_fill: 0.0,
            slippage_per_share: 0.0,
        }
    }
}

impl RunFormDraft {
    fn move_up(&mut self) {
        cycle_selection_up(&mut self.selected_field, RunFormField::ALL.len());
    }

    fn move_down(&mut self) {
        cycle_selection_down(&mut self.selected_field, RunFormField::ALL.len());
    }

    fn selected_field(&self) -> RunFormField {
        RunFormField::ALL[self.selected_field]
    }

    fn selected_symbol<'a>(
        &self,
        options: &'a SnapshotRunFormOptions,
    ) -> Option<&'a SnapshotRunSymbolOptions> {
        options.symbols.get(self.symbol_index)
    }

    fn clamp_to_snapshot(&mut self, options: &SnapshotRunFormOptions) {
        if options.symbols.is_empty() {
            self.symbol_index = 0;
            self.start_date_index = 0;
            self.end_date_index = 0;
            self.signal_date_index = 0;
            return;
        }

        self.symbol_index = self.symbol_index.min(options.symbols.len() - 1);
        self.clamp_date_indices(options);
    }

    fn clamp_date_indices(&mut self, options: &SnapshotRunFormOptions) {
        let Some(symbol) = self.selected_symbol(options) else {
            self.start_date_index = 0;
            self.end_date_index = 0;
            self.signal_date_index = 0;
            return;
        };

        if symbol.available_dates.is_empty() {
            self.start_date_index = 0;
            self.end_date_index = 0;
            self.signal_date_index = 0;
            return;
        }

        let last_index = symbol.available_dates.len() - 1;
        self.start_date_index = self.start_date_index.min(last_index);
        self.end_date_index = self.end_date_index.min(last_index);
        self.signal_date_index = self.signal_date_index.min(last_index);
    }

    fn adjust(&mut self, options: &SnapshotRunFormOptions, forward: bool) {
        match self.selected_field() {
            RunFormField::Symbol => {
                if options.symbols.is_empty() {
                    return;
                }
                if forward {
                    cycle_selection_down(&mut self.symbol_index, options.symbols.len());
                } else {
                    cycle_selection_up(&mut self.symbol_index, options.symbols.len());
                }
                self.clamp_date_indices(options);
            }
            RunFormField::StartDate => {
                let Some(symbol) = self.selected_symbol(options) else {
                    return;
                };
                if forward {
                    cycle_selection_down(&mut self.start_date_index, symbol.available_dates.len());
                } else {
                    cycle_selection_up(&mut self.start_date_index, symbol.available_dates.len());
                }
            }
            RunFormField::EndDate => {
                let Some(symbol) = self.selected_symbol(options) else {
                    return;
                };
                if forward {
                    cycle_selection_down(&mut self.end_date_index, symbol.available_dates.len());
                } else {
                    cycle_selection_up(&mut self.end_date_index, symbol.available_dates.len());
                }
            }
            RunFormField::SignalDate => {
                let Some(symbol) = self.selected_symbol(options) else {
                    return;
                };
                if forward {
                    cycle_selection_down(&mut self.signal_date_index, symbol.available_dates.len());
                } else {
                    cycle_selection_up(&mut self.signal_date_index, symbol.available_dates.len());
                }
            }
            RunFormField::InitialCash => {
                self.initial_cash = adjust_f64(
                    self.initial_cash,
                    if forward { 100.0 } else { -100.0 },
                    0.0,
                    None,
                );
            }
            RunFormField::EntryShares => {
                self.entry_shares =
                    adjust_u32(self.entry_shares, if forward { 1 } else { -1 }, 0, None);
            }
            RunFormField::ProtectiveStopFraction => {
                self.protective_stop_fraction = adjust_f64(
                    self.protective_stop_fraction,
                    if forward { 0.01 } else { -0.01 },
                    0.0,
                    Some(1.0),
                );
            }
            RunFormField::CommissionPerFill => {
                self.commission_per_fill = adjust_f64(
                    self.commission_per_fill,
                    if forward { 0.25 } else { -0.25 },
                    0.0,
                    None,
                );
            }
            RunFormField::SlippagePerShare => {
                self.slippage_per_share = adjust_f64(
                    self.slippage_per_share,
                    if forward { 0.01 } else { -0.01 },
                    0.0,
                    None,
                );
            }
            RunFormField::GapPolicy => {}
        }
    }

    fn value_for_field(
        &self,
        field: RunFormField,
        options: Option<&SnapshotRunFormOptions>,
    ) -> String {
        match field {
            RunFormField::Symbol => options
                .and_then(|options| self.selected_symbol(options))
                .map(|symbol| symbol.symbol.clone())
                .unwrap_or_else(|| "not selected".to_string()),
            RunFormField::StartDate => {
                selected_symbol_date(options, self.symbol_index, self.start_date_index)
            }
            RunFormField::EndDate => {
                selected_symbol_date(options, self.symbol_index, self.end_date_index)
            }
            RunFormField::SignalDate => {
                selected_symbol_date(options, self.symbol_index, self.signal_date_index)
            }
            RunFormField::InitialCash => format!("{:.2}", self.initial_cash),
            RunFormField::EntryShares => self.entry_shares.to_string(),
            RunFormField::ProtectiveStopFraction => {
                format!("{:.4}", self.protective_stop_fraction)
            }
            RunFormField::CommissionPerFill => format!("{:.2}", self.commission_per_fill),
            RunFormField::SlippagePerShare => format!("{:.2}", self.slippage_per_share),
            RunFormField::GapPolicy => GapPolicy::M1Default.as_str().to_string(),
        }
    }

    fn build_spec(
        &self,
        snapshot_path: &Path,
        options: &SnapshotRunFormOptions,
    ) -> Option<OperatorRunSpec> {
        let symbol = self.selected_symbol(options)?;
        let start_date = symbol.available_dates.get(self.start_date_index)?.clone();
        let end_date = symbol.available_dates.get(self.end_date_index)?.clone();
        let signal_date = symbol.available_dates.get(self.signal_date_index)?.clone();

        Some(OperatorRunSpec {
            request_path: None,
            request: None,
            snapshot_source: Some(OperatorSnapshotSourceSpec {
                snapshot_dir: snapshot_path.display().to_string(),
                symbol: symbol.symbol.clone(),
                start_date,
                end_date,
            }),
            request_template: Some(OperatorRunRequestTemplate {
                entry_intents: vec![EntryIntent {
                    signal_date,
                    intent: OrderIntent::QueueMarketEntry,
                    shares: self.entry_shares,
                }],
                reference_flow: ReferenceFlowSpec {
                    initial_cash: self.initial_cash,
                    entry_shares: self.entry_shares,
                    protective_stop_fraction: self.protective_stop_fraction,
                    cost_model: CostModel {
                        commission_per_fill: self.commission_per_fill,
                        slippage_per_share: self.slippage_per_share,
                    },
                },
                gap_policy: GapPolicy::M1Default,
            }),
            manifest: OperatorRunManifestSpec::default(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct App {
    mode: AppMode,
    inspect: Option<InspectApp>,
    research: Option<ResearchApp>,
    snapshots: SnapshotBrowser,
    history: RunHistoryBrowser,
    home_focus: HomeFocusPane,
    run_form: RunFormDraft,
    launch_error: Option<String>,
    reopen_error: Option<String>,
    should_quit: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct StartupOptions {
    bundle_path: Option<PathBuf>,
    snapshot_paths: Vec<PathBuf>,
}

impl App {
    #[cfg(test)]
    fn home() -> Self {
        Self::home_with_snapshots_and_history_root(
            Vec::new(),
            workspace_root()
                .join("target")
                .join("test-output")
                .join("tui-home"),
        )
    }

    fn home_with_snapshots(snapshot_paths: Vec<PathBuf>) -> Self {
        Self::home_with_snapshots_and_history_root(snapshot_paths, default_tui_output_root())
    }

    fn home_with_snapshots_and_history_root(
        snapshot_paths: Vec<PathBuf>,
        history_root: PathBuf,
    ) -> Self {
        let mut app = Self {
            mode: AppMode::Home,
            inspect: None,
            research: None,
            snapshots: SnapshotBrowser::from_paths(snapshot_paths),
            history: RunHistoryBrowser::load(history_root),
            home_focus: HomeFocusPane::Snapshots,
            run_form: RunFormDraft::default(),
            launch_error: None,
            reopen_error: None,
            should_quit: false,
        };
        app.sync_run_form_to_selected_snapshot();
        app
    }

    fn from_bundle(
        bundle_path: PathBuf,
        bundle: ReplayBundle,
        snapshot_paths: Vec<PathBuf>,
    ) -> Self {
        Self::from_bundle_with_history_root(
            bundle_path,
            bundle,
            snapshot_paths,
            default_tui_output_root(),
        )
    }

    fn from_bundle_with_history_root(
        bundle_path: PathBuf,
        bundle: ReplayBundle,
        snapshot_paths: Vec<PathBuf>,
        history_root: PathBuf,
    ) -> Self {
        let mut app = Self {
            mode: AppMode::Inspect,
            inspect: Some(InspectApp::from_bundle(bundle_path, bundle)),
            research: None,
            snapshots: SnapshotBrowser::from_paths(snapshot_paths),
            history: RunHistoryBrowser::load(history_root),
            home_focus: HomeFocusPane::Snapshots,
            run_form: RunFormDraft::default(),
            launch_error: None,
            reopen_error: None,
            should_quit: false,
        };
        app.sync_run_form_to_selected_snapshot();
        app
    }

    fn from_report(
        report_path: PathBuf,
        report: ResearchReport,
        snapshot_paths: Vec<PathBuf>,
    ) -> Self {
        Self::from_report_with_history_root(
            report_path,
            report,
            snapshot_paths,
            default_tui_output_root(),
        )
    }

    fn from_report_with_history_root(
        report_path: PathBuf,
        report: ResearchReport,
        snapshot_paths: Vec<PathBuf>,
        history_root: PathBuf,
    ) -> Self {
        let mut app = Self {
            mode: AppMode::Research,
            inspect: None,
            research: Some(ResearchApp::from_report(report_path, report)),
            snapshots: SnapshotBrowser::from_paths(snapshot_paths),
            history: RunHistoryBrowser::load(history_root),
            home_focus: HomeFocusPane::Snapshots,
            run_form: RunFormDraft::default(),
            launch_error: None,
            reopen_error: None,
            should_quit: false,
        };
        app.sync_run_form_to_selected_snapshot();
        app
    }

    fn apply(&mut self, command: AppCommand) {
        match command {
            AppCommand::ShowHome => self.mode = AppMode::Home,
            AppCommand::ShowInspect => {
                if self.inspect.is_some() && self.research.is_some() {
                    self.mode = if matches!(self.mode, AppMode::Inspect) {
                        AppMode::Research
                    } else {
                        AppMode::Inspect
                    };
                } else if self.inspect.is_some() {
                    self.mode = AppMode::Inspect;
                } else if self.research.is_some() {
                    self.mode = AppMode::Research;
                }
            }
            AppCommand::ShowHelp => self.mode = AppMode::Help,
            AppCommand::ToggleHelp if !matches!(self.mode, AppMode::Inspect) => {
                self.mode = AppMode::Help
            }
            AppCommand::Quit => self.should_quit = true,
            AppCommand::NextFocus if matches!(self.mode, AppMode::Home) => {
                self.clear_home_errors();
                self.home_focus = self.home_focus.next();
            }
            AppCommand::PreviousFocus if matches!(self.mode, AppMode::Home) => {
                self.clear_home_errors();
                self.home_focus = self.home_focus.previous();
            }
            AppCommand::MoveUp if matches!(self.mode, AppMode::Home) => match self.home_focus {
                HomeFocusPane::Snapshots => {
                    self.clear_home_errors();
                    self.snapshots.move_up();
                    self.sync_run_form_to_selected_snapshot();
                }
                HomeFocusPane::RunForm => {
                    self.clear_home_errors();
                    self.run_form.move_up();
                }
                HomeFocusPane::History => {
                    self.clear_home_errors();
                    self.history.move_up();
                }
            },
            AppCommand::MoveDown if matches!(self.mode, AppMode::Home) => match self.home_focus {
                HomeFocusPane::Snapshots => {
                    self.clear_home_errors();
                    self.snapshots.move_down();
                    self.sync_run_form_to_selected_snapshot();
                }
                HomeFocusPane::RunForm => {
                    self.clear_home_errors();
                    self.run_form.move_down();
                }
                HomeFocusPane::History => {
                    self.clear_home_errors();
                    self.history.move_down();
                }
            },
            AppCommand::AdjustPrevious if matches!(self.mode, AppMode::Home) => {
                if self.home_focus == HomeFocusPane::RunForm
                    && let Some(run_form_options) = self
                        .selected_loaded_snapshot()
                        .map(|loaded| loaded.run_form_options.clone())
                {
                    self.clear_home_errors();
                    self.run_form.adjust(&run_form_options, false);
                }
            }
            AppCommand::AdjustNext if matches!(self.mode, AppMode::Home) => {
                if self.home_focus == HomeFocusPane::RunForm
                    && let Some(run_form_options) = self
                        .selected_loaded_snapshot()
                        .map(|loaded| loaded.run_form_options.clone())
                {
                    self.clear_home_errors();
                    self.run_form.adjust(&run_form_options, true);
                }
            }
            AppCommand::LaunchRun if matches!(self.mode, AppMode::Home) => match self.home_focus {
                HomeFocusPane::History => self.reopen_selected_history(),
                HomeFocusPane::Snapshots | HomeFocusPane::RunForm => self.launch_run(),
            },
            AppCommand::NextFocus
            | AppCommand::PreviousFocus
            | AppCommand::MoveUp
            | AppCommand::MoveDown
                if matches!(self.mode, AppMode::Research) =>
            {
                self.reopen_error = None;
                if let Some(research) = &mut self.research {
                    research.apply(command);
                }
            }
            AppCommand::LaunchRun if matches!(self.mode, AppMode::Research) => {
                self.reopen_selected_research_bundle();
            }
            command => {
                if matches!(self.mode, AppMode::Inspect)
                    && let Some(inspect) = &mut self.inspect
                {
                    inspect.apply(command);
                    self.should_quit |= inspect.should_quit;
                }
            }
        }
    }

    fn inspect(&self) -> Option<&InspectApp> {
        self.inspect.as_ref()
    }

    fn research(&self) -> Option<&ResearchApp> {
        self.research.as_ref()
    }

    fn selected_snapshot(&self) -> Option<&SnapshotBrowserEntry> {
        self.snapshots.selected()
    }

    fn selected_loaded_snapshot(&self) -> Option<&LoadedSnapshot> {
        match &self.selected_snapshot()?.state {
            SnapshotLoadState::Loaded(loaded) => Some(loaded.as_ref()),
            SnapshotLoadState::Failed(_) => None,
        }
    }

    fn sync_run_form_to_selected_snapshot(&mut self) {
        if let Some(run_form_options) = self
            .selected_loaded_snapshot()
            .map(|loaded| loaded.run_form_options.clone())
        {
            self.run_form.clamp_to_snapshot(&run_form_options);
        }
    }

    fn clear_home_errors(&mut self) {
        self.launch_error = None;
        self.reopen_error = None;
    }

    fn validated_run_spec(&self) -> Result<(OperatorRunSpec, RunSpecPreview), String> {
        let entry = self
            .selected_snapshot()
            .ok_or_else(|| "select a stored snapshot before launching a run".to_string())?;
        let loaded = match &entry.state {
            SnapshotLoadState::Loaded(loaded) => loaded.as_ref(),
            SnapshotLoadState::Failed(error) => {
                return Err(format!(
                    "selected snapshot directory failed to load: {error}"
                ));
            }
        };
        let spec = self
            .run_form
            .build_spec(&entry.path, &loaded.run_form_options)
            .ok_or_else(|| {
                "the selected snapshot does not expose a runnable symbol/date form".to_string()
            })?;
        let preview = preview_run_spec(&spec, None).map_err(|err| err.to_string())?;
        Ok((spec, preview))
    }

    fn launch_run(&mut self) {
        self.launch_error = None;
        self.reopen_error = None;

        let (spec, preview) = match self.validated_run_spec() {
            Ok(value) => value,
            Err(error) => {
                self.launch_error = Some(error);
                return;
            }
        };
        let output_dir = default_output_dir_for_launch_under(&self.history.root, &preview, &spec);

        let outcome = match execute_run_spec(&spec, output_dir.clone()) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.launch_error = Some(error.to_string());
                return;
            }
        };

        let bundle = match load_replay_bundle(&outcome.output_dir) {
            Ok(bundle) => bundle,
            Err(error) => {
                self.launch_error = Some(format!(
                    "launched run at {} but failed to reopen replay bundle: {error}",
                    outcome.output_dir.display()
                ));
                return;
            }
        };

        self.history.refresh();
        self.inspect = Some(InspectApp::from_bundle(outcome.output_dir.clone(), bundle));
        self.research = None;
        self.mode = AppMode::Inspect;
    }

    fn reopen_selected_history(&mut self) {
        self.launch_error = None;
        self.reopen_error = None;

        let Some(path) = self.history.selected().map(|entry| entry.path.clone()) else {
            self.reopen_error =
                Some("no prior runs available under the TUI output root".to_string());
            return;
        };

        let bundle = match load_replay_bundle(&path) {
            Ok(bundle) => bundle,
            Err(error) => {
                self.history.refresh();
                self.reopen_error = Some(format!(
                    "failed to reopen prior run {}: {error}",
                    path.display()
                ));
                return;
            }
        };

        self.inspect = Some(InspectApp::from_bundle(path, bundle));
        self.research = None;
        self.mode = AppMode::Inspect;
    }

    fn reopen_selected_research_bundle(&mut self) {
        self.launch_error = None;
        self.reopen_error = None;

        let Some(path) = self
            .research()
            .and_then(|research| research.selected_link_path().cloned())
        else {
            self.reopen_error =
                Some("the selected research item does not link to a replay bundle".to_string());
            return;
        };

        let bundle = match load_replay_bundle(&path) {
            Ok(bundle) => bundle,
            Err(error) => {
                self.reopen_error = Some(format!(
                    "failed to reopen linked replay bundle {}: {error}",
                    path.display()
                ));
                return;
            }
        };

        self.inspect = Some(InspectApp::from_bundle(path, bundle));
        self.mode = AppMode::Inspect;
    }
}

fn load_snapshot_entry(path: PathBuf) -> SnapshotBrowserEntry {
    let state = load_snapshot_bundle(&path)
        .and_then(|bundle| {
            let report = inspect_snapshot_bundle(&bundle)?;
            let run_form_options = snapshot_run_form_options(&bundle)?;
            Ok(Box::new(LoadedSnapshot {
                report,
                run_form_options,
            }))
        })
        .map(SnapshotLoadState::Loaded)
        .unwrap_or_else(|err| SnapshotLoadState::Failed(err.to_string()));

    SnapshotBrowserEntry { path, state }
}

pub fn run_from_args<I, S>(args: I) -> Result<(), TuiError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let options = parse_startup_options(args)?;
    run_app(options)
}

fn parse_startup_options(args: Vec<String>) -> Result<StartupOptions, TuiError> {
    let mut options = StartupOptions::default();
    let mut iter = args.into_iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "open" => {
                if options.bundle_path.is_some() {
                    return Err(TuiError::invalid(USAGE));
                }
                let Some(bundle_dir) = iter.next() else {
                    return Err(TuiError::invalid(USAGE));
                };
                options.bundle_path = Some(PathBuf::from(bundle_dir));
            }
            "--snapshot" => {
                let Some(snapshot_dir) = iter.next() else {
                    return Err(TuiError::invalid(USAGE));
                };
                options.snapshot_paths.push(PathBuf::from(snapshot_dir));
            }
            _ if options.bundle_path.is_none() => {
                options.bundle_path = Some(PathBuf::from(arg));
            }
            _ => return Err(TuiError::invalid(USAGE)),
        }
    }

    Ok(options)
}

#[cfg(test)]
fn parse_bundle_path(args: Vec<String>) -> Result<Option<PathBuf>, TuiError> {
    parse_startup_options(args).map(|options| options.bundle_path)
}

fn run_app(options: StartupOptions) -> Result<(), TuiError> {
    let mut app = match options.bundle_path.as_deref() {
        Some(bundle_path) => {
            load_startup_app_for_artifact_path(bundle_path, options.snapshot_paths)?
        }
        None => App::home_with_snapshots(options.snapshot_paths),
    };
    let mut terminal =
        ratatui::try_init().map_err(|err| TuiError::io("failed to initialize terminal", &err))?;
    let _restore_guard = RestoreTerminalGuard;

    loop {
        terminal
            .draw(|frame| render(frame, &app))
            .map_err(|err| TuiError::io("failed to draw terminal frame", &err))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(250))
            .map_err(|err| TuiError::io("failed to poll terminal events", &err))?
            && let Event::Key(key) =
                event::read().map_err(|err| TuiError::io("failed to read terminal event", &err))?
            && let Some(command) = AppCommand::from_key_event(&key)
        {
            app.apply(command);
        }
    }

    Ok(())
}

struct RestoreTerminalGuard;

impl Drop for RestoreTerminalGuard {
    fn drop(&mut self) {
        let _ = ratatui::try_restore();
    }
}

fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Inspect => {
            if let Some(inspect) = app.inspect() {
                render_inspect(frame, app, inspect);
            } else if let Some(research) = app.research() {
                render_research(frame, app, research);
            } else {
                render_home(frame, app);
            }
        }
        AppMode::Research => {
            if let Some(research) = app.research() {
                render_research(frame, app, research);
            } else {
                render_home(frame, app);
            }
        }
        AppMode::Home => render_home(frame, app),
        AppMode::Help => render_shell_help(frame, app),
    }
}

fn render_inspect(frame: &mut Frame, app: &App, inspect: &InspectApp) {
    let help_height = if inspect.help_expanded { 7 } else { 3 };
    let layout = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(12),
        Constraint::Length(help_height),
    ])
    .split(frame.area());
    let body = Layout::horizontal([
        Constraint::Percentage(26),
        Constraint::Percentage(40),
        Constraint::Percentage(34),
    ])
    .split(layout[1]);
    let center = Layout::vertical([Constraint::Length(12), Constraint::Min(8)]).split(body[1]);

    render_header(frame, layout[0], app, inspect);
    render_results(frame, body[0], inspect);
    render_chart(frame, center[0], inspect);
    render_ledger(frame, center[1], inspect);
    render_audit(frame, body[2], inspect);
    render_help(frame, layout[2], inspect);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, inspect: &InspectApp) {
    let provenance = bundle_provenance_summary(&inspect.bundle);
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                APP_TITLE,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("mode: {}", app.mode.label())),
            Span::raw("  "),
            Span::raw(format!("bundle: {}", inspect.bundle_path.display())),
        ]),
        Line::from(format!(
            "symbol={} snapshot={} provider={} focus={}",
            inspect.bundle.manifest.symbol_or_universe,
            inspect.bundle.manifest.data_snapshot_id,
            inspect.bundle.manifest.provider_identity,
            inspect.focus.label()
        )),
        Line::from(provenance),
    ]);

    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Run"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_research(frame: &mut Frame, app: &App, research: &ResearchApp) {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(12),
        Constraint::Length(8),
    ])
    .split(frame.area());
    let body = Layout::horizontal([
        Constraint::Percentage(34),
        Constraint::Percentage(30),
        Constraint::Percentage(36),
    ])
    .split(layout[1]);

    let header = Text::from(vec![
        Line::from(vec![
            Span::styled(
                APP_TITLE,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("mode: {}", app.mode.label())),
            Span::raw("  "),
            Span::raw(format!("report: {}", research.report_path.display())),
        ]),
        Line::from(format!(
            "report_kind={} linked_replay_bundles={}",
            research.report.kind(),
            research_report_link_count(&research.report)
        )),
    ]);
    frame.render_widget(
        Paragraph::new(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Research Report"),
            )
            .wrap(Wrap { trim: true }),
        layout[0],
    );

    frame.render_widget(
        Paragraph::new(Text::from(
            research_report_summary_lines(&research.report)
                .into_iter()
                .map(Line::from)
                .collect::<Vec<_>>(),
        ))
        .block(pane_block(
            "Summary",
            research.focus == ResearchFocusPane::Summary,
        ))
        .wrap(Wrap { trim: false }),
        body[0],
    );
    render_research_items(frame, body[1], research);
    render_research_detail(frame, body[2], app, research);

    render_mode_footer(frame, layout[2], app);
}

fn render_research_items(frame: &mut Frame, area: Rect, research: &ResearchApp) {
    let items = if research.items.is_empty() {
        vec![ListItem::new("No report items loaded.")]
    } else {
        research
            .items
            .iter()
            .map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{: <14}", item.label),
                        Style::default().fg(Color::LightCyan),
                    ),
                    Span::raw(item.value.clone()),
                ]))
            })
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .block(pane_block(
            research_item_pane_title(&research.report),
            research.focus == ResearchFocusPane::Items,
        ))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));
    let mut state = ListState::default();
    state.select(research.selected_item_index());
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_research_detail(frame: &mut Frame, area: Rect, app: &App, research: &ResearchApp) {
    let mut lines = if let Some(item) = research.selected_item() {
        let mut lines = vec![format!("item: {}", item.detail_title), String::new()];
        lines.extend(item.detail_lines.clone());
        lines.push(String::new());
        lines.push("linked_replay_bundles:".to_string());
        if item.linked_bundle_paths.is_empty() {
            lines.push("  none".to_string());
        } else {
            for (index, path) in item.linked_bundle_paths.iter().enumerate() {
                let marker = if research.selected_link_index() == Some(index) {
                    ">>"
                } else {
                    "  "
                };
                let state = if path.exists() { "present" } else { "missing" };
                lines.push(format!("{marker} {} ({state})", path.display()));
            }
        }
        lines
    } else {
        vec![
            "item: none".to_string(),
            String::new(),
            "No report item is selected.".to_string(),
        ]
    };
    lines.push(String::new());
    if research.selected_link_path().is_some() {
        lines.push("drilldown: press Enter or r to reopen the selected replay bundle".to_string());
    } else {
        lines.push(
            "drilldown: blocked because the selected report item has no replay bundle".to_string(),
        );
    }
    if let Some(error) = &app.reopen_error {
        lines.push(String::new());
        lines.push("reopen_status: failed".to_string());
        lines.push(format!("reopen_error: {error}"));
    }

    frame.render_widget(
        Paragraph::new(Text::from(
            lines.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .block(pane_block(
            "Detail",
            research.focus == ResearchFocusPane::Detail,
        ))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_results(frame: &mut Frame, area: Rect, app: &InspectApp) {
    let items = if app.results.is_empty() {
        vec![ListItem::new("No run results loaded.")]
    } else {
        app.results
            .iter()
            .map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{: <12}", item.label),
                        Style::default().fg(Color::LightCyan),
                    ),
                    Span::raw(item.value.clone()),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(pane_block("Inspect", app.focus == FocusPane::Results))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));
    let mut state = ListState::default();
    state.select(app.selected_result_index());
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_home(frame: &mut Frame, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(17),
        Constraint::Length(10),
        Constraint::Length(9),
    ])
    .split(frame.area());
    let body = Layout::horizontal([
        Constraint::Percentage(28),
        Constraint::Percentage(34),
        Constraint::Percentage(38),
    ])
    .split(layout[1]);
    let run_area =
        Layout::vertical([Constraint::Percentage(56), Constraint::Percentage(44)]).split(body[2]);
    let history_area = Layout::horizontal([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(layout[2]);

    let header = Text::from(vec![
        Line::from(vec![
            Span::styled(
                APP_TITLE,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("mode: {}", app.mode.label())),
        ]),
        Line::from("operator_workspace: snapshot-backed single-symbol daily runs"),
    ]);
    frame.render_widget(
        Paragraph::new(header)
            .block(Block::default().borders(Borders::ALL).title("Home"))
            .wrap(Wrap { trim: true }),
        layout[0],
    );

    render_snapshot_browser(frame, body[0], app);
    render_snapshot_summary(frame, body[1], app);
    render_run_form(frame, run_area[0], app);
    render_run_validation(frame, run_area[1], app);
    render_run_history_browser(frame, history_area[0], app);
    render_run_history_preview(frame, history_area[1], app);

    render_mode_footer(frame, layout[3], app);
}

fn render_snapshot_browser(frame: &mut Frame, area: Rect, app: &App) {
    let items = if app.snapshots.entries.is_empty() {
        vec![ListItem::new("No stored snapshots configured.")]
    } else {
        app.snapshots
            .entries
            .iter()
            .map(|entry| {
                let status = match &entry.state {
                    SnapshotLoadState::Loaded(loaded) => {
                        format!("ok {} symbols", loaded.report.symbol_count)
                    }
                    SnapshotLoadState::Failed(_) => "invalid".to_string(),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{: <8}", status),
                        Style::default().fg(match &entry.state {
                            SnapshotLoadState::Loaded(_) => Color::Green,
                            SnapshotLoadState::Failed(_) => Color::Red,
                        }),
                    ),
                    Span::raw(entry.path.display().to_string()),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(pane_block(
            "Snapshots",
            app.mode == AppMode::Home && app.home_focus == HomeFocusPane::Snapshots,
        ))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));
    let mut state = ListState::default();
    state.select((!app.snapshots.entries.is_empty()).then_some(app.snapshots.selected_snapshot));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_snapshot_summary(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = match app.selected_snapshot() {
        Some(entry) => snapshot_entry_lines(entry),
        None => vec![
            "Snapshot Summary".to_string(),
            "snapshot_source: not selected".to_string(),
            "provider: none".to_string(),
            "requested_window: none".to_string(),
            "symbols: none".to_string(),
        ],
    };

    lines.push(String::new());
    if let Some(inspect) = app.inspect() {
        vec![
            format!("loaded_bundle: {}", inspect.bundle_path.display()),
            format!(
                "loaded_symbol: {}",
                inspect.bundle.manifest.symbol_or_universe
            ),
            format!(
                "loaded_snapshot: {}",
                inspect.bundle.manifest.data_snapshot_id
            ),
        ]
        .into_iter()
        .for_each(|line| lines.push(line));
    } else if let Some(research) = app.research() {
        vec![
            format!("loaded_report: {}", research.report_path.display()),
            format!("loaded_report_kind: {}", research.report.kind()),
            format!(
                "linked_replay_bundles: {}",
                research_report_link_count(&research.report)
            ),
        ]
        .into_iter()
        .for_each(|line| lines.push(line));
    } else {
        lines.push("loaded_bundle: none".to_string());
    }

    frame.render_widget(
        Paragraph::new(Text::from(
            lines.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Snapshot Summary"),
        )
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn snapshot_entry_lines(entry: &SnapshotBrowserEntry) -> Vec<String> {
    match &entry.state {
        SnapshotLoadState::Loaded(loaded) => {
            let report = &loaded.report;
            let mut lines = vec![
                "Snapshot Summary".to_string(),
                format!("snapshot_source: {}", entry.path.display()),
                format!("snapshot_id: {}", report.snapshot_id),
                format!("provider: {}", report.provider_identity.as_str()),
                format!(
                    "requested_window: {}..{}",
                    report.requested_start_date, report.requested_end_date
                ),
                format!("capture_mode: {}", report.capture_mode),
                format!("entrypoint: {}", report.entrypoint),
                format!("symbols: {}", report.symbol_count),
            ];

            for symbol in report.symbols.iter().take(4) {
                lines.push(format!(
                    "symbol: {} raw={} actions={} splits={} dividends={} daily={} weekly={} monthly={} adjusted={} max_gap={}",
                    symbol.symbol,
                    symbol.raw_bar_count,
                    symbol.corporate_action_count,
                    symbol.split_action_count,
                    symbol.cash_dividend_action_count,
                    symbol.normalized_daily_bar_count,
                    symbol.weekly_bar_count,
                    symbol.monthly_bar_count,
                    symbol.analysis_adjusted_bar_count,
                    format_optional_f64(symbol.max_analysis_close_gap)
                ));
            }

            if report.symbols.len() > 4 {
                lines.push(format!("more_symbols: {}", report.symbols.len() - 4));
            }

            lines
        }
        SnapshotLoadState::Failed(error) => vec![
            "Snapshot Summary".to_string(),
            format!("snapshot_source: {}", entry.path.display()),
            "status: invalid".to_string(),
            format!("error: {error}"),
        ],
    }
}

fn render_run_form(frame: &mut Frame, area: Rect, app: &App) {
    let options = app
        .selected_loaded_snapshot()
        .map(|loaded| &loaded.run_form_options);
    let items = RunFormField::ALL
        .into_iter()
        .map(|field| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{: <24}", field.label()),
                    Style::default().fg(Color::LightCyan),
                ),
                Span::raw(app.run_form.value_for_field(field, options)),
            ]))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(pane_block(
            "Run Form",
            app.mode == AppMode::Home && app.home_focus == HomeFocusPane::RunForm,
        ))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    let mut state = ListState::default();
    state.select(Some(app.run_form.selected_field));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_run_validation(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(Text::from(
            run_validation_lines(app)
                .into_iter()
                .map(Line::from)
                .collect::<Vec<_>>(),
        ))
        .block(Block::default().borders(Borders::ALL).title("Validation"))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_run_history_browser(frame: &mut Frame, area: Rect, app: &App) {
    let items = if app.history.entries.is_empty() {
        vec![ListItem::new("No prior TUI runs found.")]
    } else {
        app.history
            .entries
            .iter()
            .map(|entry| match &entry.state {
                RunHistoryState::Loaded(preview) => ListItem::new(Line::from(vec![
                    Span::styled(format!("{: <8}", "ok"), Style::default().fg(Color::Green)),
                    Span::raw(format!(
                        "{} {} {}",
                        preview.symbol,
                        preview.start_date,
                        entry
                            .path
                            .file_name()
                            .and_then(|value| value.to_str())
                            .unwrap_or("run")
                    )),
                ])),
                RunHistoryState::Failed(_) => ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{: <8}", "invalid"),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(
                        entry
                            .path
                            .file_name()
                            .and_then(|value| value.to_str())
                            .unwrap_or("run")
                            .to_string(),
                    ),
                ])),
            })
            .collect::<Vec<_>>()
    };
    let list = List::new(items)
        .block(pane_block(
            "Run History",
            app.mode == AppMode::Home && app.home_focus == HomeFocusPane::History,
        ))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));
    let mut state = ListState::default();
    state.select((!app.history.entries.is_empty()).then_some(app.history.selected_run));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_run_history_preview(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(Text::from(
            run_history_preview_lines(app)
                .into_iter()
                .map(Line::from)
                .collect::<Vec<_>>(),
        ))
        .block(Block::default().borders(Borders::ALL).title("Run Preview"))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn run_validation_lines(app: &App) -> Vec<String> {
    let mut lines = match app.validated_run_spec() {
        Ok((spec, preview)) => run_preview_lines(
            &preview,
            &default_output_dir_for_launch_under(&app.history.root, &preview, &spec),
        ),
        Err(error) => vec![
            "status: invalid".to_string(),
            format!("error: {error}"),
            format!("output_root: {}", app.history.root.display()),
            "launch: blocked until validation passes".to_string(),
            "entry_intent.intent: queue_market_entry".to_string(),
            format!("entry_intent.shares: {}", app.run_form.entry_shares),
            "gap_policy: m1_default".to_string(),
        ],
    };

    if let Some(error) = &app.launch_error {
        let mut prefixed = vec![
            "launch_status: failed".to_string(),
            format!("launch_error: {error}"),
            String::new(),
        ];
        prefixed.append(&mut lines);
        prefixed
    } else {
        lines
    }
}

fn run_history_preview_lines(app: &App) -> Vec<String> {
    let mut lines = match app.history.selected() {
        Some(entry) => match &entry.state {
            RunHistoryState::Loaded(preview) => vec![
                format!("bundle: {}", entry.path.display()),
                format!("snapshot_id: {}", preview.snapshot_id),
                format!("provider: {}", preview.provider_identity),
                format!("symbol: {}", preview.symbol),
                format!("date_range: {}..{}", preview.start_date, preview.end_date),
                format!("run_source_kind: {}", preview.run_source_kind),
                format!("request_source: {}", preview.request_source),
                format!("spec_source: {}", preview.spec_source),
                format!("snapshot_source_path: {}", preview.snapshot_source_path),
                format!("snapshot_selection: {}", preview.snapshot_selection),
                format!("rows: {}", preview.row_count),
                format!("warning_count: {}", preview.warning_count),
                format!("ending_equity: {:.4}", preview.ending_equity),
                "reopen: press Enter or r while Run History is focused".to_string(),
            ],
            RunHistoryState::Failed(error) => vec![
                format!("bundle: {}", entry.path.display()),
                "status: invalid".to_string(),
                format!("error: {error}"),
                "reopen: blocked until the bundle loads cleanly".to_string(),
            ],
        },
        None => vec![
            format!("output_root: {}", app.history.root.display()),
            "status: empty".to_string(),
            "reopen: no prior TUI runs found".to_string(),
            "next: launch a Home-mode run to populate this history".to_string(),
        ],
    };

    if let Some(error) = &app.reopen_error {
        let mut prefixed = vec![
            "reopen_status: failed".to_string(),
            format!("reopen_error: {error}"),
            String::new(),
        ];
        prefixed.append(&mut lines);
        prefixed
    } else {
        lines
    }
}

fn run_preview_lines(preview: &RunSpecPreview, output_dir: &Path) -> Vec<String> {
    vec![
        "status: ready".to_string(),
        format!("run_source_kind: {}", preview.run_source_kind.as_str()),
        format!(
            "snapshot_source: {}",
            format_optional_text(preview.snapshot_source_path.as_deref())
        ),
        format!("snapshot_id: {}", preview.snapshot_id),
        format!("provider: {}", preview.provider_identity),
        format!("symbol: {}", preview.symbol),
        format!(
            "selected_window: {}..{}",
            preview.start_date, preview.end_date
        ),
        format!("bars: {}", preview.row_count),
        format!("output_dir: {}", output_dir.display()),
        format!("request_source: {}", preview.request_source),
        format!(
            "spec_source: {}",
            format_optional_text(preview.spec_source.as_deref())
        ),
        format!(
            "snapshot_selection: {}..{}",
            preview.start_date, preview.end_date
        ),
        "entry_intent.intent: queue_market_entry".to_string(),
        "launch: press Enter or r".to_string(),
    ]
}

fn render_shell_help(frame: &mut Frame, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(10),
        Constraint::Length(6),
    ])
    .split(frame.area());

    let header = Text::from(vec![
        Line::from(vec![
            Span::styled(
                APP_TITLE,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("mode: {}", app.mode.label())),
        ]),
        Line::from("workspace_scope: stored snapshots now, live capture and research stay outside"),
    ]);
    frame.render_widget(
        Paragraph::new(header)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true }),
        layout[0],
    );

    let lines = vec![
        Line::from("Modes"),
        Line::from("1 Home"),
        Line::from("2 Inspect or Research"),
        Line::from("3 Help"),
        Line::from(""),
        Line::from("Home"),
        Line::from("tab/shift-tab switches between snapshot browser, run form, and run history"),
        Line::from("j/k moves selection"),
        Line::from("[/] adjusts the selected run-form field"),
        Line::from("enter or r launches the run form or reopens the selected prior run"),
        Line::from(""),
        Line::from("Inspect"),
        Line::from("tab/h/l changes pane focus"),
        Line::from("j/k changes selected result or ledger row"),
        Line::from("? toggles the inspect help panel"),
        Line::from(""),
        Line::from("Research"),
        Line::from("tab/h/l changes pane focus across summary, items, and detail"),
        Line::from("j/k changes the selected report item while Items is focused"),
        Line::from("j/k changes the selected linked replay bundle while Detail is focused"),
        Line::from("enter or r reopens the selected linked replay bundle into Inspect mode"),
        Line::from("saved research reports reopen through shared artifact ownership"),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Keys"))
            .wrap(Wrap { trim: false }),
        layout[1],
    );

    render_mode_footer(frame, layout[2], app);
}

fn render_mode_footer(frame: &mut Frame, area: Rect, app: &App) {
    let lines = mode_footer_lines(app);

    frame.render_widget(
        Paragraph::new(Text::from(
            lines.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_chart(frame: &mut Frame, area: Rect, app: &InspectApp) {
    if app.bundle.ledger.is_empty() {
        frame.render_widget(
            Paragraph::new("No persisted ledger rows loaded.")
                .block(pane_block("Chart", app.focus == FocusPane::Chart))
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let raw_close_points = app
        .bundle
        .ledger
        .iter()
        .enumerate()
        .map(|(index, row)| (index as f64, row.raw_close))
        .collect::<Vec<_>>();
    let analysis_close_points = app
        .bundle
        .ledger
        .iter()
        .enumerate()
        .map(|(index, row)| (index as f64, row.analysis_close))
        .collect::<Vec<_>>();
    let active_stop_points = app
        .bundle
        .ledger
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.prior_stop.map(|stop| (index as f64, stop)))
        .collect::<Vec<_>>();
    let fill_points = app
        .bundle
        .ledger
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.fill_price.map(|price| (index as f64, price)))
        .collect::<Vec<_>>();
    let selected_row_points = app
        .selected_ledger_row()
        .map(|row| vec![(app.selected_ledger as f64, row.raw_close)])
        .unwrap_or_default();
    let x_upper_bound = (app.bundle.ledger.len().saturating_sub(1).max(1)) as f64;
    let [y_min, y_max] = chart_y_bounds(&app.bundle.ledger);

    let datasets = vec![
        Dataset::default()
            .name("raw close")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&raw_close_points),
        Dataset::default()
            .name("analysis")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&analysis_close_points),
        Dataset::default()
            .name("active stop")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&active_stop_points),
        Dataset::default()
            .name("fills")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Magenta))
            .data(&fill_points),
        Dataset::default()
            .name("selected")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .data(&selected_row_points),
    ];

    let chart = Chart::new(datasets)
        .block(pane_block("Chart", app.focus == FocusPane::Chart))
        .x_axis(
            Axis::default()
                .title(chart_title(app))
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, x_upper_bound])
                .labels(chart_x_axis_labels(app)),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(chart_y_axis_labels(y_min, y_max)),
        )
        .hidden_legend_constraints((Constraint::Length(18), Constraint::Length(4)));

    frame.render_widget(chart, area);
}

fn render_ledger(frame: &mut Frame, area: Rect, app: &InspectApp) {
    let items = if app.bundle.ledger.is_empty() {
        vec![ListItem::new("No persisted ledger rows loaded.")]
    } else {
        app.bundle
            .ledger
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let primary = format!(
                    "{}  sh={} fill={} eq={:.4}",
                    row.date,
                    row.position_shares,
                    format_optional_f64(row.fill_price),
                    row.equity
                );
                let secondary = format!(
                    "signal={} pending={} reasons={}",
                    row.signal_output,
                    row.pending_order_state,
                    format_reason_codes(&row.reason_codes)
                );

                let style = if Some(index) == app.selected_ledger_index() {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(vec![
                    Line::styled(primary, style),
                    Line::styled(secondary, style),
                ])
            })
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .block(pane_block("Ledger", app.focus == FocusPane::Ledger))
        .highlight_symbol(">> ")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    let mut state = ListState::default();
    state.select(app.selected_ledger_index());
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_audit(frame: &mut Frame, area: Rect, app: &InspectApp) {
    frame.render_widget(
        Paragraph::new(Text::from(
            build_audit_lines(app)
                .into_iter()
                .map(Line::from)
                .collect::<Vec<_>>(),
        ))
        .block(Block::default().borders(Borders::ALL).title("Audit"))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_help(frame: &mut Frame, area: Rect, app: &InspectApp) {
    let lines = if app.help_expanded {
        expanded_help_lines()
    } else {
        vec!["tab/shift-tab focus  h/l change pane  j/k move  ? expand help  q quit".to_string()]
    };

    frame.render_widget(
        Paragraph::new(Text::from(
            lines.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .block(pane_block("Help", app.focus == FocusPane::Help))
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn pane_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let border_style = if focused {
        Style::default()
            .fg(Color::Rgb(217, 119, 6))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
}

fn build_result_items(
    bundle_path: &Path,
    bundle: &ReplayBundle,
    audit_report: &DataAuditReport,
) -> Vec<ResultItem> {
    let trade_summaries = derive_trade_summaries(&bundle.ledger);
    let trade_count = trade_summaries.len();
    let last_row_index = bundle.ledger.len().checked_sub(1);
    let audit_anchor = audit_report
        .max_analysis_close_gap_date
        .as_deref()
        .and_then(|date| ledger_index_for_date(&bundle.ledger, date));
    let mut items = vec![ResultItem {
        label: "run".to_string(),
        value: format!(
            "{} rows / {}",
            bundle.summary.row_count,
            count_label(trade_count, "trade", "trades")
        ),
        detail_title: "Run Overview".to_string(),
        detail_lines: vec![
            format!("symbol: {}", bundle.manifest.symbol_or_universe),
            format!("provider: {}", bundle.manifest.provider_identity),
            format!("universe_mode: {}", bundle.manifest.universe_mode),
            format!(
                "date_range: {}..{}",
                bundle.manifest.date_range.start_date, bundle.manifest.date_range.end_date
            ),
            format!("trade_count: {}", trade_count),
            format!("bundle_path: {}", bundle_path.display()),
        ],
        anchor_ledger_index: if bundle.ledger.is_empty() {
            None
        } else {
            Some(0)
        },
    }];

    items.extend(
        trade_summaries
            .into_iter()
            .map(result_item_from_trade_summary),
    );
    items.extend([
        ResultItem {
            label: "terminal".to_string(),
            value: format!("{:.4}", bundle.summary.ending_equity),
            detail_title: "Terminal Result".to_string(),
            detail_lines: vec![
                format!("ending_equity: {:.4}", bundle.summary.ending_equity),
                format!("ending_cash: {:.4}", bundle.summary.ending_cash),
                format!("row_count: {}", bundle.summary.row_count),
                format!("warning_count: {}", bundle.summary.warning_count),
            ],
            anchor_ledger_index: last_row_index,
        },
        ResultItem {
            label: "warnings".to_string(),
            value: count_label(bundle.manifest.warnings.len(), "warning", "warnings"),
            detail_title: "Run Warnings".to_string(),
            detail_lines: format_warning_lines(&bundle.manifest.warnings),
            anchor_ledger_index: None,
        },
        ResultItem {
            label: "data audit".to_string(),
            value: if audit_report.is_clean() {
                "clean".to_string()
            } else {
                count_label(audit_report.findings.len(), "finding", "findings")
            },
            detail_title: "Data Audit".to_string(),
            detail_lines: format_audit_detail_lines(audit_report),
            anchor_ledger_index: audit_anchor,
        },
        ResultItem {
            label: "artifact".to_string(),
            value: bundle.manifest.data_snapshot_id.clone(),
            detail_title: "Shared Artifact".to_string(),
            detail_lines: vec![
                format!(
                    "bundle schema_version: {}",
                    bundle.descriptor.schema_version
                ),
                format!(
                    "manifest schema_version: {}",
                    bundle.manifest.schema_version
                ),
                format!("engine_version: {}", bundle.manifest.engine_version),
                format!("gap_policy: {}", bundle.manifest.gap_policy.as_str()),
            ],
            anchor_ledger_index: None,
        },
    ]);

    items
}

fn format_warning_lines(warnings: &[String]) -> Vec<String> {
    if warnings.is_empty() {
        vec!["No warnings recorded in the replay bundle.".to_string()]
    } else {
        warnings
            .iter()
            .enumerate()
            .map(|(index, warning)| format!("warning {}: {}", index + 1, warning))
            .collect()
    }
}

fn format_audit_detail_lines(audit_report: &DataAuditReport) -> Vec<String> {
    let mut lines = vec![
        format!("bars: {}", audit_report.bar_count),
        format!(
            "date_range: {}..{}",
            format_optional_text(audit_report.start_date.as_deref()),
            format_optional_text(audit_report.end_date.as_deref())
        ),
        format!(
            "analysis_adjusted_bars: {}",
            audit_report.analysis_adjusted_bar_count
        ),
        format!(
            "analysis_matches_raw_close: {}",
            audit_report.analysis_matches_raw_close_count
        ),
        format!(
            "max_analysis_close_gap: {}",
            format_optional_f64(audit_report.max_analysis_close_gap)
        ),
    ];

    if audit_report.findings.is_empty() {
        lines.push("findings: none".to_string());
    } else {
        lines.push(format!(
            "findings: {}",
            count_label(audit_report.findings.len(), "finding", "findings")
        ));
        lines.extend(audit_report.findings.iter().take(3).map(|finding| {
            format!(
                "{} {}",
                format_optional_text(finding.date.as_deref()),
                finding.code
            )
        }));
        if audit_report.findings.len() > 3 {
            lines.push(format!(
                "more_findings: {}",
                audit_report.findings.len() - 3
            ));
        }
    }

    lines
}

fn derive_trade_summaries(ledger: &[PersistedLedgerRow]) -> Vec<TradeSummary> {
    #[derive(Clone, Debug)]
    struct OpenTrade {
        sequence: usize,
        entry_row_index: usize,
        entry_date: String,
        entry_price: f64,
        entry_equity: f64,
        entry_stop: Option<f64>,
        entry_reasons: Vec<String>,
    }

    let mut trade_summaries = Vec::new();
    let mut previous_shares = 0_u32;
    let mut open_trade = None;
    let mut sequence = 0_usize;

    for (index, row) in ledger.iter().enumerate() {
        if let Some(fill_price) = row.fill_price {
            if row.position_shares > previous_shares {
                sequence += 1;
                open_trade = Some(OpenTrade {
                    sequence,
                    entry_row_index: index,
                    entry_date: row.date.clone(),
                    entry_price: fill_price,
                    entry_equity: row.equity,
                    entry_stop: row.next_stop,
                    entry_reasons: row.reason_codes.clone(),
                });
            } else if row.position_shares < previous_shares
                && let Some(open_trade) = open_trade.take()
            {
                trade_summaries.push(TradeSummary {
                    sequence: open_trade.sequence,
                    entry_row_index: open_trade.entry_row_index,
                    entry_date: open_trade.entry_date,
                    entry_price: open_trade.entry_price,
                    entry_equity: open_trade.entry_equity,
                    entry_stop: open_trade.entry_stop,
                    entry_reasons: open_trade.entry_reasons,
                    exit_row_index: Some(index),
                    exit_date: Some(row.date.clone()),
                    exit_price: Some(fill_price),
                    exit_equity: Some(row.equity),
                    exit_reasons: row.reason_codes.clone(),
                    mark_date: row.date.clone(),
                    mark_price: fill_price,
                    bars_held: index + 1 - open_trade.entry_row_index,
                });
            }
        }

        previous_shares = row.position_shares;
    }

    if let Some(open_trade) = open_trade
        && let Some(mark_row) = ledger.last()
    {
        trade_summaries.push(TradeSummary {
            sequence: open_trade.sequence,
            entry_row_index: open_trade.entry_row_index,
            entry_date: open_trade.entry_date,
            entry_price: open_trade.entry_price,
            entry_equity: open_trade.entry_equity,
            entry_stop: open_trade.entry_stop,
            entry_reasons: open_trade.entry_reasons,
            exit_row_index: None,
            exit_date: None,
            exit_price: None,
            exit_equity: None,
            exit_reasons: Vec::new(),
            mark_date: mark_row.date.clone(),
            mark_price: mark_row.raw_close,
            bars_held: ledger.len() - open_trade.entry_row_index,
        });
    }

    trade_summaries
}

fn result_item_from_trade_summary(trade: TradeSummary) -> ResultItem {
    let pnl_points = trade.exit_price.unwrap_or(trade.mark_price) - trade.entry_price;
    let status = if trade.exit_row_index.is_some() {
        "closed"
    } else {
        "open"
    };
    let mut detail_lines = vec![
        format!("status: {}", status),
        format!("entry: {} @ {:.4}", trade.entry_date, trade.entry_price),
        format!("bars_held: {}", trade.bars_held),
        format!("pnl_points: {}", format_signed_f64(pnl_points)),
        format!("equity_at_entry: {:.4}", trade.entry_equity),
        format!("entry_stop: {}", format_optional_f64(trade.entry_stop)),
        format!(
            "entry_reasons: {}",
            format_reason_codes(&trade.entry_reasons)
        ),
    ];

    if let (Some(exit_date), Some(exit_price), Some(exit_equity)) = (
        trade.exit_date.as_deref(),
        trade.exit_price,
        trade.exit_equity,
    ) {
        detail_lines.push(format!("exit: {} @ {:.4}", exit_date, exit_price));
        detail_lines.push(format!("equity_at_exit: {:.4}", exit_equity));
        detail_lines.push(format!(
            "exit_reasons: {}",
            format_reason_codes(&trade.exit_reasons)
        ));
    } else {
        detail_lines.push(format!(
            "mark: {} @ {:.4}",
            trade.mark_date, trade.mark_price
        ));
    }

    ResultItem {
        label: format!("trade {}", trade.sequence),
        value: format!("{} {}", status, format_signed_f64(pnl_points)),
        detail_title: format!("Trade {}", trade.sequence),
        detail_lines,
        anchor_ledger_index: Some(trade.entry_row_index),
    }
}

fn ledger_index_for_date(ledger: &[PersistedLedgerRow], date: &str) -> Option<usize> {
    ledger.iter().position(|row| row.date == date)
}

fn build_audit_lines(app: &InspectApp) -> Vec<String> {
    let mut lines = vec![
        "Run".to_string(),
        format!("symbol: {}", app.bundle.manifest.symbol_or_universe),
        format!("provider: {}", app.bundle.manifest.provider_identity),
        format!("snapshot: {}", app.bundle.manifest.data_snapshot_id),
        format!(
            "date_range: {}..{}",
            app.bundle.manifest.date_range.start_date, app.bundle.manifest.date_range.end_date
        ),
        format!("gap_policy: {}", app.bundle.manifest.gap_policy.as_str()),
        String::new(),
        "Provenance".to_string(),
        format!(
            "run_source_kind: {}",
            manifest_parameter_value_or(&app.bundle.manifest, RUN_SOURCE_KIND_PARAMETER, "request")
        ),
        format!(
            "request_source: {}",
            manifest_parameter_value_or(&app.bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER, "none")
        ),
        format!(
            "spec_source: {}",
            manifest_parameter_value_or(&app.bundle.manifest, RUN_SPEC_SOURCE_PARAMETER, "none")
        ),
        format!(
            "snapshot_source_path: {}",
            manifest_parameter_value_or(
                &app.bundle.manifest,
                SNAPSHOT_SOURCE_PATH_PARAMETER,
                "none"
            )
        ),
        format!(
            "snapshot_selection: {}..{}",
            manifest_parameter_value_or(
                &app.bundle.manifest,
                SNAPSHOT_SELECTION_START_PARAMETER,
                "none"
            ),
            manifest_parameter_value_or(
                &app.bundle.manifest,
                SNAPSHOT_SELECTION_END_PARAMETER,
                "none"
            )
        ),
        String::new(),
        "Warnings".to_string(),
        format!(
            "count: {}",
            count_label(app.bundle.manifest.warnings.len(), "warning", "warnings")
        ),
    ];

    if app.bundle.manifest.warnings.is_empty() {
        lines.push("warning 1: none".to_string());
    } else {
        lines.extend(
            app.bundle
                .manifest
                .warnings
                .iter()
                .take(2)
                .enumerate()
                .map(|(index, warning)| format!("warning {}: {}", index + 1, warning)),
        );
        if app.bundle.manifest.warnings.len() > 2 {
            lines.push(format!(
                "more_warnings: {}",
                app.bundle.manifest.warnings.len() - 2
            ));
        }
    }

    lines.push(String::new());
    lines.push("Selected Row".to_string());
    lines.extend(
        app.selected_ledger_row()
            .map(selected_row_audit_lines)
            .unwrap_or_else(|| vec!["No persisted ledger rows loaded.".to_string()]),
    );
    lines.push(String::new());
    lines.push("Data Audit".to_string());
    lines.extend(format_audit_detail_lines(&app.audit_report));

    if let Some(item) = app.selected_result() {
        lines.push(String::new());
        lines.push("Result Focus".to_string());
        lines.push(format!("{}: {}", item.label, item.value));
        lines.extend(item.detail_lines.iter().take(3).cloned());
        if item.detail_lines.len() > 3 {
            lines.push(format!(
                "more_detail_lines: {}",
                item.detail_lines.len() - 3
            ));
        }
    }

    lines
}

fn selected_row_audit_lines(row: &PersistedLedgerRow) -> Vec<String> {
    vec![
        format!("date: {}", row.date),
        format!(
            "raw_close/analysis: {:.4}/{:.4}",
            row.raw_close, row.analysis_close
        ),
        format!(
            "signal/filter: {}/{}",
            row.signal_output, row.filter_outcome
        ),
        format!("pending: {}", row.pending_order_state),
        format!("fill: {}", format_optional_f64(row.fill_price)),
        format!(
            "stops: prior={} next={}",
            format_optional_f64(row.prior_stop),
            format_optional_f64(row.next_stop)
        ),
        format!("cash/equity: {:.4}/{:.4}", row.cash, row.equity),
        format!("reasons: {}", format_reason_codes(&row.reason_codes)),
    ]
}

fn expanded_help_lines() -> Vec<String> {
    vec![
        "The TUI still reopens shared replay bundles only through trendlab-artifact.".to_string(),
        "Home keeps launch and reopen targets explicit so Enter or r commits an auditable action.".to_string(),
        "Inspect now mixes run-level and per-trade checkpoints instead of only bundle-summary fields.".to_string(),
        "Chart and Ledger share the selected bar so price movement and persisted reasoning stay aligned.".to_string(),
        "Audit keeps provenance, warnings, data-audit summary, and selected-row reasoning visible across panes.".to_string(),
        "Keys: tab/shift-tab or h/l switch focus, j/k or arrows move selection, ? collapses help, q quits.".to_string(),
    ]
}

fn chart_title(app: &InspectApp) -> String {
    app.selected_ledger_row()
        .map(|row| {
            format!(
                "selected={} raw_close={:.4} analysis={:.4}",
                row.date, row.raw_close, row.analysis_close
            )
        })
        .unwrap_or_else(|| "selected=none".to_string())
}

fn chart_x_axis_labels(app: &InspectApp) -> Vec<Span<'static>> {
    if app.bundle.ledger.is_empty() {
        return vec![Span::raw("0"), Span::raw("1")];
    }

    let first = app
        .bundle
        .ledger
        .first()
        .map(|row| row.date.clone())
        .unwrap_or_else(|| "start".to_string());
    let selected = app
        .selected_ledger_row()
        .map(|row| row.date.clone())
        .unwrap_or_else(|| "selected".to_string());
    let last = app
        .bundle
        .ledger
        .last()
        .map(|row| row.date.clone())
        .unwrap_or_else(|| "end".to_string());
    let mut labels = vec![Span::raw(first)];

    if labels.last().map(|label| label.content.as_ref()) != Some(selected.as_str()) {
        labels.push(Span::raw(selected));
    }

    if labels.last().map(|label| label.content.as_ref()) != Some(last.as_str()) {
        labels.push(Span::raw(last));
    }

    labels
}

fn chart_y_axis_labels(y_min: f64, y_max: f64) -> Vec<Span<'static>> {
    let midpoint = (y_min + y_max) / 2.0;
    vec![
        Span::raw(format!("{y_min:.2}")),
        Span::raw(format!("{midpoint:.2}")),
        Span::raw(format!("{y_max:.2}")),
    ]
}

fn chart_y_bounds(ledger: &[PersistedLedgerRow]) -> [f64; 2] {
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;

    for row in ledger {
        min_value = min_value.min(row.raw_low).min(row.analysis_close);
        max_value = max_value.max(row.raw_high).max(row.analysis_close);

        if let Some(fill_price) = row.fill_price {
            min_value = min_value.min(fill_price);
            max_value = max_value.max(fill_price);
        }

        if let Some(prior_stop) = row.prior_stop {
            min_value = min_value.min(prior_stop);
            max_value = max_value.max(prior_stop);
        }
    }

    if !min_value.is_finite() || !max_value.is_finite() {
        return [0.0, 1.0];
    }

    let padding = ((max_value - min_value) * 0.08).max(1.0);
    [min_value - padding, max_value + padding]
}

fn cycle_selection_up(selected: &mut usize, len: usize) {
    if len == 0 {
        return;
    }

    *selected = if *selected == 0 {
        len - 1
    } else {
        *selected - 1
    };
}

fn cycle_selection_down(selected: &mut usize, len: usize) {
    if len == 0 {
        return;
    }

    *selected = (*selected + 1) % len;
}

fn adjust_f64(value: f64, delta: f64, minimum: f64, maximum: Option<f64>) -> f64 {
    let mut next = ((value + delta) * 10_000.0).round() / 10_000.0;
    if next < minimum {
        next = minimum;
    }
    if let Some(maximum) = maximum
        && next > maximum
    {
        next = maximum;
    }
    next
}

fn adjust_u32(value: u32, delta: i32, minimum: u32, maximum: Option<u32>) -> u32 {
    let mut next = value as i32 + delta;
    if next < minimum as i32 {
        next = minimum as i32;
    }
    if let Some(maximum) = maximum
        && next > maximum as i32
    {
        next = maximum as i32;
    }
    next as u32
}

fn selected_symbol_date(
    options: Option<&SnapshotRunFormOptions>,
    symbol_index: usize,
    date_index: usize,
) -> String {
    options
        .and_then(|options| options.symbols.get(symbol_index))
        .and_then(|symbol| symbol.available_dates.get(date_index))
        .cloned()
        .unwrap_or_else(|| "not selected".to_string())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("trendlab-tui lives under crates/")
        .to_path_buf()
}

fn default_tui_output_root() -> PathBuf {
    workspace_root().join("target").join("tui-runs")
}

fn default_output_dir_for_launch_under(
    root: &Path,
    preview: &RunSpecPreview,
    spec: &OperatorRunSpec,
) -> PathBuf {
    let signal_date = spec
        .request_template
        .as_ref()
        .and_then(|template| template.entry_intents.first())
        .map(|intent| intent.signal_date.as_str())
        .unwrap_or("none");
    let base_name = format!(
        "{}__{}__{}__{}__sig_{}",
        sanitize_path_component(&preview.snapshot_id),
        sanitize_path_component(&preview.symbol),
        sanitize_path_component(&preview.start_date),
        sanitize_path_component(&preview.end_date),
        sanitize_path_component(signal_date),
    );

    next_available_output_dir(root.join(base_name))
}

fn next_available_output_dir(base: PathBuf) -> PathBuf {
    if !base.exists() {
        return base;
    }

    let base_name = base
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("run")
        .to_string();
    let parent = base.parent().map(Path::to_path_buf).unwrap_or_default();

    for suffix in 2.. {
        let candidate = parent.join(format!("{base_name}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("integer suffix iteration should eventually find an unused output path")
}

fn sanitize_path_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    let mut last_was_separator = false;

    for ch in value.chars() {
        let keep = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        if keep {
            sanitized.push(ch);
            last_was_separator = false;
        } else if !last_was_separator {
            sanitized.push('_');
            last_was_separator = true;
        }
    }

    let sanitized = sanitized.trim_matches('_').to_string();
    if sanitized.is_empty() {
        "run".to_string()
    } else {
        sanitized
    }
}

fn load_history_entries(root: &Path) -> Vec<RunHistoryEntry> {
    let mut entries = Vec::new();
    let read_dir = match fs::read_dir(root) {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return entries,
        Err(err) => {
            entries.push(RunHistoryEntry {
                path: root.to_path_buf(),
                state: RunHistoryState::Failed(format!(
                    "failed to read run-history root {}: {err}",
                    root.display()
                )),
            });
            return entries;
        }
    };

    for child in read_dir.flatten() {
        let path = child.path();
        if !path.is_dir() {
            continue;
        }

        let state = match load_replay_bundle(&path) {
            Ok(bundle) => {
                RunHistoryState::Loaded(Box::new(run_history_preview_from_bundle(&bundle)))
            }
            Err(err) => RunHistoryState::Failed(err.to_string()),
        };
        entries.push(RunHistoryEntry { path, state });
    }

    entries
}

fn run_history_preview_from_bundle(bundle: &ReplayBundle) -> RunHistoryPreview {
    let run_source_kind =
        manifest_parameter_value_or(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER, "request");
    let snapshot_selection = format!(
        "{}..{}",
        manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SELECTION_START_PARAMETER, "none"),
        manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SELECTION_END_PARAMETER, "none"),
    );

    RunHistoryPreview {
        snapshot_id: bundle.manifest.data_snapshot_id.clone(),
        provider_identity: bundle.manifest.provider_identity.clone(),
        symbol: bundle.manifest.symbol_or_universe.clone(),
        start_date: bundle.manifest.date_range.start_date.clone(),
        end_date: bundle.manifest.date_range.end_date.clone(),
        row_count: bundle.summary.row_count,
        warning_count: bundle.summary.warning_count,
        ending_equity: bundle.summary.ending_equity,
        run_source_kind: run_source_kind.to_string(),
        request_source: manifest_parameter_value_or(
            &bundle.manifest,
            RUN_REQUEST_SOURCE_PARAMETER,
            "none",
        )
        .to_string(),
        spec_source: manifest_parameter_value_or(
            &bundle.manifest,
            RUN_SPEC_SOURCE_PARAMETER,
            "none",
        )
        .to_string(),
        snapshot_source_path: manifest_parameter_value_or(
            &bundle.manifest,
            SNAPSHOT_SOURCE_PATH_PARAMETER,
            "none",
        )
        .to_string(),
        snapshot_selection,
    }
}

fn bundle_provenance_summary(bundle: &ReplayBundle) -> String {
    format!(
        "run_source={} request_source={} spec_source={} snapshot_source={} selection={}..{}",
        manifest_parameter_value_or(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER, "request"),
        manifest_parameter_value_or(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER, "none"),
        manifest_parameter_value_or(&bundle.manifest, RUN_SPEC_SOURCE_PARAMETER, "none"),
        manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SOURCE_PATH_PARAMETER, "none"),
        manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SELECTION_START_PARAMETER, "none"),
        manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SELECTION_END_PARAMETER, "none"),
    )
}

fn mode_footer_lines(app: &App) -> Vec<String> {
    let loaded_status = loaded_artifact_status(app);
    let mut lines = vec![format!("mode: {}", app.mode.label()), loaded_status];

    if app.mode == AppMode::Home {
        lines.push(format!("home_focus: {}", home_focus_label(app.home_focus)));
        lines.push(format!(
            "selected_snapshot: {}",
            selected_snapshot_status(app).unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "launch_target: {}",
            next_launch_target(app)
                .unwrap_or_else(|| "blocked until validation passes".to_string())
        ));
        lines.push(format!(
            "history_target: {}",
            selected_history_status(app).unwrap_or_else(|| "none".to_string())
        ));
    } else if app.mode == AppMode::Research {
        lines.push(format!(
            "research_focus: {}",
            app.research()
                .map(|research| research.focus.label())
                .unwrap_or("none")
        ));
        lines.push(format!(
            "report_item: {}",
            selected_research_item_status(app).unwrap_or_else(|| "none".to_string())
        ));
        lines.push(format!(
            "drilldown_target: {}",
            selected_research_link_status(app).unwrap_or_else(|| "none".to_string())
        ));
    } else {
        lines.push(format!(
            "default_output_root: {}",
            default_tui_output_root().display()
        ));
    }

    lines.push("Research reports can reopen linked replay bundles into Inspect mode.".to_string());
    lines
}

fn loaded_artifact_status(app: &App) -> String {
    if let Some(inspect) = app.inspect() {
        format!("loaded_bundle: {}", inspect.bundle_path.display())
    } else if let Some(research) = app.research() {
        format!("loaded_report: {}", research.report_path.display())
    } else {
        "loaded_bundle: none".to_string()
    }
}

fn home_focus_label(focus: HomeFocusPane) -> &'static str {
    match focus {
        HomeFocusPane::Snapshots => "snapshots",
        HomeFocusPane::RunForm => "run_form",
        HomeFocusPane::History => "history",
    }
}

fn selected_snapshot_status(app: &App) -> Option<String> {
    let entry = app.selected_snapshot()?;
    Some(match &entry.state {
        SnapshotLoadState::Loaded(loaded) => format!(
            "{} ({})",
            loaded.report.snapshot_id,
            entry
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("snapshot")
        ),
        SnapshotLoadState::Failed(_) => format!(
            "invalid {}",
            entry
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("snapshot")
        ),
    })
}

fn next_launch_target(app: &App) -> Option<String> {
    let (spec, preview) = app.validated_run_spec().ok()?;
    Some(
        default_output_dir_for_launch_under(&app.history.root, &preview, &spec)
            .display()
            .to_string(),
    )
}

fn selected_history_status(app: &App) -> Option<String> {
    let entry = app.history.selected()?;
    Some(match &entry.state {
        RunHistoryState::Loaded(preview) => format!(
            "{} {}..{}",
            preview.symbol, preview.start_date, preview.end_date
        ),
        RunHistoryState::Failed(_) => format!(
            "invalid {}",
            entry
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("run")
        ),
    })
}

fn selected_research_item_status(app: &App) -> Option<String> {
    let research = app.research()?;
    let item = research.selected_item()?;
    Some(format!("{} {}", item.label, item.value))
}

fn selected_research_link_status(app: &App) -> Option<String> {
    let research = app.research()?;
    if let Some(path) = research.selected_link_path() {
        Some(path.display().to_string())
    } else if research.selected_item().is_some() {
        Some("blocked: selected item has no linked replay bundle".to_string())
    } else {
        None
    }
}

fn build_research_items(report: &ResearchReport) -> Vec<ResearchItem> {
    match report {
        ResearchReport::Aggregate(report) => report
            .members
            .iter()
            .map(|member| ResearchItem {
                label: member.symbol.clone(),
                value: member.net_equity_change.clone(),
                detail_title: format!("aggregate member {}", member.symbol),
                detail_lines: vec![
                    format!("bundle_path: {}", member.bundle_path.display()),
                    format!("rows: {}", member.row_count),
                    format!("warnings: {}", member.warning_count),
                    format!("trades: {}", member.trade_count),
                    format!("starting_equity: {}", member.starting_equity),
                    format!("ending_equity: {}", member.ending_equity),
                    format!("net_equity_change: {}", member.net_equity_change),
                ],
                linked_bundle_paths: vec![member.bundle_path.clone()],
            })
            .collect(),
        ResearchReport::WalkForward(report) => report
            .splits
            .iter()
            .map(|split| ResearchItem {
                label: format!("split {}", split.sequence),
                value: split.test_date_range.clone(),
                detail_title: format!("walk-forward split {}", split.sequence),
                detail_lines: vec![
                    format!("train_rows: {}", split.train_row_range),
                    format!("train_dates: {}", split.train_date_range),
                    format!("test_rows: {}", split.test_row_range),
                    format!("test_dates: {}", split.test_date_range),
                    format!(
                        "children: {}",
                        count_label(split.children.len(), "bundle", "bundles")
                    ),
                ],
                linked_bundle_paths: split
                    .children
                    .iter()
                    .map(|child| child.bundle_path.clone())
                    .collect(),
            })
            .collect(),
        ResearchReport::BootstrapAggregate(report) => {
            let mut items = vec![ResearchItem {
                label: "distribution".to_string(),
                value: report.distribution.metric.clone(),
                detail_title: "bootstrap aggregate distribution".to_string(),
                detail_lines: vec![
                    format!("seed: {}", report.distribution.seed),
                    format!("samples: {}", report.distribution.sample_count),
                    format!("resample_size: {}", report.distribution.resample_size),
                    format!("baseline_metric: {}", report.distribution.baseline_metric),
                    format!("bootstrap_mean: {}", report.distribution.bootstrap_mean),
                    format!(
                        "bootstrap_interval_95: {}..{}",
                        report.distribution.bootstrap_interval_95_lower,
                        report.distribution.bootstrap_interval_95_upper
                    ),
                ],
                linked_bundle_paths: Vec::new(),
            }];
            items.extend(report.baseline.members.iter().map(|member| ResearchItem {
                label: member.symbol.clone(),
                value: member.net_equity_change.clone(),
                detail_title: format!("baseline member {}", member.symbol),
                detail_lines: vec![
                    format!("bundle_path: {}", member.bundle_path.display()),
                    format!("rows: {}", member.row_count),
                    format!("warnings: {}", member.warning_count),
                    format!("trades: {}", member.trade_count),
                    format!("net_equity_change: {}", member.net_equity_change),
                ],
                linked_bundle_paths: vec![member.bundle_path.clone()],
            }));
            items
        }
        ResearchReport::BootstrapWalkForward(report) => {
            let mut items = vec![ResearchItem {
                label: "distribution".to_string(),
                value: report.distribution.metric.clone(),
                detail_title: "bootstrap walk-forward distribution".to_string(),
                detail_lines: vec![
                    format!("seed: {}", report.distribution.seed),
                    format!("samples: {}", report.distribution.sample_count),
                    format!("resample_size: {}", report.distribution.resample_size),
                    format!("baseline_metric: {}", report.distribution.baseline_metric),
                    format!("bootstrap_mean: {}", report.distribution.bootstrap_mean),
                    format!(
                        "bootstrap_interval_95: {}..{}",
                        report.distribution.bootstrap_interval_95_lower,
                        report.distribution.bootstrap_interval_95_upper
                    ),
                ],
                linked_bundle_paths: Vec::new(),
            }];
            items.extend(report.splits.iter().map(|split| {
                ResearchItem {
                    label: format!("split {}", split.sequence),
                    value: split.test_date_range.clone(),
                    detail_title: format!("bootstrap split {}", split.sequence),
                    detail_lines: vec![
                        format!("train_rows: {}", split.train_row_range),
                        format!("train_dates: {}", split.train_date_range),
                        format!("test_rows: {}", split.test_row_range),
                        format!("test_dates: {}", split.test_date_range),
                        format!(
                            "baseline_test_total_net_equity_change: {}",
                            split.baseline_test_total_net_equity_change
                        ),
                        format!(
                            "children: {}",
                            count_label(split.children.len(), "bundle", "bundles")
                        ),
                    ],
                    linked_bundle_paths: split
                        .children
                        .iter()
                        .map(|child| child.bundle_path.clone())
                        .collect(),
                }
            }));
            items
        }
        ResearchReport::Leaderboard(report) => report
            .rows
            .iter()
            .map(|row| ResearchItem {
                label: format!("rank {}", row.rank),
                value: row.label.clone(),
                detail_title: format!("leaderboard row {}", row.rank),
                detail_lines: vec![
                    format!("signal_id: {}", row.signal_id),
                    format!("filter_id: {}", row.filter_id),
                    format!("position_manager_id: {}", row.position_manager_id),
                    format!("execution_model_id: {}", row.execution_model_id),
                    format!("symbols: {}", row.aggregate.symbols.join("|")),
                    format!(
                        "member_bundles: {}",
                        count_label(row.aggregate.members.len(), "bundle", "bundles")
                    ),
                    format!(
                        "net_equity_change_total: {}",
                        row.aggregate.net_equity_change_total
                    ),
                ],
                linked_bundle_paths: row
                    .aggregate
                    .members
                    .iter()
                    .map(|member| member.bundle_path.clone())
                    .collect(),
            })
            .collect(),
    }
}

fn research_item_pane_title(report: &ResearchReport) -> &'static str {
    match report {
        ResearchReport::Aggregate(_) => "Members",
        ResearchReport::WalkForward(_) => "Splits",
        ResearchReport::BootstrapAggregate(_) => "Baseline",
        ResearchReport::BootstrapWalkForward(_) => "Bootstrap Splits",
        ResearchReport::Leaderboard(_) => "Rows",
    }
}

fn research_report_summary_lines(report: &ResearchReport) -> Vec<String> {
    match report {
        ResearchReport::Aggregate(report) => {
            vec![
                "kind: aggregate".to_string(),
                format!("snapshot_id: {}", report.snapshot_id),
                format!("provider: {}", report.provider_identity),
                format!("date_range: {}", report.date_range),
                format!("symbols: {}", report.symbols.join("|")),
                format!("symbol_count: {}", report.symbol_count),
                format!(
                    "members: {}",
                    count_label(report.members.len(), "member", "members")
                ),
                format!("total_trades: {}", report.total_trade_count),
                format!(
                    "net_equity_change_total: {}",
                    report.net_equity_change_total
                ),
                "drilldown_status: linked replay bundles reopen from this report shell".to_string(),
            ]
        }
        ResearchReport::WalkForward(report) => vec![
            "kind: walk_forward".to_string(),
            format!("snapshot_id: {}", report.snapshot_id),
            format!("provider: {}", report.provider_identity),
            format!("date_range: {}", report.date_range),
            format!("symbols: {}", report.symbols.join("|")),
            format!("train_bars: {}", report.train_bars),
            format!("test_bars: {}", report.test_bars),
            format!("split_count: {}", report.split_count),
            "drilldown_status: linked replay bundles reopen from this report shell".to_string(),
        ],
        ResearchReport::BootstrapAggregate(report) => vec![
            "kind: bootstrap_aggregate".to_string(),
            format!("baseline_snapshot_id: {}", report.baseline.snapshot_id),
            format!("baseline_symbols: {}", report.baseline.symbols.join("|")),
            format!("samples: {}", report.distribution.sample_count),
            format!("seed: {}", report.distribution.seed),
            format!("metric: {}", report.distribution.metric),
            format!(
                "bootstrap_interval_95: {}..{}",
                report.distribution.bootstrap_interval_95_lower,
                report.distribution.bootstrap_interval_95_upper
            ),
            "drilldown_status: linked replay bundles reopen from this report shell".to_string(),
        ],
        ResearchReport::BootstrapWalkForward(report) => vec![
            "kind: bootstrap_walk_forward".to_string(),
            format!("baseline_snapshot_id: {}", report.baseline.snapshot_id),
            format!("baseline_symbols: {}", report.baseline.symbols.join("|")),
            format!("baseline_splits: {}", report.baseline.split_count),
            format!("samples: {}", report.distribution.sample_count),
            format!("seed: {}", report.distribution.seed),
            format!("metric: {}", report.distribution.metric),
            format!(
                "bootstrap_interval_95: {}..{}",
                report.distribution.bootstrap_interval_95_lower,
                report.distribution.bootstrap_interval_95_upper
            ),
            "drilldown_status: linked replay bundles reopen from this report shell".to_string(),
        ],
        ResearchReport::Leaderboard(report) => vec![
            "kind: leaderboard".to_string(),
            format!("view: {}", report.view.as_str()),
            format!("snapshot_id: {}", report.snapshot_id),
            format!("provider: {}", report.provider_identity),
            format!("date_range: {}", report.date_range),
            format!("symbols: {}", report.symbols.join("|")),
            format!("rows: {}", count_label(report.rows.len(), "row", "rows")),
            "drilldown_status: linked replay bundles reopen from this report shell".to_string(),
        ],
    }
}

fn research_report_link_count(report: &ResearchReport) -> usize {
    research_report_link_paths(report).len()
}

fn research_report_link_paths(report: &ResearchReport) -> Vec<&PathBuf> {
    match report {
        ResearchReport::Aggregate(report) => report
            .members
            .iter()
            .map(|member| &member.bundle_path)
            .collect(),
        ResearchReport::WalkForward(report) => report
            .splits
            .iter()
            .flat_map(|split| split.children.iter().map(|child| &child.bundle_path))
            .collect(),
        ResearchReport::BootstrapAggregate(report) => report
            .baseline
            .members
            .iter()
            .map(|member| &member.bundle_path)
            .collect(),
        ResearchReport::BootstrapWalkForward(report) => report
            .splits
            .iter()
            .flat_map(|split| split.children.iter().map(|child| &child.bundle_path))
            .collect(),
        ResearchReport::Leaderboard(report) => report
            .rows
            .iter()
            .flat_map(|row| {
                row.aggregate
                    .members
                    .iter()
                    .map(|member| &member.bundle_path)
            })
            .collect(),
    }
}

fn manifest_parameter_value_or<'a>(
    manifest: &'a trendlab_artifact::RunManifest,
    name: &str,
    fallback: &'a str,
) -> &'a str {
    manifest
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .map(|parameter| parameter.value.as_str())
        .unwrap_or(fallback)
}

fn count_label(count: usize, singular: &str, plural: &str) -> String {
    match count {
        0 => format!("0 {plural}"),
        1 => format!("1 {singular}"),
        _ => format!("{count} {plural}"),
    }
}

fn format_optional_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_text(value: Option<&str>) -> String {
    value.unwrap_or("none").to_string()
}

fn format_signed_f64(value: f64) -> String {
    if value >= 0.0 {
        format!("+{value:.4}")
    } else {
        format!("{value:.4}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupArtifactKind {
    ReplayBundle,
    ResearchReport,
}

fn load_startup_app_for_artifact_path(
    artifact_path: &Path,
    snapshot_paths: Vec<PathBuf>,
) -> Result<App, TuiError> {
    match detect_startup_artifact_kind(artifact_path)? {
        StartupArtifactKind::ReplayBundle => {
            let bundle = load_replay_bundle(artifact_path)
                .map_err(|err| TuiError::invalid(err.to_string()))?;
            Ok(App::from_bundle(
                artifact_path.to_path_buf(),
                bundle,
                snapshot_paths,
            ))
        }
        StartupArtifactKind::ResearchReport => {
            let report = load_research_report_bundle(artifact_path)
                .map_err(|err| TuiError::invalid(err.to_string()))?;
            Ok(App::from_report(
                artifact_path.to_path_buf(),
                report,
                snapshot_paths,
            ))
        }
    }
}

fn detect_startup_artifact_kind(path: &Path) -> Result<StartupArtifactKind, TuiError> {
    let has_bundle = path.join(BUNDLE_FILE_NAME).is_file();
    let has_report = path.join(RESEARCH_REPORT_FILE_NAME).is_file();

    match (has_bundle, has_report) {
        (true, false) => Ok(StartupArtifactKind::ReplayBundle),
        (false, true) => Ok(StartupArtifactKind::ResearchReport),
        (true, true) => Err(TuiError::invalid(format!(
            "artifact directory {} contains both {} and {}; choose one shared artifact type",
            path.display(),
            BUNDLE_FILE_NAME,
            RESEARCH_REPORT_FILE_NAME
        ))),
        (false, false) => Err(TuiError::invalid(format!(
            "artifact directory {} does not contain {} or {}",
            path.display(),
            BUNDLE_FILE_NAME,
            RESEARCH_REPORT_FILE_NAME
        ))),
    }
}

fn format_reason_codes(reason_codes: &[String]) -> String {
    if reason_codes.is_empty() {
        "none".to_string()
    } else {
        reason_codes.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use trendlab_artifact::{
        BUNDLE_FILE_NAME, BootstrapDistributionSummary, BundleDescriptor, DateRange,
        LeaderboardView, ManifestParameter, RESEARCH_REPORT_FILE_NAME, ReferenceFlowDefinition,
        ResearchAggregateMember, ResearchAggregateReport, ResearchBootstrapAggregateReport,
        ResearchBootstrapWalkForwardReport, ResearchBootstrapWalkForwardSplit,
        ResearchLeaderboardReport, ResearchLeaderboardRow, ResearchReport,
        ResearchWalkForwardReport, ResearchWalkForwardSplit, ResearchWalkForwardSplitChild,
        RunManifest, RunSummary, SCHEMA_VERSION, write_replay_bundle, write_research_report_bundle,
    };
    use trendlab_core::accounting::CostModel;
    use trendlab_core::orders::GapPolicy;

    use super::{
        App, AppCommand, AppMode, FocusPane, HomeFocusPane, InspectApp, ResearchFocusPane,
        RunFormField, SNAPSHOT_SELECTION_END_PARAMETER, SNAPSHOT_SELECTION_START_PARAMETER,
        SnapshotLoadState, StartupArtifactKind, build_audit_lines, bundle_provenance_summary,
        detect_startup_artifact_kind, load_startup_app_for_artifact_path,
        manifest_parameter_value_or, parse_bundle_path, render, run_validation_lines,
    };

    #[test]
    fn app_surfaces_bundle_provenance_and_audit_summary() {
        let app = sample_app();
        let inspect = inspect_app(&app);

        assert_eq!(app.mode, AppMode::Inspect);
        assert_eq!(inspect.focus, FocusPane::Results);
        assert_eq!(inspect.results.len(), 6);
        assert_eq!(inspect.results[0].label, "run");
        assert_eq!(inspect.results[1].label, "trade 1");
        assert_eq!(inspect.results[1].value, "closed -10.2000");
        assert_eq!(inspect.results[4].label, "data audit");
        assert_eq!(inspect.results[5].label, "artifact");
        assert_eq!(inspect.audit_report.analysis_adjusted_bar_count, 1);
    }

    #[test]
    fn navigation_is_scoped_by_the_focused_pane() {
        let mut app = sample_app();

        app.apply(AppCommand::MoveDown);
        assert_eq!(inspect_app(&app).selected_result, 1);
        assert_eq!(inspect_app(&app).selected_ledger, 1);

        app.apply(AppCommand::NextFocus);
        assert_eq!(inspect_app(&app).focus, FocusPane::Chart);
        app.apply(AppCommand::MoveDown);
        assert_eq!(inspect_app(&app).selected_result, 1);
        assert_eq!(inspect_app(&app).selected_ledger, 2);

        app.apply(AppCommand::NextFocus);
        assert_eq!(inspect_app(&app).focus, FocusPane::Ledger);
        app.apply(AppCommand::MoveDown);
        assert_eq!(inspect_app(&app).selected_result, 1);
        assert_eq!(inspect_app(&app).selected_ledger, 0);

        app.apply(AppCommand::NextFocus);
        assert_eq!(inspect_app(&app).focus, FocusPane::Help);
        app.apply(AppCommand::MoveDown);
        assert_eq!(inspect_app(&app).selected_ledger, 0);
    }

    #[test]
    fn top_level_modes_preserve_loaded_inspect_state() {
        let mut app = sample_app();
        app.apply(AppCommand::MoveDown);
        app.apply(AppCommand::ShowHome);

        assert_eq!(app.mode, AppMode::Home);
        assert_eq!(inspect_app(&app).selected_result, 1);

        app.apply(AppCommand::ShowInspect);

        assert_eq!(app.mode, AppMode::Inspect);
        assert_eq!(inspect_app(&app).selected_result, 1);
        assert_eq!(inspect_app(&app).selected_ledger, 1);

        app.apply(AppCommand::ShowHelp);
        assert_eq!(app.mode, AppMode::Help);
    }

    #[test]
    fn render_keeps_provenance_and_ledger_reasoning_visible() {
        let mut app = sample_app();
        app.apply(AppCommand::MoveDown);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("TrendLab TUI"));
        assert!(screen.contains("snapshot=fixture:m1_intrabar_stop_exit"));
        assert!(screen.contains("Inspect"));
        assert!(screen.contains("Chart"));
        assert!(screen.contains("Audit"));
        assert!(screen.contains("trade 1"));
        assert!(screen.contains("Selected Row"));
        assert!(
            build_audit_lines(inspect_app(&app))
                .iter()
                .any(|line| line.contains("entry_filled_at_open"))
        );
    }

    #[test]
    fn app_can_start_without_a_replay_bundle() {
        let app = App::home();
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert_eq!(app.mode, AppMode::Home);
        assert!(screen.contains("operator_workspace"));
        assert!(screen.contains("snapshot_source: not selected"));
        assert!(screen.contains("loaded_bundle: none"));
    }

    #[test]
    fn startup_artifact_detection_distinguishes_replay_and_research_bundles() {
        let replay_dir = test_output_dir("tui-startup-replay");
        let report_dir = test_output_dir("tui-startup-report");
        write_sample_replay_bundle(&replay_dir);
        write_sample_research_report_bundle(&report_dir);

        assert_eq!(
            detect_startup_artifact_kind(&replay_dir).unwrap(),
            StartupArtifactKind::ReplayBundle
        );
        assert_eq!(
            detect_startup_artifact_kind(&report_dir).unwrap(),
            StartupArtifactKind::ResearchReport
        );

        remove_dir_all_if_exists(&replay_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn startup_report_bundle_loads_research_mode_and_renders_summary_shell() {
        let report_dir = test_output_dir("tui-startup-report-shell");
        write_sample_research_report_bundle(&report_dir);

        let app = load_startup_app_for_artifact_path(&report_dir, Vec::new()).unwrap();

        assert_eq!(app.mode, AppMode::Research);
        assert!(app.inspect().is_none());
        let research = app
            .research()
            .expect("report app should load research state");
        assert_eq!(research.report_path, report_dir);
        assert_eq!(research.report.kind(), "aggregate");
        assert_eq!(research.focus, ResearchFocusPane::Summary);

        let backend = TestBackend::new(150, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Research Report"));
        assert!(screen.contains("report_kind=aggregate"));
        assert!(screen.contains("Summary"));
        assert!(screen.contains("Members"));
        assert!(screen.contains("Detail"));
        assert!(screen.contains("TEST"));
        assert!(screen.contains("drilldown_status"));
        assert!(screen.contains("research_focus: Summary"));

        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn startup_report_bundle_reopens_selected_linked_bundle_into_inspect_mode() {
        let report_dir = test_output_dir("tui-startup-report-drilldown");
        write_sample_research_report_bundle(&report_dir);
        let linked_bundle_dir = report_dir.join("linked-run");

        let mut app = load_startup_app_for_artifact_path(&report_dir, Vec::new()).unwrap();

        let backend = TestBackend::new(150, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Research Report"));
        assert!(screen.contains("drilldown_status"));

        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Inspect);
        assert_eq!(inspect_app(&app).bundle_path, linked_bundle_dir);
        assert!(app.research().is_some());

        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn startup_report_bundle_rejects_malformed_research_json() {
        let report_dir = test_output_dir("tui-startup-malformed-report");
        fs::create_dir_all(&report_dir).unwrap();
        fs::write(
            report_dir.join(RESEARCH_REPORT_FILE_NAME),
            "{ this is not valid json",
        )
        .unwrap();

        let error = load_startup_app_for_artifact_path(&report_dir, Vec::new()).unwrap_err();
        assert!(error.to_string().contains("failed to read"));
        assert!(error.to_string().contains(RESEARCH_REPORT_FILE_NAME));

        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_mode_navigation_is_scoped_to_the_item_pane() {
        let report_path = PathBuf::from("target/test-output/research-navigation-report");
        let history_root = test_output_dir("tui-research-navigation-history");
        let linked_bundle_path = PathBuf::from("target/test-output/research-linked-run");
        let report = sample_multi_member_aggregate_report(linked_bundle_path);
        let mut app =
            App::from_report_with_history_root(report_path, report, Vec::new(), history_root);

        assert_eq!(app.mode, AppMode::Research);
        assert_eq!(
            app.research().expect("research state").focus,
            ResearchFocusPane::Summary
        );
        assert_eq!(
            app.research()
                .expect("research state")
                .selected_item_index(),
            Some(0)
        );

        app.apply(AppCommand::MoveDown);
        assert_eq!(
            app.research()
                .expect("research state")
                .selected_item_index(),
            Some(0)
        );

        app.apply(AppCommand::NextFocus);
        assert_eq!(
            app.research().expect("research state").focus,
            ResearchFocusPane::Items
        );

        app.apply(AppCommand::MoveDown);
        assert_eq!(
            app.research()
                .expect("research state")
                .selected_item_index(),
            Some(1)
        );

        app.apply(AppCommand::NextFocus);
        assert_eq!(
            app.research().expect("research state").focus,
            ResearchFocusPane::Detail
        );

        app.apply(AppCommand::MoveDown);
        assert_eq!(
            app.research()
                .expect("research state")
                .selected_item_index(),
            Some(1)
        );

        app.apply(AppCommand::PreviousFocus);
        assert_eq!(
            app.research().expect("research state").focus,
            ResearchFocusPane::Items
        );
    }

    #[test]
    fn research_render_surfaces_supported_report_kinds() {
        let cases = vec![
            (
                "aggregate",
                sample_multi_member_aggregate_report(PathBuf::from(
                    "target/test-output/research-aggregate-linked",
                )),
                vec!["kind: aggregate", "Members", "TEST", "drilldown_status"],
            ),
            (
                "walk_forward",
                sample_walk_forward_report(PathBuf::from(
                    "target/test-output/research-walk-forward-linked",
                )),
                vec!["kind: walk_forward", "Splits", "split 1", "train_bars: 20"],
            ),
            (
                "bootstrap_aggregate",
                sample_bootstrap_aggregate_report(PathBuf::from(
                    "target/test-output/research-bootstrap-aggregate-linked",
                )),
                vec![
                    "kind: bootstrap_aggregate",
                    "Baseline",
                    "distribution",
                    "samples: 64",
                ],
            ),
            (
                "bootstrap_walk_forward",
                sample_bootstrap_walk_forward_report(PathBuf::from(
                    "target/test-output/research-bootstrap-walk-forward-linked",
                )),
                vec![
                    "kind: bootstrap_walk_forward",
                    "Bootstrap Splits",
                    "distribution",
                    "baseline_splits: 2",
                ],
            ),
            (
                "leaderboard",
                sample_leaderboard_report(PathBuf::from(
                    "target/test-output/research-leaderboard-linked",
                )),
                vec!["kind: leaderboard", "Rows", "rank 1", "view: signal"],
            ),
        ];

        for (label, report, expected_lines) in cases {
            let app = App::from_report_with_history_root(
                PathBuf::from(format!("target/test-output/{label}-report")),
                report,
                Vec::new(),
                test_output_dir(&format!("tui-{label}-history")),
            );

            let backend = TestBackend::new(170, 36);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(frame, &app)).unwrap();
            let screen = buffer_text(terminal.backend().buffer());

            assert!(screen.contains("Research Report"));
            for expected in expected_lines {
                assert!(
                    screen.contains(expected),
                    "expected {expected:?} in rendered screen for {label}, got:\n{screen}"
                );
            }
        }
    }

    #[test]
    fn research_drilldown_reopens_selected_linked_bundle_into_inspect_mode() {
        let report_root = test_output_dir("tui-research-drilldown-success");
        let first_bundle = report_root.join("walk-forward-child-1");
        let second_bundle = report_root.join("walk-forward-child-2");
        write_sample_replay_bundle(&first_bundle);
        write_sample_replay_bundle(&second_bundle);

        let report = sample_walk_forward_report(first_bundle.clone());
        let mut app = App::from_report_with_history_root(
            report_root.join("research-report"),
            report,
            Vec::new(),
            report_root.join("history"),
        );

        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::NextFocus);
        assert_eq!(
            app.research().expect("research state").focus,
            ResearchFocusPane::Detail
        );

        app.apply(AppCommand::MoveDown);
        assert_eq!(
            app.research()
                .expect("research state")
                .selected_link_path()
                .cloned(),
            Some(second_bundle.clone())
        );

        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Inspect);
        assert_eq!(inspect_app(&app).bundle_path, second_bundle);
        assert!(app.research().is_some());

        remove_dir_all_if_exists(&report_root);
    }

    #[test]
    fn research_drilldown_failure_stays_in_research_mode_with_explicit_error() {
        let report_dir = test_output_dir("tui-research-drilldown-failure");
        write_sample_research_report_bundle(&report_dir);

        let linked_bundle_dir = report_dir.join("linked-run");
        let mut app = load_startup_app_for_artifact_path(&report_dir, Vec::new()).unwrap();
        remove_dir_all_if_exists(&linked_bundle_dir);

        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Research);
        assert!(
            app.reopen_error
                .as_deref()
                .expect("reopen error should be captured")
                .contains("failed to reopen linked replay bundle")
        );

        let backend = TestBackend::new(170, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("reopen_status: failed"));
        assert!(screen.contains("reopen_error:"));
        assert!(screen.contains("(missing)"));

        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn startup_artifact_detection_rejects_directories_without_shared_artifact_files() {
        let invalid_dir = test_output_dir("tui-startup-invalid-artifact");
        fs::create_dir_all(&invalid_dir).unwrap();

        let error = detect_startup_artifact_kind(&invalid_dir).unwrap_err();
        assert!(
            error.to_string().contains(BUNDLE_FILE_NAME)
                && error.to_string().contains(RESEARCH_REPORT_FILE_NAME)
        );

        remove_dir_all_if_exists(&invalid_dir);
    }

    #[test]
    fn home_footer_surfaces_selected_snapshot_launch_and_history_targets() {
        let snapshot_dir = test_output_dir("tui-home-footer-snapshot");
        let history_root = test_output_dir("tui-home-footer-history");
        let bundle_dir = history_root.join("valid-run");
        write_sample_snapshot_bundle(&snapshot_dir);
        write_sample_replay_bundle(&bundle_dir);
        let app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        let backend = TestBackend::new(180, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("home_focus: snapshots"));
        assert!(screen.contains("selected_snapshot:"));
        assert!(screen.contains("launch_target:"));
        assert!(screen.contains("history_target:"));

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn home_browser_surfaces_valid_snapshot_summary() {
        let snapshot_dir = test_output_dir("tui-snapshot-browser-valid");
        write_sample_snapshot_bundle(&snapshot_dir);
        let app = App::home_with_snapshots(vec![snapshot_dir.clone()]);

        let selected = app
            .selected_snapshot()
            .expect("snapshot should be selected");
        assert!(matches!(selected.state, SnapshotLoadState::Loaded(_)));

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Snapshot Summary"));
        assert!(screen.contains("snapshot_id:"));
        assert!(screen.contains("live:tiingo:TEST"));
        assert!(screen.contains("provider: tiingo"));
        assert!(screen.contains("requested_window:"));
        assert!(screen.contains("2025-01-10"));
        assert!(screen.contains("symbol: TEST raw=4 actions=2"));

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn home_browser_keeps_malformed_snapshot_visible() {
        let valid_snapshot_dir = test_output_dir("tui-snapshot-browser-valid-mixed");
        let invalid_snapshot_dir = test_output_dir("tui-snapshot-browser-invalid");
        write_sample_snapshot_bundle(&valid_snapshot_dir);

        let mut app = App::home_with_snapshots(vec![
            valid_snapshot_dir.clone(),
            invalid_snapshot_dir.clone(),
        ]);
        app.apply(AppCommand::MoveDown);

        let selected = app
            .selected_snapshot()
            .expect("snapshot should be selected");
        assert_eq!(selected.path, invalid_snapshot_dir);
        assert!(matches!(selected.state, SnapshotLoadState::Failed(_)));

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("status: invalid"));
        assert!(screen.contains("snapshot.json"));

        remove_dir_all_if_exists(&valid_snapshot_dir);
        remove_dir_all_if_exists(&invalid_snapshot_dir);
    }

    #[test]
    fn home_run_form_surfaces_ready_snapshot_backed_preview() {
        let snapshot_dir = test_output_dir("tui-run-form-ready");
        write_sample_snapshot_bundle(&snapshot_dir);
        let app = App::home_with_snapshots(vec![snapshot_dir.clone()]);

        let backend = TestBackend::new(140, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Run Form"));
        assert!(screen.contains("status: ready"));
        assert!(screen.contains("run_source_kind: snapshot"));

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn home_run_form_surfaces_validation_error_before_launch() {
        let snapshot_dir = test_output_dir("tui-run-form-invalid");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots(vec![snapshot_dir.clone()]);

        focus_run_form(&mut app);
        set_run_form_field(&mut app, RunFormField::StartDate);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);
        set_run_form_field(&mut app, RunFormField::EndDate);
        app.apply(AppCommand::AdjustPrevious);
        app.apply(AppCommand::AdjustPrevious);
        app.apply(AppCommand::AdjustPrevious);

        let backend = TestBackend::new(140, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("status: invalid"));
        assert!(screen.contains("end_date on or after start_date"));
        assert!(
            run_validation_lines(&app)
                .iter()
                .any(|line| line.contains("blocked until validation passes"))
        );

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn home_launch_runs_non_default_tui_configuration_end_to_end() {
        let snapshot_dir = test_output_dir("tui-launch-configured-snapshot");
        let history_root = test_output_dir("tui-launch-configured-history");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        configure_non_default_run_form(&mut app);
        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Inspect);
        let inspect = inspect_app(&app);
        let bundle_path = inspect.bundle_path.clone();
        assert!(bundle_path.starts_with(&history_root));
        assert!(bundle_path.join("bundle.json").is_file());
        assert_eq!(inspect.bundle.manifest.date_range.start_date, "2025-01-03");
        assert_eq!(inspect.bundle.manifest.date_range.end_date, "2025-01-07");
        assert_eq!(inspect.bundle.summary.row_count, 3);
        assert_eq!(
            inspect.bundle.ledger.first().map(|row| row.date.as_str()),
            Some("2025-01-03")
        );
        assert_eq!(inspect.bundle.manifest.reference_flow.entry_shares, 2);
        assert_eq!(
            inspect
                .bundle
                .manifest
                .reference_flow
                .protective_stop_fraction,
            0.11
        );
        assert_eq!(inspect.bundle.manifest.cost_model.commission_per_fill, 0.25);
        assert_eq!(inspect.bundle.manifest.cost_model.slippage_per_share, 0.01);
        assert_eq!(
            manifest_parameter_value_or(
                &inspect.bundle.manifest,
                SNAPSHOT_SELECTION_START_PARAMETER,
                "missing"
            ),
            "2025-01-03"
        );
        assert_eq!(
            manifest_parameter_value_or(
                &inspect.bundle.manifest,
                SNAPSHOT_SELECTION_END_PARAMETER,
                "missing"
            ),
            "2025-01-07"
        );
        assert!(
            app.history
                .entries
                .iter()
                .any(|entry| entry.path == bundle_path)
        );

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
        remove_dir_all_if_exists(&bundle_path);
    }

    #[test]
    fn home_launch_executes_and_auto_opens_inspect_mode() {
        let snapshot_dir = test_output_dir("tui-launch-success");
        let history_root = test_output_dir("tui-launch-success-history");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Inspect);
        let inspect = inspect_app(&app);
        let bundle_path = inspect.bundle_path.clone();
        assert!(inspect.bundle_path.starts_with(&history_root));
        assert!(inspect.bundle_path.join("bundle.json").is_file());
        assert_eq!(inspect.bundle.manifest.symbol_or_universe, "TEST");
        assert_eq!(inspect.bundle.manifest.provider_identity, "tiingo");

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
        remove_dir_all_if_exists(&bundle_path);
    }

    #[test]
    fn inspect_render_preserves_snapshot_backed_provenance_after_home_launch() {
        let snapshot_dir = test_output_dir("tui-launch-provenance-snapshot");
        let history_root = test_output_dir("tui-launch-provenance-history");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        app.apply(AppCommand::LaunchRun);

        let backend = TestBackend::new(180, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Inspect"));
        let provenance = bundle_provenance_summary(&inspect_app(&app).bundle);
        assert!(provenance.contains("run_source=snapshot"));
        assert!(provenance.contains("request_source=inline_template"));

        let bundle_path = inspect_app(&app).bundle_path.clone();
        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
        remove_dir_all_if_exists(&bundle_path);
    }

    #[test]
    fn invalid_home_launch_stays_in_home_mode_with_error_message() {
        let snapshot_dir = test_output_dir("tui-launch-invalid");
        let history_root = test_output_dir("tui-launch-invalid-history");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::MoveDown);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::MoveDown);
        app.apply(AppCommand::AdjustPrevious);
        app.apply(AppCommand::AdjustPrevious);
        app.apply(AppCommand::AdjustPrevious);
        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Home);
        assert!(
            app.launch_error
                .as_deref()
                .expect("launch error should be captured")
                .contains("end_date on or after start_date")
        );

        let backend = TestBackend::new(140, 34);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("launch_status: failed"));
        assert!(screen.contains("launch_error:"));

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn launch_without_any_snapshot_selection_stays_in_home_mode_with_explicit_error() {
        let history_root = test_output_dir("tui-launch-no-snapshot-history");
        let mut app = App::home_with_snapshots_and_history_root(Vec::new(), history_root.clone());

        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Home);
        assert_eq!(
            app.launch_error.as_deref(),
            Some("select a stored snapshot before launching a run")
        );

        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn launch_with_invalid_selected_snapshot_stays_in_home_mode_with_snapshot_error() {
        let valid_snapshot_dir = test_output_dir("tui-launch-invalid-selected-valid");
        let invalid_snapshot_dir = test_output_dir("tui-launch-invalid-selected-invalid");
        let history_root = test_output_dir("tui-launch-invalid-selected-history");
        write_sample_snapshot_bundle(&valid_snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![valid_snapshot_dir.clone(), invalid_snapshot_dir.clone()],
            history_root.clone(),
        );
        app.apply(AppCommand::MoveDown);

        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Home);
        assert!(
            app.launch_error
                .as_deref()
                .expect("launch error should be captured")
                .contains("selected snapshot directory failed to load")
        );

        remove_dir_all_if_exists(&valid_snapshot_dir);
        remove_dir_all_if_exists(&invalid_snapshot_dir);
        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn launch_rejects_zero_entry_shares_and_stays_in_home_mode() {
        let snapshot_dir = test_output_dir("tui-launch-zero-shares");
        let history_root = test_output_dir("tui-launch-zero-shares-history");
        write_sample_snapshot_bundle(&snapshot_dir);
        let mut app = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );

        focus_run_form(&mut app);
        set_run_form_field(&mut app, RunFormField::EntryShares);
        app.apply(AppCommand::AdjustPrevious);
        assert!(
            run_validation_lines(&app)
                .iter()
                .any(|line| line.contains("entry_shares must be greater than zero"))
        );

        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Home);
        assert!(
            app.launch_error
                .as_deref()
                .expect("launch error should be captured")
                .contains("entry_shares must be greater than zero")
        );

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn home_history_lists_valid_and_invalid_prior_runs() {
        let history_root = test_output_dir("tui-history-list");
        let valid_bundle_dir = history_root.join("valid-run");
        let invalid_bundle_dir = history_root.join("broken-run");
        write_sample_replay_bundle(&valid_bundle_dir);
        fs::create_dir_all(&invalid_bundle_dir).unwrap();

        let mut app = App::home_with_snapshots_and_history_root(Vec::new(), history_root.clone());
        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::NextFocus);

        let backend = TestBackend::new(150, 38);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Run History"));
        assert!(screen.contains("valid-run"));
        assert!(screen.contains("broken-run"));
        assert!(screen.contains("Run Preview"));

        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn home_history_keeps_invalid_bundle_visible_with_error() {
        let history_root = test_output_dir("tui-history-invalid");
        let invalid_bundle_dir = history_root.join("broken-run");
        fs::create_dir_all(&invalid_bundle_dir).unwrap();

        let app = App::home_with_snapshots_and_history_root(Vec::new(), history_root.clone());

        let selected = app.history.selected().expect("history entry should exist");
        assert_eq!(selected.path, invalid_bundle_dir);

        let backend = TestBackend::new(150, 38);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("status: invalid"));
        assert!(screen.contains("reopen: blocked"));

        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn home_history_reopens_selected_bundle_into_inspect_mode() {
        let history_root = test_output_dir("tui-history-open");
        let bundle_dir = history_root.join("valid-run");
        write_sample_replay_bundle(&bundle_dir);
        let mut app = App::home_with_snapshots_and_history_root(Vec::new(), history_root.clone());

        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::NextFocus);
        app.apply(AppCommand::LaunchRun);

        assert_eq!(app.mode, AppMode::Inspect);
        assert_eq!(inspect_app(&app).bundle_path, bundle_dir);

        remove_dir_all_if_exists(&history_root);
    }

    #[test]
    fn inspect_render_preserves_snapshot_backed_provenance_after_history_reopen() {
        let snapshot_dir = test_output_dir("tui-history-provenance-snapshot");
        let history_root = test_output_dir("tui-history-provenance-history");
        write_sample_snapshot_bundle(&snapshot_dir);

        let mut launched = App::home_with_snapshots_and_history_root(
            vec![snapshot_dir.clone()],
            history_root.clone(),
        );
        launched.apply(AppCommand::LaunchRun);
        let launched_bundle_path = inspect_app(&launched).bundle_path.clone();

        let mut reopened =
            App::home_with_snapshots_and_history_root(Vec::new(), history_root.clone());
        reopened.apply(AppCommand::NextFocus);
        reopened.apply(AppCommand::NextFocus);
        reopened.apply(AppCommand::LaunchRun);

        assert_eq!(reopened.mode, AppMode::Inspect);
        assert_eq!(inspect_app(&reopened).bundle_path, launched_bundle_path);

        let backend = TestBackend::new(180, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &reopened)).unwrap();
        let screen = buffer_text(terminal.backend().buffer());

        assert!(screen.contains("Inspect"));
        let provenance = bundle_provenance_summary(&inspect_app(&reopened).bundle);
        assert!(provenance.contains("run_source=snapshot"));
        assert!(provenance.contains("snapshot_source="));
        assert!(provenance.contains("selection="));
        assert!(!provenance.contains("selection=none..none"));

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&history_root);
        remove_dir_all_if_exists(&launched_bundle_path);
    }

    #[test]
    fn empty_args_start_home_mode_and_open_args_start_inspect_mode() {
        assert_eq!(parse_bundle_path(Vec::new()).unwrap(), None);
        assert_eq!(
            parse_bundle_path(vec!["open".to_string(), "bundle-dir".to_string()]).unwrap(),
            Some(PathBuf::from("bundle-dir"))
        );
        assert_eq!(
            parse_bundle_path(vec!["bundle-dir".to_string()]).unwrap(),
            Some(PathBuf::from("bundle-dir"))
        );
        let options = super::parse_startup_options(vec![
            "--snapshot".to_string(),
            "snapshot-a".to_string(),
            "open".to_string(),
            "bundle-dir".to_string(),
            "--snapshot".to_string(),
            "snapshot-b".to_string(),
        ])
        .unwrap();
        assert_eq!(options.bundle_path, Some(PathBuf::from("bundle-dir")));
        assert_eq!(
            options.snapshot_paths,
            vec![PathBuf::from("snapshot-a"), PathBuf::from("snapshot-b")]
        );
    }

    fn sample_app() -> App {
        App::from_bundle_with_history_root(
            "target/test-output/sample-bundle".into(),
            trendlab_artifact::ReplayBundle {
                descriptor: BundleDescriptor::canonical(),
                manifest: RunManifest {
                    schema_version: SCHEMA_VERSION,
                    engine_version: "m1-reference-flow".to_string(),
                    data_snapshot_id: "fixture:m1_intrabar_stop_exit".to_string(),
                    provider_identity: "fixture".to_string(),
                    symbol_or_universe: "TEST".to_string(),
                    universe_mode: "single_symbol".to_string(),
                    historical_limitations: Vec::new(),
                    date_range: DateRange {
                        start_date: "2025-01-02".to_string(),
                        end_date: "2025-01-06".to_string(),
                    },
                    reference_flow: ReferenceFlowDefinition {
                        kind: "m1_reference_flow".to_string(),
                        entry_shares: 1,
                        protective_stop_fraction: 0.10,
                    },
                    parameters: vec![ManifestParameter {
                        name: "fixture_scenario".to_string(),
                        value: "m1_intrabar_stop_exit".to_string(),
                    }],
                    cost_model: CostModel::default(),
                    gap_policy: GapPolicy::M1Default,
                    seed: None,
                    warnings: vec!["example_warning".to_string()],
                },
                summary: RunSummary {
                    row_count: 3,
                    warning_count: 1,
                    ending_cash: 989.8,
                    ending_equity: 989.8,
                },
                ledger: vec![
                    trendlab_artifact::PersistedLedgerRow {
                        date: "2025-01-02".to_string(),
                        raw_open: 100.0,
                        raw_high: 101.0,
                        raw_low: 99.0,
                        raw_close: 100.5,
                        analysis_close: 50.25,
                        position_shares: 0,
                        signal_output: "queue_market_entry".to_string(),
                        filter_outcome: "pass".to_string(),
                        pending_order_state: "queue_market_entry:1".to_string(),
                        fill_price: None,
                        prior_stop: None,
                        next_stop: None,
                        cash: 1000.0,
                        equity: 1000.0,
                        reason_codes: vec!["entry_intent_queued".to_string()],
                    },
                    trendlab_artifact::PersistedLedgerRow {
                        date: "2025-01-03".to_string(),
                        raw_open: 102.0,
                        raw_high: 104.0,
                        raw_low: 101.0,
                        raw_close: 103.5,
                        analysis_close: 103.5,
                        position_shares: 1,
                        signal_output: "none".to_string(),
                        filter_outcome: "not_checked".to_string(),
                        pending_order_state: "none".to_string(),
                        fill_price: Some(102.0),
                        prior_stop: None,
                        next_stop: Some(91.8),
                        cash: 898.0,
                        equity: 1001.5,
                        reason_codes: vec![
                            "entry_filled_at_open".to_string(),
                            "protective_stop_set".to_string(),
                        ],
                    },
                    trendlab_artifact::PersistedLedgerRow {
                        date: "2025-01-06".to_string(),
                        raw_open: 103.0,
                        raw_high: 103.5,
                        raw_low: 91.0,
                        raw_close: 92.0,
                        analysis_close: 92.0,
                        position_shares: 0,
                        signal_output: "none".to_string(),
                        filter_outcome: "not_checked".to_string(),
                        pending_order_state: "none".to_string(),
                        fill_price: Some(91.8),
                        prior_stop: Some(91.8),
                        next_stop: None,
                        cash: 989.8,
                        equity: 989.8,
                        reason_codes: vec!["protective_stop_hit_intrabar".to_string()],
                    },
                ],
            },
            Vec::new(),
            test_output_dir("tui-sample-history"),
        )
    }

    fn inspect_app(app: &App) -> &InspectApp {
        app.inspect().expect("sample app should have inspect state")
    }

    fn write_sample_research_report_bundle(report_dir: &Path) {
        remove_dir_all_if_exists(report_dir);
        let bundle_dir = report_dir.join("linked-run");
        write_sample_replay_bundle(&bundle_dir);

        let report = sample_startup_research_report(bundle_dir.clone());

        write_research_report_bundle(report_dir, &report).unwrap();
    }

    fn sample_startup_research_report(bundle_path: PathBuf) -> ResearchReport {
        ResearchReport::Aggregate(ResearchAggregateReport {
            engine_version: "m13-research-audit".to_string(),
            snapshot_id: "fixture:m13_research_report".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-01-06".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "none".to_string(),
            symbol_count: 1,
            total_row_count: 3,
            total_warning_count: 1,
            total_trade_count: 1,
            starting_equity_total: "+1000.0000".to_string(),
            ending_equity_total: "+989.8000".to_string(),
            net_equity_change_total: "-10.2000".to_string(),
            average_net_equity_change: "-10.2000".to_string(),
            symbols: vec!["TEST".to_string()],
            members: vec![ResearchAggregateMember {
                symbol: "TEST".to_string(),
                bundle_path,
                row_count: 3,
                warning_count: 1,
                trade_count: 1,
                starting_equity: "+1000.0000".to_string(),
                ending_equity: "+989.8000".to_string(),
                net_equity_change: "-10.2000".to_string(),
            }],
        })
    }

    fn sample_multi_member_aggregate_report(bundle_path: PathBuf) -> ResearchReport {
        ResearchReport::Aggregate(sample_aggregate_report_model(bundle_path))
    }

    fn sample_walk_forward_report(bundle_path: PathBuf) -> ResearchReport {
        ResearchReport::WalkForward(sample_walk_forward_report_model(bundle_path))
    }

    fn sample_bootstrap_aggregate_report(bundle_path: PathBuf) -> ResearchReport {
        ResearchReport::BootstrapAggregate(ResearchBootstrapAggregateReport {
            baseline: sample_aggregate_report_model(bundle_path),
            distribution: sample_distribution_summary("aggregate_net_equity_change"),
        })
    }

    fn sample_bootstrap_walk_forward_report(bundle_path: PathBuf) -> ResearchReport {
        let sibling_bundle = sibling_bundle_path(&bundle_path, "bootstrap-split-2");
        ResearchReport::BootstrapWalkForward(ResearchBootstrapWalkForwardReport {
            baseline: sample_walk_forward_report_model(bundle_path.clone()),
            distribution: sample_distribution_summary("walk_forward_test_net_equity_change"),
            splits: vec![
                ResearchBootstrapWalkForwardSplit {
                    sequence: 1,
                    train_row_range: "0..19".to_string(),
                    train_date_range: "2025-01-02..2025-01-31".to_string(),
                    test_row_range: "20..29".to_string(),
                    test_date_range: "2025-02-03..2025-02-14".to_string(),
                    baseline_test_total_net_equity_change: "+14.5000".to_string(),
                    baseline_test_average_net_equity_change: "+7.2500".to_string(),
                    children: vec![
                        ResearchWalkForwardSplitChild {
                            symbol: "TEST".to_string(),
                            bundle_path: bundle_path.clone(),
                        },
                        ResearchWalkForwardSplitChild {
                            symbol: "MOCK".to_string(),
                            bundle_path: sibling_bundle.clone(),
                        },
                    ],
                },
                ResearchBootstrapWalkForwardSplit {
                    sequence: 2,
                    train_row_range: "10..29".to_string(),
                    train_date_range: "2025-01-16..2025-02-14".to_string(),
                    test_row_range: "30..39".to_string(),
                    test_date_range: "2025-02-17..2025-02-28".to_string(),
                    baseline_test_total_net_equity_change: "+9.1000".to_string(),
                    baseline_test_average_net_equity_change: "+4.5500".to_string(),
                    children: vec![
                        ResearchWalkForwardSplitChild {
                            symbol: "TEST".to_string(),
                            bundle_path,
                        },
                        ResearchWalkForwardSplitChild {
                            symbol: "MOCK".to_string(),
                            bundle_path: sibling_bundle,
                        },
                    ],
                },
            ],
        })
    }

    fn sample_leaderboard_report(bundle_path: PathBuf) -> ResearchReport {
        let second_bundle = sibling_bundle_path(&bundle_path, "leaderboard-row-2");
        ResearchReport::Leaderboard(ResearchLeaderboardReport {
            view: LeaderboardView::Signal,
            engine_version: "m13-research-audit".to_string(),
            snapshot_id: "fixture:m13_leaderboard".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-02-28".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "single-symbol reference flow only".to_string(),
            symbol_count: 2,
            symbols: vec!["TEST".to_string(), "MOCK".to_string()],
            fixed_signal_id: None,
            fixed_filter_id: Some("pass_through".to_string()),
            fixed_position_manager_id: Some("keep_position".to_string()),
            fixed_execution_model_id: Some("next_open_long".to_string()),
            rows: vec![
                ResearchLeaderboardRow {
                    rank: 1,
                    label: "close_confirmed_breakout".to_string(),
                    signal_id: "close_confirmed_breakout".to_string(),
                    filter_id: "pass_through".to_string(),
                    position_manager_id: "keep_position".to_string(),
                    execution_model_id: "next_open_long".to_string(),
                    aggregate: sample_aggregate_report_model(bundle_path.clone()),
                },
                ResearchLeaderboardRow {
                    rank: 2,
                    label: "stop_entry_breakout".to_string(),
                    signal_id: "stop_entry_breakout".to_string(),
                    filter_id: "pass_through".to_string(),
                    position_manager_id: "keep_position".to_string(),
                    execution_model_id: "next_open_long".to_string(),
                    aggregate: sample_aggregate_report_model(second_bundle),
                },
            ],
        })
    }

    fn sample_aggregate_report_model(bundle_path: PathBuf) -> ResearchAggregateReport {
        let second_bundle = sibling_bundle_path(&bundle_path, "aggregate-member-2");
        ResearchAggregateReport {
            engine_version: "m13-research-audit".to_string(),
            snapshot_id: "fixture:m13_research_report".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-02-28".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "single-symbol reference flow only".to_string(),
            symbol_count: 2,
            total_row_count: 6,
            total_warning_count: 1,
            total_trade_count: 2,
            starting_equity_total: "+2000.0000".to_string(),
            ending_equity_total: "+2015.5000".to_string(),
            net_equity_change_total: "+15.5000".to_string(),
            average_net_equity_change: "+7.7500".to_string(),
            symbols: vec!["TEST".to_string(), "MOCK".to_string()],
            members: vec![
                ResearchAggregateMember {
                    symbol: "TEST".to_string(),
                    bundle_path,
                    row_count: 3,
                    warning_count: 1,
                    trade_count: 1,
                    starting_equity: "+1000.0000".to_string(),
                    ending_equity: "+1012.3000".to_string(),
                    net_equity_change: "+12.3000".to_string(),
                },
                ResearchAggregateMember {
                    symbol: "MOCK".to_string(),
                    bundle_path: second_bundle,
                    row_count: 3,
                    warning_count: 0,
                    trade_count: 1,
                    starting_equity: "+1000.0000".to_string(),
                    ending_equity: "+1003.2000".to_string(),
                    net_equity_change: "+3.2000".to_string(),
                },
            ],
        }
    }

    fn sample_walk_forward_report_model(bundle_path: PathBuf) -> ResearchWalkForwardReport {
        let second_bundle = sibling_bundle_path(&bundle_path, "walk-forward-child-2");
        ResearchWalkForwardReport {
            engine_version: "m13-research-audit".to_string(),
            snapshot_id: "fixture:m13_walk_forward".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-02-28".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "single-symbol reference flow only".to_string(),
            symbols: vec!["TEST".to_string(), "MOCK".to_string()],
            train_bars: 20,
            test_bars: 10,
            step_bars: 10,
            split_count: 2,
            splits: vec![
                ResearchWalkForwardSplit {
                    sequence: 1,
                    train_start_index: 0,
                    train_end_index: 19,
                    test_start_index: 20,
                    test_end_index: 29,
                    train_row_range: "0..19".to_string(),
                    train_date_range: "2025-01-02..2025-01-31".to_string(),
                    test_row_range: "20..29".to_string(),
                    test_date_range: "2025-02-03..2025-02-14".to_string(),
                    children: vec![
                        ResearchWalkForwardSplitChild {
                            symbol: "TEST".to_string(),
                            bundle_path: bundle_path.clone(),
                        },
                        ResearchWalkForwardSplitChild {
                            symbol: "MOCK".to_string(),
                            bundle_path: second_bundle.clone(),
                        },
                    ],
                },
                ResearchWalkForwardSplit {
                    sequence: 2,
                    train_start_index: 10,
                    train_end_index: 29,
                    test_start_index: 30,
                    test_end_index: 39,
                    train_row_range: "10..29".to_string(),
                    train_date_range: "2025-01-16..2025-02-14".to_string(),
                    test_row_range: "30..39".to_string(),
                    test_date_range: "2025-02-17..2025-02-28".to_string(),
                    children: vec![
                        ResearchWalkForwardSplitChild {
                            symbol: "TEST".to_string(),
                            bundle_path,
                        },
                        ResearchWalkForwardSplitChild {
                            symbol: "MOCK".to_string(),
                            bundle_path: second_bundle,
                        },
                    ],
                },
            ],
        }
    }

    fn sample_distribution_summary(metric: &str) -> BootstrapDistributionSummary {
        BootstrapDistributionSummary {
            seed: 7,
            sample_count: 64,
            resample_size: 8,
            metric: metric.to_string(),
            baseline_metric: "+12.3000".to_string(),
            bootstrap_mean: "+11.9000".to_string(),
            bootstrap_median: "+11.5000".to_string(),
            bootstrap_min: "-4.2000".to_string(),
            bootstrap_max: "+20.1000".to_string(),
            bootstrap_interval_95_lower: "-1.8000".to_string(),
            bootstrap_interval_95_upper: "+18.7000".to_string(),
        }
    }

    fn sibling_bundle_path(bundle_path: &Path, leaf: &str) -> PathBuf {
        bundle_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(leaf)
    }

    fn write_sample_snapshot_bundle(snapshot_dir: &Path) {
        use trendlab_data::snapshot::{
            PersistedSnapshotBundle, SnapshotBundleDescriptor, SnapshotCaptureMetadata,
            SnapshotRequestedWindow,
        };
        use trendlab_data::snapshot_store::write_snapshot_bundle;

        let stored = sample_stored_snapshot_symbol();
        let descriptor = SnapshotBundleDescriptor::from_stored_symbols(
            stored.metadata.snapshot_id.clone(),
            stored.metadata.provider_identity,
            SnapshotRequestedWindow {
                start_date: "2025-01-02".to_string(),
                end_date: "2025-01-10".to_string(),
            },
            SnapshotCaptureMetadata {
                capture_mode: "live_provider_fetch".to_string(),
                entrypoint: "cargo xtask capture-live-snapshot".to_string(),
                captured_at_unix_epoch_seconds: Some(1_736_400_000),
            },
            std::slice::from_ref(&stored),
        )
        .unwrap();

        let bundle = PersistedSnapshotBundle {
            descriptor,
            symbols: vec![stored],
        };

        remove_dir_all_if_exists(snapshot_dir);
        write_snapshot_bundle(snapshot_dir, &bundle).unwrap();
    }

    fn write_sample_replay_bundle(bundle_dir: &Path) {
        let sample = sample_app();
        let inspect = inspect_app(&sample);

        remove_dir_all_if_exists(bundle_dir);
        write_replay_bundle(
            bundle_dir,
            &inspect.bundle.manifest,
            &inspect.bundle.summary,
            &inspect.bundle.ledger,
        )
        .unwrap();
    }

    fn sample_stored_snapshot_symbol() -> trendlab_data::snapshot::StoredSymbolData {
        use trendlab_data::provider::ProviderIdentity;
        use trendlab_data::snapshot::{
            CorporateAction, RawDailyBar, SnapshotMetadata, StoredSymbolData,
        };

        StoredSymbolData {
            metadata: SnapshotMetadata {
                schema_version: trendlab_data::SNAPSHOT_SCHEMA_VERSION,
                snapshot_id: "live:tiingo:TEST:2025-01-03:2025-01-08".to_string(),
                provider_identity: ProviderIdentity::Tiingo,
            },
            symbol: "TEST".to_string(),
            raw_bars: vec![
                RawDailyBar {
                    symbol: "TEST".to_string(),
                    date: "2025-01-02".to_string(),
                    raw_open: 100.0,
                    raw_high: 103.0,
                    raw_low: 99.0,
                    raw_close: 102.0,
                    volume: 1_000,
                },
                RawDailyBar {
                    symbol: "TEST".to_string(),
                    date: "2025-01-03".to_string(),
                    raw_open: 104.0,
                    raw_high: 105.0,
                    raw_low: 101.0,
                    raw_close: 104.0,
                    volume: 1_100,
                },
                RawDailyBar {
                    symbol: "TEST".to_string(),
                    date: "2025-01-06".to_string(),
                    raw_open: 52.0,
                    raw_high: 53.0,
                    raw_low: 50.0,
                    raw_close: 51.0,
                    volume: 2_200,
                },
                RawDailyBar {
                    symbol: "TEST".to_string(),
                    date: "2025-01-07".to_string(),
                    raw_open: 52.5,
                    raw_high: 54.0,
                    raw_low: 52.0,
                    raw_close: 53.0,
                    volume: 2_100,
                },
            ],
            corporate_actions: vec![
                CorporateAction::Split {
                    symbol: "TEST".to_string(),
                    ex_date: "2025-01-06".to_string(),
                    ratio: 2.0,
                },
                CorporateAction::CashDividend {
                    symbol: "TEST".to_string(),
                    ex_date: "2025-01-07".to_string(),
                    cash_amount: 0.25,
                },
            ],
        }
    }

    fn test_output_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("trendlab-tui lives under crates/");
        workspace_root
            .join("target")
            .join("test-output")
            .join(format!(
                "{label}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ))
    }

    fn remove_dir_all_if_exists(path: &Path) {
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
    }

    fn focus_run_form(app: &mut App) {
        while app.home_focus != HomeFocusPane::RunForm {
            app.apply(AppCommand::NextFocus);
        }
    }

    fn set_run_form_field(app: &mut App, target: RunFormField) {
        while app.run_form.selected_field() != target {
            app.apply(AppCommand::MoveDown);
        }
    }

    fn configure_non_default_run_form(app: &mut App) {
        focus_run_form(app);

        set_run_form_field(app, RunFormField::StartDate);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::EndDate);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::SignalDate);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::InitialCash);
        app.apply(AppCommand::AdjustNext);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::EntryShares);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::ProtectiveStopFraction);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::CommissionPerFill);
        app.apply(AppCommand::AdjustNext);

        set_run_form_field(app, RunFormField::SlippagePerShare);
        app.apply(AppCommand::AdjustNext);
    }

    fn buffer_text(buffer: &Buffer) -> String {
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }
}
