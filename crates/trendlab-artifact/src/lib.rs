#![forbid(unsafe_code)]

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleDescriptor {
    pub schema_version: u32,
    pub manifest_path: String,
    pub summary_path: String,
    pub ledger_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: String,
    pub end_date: String,
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

    let descriptor = BundleDescriptor::canonical();

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
    validate_replay_bundle_parts(&manifest, &summary, &ledger)?;

    Ok(ReplayBundle {
        descriptor,
        manifest,
        summary,
        ledger,
    })
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
        BUNDLE_FILE_NAME, BundleDescriptor, DateRange, LEDGER_FILE_NAME, MANIFEST_FILE_NAME,
        ManifestParameter, PersistedLedgerRow, ReferenceFlowDefinition, ReplayBundle, RunManifest,
        RunSummary, SCHEMA_VERSION, SUMMARY_FILE_NAME, diff_replay_bundles, load_replay_bundle,
        write_json_lines, write_json_pretty, write_replay_bundle,
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

        assert_eq!(descriptor, BundleDescriptor::canonical());
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
