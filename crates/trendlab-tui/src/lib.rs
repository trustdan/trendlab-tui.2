#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::Marker;
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph, Wrap,
};
use trendlab_artifact::{PersistedLedgerRow, ReplayBundle, load_replay_bundle};
use trendlab_data::audit::{DataAuditReport, audit_daily_bars};

const APP_TITLE: &str = "TrendLab TUI";
const USAGE: &str = "usage: trendlab-tui open <bundle-dir>";

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
    MoveUp,
    MoveDown,
    ToggleHelp,
    Quit,
}

impl AppCommand {
    fn from_key_event(key: &KeyEvent) -> Option<Self> {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return None;
        }

        match key.code {
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Some(Self::NextFocus),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => Some(Self::PreviousFocus),
            KeyCode::Up | KeyCode::Char('k') => Some(Self::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Self::MoveDown),
            KeyCode::Char('?') => Some(Self::ToggleHelp),
            KeyCode::Esc | KeyCode::Char('q') => Some(Self::Quit),
            _ => None,
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
struct App {
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

impl App {
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

pub fn run_from_args<I, S>(args: I) -> Result<(), TuiError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let bundle_path = parse_bundle_path(args)?;
    run_bundle_viewer(&bundle_path)
}

fn parse_bundle_path(args: Vec<String>) -> Result<PathBuf, TuiError> {
    match args.as_slice() {
        [bundle_dir] => Ok(PathBuf::from(bundle_dir)),
        [command, bundle_dir] if command == "open" => Ok(PathBuf::from(bundle_dir)),
        _ => Err(TuiError::invalid(USAGE)),
    }
}

fn run_bundle_viewer(bundle_path: &Path) -> Result<(), TuiError> {
    let bundle =
        load_replay_bundle(bundle_path).map_err(|err| TuiError::invalid(err.to_string()))?;
    let mut app = App::from_bundle(bundle_path.to_path_buf(), bundle);
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
    let help_height = if app.help_expanded { 7 } else { 3 };
    let layout = Layout::vertical([
        Constraint::Length(4),
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

    render_header(frame, layout[0], app);
    render_results(frame, body[0], app);
    render_chart(frame, center[0], app);
    render_ledger(frame, center[1], app);
    render_audit(frame, body[2], app);
    render_help(frame, layout[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                APP_TITLE,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("bundle: {}", app.bundle_path.display())),
        ]),
        Line::from(format!(
            "symbol={} snapshot={} provider={} focus={}",
            app.bundle.manifest.symbol_or_universe,
            app.bundle.manifest.data_snapshot_id,
            app.bundle.manifest.provider_identity,
            app.focus.label()
        )),
    ]);

    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Run"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
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

fn render_chart(frame: &mut Frame, area: Rect, app: &App) {
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

fn render_ledger(frame: &mut Frame, area: Rect, app: &App) {
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

fn render_audit(frame: &mut Frame, area: Rect, app: &App) {
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

fn render_help(frame: &mut Frame, area: Rect, app: &App) {
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

fn build_audit_lines(app: &App) -> Vec<String> {
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
        "Inspect now mixes run-level and per-trade checkpoints instead of only bundle-summary fields.".to_string(),
        "Chart and Ledger share the selected bar so price movement and persisted reasoning stay aligned.".to_string(),
        "Audit keeps provenance, warnings, data-audit summary, and selected-row reasoning visible across panes.".to_string(),
        "Keys: tab/shift-tab or h/l switch focus, j/k or arrows move selection, ? collapses help, q quits.".to_string(),
    ]
}

fn chart_title(app: &App) -> String {
    app.selected_ledger_row()
        .map(|row| {
            format!(
                "selected={} raw_close={:.4} analysis={:.4}",
                row.date, row.raw_close, row.analysis_close
            )
        })
        .unwrap_or_else(|| "selected=none".to_string())
}

fn chart_x_axis_labels(app: &App) -> Vec<Span<'static>> {
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

fn format_reason_codes(reason_codes: &[String]) -> String {
    if reason_codes.is_empty() {
        "none".to_string()
    } else {
        reason_codes.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use trendlab_artifact::{
        BundleDescriptor, DateRange, ManifestParameter, ReferenceFlowDefinition, RunManifest,
        RunSummary, SCHEMA_VERSION,
    };
    use trendlab_core::accounting::CostModel;
    use trendlab_core::orders::GapPolicy;

    use super::{App, AppCommand, FocusPane, render};

    #[test]
    fn app_surfaces_bundle_provenance_and_audit_summary() {
        let app = sample_app();

        assert_eq!(app.focus, FocusPane::Results);
        assert_eq!(app.results.len(), 6);
        assert_eq!(app.results[0].label, "run");
        assert_eq!(app.results[1].label, "trade 1");
        assert_eq!(app.results[1].value, "closed -10.2000");
        assert_eq!(app.results[4].label, "data audit");
        assert_eq!(app.results[5].label, "artifact");
        assert_eq!(app.audit_report.analysis_adjusted_bar_count, 1);
    }

    #[test]
    fn navigation_is_scoped_by_the_focused_pane() {
        let mut app = sample_app();

        app.apply(AppCommand::MoveDown);
        assert_eq!(app.selected_result, 1);
        assert_eq!(app.selected_ledger, 1);

        app.apply(AppCommand::NextFocus);
        assert_eq!(app.focus, FocusPane::Chart);
        app.apply(AppCommand::MoveDown);
        assert_eq!(app.selected_result, 1);
        assert_eq!(app.selected_ledger, 2);

        app.apply(AppCommand::NextFocus);
        assert_eq!(app.focus, FocusPane::Ledger);
        app.apply(AppCommand::MoveDown);
        assert_eq!(app.selected_result, 1);
        assert_eq!(app.selected_ledger, 0);

        app.apply(AppCommand::NextFocus);
        assert_eq!(app.focus, FocusPane::Help);
        app.apply(AppCommand::MoveDown);
        assert_eq!(app.selected_ledger, 0);
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
        assert!(screen.contains("entry_filled_at_open"));
        assert!(screen.contains("Selected Row"));
    }

    fn sample_app() -> App {
        App::from_bundle(
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
        )
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
