#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use trendlab_core::accounting::CostModel;
use trendlab_core::ledger::LedgerRow;
use trendlab_core::market::DailyBar;
use trendlab_core::orders::GapPolicy;

pub const SCHEMA_VERSION: u32 = 1;
pub const BUNDLE_FILE_NAME: &str = "bundle.json";
pub const MANIFEST_FILE_NAME: &str = "manifest.json";
pub const SUMMARY_FILE_NAME: &str = "summary.json";
pub const LEDGER_FILE_NAME: &str = "ledger.jsonl";
pub const RESEARCH_REPORT_FILE_NAME: &str = "research.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleDescriptor {
    pub schema_version: u32,
    pub manifest_path: String,
    pub summary_path: String,
    pub ledger_path: String,
    #[serde(default)]
    pub integrity: Option<ReplayBundleIntegrity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentFingerprint {
    pub byte_count: usize,
    pub fnv1a64: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBundleIntegrity {
    pub manifest: ContentFingerprint,
    pub summary: ContentFingerprint,
    pub ledger: ContentFingerprint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestParameter {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceFlowDefinition {
    pub kind: String,
    pub entry_shares: u32,
    pub protective_stop_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: u32,
    pub engine_version: String,
    pub data_snapshot_id: String,
    pub provider_identity: String,
    pub symbol_or_universe: String,
    pub universe_mode: String,
    pub historical_limitations: Vec<String>,
    pub date_range: DateRange,
    pub reference_flow: ReferenceFlowDefinition,
    pub parameters: Vec<ManifestParameter>,
    pub cost_model: CostModel,
    pub gap_policy: GapPolicy,
    pub seed: Option<u64>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub row_count: usize,
    pub warning_count: usize,
    pub ending_cash: f64,
    pub ending_equity: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PersistedLedgerRow {
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

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayBundle {
    pub descriptor: BundleDescriptor,
    pub manifest: RunManifest,
    pub summary: RunSummary,
    pub ledger: Vec<PersistedLedgerRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchAggregateMember {
    pub symbol: String,
    pub bundle_path: PathBuf,
    pub row_count: usize,
    pub warning_count: usize,
    pub trade_count: usize,
    pub starting_equity: String,
    pub ending_equity: String,
    pub net_equity_change: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchAggregateReport {
    pub engine_version: String,
    pub snapshot_id: String,
    pub provider_identity: String,
    pub date_range: String,
    pub gap_policy: String,
    pub historical_limitations: String,
    pub symbol_count: usize,
    pub total_row_count: usize,
    pub total_warning_count: usize,
    pub total_trade_count: usize,
    pub starting_equity_total: String,
    pub ending_equity_total: String,
    pub net_equity_change_total: String,
    pub average_net_equity_change: String,
    pub symbols: Vec<String>,
    pub members: Vec<ResearchAggregateMember>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchWalkForwardSplitChild {
    pub symbol: String,
    pub bundle_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchWalkForwardSplit {
    pub sequence: usize,
    pub train_start_index: usize,
    pub train_end_index: usize,
    pub test_start_index: usize,
    pub test_end_index: usize,
    pub train_row_range: String,
    pub train_date_range: String,
    pub test_row_range: String,
    pub test_date_range: String,
    pub children: Vec<ResearchWalkForwardSplitChild>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchWalkForwardReport {
    pub engine_version: String,
    pub snapshot_id: String,
    pub provider_identity: String,
    pub date_range: String,
    pub gap_policy: String,
    pub historical_limitations: String,
    pub symbols: Vec<String>,
    pub train_bars: usize,
    pub test_bars: usize,
    pub step_bars: usize,
    pub split_count: usize,
    pub splits: Vec<ResearchWalkForwardSplit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapDistributionSummary {
    pub seed: u64,
    pub sample_count: usize,
    pub resample_size: usize,
    pub metric: String,
    pub baseline_metric: String,
    pub bootstrap_mean: String,
    pub bootstrap_median: String,
    pub bootstrap_min: String,
    pub bootstrap_max: String,
    pub bootstrap_interval_95_lower: String,
    pub bootstrap_interval_95_upper: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchBootstrapAggregateReport {
    pub baseline: ResearchAggregateReport,
    pub distribution: BootstrapDistributionSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchBootstrapWalkForwardSplit {
    pub sequence: usize,
    pub train_row_range: String,
    pub train_date_range: String,
    pub test_row_range: String,
    pub test_date_range: String,
    pub baseline_test_total_net_equity_change: String,
    pub baseline_test_average_net_equity_change: String,
    pub children: Vec<ResearchWalkForwardSplitChild>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchBootstrapWalkForwardReport {
    pub baseline: ResearchWalkForwardReport,
    pub distribution: BootstrapDistributionSummary,
    pub splits: Vec<ResearchBootstrapWalkForwardSplit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LeaderboardView {
    Signal,
    PositionManager,
    ExecutionModel,
    System,
}

impl LeaderboardView {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Signal => "signal",
            Self::PositionManager => "position-manager",
            Self::ExecutionModel => "execution-model",
            Self::System => "system",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "signal" => Some(Self::Signal),
            "position-manager" => Some(Self::PositionManager),
            "execution-model" => Some(Self::ExecutionModel),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchLeaderboardRow {
    pub rank: usize,
    pub label: String,
    pub signal_id: String,
    pub filter_id: String,
    pub position_manager_id: String,
    pub execution_model_id: String,
    pub aggregate: ResearchAggregateReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchLeaderboardReport {
    pub view: LeaderboardView,
    pub engine_version: String,
    pub snapshot_id: String,
    pub provider_identity: String,
    pub date_range: String,
    pub gap_policy: String,
    pub historical_limitations: String,
    pub symbol_count: usize,
    pub symbols: Vec<String>,
    pub fixed_signal_id: Option<String>,
    pub fixed_filter_id: Option<String>,
    pub fixed_position_manager_id: Option<String>,
    pub fixed_execution_model_id: Option<String>,
    pub rows: Vec<ResearchLeaderboardRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "report", rename_all = "snake_case")]
pub enum ResearchReport {
    Aggregate(ResearchAggregateReport),
    WalkForward(ResearchWalkForwardReport),
    BootstrapAggregate(ResearchBootstrapAggregateReport),
    BootstrapWalkForward(ResearchBootstrapWalkForwardReport),
    Leaderboard(ResearchLeaderboardReport),
}

impl ResearchReport {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Aggregate(_) => "aggregate",
            Self::WalkForward(_) => "walk_forward",
            Self::BootstrapAggregate(_) => "bootstrap_aggregate",
            Self::BootstrapWalkForward(_) => "bootstrap_walk_forward",
            Self::Leaderboard(_) => "leaderboard",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct StoredResearchReport {
    schema_version: u32,
    #[serde(default)]
    linked_replay_bundles: Vec<StoredResearchBundleLink>,
    report: ResearchReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct StoredResearchBundleLink {
    path: PathBuf,
    integrity: ReplayBundleIntegrity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueDiff {
    pub field: String,
    pub left: String,
    pub right: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerRowDiff {
    pub index: usize,
    pub left_date: Option<String>,
    pub right_date: Option<String>,
    pub field_diffs: Vec<ValueDiff>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplayBundleDiff {
    pub manifest_diffs: Vec<ValueDiff>,
    pub summary_diffs: Vec<ValueDiff>,
    pub ledger_row_diffs: Vec<LedgerRowDiff>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactError {
    message: String,
}

impl ArtifactError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn io(action: &str, path: &Path, err: &std::io::Error) -> Self {
        Self::invalid(format!("{action} {}: {err}", path.display()))
    }

    fn json(action: &str, path: &Path, err: &serde_json::Error) -> Self {
        Self::invalid(format!("{action} {}: {err}", path.display()))
    }
}

impl Display for ArtifactError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ArtifactError {}

impl BundleDescriptor {
    pub fn canonical() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            manifest_path: MANIFEST_FILE_NAME.to_string(),
            summary_path: SUMMARY_FILE_NAME.to_string(),
            ledger_path: LEDGER_FILE_NAME.to_string(),
            integrity: None,
        }
    }

    fn canonical_with_integrity(integrity: ReplayBundleIntegrity) -> Self {
        Self {
            integrity: Some(integrity),
            ..Self::canonical()
        }
    }
}

impl From<&LedgerRow> for PersistedLedgerRow {
    fn from(value: &LedgerRow) -> Self {
        Self {
            date: value.date.clone(),
            raw_open: value.raw_open,
            raw_high: value.raw_high,
            raw_low: value.raw_low,
            raw_close: value.raw_close,
            analysis_close: value.analysis_close,
            position_shares: value.position_shares,
            signal_output: value.signal_output.clone(),
            filter_outcome: value.filter_outcome.clone(),
            pending_order_state: value.pending_order_state.clone(),
            fill_price: value.fill_price,
            prior_stop: value.prior_stop,
            next_stop: value.next_stop,
            cash: value.cash,
            equity: value.equity,
            reason_codes: value.reason_codes.clone(),
        }
    }
}

impl PersistedLedgerRow {
    pub fn market_bar(&self) -> DailyBar {
        DailyBar {
            date: self.date.clone(),
            raw_open: self.raw_open,
            raw_high: self.raw_high,
            raw_low: self.raw_low,
            raw_close: self.raw_close,
            analysis_close: self.analysis_close,
        }
    }
}

impl ReplayBundleDiff {
    pub fn is_empty(&self) -> bool {
        self.manifest_diffs.is_empty()
            && self.summary_diffs.is_empty()
            && self.ledger_row_diffs.is_empty()
    }
}

pub fn write_replay_bundle(
    bundle_dir: &Path,
    manifest: &RunManifest,
    summary: &RunSummary,
    ledger: &[PersistedLedgerRow],
) -> Result<BundleDescriptor, ArtifactError> {
    validate_replay_bundle_parts(manifest, summary, ledger)?;

    let descriptor = BundleDescriptor::canonical_with_integrity(compute_replay_bundle_integrity(
        manifest, summary, ledger,
    )?);

    fs::create_dir_all(bundle_dir)
        .map_err(|err| ArtifactError::io("failed to create", bundle_dir, &err))?;

    write_json_pretty(
        &bundle_dir.join(&descriptor.manifest_path),
        manifest,
        "failed to write",
    )?;
    write_json_pretty(
        &bundle_dir.join(&descriptor.summary_path),
        summary,
        "failed to write",
    )?;
    write_json_lines(
        &bundle_dir.join(&descriptor.ledger_path),
        ledger,
        "failed to write",
    )?;
    write_json_pretty(
        &bundle_dir.join(BUNDLE_FILE_NAME),
        &descriptor,
        "failed to write",
    )?;

    Ok(descriptor)
}

pub fn load_replay_bundle(bundle_dir: &Path) -> Result<ReplayBundle, ArtifactError> {
    let descriptor: BundleDescriptor =
        read_json(&bundle_dir.join(BUNDLE_FILE_NAME), "failed to read")?;

    if descriptor.schema_version != SCHEMA_VERSION {
        return Err(ArtifactError::invalid(format!(
            "unsupported bundle schema version {}; expected {}",
            descriptor.schema_version, SCHEMA_VERSION
        )));
    }

    let manifest_path =
        resolve_bundle_path(bundle_dir, &descriptor.manifest_path, "manifest_path")?;
    let summary_path = resolve_bundle_path(bundle_dir, &descriptor.summary_path, "summary_path")?;
    let ledger_path = resolve_bundle_path(bundle_dir, &descriptor.ledger_path, "ledger_path")?;

    let manifest: RunManifest = read_json(&manifest_path, "failed to read")?;
    if manifest.schema_version != descriptor.schema_version {
        return Err(ArtifactError::invalid(format!(
            "manifest schema version {} does not match bundle schema version {}",
            manifest.schema_version, descriptor.schema_version
        )));
    }

    let summary: RunSummary = read_json(&summary_path, "failed to read")?;
    let ledger: Vec<PersistedLedgerRow> = read_json_lines(&ledger_path, "failed to read")?;
    if let Some(expected_integrity) = &descriptor.integrity {
        let actual_integrity = compute_replay_bundle_integrity(&manifest, &summary, &ledger)?;
        validate_replay_bundle_integrity(
            expected_integrity,
            &actual_integrity,
            &bundle_dir.join(BUNDLE_FILE_NAME),
        )?;
    }
    validate_replay_bundle_parts(&manifest, &summary, &ledger)?;

    Ok(ReplayBundle {
        descriptor,
        manifest,
        summary,
        ledger,
    })
}

pub fn write_research_report_bundle(
    report_dir: &Path,
    report: &ResearchReport,
) -> Result<(), ArtifactError> {
    validate_research_report(report)?;

    fs::create_dir_all(report_dir)
        .map_err(|err| ArtifactError::io("failed to create", report_dir, &err))?;
    let normalized_report_dir = normalize_external_path(report_dir)?;
    let normalized_report = map_research_report_bundle_paths(report, &mut |bundle_path| {
        let normalized_bundle_path = normalize_external_path(bundle_path)?;
        Ok(
            relative_external_path(&normalized_report_dir, &normalized_bundle_path)
                .unwrap_or(normalized_bundle_path),
        )
    })?;
    let linked_replay_bundles = collect_research_report_bundle_paths(&normalized_report)
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|stored_path| {
            let resolved_path = resolve_research_bundle_path(&normalized_report_dir, &stored_path);
            let bundle = load_replay_bundle(&resolved_path).map_err(|err| {
                ArtifactError::invalid(format!(
                    "research report requires replay bundle {}: {err}",
                    resolved_path.display()
                ))
            })?;
            Ok(StoredResearchBundleLink {
                path: stored_path,
                integrity: compute_replay_bundle_integrity(
                    &bundle.manifest,
                    &bundle.summary,
                    &bundle.ledger,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let stored = StoredResearchReport {
        schema_version: SCHEMA_VERSION,
        linked_replay_bundles,
        report: normalized_report,
    };

    write_json_pretty(
        &report_dir.join(RESEARCH_REPORT_FILE_NAME),
        &stored,
        "failed to write",
    )
}

pub fn load_research_report_bundle(report_dir: &Path) -> Result<ResearchReport, ArtifactError> {
    let path = report_dir.join(RESEARCH_REPORT_FILE_NAME);
    let stored: StoredResearchReport = read_json(&path, "failed to read")?;

    if stored.schema_version != SCHEMA_VERSION {
        return Err(ArtifactError::invalid(format!(
            "unsupported research report schema version {}; expected {}",
            stored.schema_version, SCHEMA_VERSION
        )));
    }

    let normalized_report_dir = normalize_external_path(report_dir)?;
    validate_research_report_bundle_links(&stored, &path)?;
    let report = map_research_report_bundle_paths(&stored.report, &mut |bundle_path| {
        let resolved_path = resolve_research_bundle_path(&normalized_report_dir, bundle_path);
        if let Some(expected_integrity) = stored
            .linked_replay_bundles
            .iter()
            .find(|link| link.path == bundle_path)
            .map(|link| &link.integrity)
        {
            let bundle = load_replay_bundle(&resolved_path).map_err(|err| {
                ArtifactError::invalid(format!(
                    "research report requires replay bundle {}: {err}",
                    resolved_path.display()
                ))
            })?;
            let actual_integrity =
                compute_replay_bundle_integrity(&bundle.manifest, &bundle.summary, &bundle.ledger)?;
            validate_replay_bundle_integrity(
                expected_integrity,
                &actual_integrity,
                &path,
            )
            .map_err(|_| {
                ArtifactError::invalid(format!(
                    "research report linked replay bundle {} no longer matches stored integrity metadata",
                    resolved_path.display()
                ))
            })?;
        }
        Ok(resolved_path)
    })?;

    validate_research_report(&report)?;

    Ok(report)
}

pub fn validate_research_report(report: &ResearchReport) -> Result<(), ArtifactError> {
    match report {
        ResearchReport::Aggregate(report) => {
            validate_aggregate_report(report, "research aggregate")
        }
        ResearchReport::WalkForward(report) => {
            validate_walk_forward_report(report, "research walk-forward")
        }
        ResearchReport::BootstrapAggregate(report) => {
            validate_bootstrap_aggregate_report(report, "research bootstrap aggregate")
        }
        ResearchReport::BootstrapWalkForward(report) => {
            validate_bootstrap_walk_forward_report(report, "research bootstrap walk-forward")
        }
        ResearchReport::Leaderboard(report) => {
            validate_leaderboard_report(report, "research leaderboard")
        }
    }
}

fn validate_aggregate_report(
    report: &ResearchAggregateReport,
    context: &str,
) -> Result<(), ArtifactError> {
    validate_common_report_fields(
        context,
        &report.engine_version,
        &report.snapshot_id,
        &report.provider_identity,
        &report.date_range,
        &report.gap_policy,
        &report.historical_limitations,
    )?;

    if report.members.is_empty() {
        return Err(ArtifactError::invalid(format!(
            "{context} must contain at least one member"
        )));
    }

    if report.symbol_count != report.members.len() {
        return Err(ArtifactError::invalid(format!(
            "{context} symbol_count {} does not match member count {}",
            report.symbol_count,
            report.members.len()
        )));
    }

    if report.symbol_count != report.symbols.len() {
        return Err(ArtifactError::invalid(format!(
            "{context} symbol_count {} does not match symbols length {}",
            report.symbol_count,
            report.symbols.len()
        )));
    }

    let mut seen_symbols = BTreeSet::new();
    let mut total_starting_equity = 0.0_f64;
    let mut total_ending_equity = 0.0_f64;
    let mut total_trade_count = 0_usize;
    let mut total_warning_count = 0_usize;
    let mut total_row_count = 0_usize;

    for (index, member) in report.members.iter().enumerate() {
        validate_non_empty_text(&format!("{context} member[{index}].symbol"), &member.symbol)?;
        validate_non_empty_path(
            &format!("{context} member[{index}].bundle_path"),
            &member.bundle_path,
        )?;

        if !seen_symbols.insert(member.symbol.as_str()) {
            return Err(ArtifactError::invalid(format!(
                "{context} contains duplicate member symbol `{}`",
                member.symbol
            )));
        }

        if report.symbols.get(index) != Some(&member.symbol) {
            return Err(ArtifactError::invalid(format!(
                "{context} symbols[{index}] does not match member symbol `{}`",
                member.symbol
            )));
        }

        let starting_equity = parse_report_f64(
            &format!("{context} member[{index}].starting_equity"),
            &member.starting_equity,
        )?;
        let ending_equity = parse_report_f64(
            &format!("{context} member[{index}].ending_equity"),
            &member.ending_equity,
        )?;
        let net_equity_change = parse_report_f64(
            &format!("{context} member[{index}].net_equity_change"),
            &member.net_equity_change,
        )?;
        let expected_change = ending_equity - starting_equity;

        if round4(net_equity_change) != round4(expected_change) {
            return Err(ArtifactError::invalid(format!(
                "{context} member[{index}] net_equity_change {} does not match ending minus starting {}",
                member.net_equity_change,
                format_signed_decimal(expected_change)
            )));
        }

        total_starting_equity += starting_equity;
        total_ending_equity += ending_equity;
        total_trade_count += member.trade_count;
        total_warning_count += member.warning_count;
        total_row_count += member.row_count;
    }

    let total_net_equity_change = total_ending_equity - total_starting_equity;
    let average_net_equity_change = total_net_equity_change / report.members.len() as f64;

    validate_report_usize(
        &format!("{context} total_row_count"),
        report.total_row_count,
        total_row_count,
    )?;
    validate_report_usize(
        &format!("{context} total_warning_count"),
        report.total_warning_count,
        total_warning_count,
    )?;
    validate_report_usize(
        &format!("{context} total_trade_count"),
        report.total_trade_count,
        total_trade_count,
    )?;
    validate_report_f64_value(
        &format!("{context} starting_equity_total"),
        &report.starting_equity_total,
        total_starting_equity,
    )?;
    validate_report_f64_value(
        &format!("{context} ending_equity_total"),
        &report.ending_equity_total,
        total_ending_equity,
    )?;
    validate_report_f64_value(
        &format!("{context} net_equity_change_total"),
        &report.net_equity_change_total,
        total_net_equity_change,
    )?;
    validate_report_f64_value(
        &format!("{context} average_net_equity_change"),
        &report.average_net_equity_change,
        average_net_equity_change,
    )?;

    Ok(())
}

fn validate_walk_forward_report(
    report: &ResearchWalkForwardReport,
    context: &str,
) -> Result<(), ArtifactError> {
    validate_common_report_fields(
        context,
        &report.engine_version,
        &report.snapshot_id,
        &report.provider_identity,
        &report.date_range,
        &report.gap_policy,
        &report.historical_limitations,
    )?;

    if report.symbols.is_empty() {
        return Err(ArtifactError::invalid(format!(
            "{context} must contain at least one symbol"
        )));
    }

    let mut seen_symbols = BTreeSet::new();
    for symbol in &report.symbols {
        validate_non_empty_text(&format!("{context} symbol"), symbol)?;
        if !seen_symbols.insert(symbol.as_str()) {
            return Err(ArtifactError::invalid(format!(
                "{context} contains duplicate symbol `{symbol}`"
            )));
        }
    }

    if report.train_bars == 0 || report.test_bars == 0 || report.step_bars == 0 {
        return Err(ArtifactError::invalid(format!(
            "{context} requires train_bars, test_bars, and step_bars to be greater than zero"
        )));
    }

    if report.splits.is_empty() {
        return Err(ArtifactError::invalid(format!(
            "{context} must contain at least one split"
        )));
    }

    validate_report_usize(
        &format!("{context} split_count"),
        report.split_count,
        report.splits.len(),
    )?;

    for (index, split) in report.splits.iter().enumerate() {
        let split_context = format!("{context} split[{}]", index + 1);
        validate_report_usize(
            &format!("{split_context} sequence"),
            split.sequence,
            index + 1,
        )?;

        if split.train_start_index > split.train_end_index {
            return Err(ArtifactError::invalid(format!(
                "{split_context} train_start_index {} must be <= train_end_index {}",
                split.train_start_index, split.train_end_index
            )));
        }

        if split.test_start_index > split.test_end_index {
            return Err(ArtifactError::invalid(format!(
                "{split_context} test_start_index {} must be <= test_end_index {}",
                split.test_start_index, split.test_end_index
            )));
        }

        if split.train_end_index >= split.test_start_index {
            return Err(ArtifactError::invalid(format!(
                "{split_context} train_end_index {} must be < test_start_index {}",
                split.train_end_index, split.test_start_index
            )));
        }

        validate_non_empty_text(
            &format!("{split_context} train_date_range"),
            &split.train_date_range,
        )?;
        validate_non_empty_text(
            &format!("{split_context} test_date_range"),
            &split.test_date_range,
        )?;

        let expected_train_row_range =
            format!("{}..{}", split.train_start_index, split.train_end_index);
        let expected_test_row_range =
            format!("{}..{}", split.test_start_index, split.test_end_index);
        validate_exact_text(
            &format!("{split_context} train_row_range"),
            &split.train_row_range,
            &expected_train_row_range,
        )?;
        validate_exact_text(
            &format!("{split_context} test_row_range"),
            &split.test_row_range,
            &expected_test_row_range,
        )?;

        validate_split_children(&split_context, &report.symbols, &split.children)?;
    }

    Ok(())
}

fn validate_bootstrap_aggregate_report(
    report: &ResearchBootstrapAggregateReport,
    context: &str,
) -> Result<(), ArtifactError> {
    validate_aggregate_report(&report.baseline, &format!("{context} baseline"))?;
    validate_bootstrap_distribution(
        &report.distribution,
        &format!("{context} distribution"),
        report.baseline.members.len(),
    )?;
    Ok(())
}

fn validate_bootstrap_walk_forward_report(
    report: &ResearchBootstrapWalkForwardReport,
    context: &str,
) -> Result<(), ArtifactError> {
    validate_walk_forward_report(&report.baseline, &format!("{context} baseline"))?;
    validate_bootstrap_distribution(
        &report.distribution,
        &format!("{context} distribution"),
        report.baseline.splits.len(),
    )?;

    validate_report_usize(
        &format!("{context} splits"),
        report.splits.len(),
        report.baseline.splits.len(),
    )?;

    for (index, (split, baseline_split)) in report
        .splits
        .iter()
        .zip(report.baseline.splits.iter())
        .enumerate()
    {
        let split_context = format!("{context} split[{}]", index + 1);
        validate_report_usize(
            &format!("{split_context} sequence"),
            split.sequence,
            baseline_split.sequence,
        )?;
        validate_exact_text(
            &format!("{split_context} train_row_range"),
            &split.train_row_range,
            &baseline_split.train_row_range,
        )?;
        validate_exact_text(
            &format!("{split_context} train_date_range"),
            &split.train_date_range,
            &baseline_split.train_date_range,
        )?;
        validate_exact_text(
            &format!("{split_context} test_row_range"),
            &split.test_row_range,
            &baseline_split.test_row_range,
        )?;
        validate_exact_text(
            &format!("{split_context} test_date_range"),
            &split.test_date_range,
            &baseline_split.test_date_range,
        )?;
        validate_split_children(
            &format!("{split_context} children"),
            &report.baseline.symbols,
            &split.children,
        )?;
        validate_exact_children(
            &format!("{split_context} children"),
            &split.children,
            &baseline_split.children,
        )?;
        parse_report_f64(
            &format!("{split_context} baseline_test_total_net_equity_change"),
            &split.baseline_test_total_net_equity_change,
        )?;
        parse_report_f64(
            &format!("{split_context} baseline_test_average_net_equity_change"),
            &split.baseline_test_average_net_equity_change,
        )?;
    }

    Ok(())
}

fn validate_leaderboard_report(
    report: &ResearchLeaderboardReport,
    context: &str,
) -> Result<(), ArtifactError> {
    validate_common_report_fields(
        context,
        &report.engine_version,
        &report.snapshot_id,
        &report.provider_identity,
        &report.date_range,
        &report.gap_policy,
        &report.historical_limitations,
    )?;

    if report.symbol_count != report.symbols.len() {
        return Err(ArtifactError::invalid(format!(
            "{context} symbol_count {} does not match symbols length {}",
            report.symbol_count,
            report.symbols.len()
        )));
    }

    if report.rows.is_empty() {
        return Err(ArtifactError::invalid(format!(
            "{context} must contain at least one row"
        )));
    }

    let mut seen_symbols = BTreeSet::new();
    for symbol in &report.symbols {
        validate_non_empty_text(&format!("{context} symbol"), symbol)?;
        if !seen_symbols.insert(symbol.as_str()) {
            return Err(ArtifactError::invalid(format!(
                "{context} contains duplicate symbol `{symbol}`"
            )));
        }
    }

    match report.view {
        LeaderboardView::Signal => {
            validate_none_text(
                &format!("{context} fixed_signal_id"),
                report.fixed_signal_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_filter_id"),
                report.fixed_filter_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_position_manager_id"),
                report.fixed_position_manager_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_execution_model_id"),
                report.fixed_execution_model_id.as_deref(),
            )?;
        }
        LeaderboardView::PositionManager => {
            validate_some_text(
                &format!("{context} fixed_signal_id"),
                report.fixed_signal_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_filter_id"),
                report.fixed_filter_id.as_deref(),
            )?;
            validate_none_text(
                &format!("{context} fixed_position_manager_id"),
                report.fixed_position_manager_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_execution_model_id"),
                report.fixed_execution_model_id.as_deref(),
            )?;
        }
        LeaderboardView::ExecutionModel => {
            validate_some_text(
                &format!("{context} fixed_signal_id"),
                report.fixed_signal_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_filter_id"),
                report.fixed_filter_id.as_deref(),
            )?;
            validate_some_text(
                &format!("{context} fixed_position_manager_id"),
                report.fixed_position_manager_id.as_deref(),
            )?;
            validate_none_text(
                &format!("{context} fixed_execution_model_id"),
                report.fixed_execution_model_id.as_deref(),
            )?;
        }
        LeaderboardView::System => {
            validate_none_text(
                &format!("{context} fixed_signal_id"),
                report.fixed_signal_id.as_deref(),
            )?;
            validate_none_text(
                &format!("{context} fixed_filter_id"),
                report.fixed_filter_id.as_deref(),
            )?;
            validate_none_text(
                &format!("{context} fixed_position_manager_id"),
                report.fixed_position_manager_id.as_deref(),
            )?;
            validate_none_text(
                &format!("{context} fixed_execution_model_id"),
                report.fixed_execution_model_id.as_deref(),
            )?;
        }
    }

    for (index, row) in report.rows.iter().enumerate() {
        let row_context = format!("{context} row[{}]", index + 1);
        validate_report_usize(&format!("{row_context} rank"), row.rank, index + 1)?;
        validate_non_empty_text(&format!("{row_context} label"), &row.label)?;
        validate_non_empty_text(&format!("{row_context} signal_id"), &row.signal_id)?;
        validate_non_empty_text(&format!("{row_context} filter_id"), &row.filter_id)?;
        validate_non_empty_text(
            &format!("{row_context} position_manager_id"),
            &row.position_manager_id,
        )?;
        validate_non_empty_text(
            &format!("{row_context} execution_model_id"),
            &row.execution_model_id,
        )?;
        validate_aggregate_report(&row.aggregate, &format!("{row_context} aggregate"))?;
        validate_exact_text(
            &format!("{row_context} aggregate.engine_version"),
            &row.aggregate.engine_version,
            &report.engine_version,
        )?;
        validate_exact_text(
            &format!("{row_context} aggregate.snapshot_id"),
            &row.aggregate.snapshot_id,
            &report.snapshot_id,
        )?;
        validate_exact_text(
            &format!("{row_context} aggregate.provider_identity"),
            &row.aggregate.provider_identity,
            &report.provider_identity,
        )?;
        validate_exact_text(
            &format!("{row_context} aggregate.date_range"),
            &row.aggregate.date_range,
            &report.date_range,
        )?;
        validate_exact_text(
            &format!("{row_context} aggregate.gap_policy"),
            &row.aggregate.gap_policy,
            &report.gap_policy,
        )?;
        validate_exact_text(
            &format!("{row_context} aggregate.historical_limitations"),
            &row.aggregate.historical_limitations,
            &report.historical_limitations,
        )?;
        validate_exact_vec(
            &format!("{row_context} aggregate.symbols"),
            &row.aggregate.symbols,
            &report.symbols,
        )?;
        validate_report_usize(
            &format!("{row_context} aggregate.symbol_count"),
            row.aggregate.symbol_count,
            report.symbol_count,
        )?;

        if let Some(value) = &report.fixed_signal_id {
            validate_exact_text(&format!("{row_context} signal_id"), &row.signal_id, value)?;
        }
        if let Some(value) = &report.fixed_filter_id {
            validate_exact_text(&format!("{row_context} filter_id"), &row.filter_id, value)?;
        }
        if let Some(value) = &report.fixed_position_manager_id {
            validate_exact_text(
                &format!("{row_context} position_manager_id"),
                &row.position_manager_id,
                value,
            )?;
        }
        if let Some(value) = &report.fixed_execution_model_id {
            validate_exact_text(
                &format!("{row_context} execution_model_id"),
                &row.execution_model_id,
                value,
            )?;
        }

        let expected_label = expected_leaderboard_label(report.view, row);
        validate_exact_text(&format!("{row_context} label"), &row.label, &expected_label)?;
    }

    Ok(())
}

fn validate_common_report_fields(
    context: &str,
    engine_version: &str,
    snapshot_id: &str,
    provider_identity: &str,
    date_range: &str,
    gap_policy: &str,
    historical_limitations: &str,
) -> Result<(), ArtifactError> {
    validate_non_empty_text(&format!("{context} engine_version"), engine_version)?;
    validate_non_empty_text(&format!("{context} snapshot_id"), snapshot_id)?;
    validate_non_empty_text(&format!("{context} provider_identity"), provider_identity)?;
    validate_non_empty_text(&format!("{context} date_range"), date_range)?;
    validate_non_empty_text(&format!("{context} gap_policy"), gap_policy)?;
    validate_non_empty_text(
        &format!("{context} historical_limitations"),
        historical_limitations,
    )?;
    Ok(())
}

fn validate_split_children(
    context: &str,
    expected_symbols: &[String],
    children: &[ResearchWalkForwardSplitChild],
) -> Result<(), ArtifactError> {
    if children.len() != expected_symbols.len() {
        return Err(ArtifactError::invalid(format!(
            "{context} child count {} does not match symbol count {}",
            children.len(),
            expected_symbols.len()
        )));
    }

    let mut actual_symbols = Vec::with_capacity(children.len());
    let mut seen_symbols = BTreeSet::new();

    for (index, child) in children.iter().enumerate() {
        validate_non_empty_text(&format!("{context} child[{index}].symbol"), &child.symbol)?;
        validate_non_empty_path(
            &format!("{context} child[{index}].bundle_path"),
            &child.bundle_path,
        )?;
        if !seen_symbols.insert(child.symbol.as_str()) {
            return Err(ArtifactError::invalid(format!(
                "{context} contains duplicate child symbol `{}`",
                child.symbol
            )));
        }
        actual_symbols.push(child.symbol.clone());
    }

    validate_exact_vec(
        &format!("{context} symbols"),
        &actual_symbols,
        expected_symbols,
    )
}

fn validate_exact_children(
    context: &str,
    actual: &[ResearchWalkForwardSplitChild],
    expected: &[ResearchWalkForwardSplitChild],
) -> Result<(), ArtifactError> {
    if actual.len() != expected.len() {
        return Err(ArtifactError::invalid(format!(
            "{context} length {} does not match expected {}",
            actual.len(),
            expected.len()
        )));
    }

    for (index, (actual_child, expected_child)) in actual.iter().zip(expected.iter()).enumerate() {
        validate_exact_text(
            &format!("{context}[{index}].symbol"),
            &actual_child.symbol,
            &expected_child.symbol,
        )?;
        if actual_child.bundle_path != expected_child.bundle_path {
            return Err(ArtifactError::invalid(format!(
                "{context}[{index}].bundle_path {} does not match expected {}",
                actual_child.bundle_path.display(),
                expected_child.bundle_path.display()
            )));
        }
    }

    Ok(())
}

fn validate_bootstrap_distribution(
    distribution: &BootstrapDistributionSummary,
    context: &str,
    expected_resample_size: usize,
) -> Result<(), ArtifactError> {
    if distribution.sample_count == 0 {
        return Err(ArtifactError::invalid(format!(
            "{context} sample_count must be greater than zero"
        )));
    }

    validate_report_usize(
        &format!("{context} resample_size"),
        distribution.resample_size,
        expected_resample_size,
    )?;
    validate_non_empty_text(&format!("{context} metric"), &distribution.metric)?;
    parse_report_f64(
        &format!("{context} baseline_metric"),
        &distribution.baseline_metric,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_mean"),
        &distribution.bootstrap_mean,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_median"),
        &distribution.bootstrap_median,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_min"),
        &distribution.bootstrap_min,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_max"),
        &distribution.bootstrap_max,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_interval_95_lower"),
        &distribution.bootstrap_interval_95_lower,
    )?;
    parse_report_f64(
        &format!("{context} bootstrap_interval_95_upper"),
        &distribution.bootstrap_interval_95_upper,
    )?;
    Ok(())
}

fn validate_non_empty_text(field: &str, value: &str) -> Result<(), ArtifactError> {
    if value.trim().is_empty() {
        Err(ArtifactError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_non_empty_path(field: &str, value: &Path) -> Result<(), ArtifactError> {
    if value.as_os_str().is_empty() {
        Err(ArtifactError::invalid(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_some_text(field: &str, value: Option<&str>) -> Result<(), ArtifactError> {
    match value {
        Some(value) => validate_non_empty_text(field, value),
        None => Err(ArtifactError::invalid(format!("{field} must be present"))),
    }
}

fn validate_none_text(field: &str, value: Option<&str>) -> Result<(), ArtifactError> {
    if value.is_some() {
        Err(ArtifactError::invalid(format!("{field} must be absent")))
    } else {
        Ok(())
    }
}

fn validate_exact_text(field: &str, actual: &str, expected: &str) -> Result<(), ArtifactError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ArtifactError::invalid(format!(
            "{field} `{actual}` does not match expected `{expected}`"
        )))
    }
}

fn validate_exact_vec(
    field: &str,
    actual: &[String],
    expected: &[String],
) -> Result<(), ArtifactError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ArtifactError::invalid(format!(
            "{field} `{}` does not match expected `{}`",
            actual.join("|"),
            expected.join("|")
        )))
    }
}

fn validate_report_usize(field: &str, actual: usize, expected: usize) -> Result<(), ArtifactError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ArtifactError::invalid(format!(
            "{field} {actual} does not match expected {expected}"
        )))
    }
}

fn validate_report_f64_value(
    field: &str,
    actual_text: &str,
    expected: f64,
) -> Result<(), ArtifactError> {
    let actual = parse_report_f64(field, actual_text)?;
    if round4(actual) == round4(expected) {
        Ok(())
    } else {
        Err(ArtifactError::invalid(format!(
            "{field} {} does not match expected {}",
            actual_text,
            format_signed_decimal(expected)
        )))
    }
}

fn parse_report_f64(field: &str, value: &str) -> Result<f64, ArtifactError> {
    value.parse::<f64>().map_err(|_| {
        ArtifactError::invalid(format!("{field} `{value}` is not a valid formatted number"))
    })
}

fn format_signed_decimal(value: f64) -> String {
    if value >= 0.0 {
        format!("+{value:.4}")
    } else {
        format!("{value:.4}")
    }
}

fn expected_leaderboard_label(view: LeaderboardView, row: &ResearchLeaderboardRow) -> String {
    match view {
        LeaderboardView::Signal => row.signal_id.clone(),
        LeaderboardView::PositionManager => row.position_manager_id.clone(),
        LeaderboardView::ExecutionModel => row.execution_model_id.clone(),
        LeaderboardView::System => format!(
            "signal={} filter={} position={} execution={}",
            row.signal_id, row.filter_id, row.position_manager_id, row.execution_model_id
        ),
    }
}

pub fn diff_replay_bundles(left: &ReplayBundle, right: &ReplayBundle) -> ReplayBundleDiff {
    let mut manifest_diffs = Vec::new();
    let mut summary_diffs = Vec::new();

    push_diff(
        &mut manifest_diffs,
        "schema_version",
        left.manifest.schema_version.to_string(),
        right.manifest.schema_version.to_string(),
    );
    push_diff(
        &mut manifest_diffs,
        "engine_version",
        left.manifest.engine_version.clone(),
        right.manifest.engine_version.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "data_snapshot_id",
        left.manifest.data_snapshot_id.clone(),
        right.manifest.data_snapshot_id.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "provider_identity",
        left.manifest.provider_identity.clone(),
        right.manifest.provider_identity.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "symbol_or_universe",
        left.manifest.symbol_or_universe.clone(),
        right.manifest.symbol_or_universe.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "universe_mode",
        left.manifest.universe_mode.clone(),
        right.manifest.universe_mode.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "historical_limitations",
        format_string_list(&left.manifest.historical_limitations),
        format_string_list(&right.manifest.historical_limitations),
    );
    push_diff(
        &mut manifest_diffs,
        "date_range.start_date",
        left.manifest.date_range.start_date.clone(),
        right.manifest.date_range.start_date.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "date_range.end_date",
        left.manifest.date_range.end_date.clone(),
        right.manifest.date_range.end_date.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "reference_flow.kind",
        left.manifest.reference_flow.kind.clone(),
        right.manifest.reference_flow.kind.clone(),
    );
    push_diff(
        &mut manifest_diffs,
        "reference_flow.entry_shares",
        left.manifest.reference_flow.entry_shares.to_string(),
        right.manifest.reference_flow.entry_shares.to_string(),
    );
    push_diff(
        &mut manifest_diffs,
        "reference_flow.protective_stop_fraction",
        format_decimal(left.manifest.reference_flow.protective_stop_fraction),
        format_decimal(right.manifest.reference_flow.protective_stop_fraction),
    );
    push_diff(
        &mut manifest_diffs,
        "parameters",
        format_manifest_parameters(&left.manifest.parameters),
        format_manifest_parameters(&right.manifest.parameters),
    );
    push_diff(
        &mut manifest_diffs,
        "cost_model.commission_per_fill",
        format_decimal(left.manifest.cost_model.commission_per_fill),
        format_decimal(right.manifest.cost_model.commission_per_fill),
    );
    push_diff(
        &mut manifest_diffs,
        "cost_model.slippage_per_share",
        format_decimal(left.manifest.cost_model.slippage_per_share),
        format_decimal(right.manifest.cost_model.slippage_per_share),
    );
    push_diff(
        &mut manifest_diffs,
        "gap_policy",
        left.manifest.gap_policy.as_str().to_string(),
        right.manifest.gap_policy.as_str().to_string(),
    );
    push_diff(
        &mut manifest_diffs,
        "seed",
        format_optional_u64(left.manifest.seed),
        format_optional_u64(right.manifest.seed),
    );
    push_diff(
        &mut manifest_diffs,
        "warnings",
        format_string_list(&left.manifest.warnings),
        format_string_list(&right.manifest.warnings),
    );

    push_diff(
        &mut summary_diffs,
        "row_count",
        left.summary.row_count.to_string(),
        right.summary.row_count.to_string(),
    );
    push_diff(
        &mut summary_diffs,
        "warning_count",
        left.summary.warning_count.to_string(),
        right.summary.warning_count.to_string(),
    );
    push_diff(
        &mut summary_diffs,
        "ending_cash",
        format_decimal(left.summary.ending_cash),
        format_decimal(right.summary.ending_cash),
    );
    push_diff(
        &mut summary_diffs,
        "ending_equity",
        format_decimal(left.summary.ending_equity),
        format_decimal(right.summary.ending_equity),
    );

    let mut ledger_row_diffs = Vec::new();
    for index in 0..left.ledger.len().max(right.ledger.len()) {
        match (left.ledger.get(index), right.ledger.get(index)) {
            (Some(left_row), Some(right_row)) => {
                let mut field_diffs = Vec::new();
                push_diff(
                    &mut field_diffs,
                    "date",
                    left_row.date.clone(),
                    right_row.date.clone(),
                );
                push_diff(
                    &mut field_diffs,
                    "raw_open",
                    format_decimal(left_row.raw_open),
                    format_decimal(right_row.raw_open),
                );
                push_diff(
                    &mut field_diffs,
                    "raw_high",
                    format_decimal(left_row.raw_high),
                    format_decimal(right_row.raw_high),
                );
                push_diff(
                    &mut field_diffs,
                    "raw_low",
                    format_decimal(left_row.raw_low),
                    format_decimal(right_row.raw_low),
                );
                push_diff(
                    &mut field_diffs,
                    "raw_close",
                    format_decimal(left_row.raw_close),
                    format_decimal(right_row.raw_close),
                );
                push_diff(
                    &mut field_diffs,
                    "analysis_close",
                    format_decimal(left_row.analysis_close),
                    format_decimal(right_row.analysis_close),
                );
                push_diff(
                    &mut field_diffs,
                    "position_shares",
                    left_row.position_shares.to_string(),
                    right_row.position_shares.to_string(),
                );
                push_diff(
                    &mut field_diffs,
                    "signal_output",
                    left_row.signal_output.clone(),
                    right_row.signal_output.clone(),
                );
                push_diff(
                    &mut field_diffs,
                    "filter_outcome",
                    left_row.filter_outcome.clone(),
                    right_row.filter_outcome.clone(),
                );
                push_diff(
                    &mut field_diffs,
                    "pending_order_state",
                    left_row.pending_order_state.clone(),
                    right_row.pending_order_state.clone(),
                );
                push_diff(
                    &mut field_diffs,
                    "fill_price",
                    format_optional_f64(left_row.fill_price),
                    format_optional_f64(right_row.fill_price),
                );
                push_diff(
                    &mut field_diffs,
                    "prior_stop",
                    format_optional_f64(left_row.prior_stop),
                    format_optional_f64(right_row.prior_stop),
                );
                push_diff(
                    &mut field_diffs,
                    "next_stop",
                    format_optional_f64(left_row.next_stop),
                    format_optional_f64(right_row.next_stop),
                );
                push_diff(
                    &mut field_diffs,
                    "cash",
                    format_decimal(left_row.cash),
                    format_decimal(right_row.cash),
                );
                push_diff(
                    &mut field_diffs,
                    "equity",
                    format_decimal(left_row.equity),
                    format_decimal(right_row.equity),
                );
                push_diff(
                    &mut field_diffs,
                    "reason_codes",
                    format_string_list(&left_row.reason_codes),
                    format_string_list(&right_row.reason_codes),
                );

                if !field_diffs.is_empty() {
                    ledger_row_diffs.push(LedgerRowDiff {
                        index,
                        left_date: Some(left_row.date.clone()),
                        right_date: Some(right_row.date.clone()),
                        field_diffs,
                    });
                }
            }
            (Some(left_row), None) => ledger_row_diffs.push(LedgerRowDiff {
                index,
                left_date: Some(left_row.date.clone()),
                right_date: None,
                field_diffs: vec![ValueDiff {
                    field: "row".to_string(),
                    left: format_ledger_row(left_row),
                    right: "missing".to_string(),
                }],
            }),
            (None, Some(right_row)) => ledger_row_diffs.push(LedgerRowDiff {
                index,
                left_date: None,
                right_date: Some(right_row.date.clone()),
                field_diffs: vec![ValueDiff {
                    field: "row".to_string(),
                    left: "missing".to_string(),
                    right: format_ledger_row(right_row),
                }],
            }),
            (None, None) => {}
        }
    }

    ReplayBundleDiff {
        manifest_diffs,
        summary_diffs,
        ledger_row_diffs,
    }
}

fn resolve_bundle_path(
    bundle_dir: &Path,
    relative_path: &str,
    field_name: &str,
) -> Result<PathBuf, ArtifactError> {
    let path = Path::new(relative_path);

    if path.is_absolute() {
        return Err(ArtifactError::invalid(format!(
            "{field_name} must be relative within the bundle"
        )));
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err(ArtifactError::invalid(format!(
            "{field_name} must not escape the bundle root"
        )));
    }

    Ok(bundle_dir.join(path))
}

fn validate_replay_bundle_integrity(
    expected: &ReplayBundleIntegrity,
    actual: &ReplayBundleIntegrity,
    path: &Path,
) -> Result<(), ArtifactError> {
    if expected == actual {
        return Ok(());
    }

    let field_name = if expected.manifest != actual.manifest {
        "manifest"
    } else if expected.summary != actual.summary {
        "summary"
    } else {
        "ledger"
    };

    Err(ArtifactError::invalid(format!(
        "replay bundle integrity mismatch for {} {field_name}",
        path.display()
    )))
}

fn compute_replay_bundle_integrity(
    manifest: &RunManifest,
    summary: &RunSummary,
    ledger: &[PersistedLedgerRow],
) -> Result<ReplayBundleIntegrity, ArtifactError> {
    Ok(ReplayBundleIntegrity {
        manifest: fingerprint_json_value(manifest, "manifest")?,
        summary: fingerprint_json_value(summary, "summary")?,
        ledger: fingerprint_json_value(ledger, "ledger")?,
    })
}

fn fingerprint_json_value<T: Serialize + ?Sized>(
    value: &T,
    label: &str,
) -> Result<ContentFingerprint, ArtifactError> {
    let bytes = serde_json::to_vec(value).map_err(|err| {
        ArtifactError::invalid(format!(
            "failed to serialize replay bundle {label} for integrity fingerprinting: {err}"
        ))
    })?;
    Ok(ContentFingerprint {
        byte_count: bytes.len(),
        fnv1a64: format!("{:016x}", fnv1a64(&bytes)),
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn validate_research_report_bundle_links(
    stored: &StoredResearchReport,
    report_path: &Path,
) -> Result<(), ArtifactError> {
    if stored.linked_replay_bundles.is_empty() {
        validate_research_report(&stored.report)?;
        return Ok(());
    }

    let expected_paths = collect_research_report_bundle_paths(&stored.report)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let actual_paths = stored
        .linked_replay_bundles
        .iter()
        .map(|link| link.path.clone())
        .collect::<BTreeSet<_>>();

    if expected_paths != actual_paths {
        return Err(ArtifactError::invalid(format!(
            "research report linked replay bundle metadata in {} does not match stored report paths",
            report_path.display()
        )));
    }

    validate_research_report(&stored.report)
}

fn collect_research_report_bundle_paths(report: &ResearchReport) -> Vec<PathBuf> {
    match report {
        ResearchReport::Aggregate(report) => report
            .members
            .iter()
            .map(|member| member.bundle_path.clone())
            .collect(),
        ResearchReport::WalkForward(report) => report
            .splits
            .iter()
            .flat_map(|split| split.children.iter().map(|child| child.bundle_path.clone()))
            .collect(),
        ResearchReport::BootstrapAggregate(report) => report
            .baseline
            .members
            .iter()
            .map(|member| member.bundle_path.clone())
            .collect(),
        ResearchReport::BootstrapWalkForward(report) => report
            .splits
            .iter()
            .flat_map(|split| split.children.iter().map(|child| child.bundle_path.clone()))
            .collect(),
        ResearchReport::Leaderboard(report) => report
            .rows
            .iter()
            .flat_map(|row| {
                row.aggregate
                    .members
                    .iter()
                    .map(|member| member.bundle_path.clone())
            })
            .collect(),
    }
}

fn map_research_report_bundle_paths(
    report: &ResearchReport,
    mapper: &mut dyn FnMut(&Path) -> Result<PathBuf, ArtifactError>,
) -> Result<ResearchReport, ArtifactError> {
    match report {
        ResearchReport::Aggregate(report) => {
            Ok(ResearchReport::Aggregate(ResearchAggregateReport {
                members: report
                    .members
                    .iter()
                    .map(|member| {
                        Ok(ResearchAggregateMember {
                            bundle_path: mapper(&member.bundle_path)?,
                            ..member.clone()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                ..report.clone()
            }))
        }
        ResearchReport::WalkForward(report) => {
            Ok(ResearchReport::WalkForward(ResearchWalkForwardReport {
                splits: report
                    .splits
                    .iter()
                    .map(|split| {
                        Ok(ResearchWalkForwardSplit {
                            children: split
                                .children
                                .iter()
                                .map(|child| {
                                    Ok(ResearchWalkForwardSplitChild {
                                        bundle_path: mapper(&child.bundle_path)?,
                                        ..child.clone()
                                    })
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                            ..split.clone()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                ..report.clone()
            }))
        }
        ResearchReport::BootstrapAggregate(report) => {
            let baseline = match map_research_report_bundle_paths(
                &ResearchReport::Aggregate(report.baseline.clone()),
                mapper,
            )? {
                ResearchReport::Aggregate(baseline) => baseline,
                _ => unreachable!("aggregate remap must return aggregate"),
            };
            Ok(ResearchReport::BootstrapAggregate(
                ResearchBootstrapAggregateReport {
                    baseline,
                    ..report.clone()
                },
            ))
        }
        ResearchReport::BootstrapWalkForward(report) => {
            let baseline = match map_research_report_bundle_paths(
                &ResearchReport::WalkForward(report.baseline.clone()),
                mapper,
            )? {
                ResearchReport::WalkForward(baseline) => baseline,
                _ => unreachable!("walk-forward remap must return walk-forward"),
            };
            Ok(ResearchReport::BootstrapWalkForward(
                ResearchBootstrapWalkForwardReport {
                    baseline,
                    splits: report
                        .splits
                        .iter()
                        .map(|split| {
                            Ok(ResearchBootstrapWalkForwardSplit {
                                children: split
                                    .children
                                    .iter()
                                    .map(|child| {
                                        Ok(ResearchWalkForwardSplitChild {
                                            bundle_path: mapper(&child.bundle_path)?,
                                            ..child.clone()
                                        })
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                                ..split.clone()
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    ..report.clone()
                },
            ))
        }
        ResearchReport::Leaderboard(report) => {
            Ok(ResearchReport::Leaderboard(ResearchLeaderboardReport {
                rows: report
                    .rows
                    .iter()
                    .map(|row| {
                        Ok(ResearchLeaderboardRow {
                            aggregate: ResearchAggregateReport {
                                members: row
                                    .aggregate
                                    .members
                                    .iter()
                                    .map(|member| {
                                        Ok(ResearchAggregateMember {
                                            bundle_path: mapper(&member.bundle_path)?,
                                            ..member.clone()
                                        })
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                                ..row.aggregate.clone()
                            },
                            ..row.clone()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                ..report.clone()
            }))
        }
    }
}

fn resolve_research_bundle_path(report_dir: &Path, stored_path: &Path) -> PathBuf {
    if stored_path.is_absolute() {
        normalize_path(stored_path)
    } else {
        normalize_path(&report_dir.join(stored_path))
    }
}

fn normalize_external_path(path: &Path) -> Result<PathBuf, ArtifactError> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| {
                ArtifactError::invalid(format!(
                    "failed to read current directory while resolving {}: {err}",
                    path.display()
                ))
            })?
            .join(path)
    };

    Ok(normalize_path(&path))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
        }
    }

    normalized
}

fn relative_external_path(from_dir: &Path, to_path: &Path) -> Option<PathBuf> {
    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_path.components().collect::<Vec<_>>();

    let mut shared = 0_usize;
    while shared < from_components.len()
        && shared < to_components.len()
        && component_os_str(from_components[shared]) == component_os_str(to_components[shared])
    {
        shared += 1;
    }

    if shared == 0 {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in &from_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &to_components[shared..] {
        relative.push(component_os_str(*component));
    }

    if relative.as_os_str().is_empty() {
        relative.push(".");
    }

    Some(relative)
}

fn component_os_str(component: Component<'_>) -> &std::ffi::OsStr {
    match component {
        Component::Prefix(prefix) => prefix.as_os_str(),
        Component::RootDir => std::ffi::OsStr::new(std::path::MAIN_SEPARATOR_STR),
        Component::CurDir => std::ffi::OsStr::new("."),
        Component::ParentDir => std::ffi::OsStr::new(".."),
        Component::Normal(part) => part,
    }
}

fn validate_replay_bundle_parts(
    manifest: &RunManifest,
    summary: &RunSummary,
    ledger: &[PersistedLedgerRow],
) -> Result<(), ArtifactError> {
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(ArtifactError::invalid(format!(
            "manifest schema version {} does not match supported schema version {}",
            manifest.schema_version, SCHEMA_VERSION
        )));
    }

    if summary.row_count != ledger.len() {
        return Err(ArtifactError::invalid(format!(
            "summary row_count {} does not match ledger row count {}",
            summary.row_count,
            ledger.len()
        )));
    }

    if summary.warning_count != manifest.warnings.len() {
        return Err(ArtifactError::invalid(format!(
            "summary warning_count {} does not match manifest warning count {}",
            summary.warning_count,
            manifest.warnings.len()
        )));
    }

    if let Some(last_row) = ledger.last() {
        if round4(summary.ending_cash) != round4(last_row.cash) {
            return Err(ArtifactError::invalid(format!(
                "summary ending_cash {} does not match terminal ledger cash {}",
                summary.ending_cash, last_row.cash
            )));
        }

        if round4(summary.ending_equity) != round4(last_row.equity) {
            return Err(ArtifactError::invalid(format!(
                "summary ending_equity {} does not match terminal ledger equity {}",
                summary.ending_equity, last_row.equity
            )));
        }
    }

    Ok(())
}

fn write_json_pretty<T: Serialize>(
    path: &Path,
    value: &T,
    error_prefix: &str,
) -> Result<(), ArtifactError> {
    let file = File::create(path).map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value)
        .map_err(|err| ArtifactError::json(error_prefix, path, &err))
}

fn read_json<T: DeserializeOwned>(path: &Path, error_prefix: &str) -> Result<T, ArtifactError> {
    let file = File::open(path).map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(|err| ArtifactError::json(error_prefix, path, &err))
}

fn write_json_lines<T: Serialize>(
    path: &Path,
    rows: &[T],
    error_prefix: &str,
) -> Result<(), ArtifactError> {
    let file = File::create(path).map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
    let mut writer = BufWriter::new(file);

    for row in rows {
        serde_json::to_writer(&mut writer, row)
            .map_err(|err| ArtifactError::json(error_prefix, path, &err))?;
        writer
            .write_all(b"\n")
            .map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
    }

    writer
        .flush()
        .map_err(|err| ArtifactError::io(error_prefix, path, &err))
}

fn read_json_lines<T: DeserializeOwned>(
    path: &Path,
    error_prefix: &str,
) -> Result<Vec<T>, ArtifactError> {
    let file = File::open(path).map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for (line_index, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|err| ArtifactError::io(error_prefix, path, &err))?;
        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str(&line).map_err(|err| {
            ArtifactError::invalid(format!(
                "{error_prefix} {} line {}: {err}",
                path.display(),
                line_index + 1
            ))
        })?;
        rows.push(value);
    }

    Ok(rows)
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn push_diff(diffs: &mut Vec<ValueDiff>, field: &str, left: String, right: String) {
    if left != right {
        diffs.push(ValueDiff {
            field: field.to_string(),
            left,
            right,
        });
    }
}

fn format_decimal(value: f64) -> String {
    format!("{value:.4}")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value
        .map(format_decimal)
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_string_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("|")
    }
}

fn format_manifest_parameters(parameters: &[ManifestParameter]) -> String {
    if parameters.is_empty() {
        "none".to_string()
    } else {
        parameters
            .iter()
            .map(|parameter| format!("{}={}", parameter.name, parameter.value))
            .collect::<Vec<_>>()
            .join("|")
    }
}

fn format_ledger_row(row: &PersistedLedgerRow) -> String {
    serde_json::to_string(row).expect("persisted ledger rows must serialize")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use trendlab_core::ledger::LedgerRow;

    use crate::{
        ArtifactError, BUNDLE_FILE_NAME, BootstrapDistributionSummary, BundleDescriptor, DateRange,
        LEDGER_FILE_NAME, MANIFEST_FILE_NAME, ManifestParameter, PersistedLedgerRow,
        RESEARCH_REPORT_FILE_NAME, ReferenceFlowDefinition, ReplayBundle, ResearchAggregateMember,
        ResearchAggregateReport, ResearchBootstrapAggregateReport,
        ResearchBootstrapWalkForwardReport, ResearchBootstrapWalkForwardSplit,
        ResearchLeaderboardReport, ResearchLeaderboardRow, ResearchReport,
        ResearchWalkForwardReport, ResearchWalkForwardSplit, ResearchWalkForwardSplitChild,
        RunManifest, RunSummary, SCHEMA_VERSION, SUMMARY_FILE_NAME, diff_replay_bundles,
        load_replay_bundle, load_research_report_bundle, map_research_report_bundle_paths,
        write_json_lines, write_json_pretty, write_replay_bundle, write_research_report_bundle,
    };

    #[test]
    fn persisted_row_copies_core_ledger_shape() {
        let row = LedgerRow {
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
            reason_codes: vec!["entry_filled_at_open".to_string()],
        };

        let persisted = PersistedLedgerRow::from(&row);

        assert_eq!(persisted.date, row.date);
        assert_eq!(persisted.position_shares, row.position_shares);
        assert_eq!(persisted.reason_codes, row.reason_codes);
    }

    #[test]
    fn replay_bundle_round_trips_on_disk() {
        let bundle_dir = test_output_dir("artifact-roundtrip");
        let manifest = sample_manifest();
        let summary = RunSummary {
            row_count: 2,
            warning_count: 1,
            ending_cash: 898.0,
            ending_equity: 1001.5,
        };
        let ledger = vec![
            PersistedLedgerRow {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.5,
                analysis_close: 100.5,
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
            PersistedLedgerRow {
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
        ];

        let descriptor = write_replay_bundle(&bundle_dir, &manifest, &summary, &ledger).unwrap();
        let loaded = load_replay_bundle(&bundle_dir).unwrap();

        assert_eq!(descriptor.manifest_path, MANIFEST_FILE_NAME);
        assert_eq!(descriptor.summary_path, SUMMARY_FILE_NAME);
        assert_eq!(descriptor.ledger_path, LEDGER_FILE_NAME);
        assert!(descriptor.integrity.is_some());
        assert!(bundle_dir.join(BUNDLE_FILE_NAME).is_file());
        assert!(bundle_dir.join(MANIFEST_FILE_NAME).is_file());
        assert!(bundle_dir.join(SUMMARY_FILE_NAME).is_file());
        assert!(bundle_dir.join(LEDGER_FILE_NAME).is_file());
        assert_eq!(
            loaded,
            ReplayBundle {
                descriptor,
                manifest,
                summary,
                ledger,
            }
        );

        fs::remove_dir_all(bundle_dir).unwrap();
    }

    #[test]
    fn write_replay_bundle_rejects_warning_count_mismatch() {
        let bundle_dir = test_output_dir("artifact-warning-mismatch");
        let manifest = sample_manifest();
        let summary = RunSummary {
            row_count: 1,
            warning_count: 0,
            ending_cash: 1000.0,
            ending_equity: 1000.0,
        };
        let ledger = vec![PersistedLedgerRow {
            date: "2025-01-02".to_string(),
            raw_open: 100.0,
            raw_high: 101.0,
            raw_low: 99.0,
            raw_close: 100.0,
            analysis_close: 100.0,
            position_shares: 0,
            signal_output: "none".to_string(),
            filter_outcome: "not_checked".to_string(),
            pending_order_state: "none".to_string(),
            fill_price: None,
            prior_stop: None,
            next_stop: None,
            cash: 1000.0,
            equity: 1000.0,
            reason_codes: vec!["hold_position".to_string()],
        }];

        let error = write_replay_bundle(&bundle_dir, &manifest, &summary, &ledger).unwrap_err();

        assert_eq!(
            error.to_string(),
            "summary warning_count 0 does not match manifest warning count 1"
        );
    }

    #[test]
    fn load_replay_bundle_rejects_terminal_summary_mismatch() {
        let bundle_dir = test_output_dir("artifact-ending-mismatch");
        let descriptor = BundleDescriptor::canonical();
        let manifest = sample_manifest();
        let summary = RunSummary {
            row_count: 1,
            warning_count: 1,
            ending_cash: 999.0,
            ending_equity: 999.0,
        };
        let ledger = vec![PersistedLedgerRow {
            date: "2025-01-02".to_string(),
            raw_open: 100.0,
            raw_high: 101.0,
            raw_low: 99.0,
            raw_close: 100.0,
            analysis_close: 100.0,
            position_shares: 0,
            signal_output: "none".to_string(),
            filter_outcome: "not_checked".to_string(),
            pending_order_state: "none".to_string(),
            fill_price: None,
            prior_stop: None,
            next_stop: None,
            cash: 1000.0,
            equity: 1000.0,
            reason_codes: vec!["hold_position".to_string()],
        }];

        fs::create_dir_all(&bundle_dir).unwrap();
        write_json_pretty(
            &bundle_dir.join(BUNDLE_FILE_NAME),
            &descriptor,
            "failed to write",
        )
        .unwrap();
        write_json_pretty(
            &bundle_dir.join(MANIFEST_FILE_NAME),
            &manifest,
            "failed to write",
        )
        .unwrap();
        write_json_pretty(
            &bundle_dir.join(SUMMARY_FILE_NAME),
            &summary,
            "failed to write",
        )
        .unwrap();
        write_json_lines(
            &bundle_dir.join(LEDGER_FILE_NAME),
            &ledger,
            "failed to write",
        )
        .unwrap();

        let error = load_replay_bundle(&bundle_dir).unwrap_err();

        assert_eq!(
            error.to_string(),
            "summary ending_cash 999 does not match terminal ledger cash 1000"
        );

        fs::remove_dir_all(bundle_dir).unwrap();
    }

    #[test]
    fn load_replay_bundle_rejects_manifest_integrity_drift() {
        let bundle_dir = test_output_dir("artifact-integrity-mismatch");
        let manifest = sample_manifest();
        let summary = RunSummary {
            row_count: 1,
            warning_count: 1,
            ending_cash: 1000.0,
            ending_equity: 1000.0,
        };
        let ledger = vec![PersistedLedgerRow {
            date: "2025-01-02".to_string(),
            raw_open: 100.0,
            raw_high: 101.0,
            raw_low: 99.0,
            raw_close: 100.0,
            analysis_close: 100.0,
            position_shares: 0,
            signal_output: "none".to_string(),
            filter_outcome: "not_checked".to_string(),
            pending_order_state: "none".to_string(),
            fill_price: None,
            prior_stop: None,
            next_stop: None,
            cash: 1000.0,
            equity: 1000.0,
            reason_codes: vec!["hold_position".to_string()],
        }];

        write_replay_bundle(&bundle_dir, &manifest, &summary, &ledger).unwrap();

        let mut tampered_manifest = manifest.clone();
        tampered_manifest.engine_version = "tampered-reference-flow".to_string();
        write_json_pretty(
            &bundle_dir.join(MANIFEST_FILE_NAME),
            &tampered_manifest,
            "failed to write",
        )
        .unwrap();

        let error = load_replay_bundle(&bundle_dir).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "replay bundle integrity mismatch for {} manifest",
                bundle_dir.join(BUNDLE_FILE_NAME).display()
            )
        );

        fs::remove_dir_all(bundle_dir).unwrap();
    }

    #[test]
    fn persisted_rows_project_back_to_core_market_bars() {
        let row = PersistedLedgerRow {
            date: "2025-01-02".to_string(),
            raw_open: 100.0,
            raw_high: 101.0,
            raw_low: 99.0,
            raw_close: 100.5,
            analysis_close: 50.25,
            position_shares: 0,
            signal_output: "none".to_string(),
            filter_outcome: "not_checked".to_string(),
            pending_order_state: "none".to_string(),
            fill_price: None,
            prior_stop: None,
            next_stop: None,
            cash: 1000.0,
            equity: 1000.0,
            reason_codes: Vec::new(),
        };

        let bar = row.market_bar();

        assert_eq!(bar.date, "2025-01-02");
        assert_eq!(bar.raw_close, 100.5);
        assert_eq!(bar.analysis_close, 50.25);
    }

    #[test]
    fn replay_bundle_diff_reports_manifest_summary_and_ledger_changes() {
        let left = ReplayBundle {
            descriptor: BundleDescriptor::canonical(),
            manifest: sample_manifest(),
            summary: RunSummary {
                row_count: 1,
                warning_count: 1,
                ending_cash: 1000.0,
                ending_equity: 1000.0,
            },
            ledger: vec![PersistedLedgerRow {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.0,
                analysis_close: 100.0,
                position_shares: 0,
                signal_output: "none".to_string(),
                filter_outcome: "not_checked".to_string(),
                pending_order_state: "none".to_string(),
                fill_price: None,
                prior_stop: None,
                next_stop: None,
                cash: 1000.0,
                equity: 1000.0,
                reason_codes: vec!["hold_position".to_string()],
            }],
        };
        let mut right = left.clone();
        right.manifest.engine_version = "m1-reference-flow-v2".to_string();
        right.summary.ending_equity = 1001.5;
        right.ledger[0].analysis_close = 50.0;
        right.ledger[0].equity = 1001.5;

        let diff = diff_replay_bundles(&left, &right);

        assert_eq!(diff.manifest_diffs.len(), 1);
        assert_eq!(diff.manifest_diffs[0].field, "engine_version");
        assert_eq!(diff.summary_diffs.len(), 1);
        assert_eq!(diff.summary_diffs[0].field, "ending_equity");
        assert_eq!(diff.ledger_row_diffs.len(), 1);
        assert_eq!(diff.ledger_row_diffs[0].index, 0);
        assert!(
            diff.ledger_row_diffs[0]
                .field_diffs
                .iter()
                .any(|entry| entry.field == "analysis_close")
        );
        assert!(
            diff.ledger_row_diffs[0]
                .field_diffs
                .iter()
                .any(|entry| entry.field == "equity")
        );
        assert!(!diff.is_empty());
    }

    #[test]
    fn research_reports_round_trip_on_disk() {
        for (index, report) in sample_research_reports().into_iter().enumerate() {
            let root_dir = test_output_dir(&format!("artifact-research-report-{index}"));
            let alpha_bundle_dir = root_dir.join("bundles").join("alpha");
            let beta_bundle_dir = root_dir.join("bundles").join("beta");
            let report_dir = root_dir.join("report");
            write_sample_bundle(&alpha_bundle_dir, "ALPHA");
            write_sample_bundle(&beta_bundle_dir, "BETA");
            let report =
                bind_sample_report_bundle_paths(&report, &alpha_bundle_dir, &beta_bundle_dir);

            write_research_report_bundle(&report_dir, &report).unwrap();
            let loaded = load_research_report_bundle(&report_dir).unwrap();

            assert_eq!(loaded, report);
            assert!(report_dir.join(RESEARCH_REPORT_FILE_NAME).is_file());

            fs::remove_dir_all(report_dir).unwrap();
        }
    }

    #[test]
    fn write_research_report_rejects_inconsistent_aggregate_totals() {
        let report_dir = test_output_dir("artifact-research-report-invalid-aggregate");
        let mut report = sample_research_aggregate_report();
        report.total_trade_count = 999;

        let error = write_research_report_bundle(&report_dir, &ResearchReport::Aggregate(report))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "research aggregate total_trade_count 999 does not match expected 2"
        );
    }

    #[test]
    fn load_research_report_rejects_unsupported_schema_version() {
        let report_dir = test_output_dir("artifact-research-report-schema-mismatch");
        let stored = serde_json::json!({
            "schema_version": SCHEMA_VERSION + 1,
            "report": sample_research_reports()
                .into_iter()
                .next()
                .expect("sample research report"),
        });

        fs::create_dir_all(&report_dir).unwrap();
        write_json_pretty(
            &report_dir.join(RESEARCH_REPORT_FILE_NAME),
            &stored,
            "failed to write",
        )
        .unwrap();

        let error = load_research_report_bundle(&report_dir).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "unsupported research report schema version {}; expected {}",
                SCHEMA_VERSION + 1,
                SCHEMA_VERSION
            )
        );

        fs::remove_dir_all(report_dir).unwrap();
    }

    #[test]
    fn load_research_report_rejects_invalid_leaderboard_shape() {
        let report_dir = test_output_dir("artifact-research-report-invalid-leaderboard");
        let report = sample_research_reports()
            .into_iter()
            .find_map(|report| match report {
                ResearchReport::Leaderboard(report) => Some(report),
                _ => None,
            })
            .expect("sample leaderboard report");
        let mut report = report;
        report.fixed_signal_id = Some("should_not_be_present".to_string());
        let stored = serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "report": ResearchReport::Leaderboard(report),
        });

        fs::create_dir_all(&report_dir).unwrap();
        write_json_pretty(
            &report_dir.join(RESEARCH_REPORT_FILE_NAME),
            &stored,
            "failed to write",
        )
        .unwrap();

        let error = load_research_report_bundle(&report_dir).unwrap_err();

        assert_eq!(
            error.to_string(),
            "research leaderboard fixed_signal_id must be absent"
        );

        fs::remove_dir_all(report_dir).unwrap();
    }

    #[test]
    fn load_research_report_rejects_missing_linked_bundle() {
        let root_dir = test_output_dir("artifact-research-report-missing-bundle");
        let alpha_bundle_dir = root_dir.join("bundles").join("alpha");
        let beta_bundle_dir = root_dir.join("bundles").join("beta");
        let report_dir = root_dir.join("report");
        write_sample_bundle(&alpha_bundle_dir, "ALPHA");
        write_sample_bundle(&beta_bundle_dir, "BETA");
        let report = bind_sample_report_bundle_paths(
            &ResearchReport::Aggregate(sample_research_aggregate_report()),
            &alpha_bundle_dir,
            &beta_bundle_dir,
        );

        write_research_report_bundle(&report_dir, &report).unwrap();
        fs::remove_dir_all(&alpha_bundle_dir).unwrap();

        let error = load_research_report_bundle(&report_dir).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("research report requires replay bundle")
        );
        assert!(
            error
                .to_string()
                .contains(&alpha_bundle_dir.display().to_string())
        );

        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn load_research_report_rejects_linked_bundle_integrity_drift() {
        let root_dir = test_output_dir("artifact-research-report-integrity-drift");
        let alpha_bundle_dir = root_dir.join("bundles").join("alpha");
        let beta_bundle_dir = root_dir.join("bundles").join("beta");
        let report_dir = root_dir.join("report");
        write_sample_bundle(&alpha_bundle_dir, "ALPHA");
        write_sample_bundle(&beta_bundle_dir, "BETA");
        let report = bind_sample_report_bundle_paths(
            &ResearchReport::Aggregate(sample_research_aggregate_report()),
            &alpha_bundle_dir,
            &beta_bundle_dir,
        );

        write_research_report_bundle(&report_dir, &report).unwrap();
        let mut tampered = load_replay_bundle(&alpha_bundle_dir).unwrap();
        tampered.manifest.engine_version = "tampered-engine-version".to_string();
        write_replay_bundle(
            &alpha_bundle_dir,
            &tampered.manifest,
            &tampered.summary,
            &tampered.ledger,
        )
        .unwrap();

        let error = load_research_report_bundle(&report_dir).unwrap_err();

        assert!(error.to_string().contains("linked replay bundle"));
        assert!(
            error
                .to_string()
                .contains(&alpha_bundle_dir.display().to_string())
        );

        fs::remove_dir_all(root_dir).unwrap();
    }

    #[test]
    fn load_research_report_resolves_relative_links_after_tree_move() {
        let source_root = test_output_dir("artifact-research-report-portable-source");
        let moved_root = test_output_dir("artifact-research-report-portable-moved");
        let alpha_bundle_dir = source_root.join("bundles").join("alpha");
        let beta_bundle_dir = source_root.join("bundles").join("beta");
        let report_dir = source_root.join("report");
        write_sample_bundle(&alpha_bundle_dir, "ALPHA");
        write_sample_bundle(&beta_bundle_dir, "BETA");
        let report = bind_sample_report_bundle_paths(
            &ResearchReport::Aggregate(sample_research_aggregate_report()),
            &alpha_bundle_dir,
            &beta_bundle_dir,
        );

        write_research_report_bundle(&report_dir, &report).unwrap();
        fs::rename(&source_root, &moved_root).unwrap();

        let moved_report_dir = moved_root.join("report");
        let moved_alpha_bundle_dir = moved_root.join("bundles").join("alpha");
        let moved_beta_bundle_dir = moved_root.join("bundles").join("beta");
        let loaded = load_research_report_bundle(&moved_report_dir).unwrap();
        let expected = bind_sample_report_bundle_paths(
            &ResearchReport::Aggregate(sample_research_aggregate_report()),
            &moved_alpha_bundle_dir,
            &moved_beta_bundle_dir,
        );

        assert_eq!(loaded, expected);

        fs::remove_dir_all(moved_root).unwrap();
    }

    fn sample_manifest() -> RunManifest {
        RunManifest {
            schema_version: SCHEMA_VERSION,
            engine_version: "m1-reference-flow".to_string(),
            data_snapshot_id: "fixture:m1_intrabar_stop_exit".to_string(),
            provider_identity: "fixture".to_string(),
            symbol_or_universe: "TEST".to_string(),
            universe_mode: "single_symbol".to_string(),
            historical_limitations: Vec::new(),
            date_range: DateRange {
                start_date: "2025-01-02".to_string(),
                end_date: "2025-01-03".to_string(),
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
            cost_model: trendlab_core::accounting::CostModel::default(),
            gap_policy: trendlab_core::orders::GapPolicy::M1Default,
            seed: None,
            warnings: vec!["example_warning".to_string()],
        }
    }

    fn sample_research_reports() -> Vec<ResearchReport> {
        let aggregate = sample_research_aggregate_report();
        let walk_forward = sample_research_walk_forward_report();

        vec![
            ResearchReport::Aggregate(aggregate.clone()),
            ResearchReport::WalkForward(walk_forward.clone()),
            ResearchReport::BootstrapAggregate(ResearchBootstrapAggregateReport {
                baseline: aggregate.clone(),
                distribution: sample_bootstrap_aggregate_distribution(),
            }),
            ResearchReport::BootstrapWalkForward(ResearchBootstrapWalkForwardReport {
                baseline: walk_forward.clone(),
                distribution: sample_bootstrap_walk_forward_distribution(),
                splits: vec![ResearchBootstrapWalkForwardSplit {
                    sequence: 1,
                    train_row_range: "0..2".to_string(),
                    train_date_range: "2025-01-02..2025-01-06".to_string(),
                    test_row_range: "3..4".to_string(),
                    test_date_range: "2025-01-07..2025-01-08".to_string(),
                    baseline_test_total_net_equity_change: "+3.0000".to_string(),
                    baseline_test_average_net_equity_change: "+1.5000".to_string(),
                    children: sample_split_children(),
                }],
            }),
            ResearchReport::Leaderboard(ResearchLeaderboardReport {
                view: super::LeaderboardView::Signal,
                engine_version: "m6-reference-flow".to_string(),
                snapshot_id: "fixture:m6_research".to_string(),
                provider_identity: "fixture".to_string(),
                date_range: "2025-01-02..2025-01-08".to_string(),
                gap_policy: "m1_default".to_string(),
                historical_limitations: "none".to_string(),
                symbol_count: 2,
                symbols: vec!["ALPHA".to_string(), "BETA".to_string()],
                fixed_signal_id: None,
                fixed_filter_id: Some("pass_filter".to_string()),
                fixed_position_manager_id: Some("keep_position_manager".to_string()),
                fixed_execution_model_id: Some("next_open_long".to_string()),
                rows: vec![ResearchLeaderboardRow {
                    rank: 1,
                    label: "close_confirmed_breakout".to_string(),
                    signal_id: "close_confirmed_breakout".to_string(),
                    filter_id: "pass_filter".to_string(),
                    position_manager_id: "keep_position_manager".to_string(),
                    execution_model_id: "next_open_long".to_string(),
                    aggregate,
                }],
            }),
        ]
    }

    fn bind_sample_report_bundle_paths(
        report: &ResearchReport,
        alpha_bundle_dir: &Path,
        beta_bundle_dir: &Path,
    ) -> ResearchReport {
        map_research_report_bundle_paths(report, &mut |bundle_path| {
            let bundle_name = bundle_path.to_string_lossy();
            if bundle_name.contains("alpha-bundle") {
                Ok(alpha_bundle_dir.to_path_buf())
            } else if bundle_name.contains("beta-bundle") {
                Ok(beta_bundle_dir.to_path_buf())
            } else {
                Err(ArtifactError::invalid(format!(
                    "unexpected sample bundle path {}",
                    bundle_path.display()
                )))
            }
        })
        .expect("sample report bundle paths should bind")
    }

    fn write_sample_bundle(bundle_dir: &Path, symbol: &str) {
        let mut manifest = sample_manifest();
        manifest.symbol_or_universe = symbol.to_string();
        let summary = RunSummary {
            row_count: 1,
            warning_count: manifest.warnings.len(),
            ending_cash: 1000.0,
            ending_equity: 1000.0,
        };
        let ledger = vec![PersistedLedgerRow {
            date: "2025-01-02".to_string(),
            raw_open: 100.0,
            raw_high: 101.0,
            raw_low: 99.0,
            raw_close: 100.0,
            analysis_close: 100.0,
            position_shares: 0,
            signal_output: "none".to_string(),
            filter_outcome: "not_checked".to_string(),
            pending_order_state: "none".to_string(),
            fill_price: None,
            prior_stop: None,
            next_stop: None,
            cash: 1000.0,
            equity: 1000.0,
            reason_codes: vec!["hold_position".to_string()],
        }];

        write_replay_bundle(bundle_dir, &manifest, &summary, &ledger)
            .expect("sample bundle should write");
    }

    fn sample_research_aggregate_report() -> ResearchAggregateReport {
        ResearchAggregateReport {
            engine_version: "m6-reference-flow".to_string(),
            snapshot_id: "fixture:m6_research".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-01-08".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "none".to_string(),
            symbol_count: 2,
            total_row_count: 10,
            total_warning_count: 0,
            total_trade_count: 2,
            starting_equity_total: "2000.0000".to_string(),
            ending_equity_total: "2003.0000".to_string(),
            net_equity_change_total: "+3.0000".to_string(),
            average_net_equity_change: "+1.5000".to_string(),
            symbols: vec!["ALPHA".to_string(), "BETA".to_string()],
            members: vec![
                ResearchAggregateMember {
                    symbol: "ALPHA".to_string(),
                    bundle_path: PathBuf::from("reports/alpha-bundle"),
                    row_count: 5,
                    warning_count: 0,
                    trade_count: 1,
                    starting_equity: "1000.0000".to_string(),
                    ending_equity: "1002.0000".to_string(),
                    net_equity_change: "+2.0000".to_string(),
                },
                ResearchAggregateMember {
                    symbol: "BETA".to_string(),
                    bundle_path: PathBuf::from("reports/beta-bundle"),
                    row_count: 5,
                    warning_count: 0,
                    trade_count: 1,
                    starting_equity: "1000.0000".to_string(),
                    ending_equity: "1001.0000".to_string(),
                    net_equity_change: "+1.0000".to_string(),
                },
            ],
        }
    }

    fn sample_research_walk_forward_report() -> ResearchWalkForwardReport {
        ResearchWalkForwardReport {
            engine_version: "m6-reference-flow".to_string(),
            snapshot_id: "fixture:m6_research".to_string(),
            provider_identity: "fixture".to_string(),
            date_range: "2025-01-02..2025-01-08".to_string(),
            gap_policy: "m1_default".to_string(),
            historical_limitations: "none".to_string(),
            symbols: vec!["ALPHA".to_string(), "BETA".to_string()],
            train_bars: 3,
            test_bars: 2,
            step_bars: 1,
            split_count: 1,
            splits: vec![ResearchWalkForwardSplit {
                sequence: 1,
                train_start_index: 0,
                train_end_index: 2,
                test_start_index: 3,
                test_end_index: 4,
                train_row_range: "0..2".to_string(),
                train_date_range: "2025-01-02..2025-01-06".to_string(),
                test_row_range: "3..4".to_string(),
                test_date_range: "2025-01-07..2025-01-08".to_string(),
                children: sample_split_children(),
            }],
        }
    }

    fn sample_bootstrap_aggregate_distribution() -> BootstrapDistributionSummary {
        BootstrapDistributionSummary {
            seed: 7,
            sample_count: 5,
            resample_size: 2,
            metric: "average_net_equity_change".to_string(),
            baseline_metric: "+1.5000".to_string(),
            bootstrap_mean: "+1.7000".to_string(),
            bootstrap_median: "+1.5000".to_string(),
            bootstrap_min: "+1.0000".to_string(),
            bootstrap_max: "+2.5000".to_string(),
            bootstrap_interval_95_lower: "+1.0000".to_string(),
            bootstrap_interval_95_upper: "+2.4000".to_string(),
        }
    }

    fn sample_bootstrap_walk_forward_distribution() -> BootstrapDistributionSummary {
        BootstrapDistributionSummary {
            seed: 11,
            sample_count: 6,
            resample_size: 1,
            metric: "mean_split_test_average_net_equity_change".to_string(),
            baseline_metric: "+1.5000".to_string(),
            bootstrap_mean: "+1.6000".to_string(),
            bootstrap_median: "+1.5000".to_string(),
            bootstrap_min: "+1.5000".to_string(),
            bootstrap_max: "+1.5000".to_string(),
            bootstrap_interval_95_lower: "+1.5000".to_string(),
            bootstrap_interval_95_upper: "+1.5000".to_string(),
        }
    }

    fn sample_split_children() -> Vec<ResearchWalkForwardSplitChild> {
        vec![
            ResearchWalkForwardSplitChild {
                symbol: "ALPHA".to_string(),
                bundle_path: PathBuf::from("reports/alpha-bundle"),
            },
            ResearchWalkForwardSplitChild {
                symbol: "BETA".to_string(),
                bundle_path: PathBuf::from("reports/beta-bundle"),
            },
        ]
    }

    fn test_output_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("artifact crate lives under crates/");
        workspace_root
            .join("target")
            .join("test-output")
            .join(format!(
                "{label}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ))
    }
}
