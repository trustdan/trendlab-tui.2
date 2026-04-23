#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use trendlab_artifact::{
    BundleDescriptor, DateRange, ManifestParameter, PersistedLedgerRow, ReferenceFlowDefinition,
    RunManifest, RunSummary, SCHEMA_VERSION, write_replay_bundle,
};
use trendlab_core::engine::{
    ReferenceFlowSpec, RunRequest, run_reference_flow, validate_run_request,
};
use trendlab_core::orders::{EntryIntent, GapPolicy};
use trendlab_data::provider::ProviderIdentity;
use trendlab_data::run_source::{SnapshotRunSliceRequest, resolve_snapshot_run_source};

pub const DEFAULT_ENGINE_VERSION: &str = "m1-reference-flow";
pub const STRATEGY_SIGNAL_PARAMETER: &str = "strategy.signal_id";
pub const STRATEGY_FILTER_PARAMETER: &str = "strategy.filter_id";
pub const STRATEGY_POSITION_PARAMETER: &str = "strategy.position_manager_id";
pub const STRATEGY_EXECUTION_PARAMETER: &str = "strategy.execution_model_id";
pub const RUN_SOURCE_KIND_PARAMETER: &str = "run_source_kind";
pub const RUN_REQUEST_SOURCE_PARAMETER: &str = "run_request_source";
pub const RUN_SPEC_SOURCE_PARAMETER: &str = "run_spec_source";
pub const SNAPSHOT_SOURCE_PATH_PARAMETER: &str = "snapshot_source_path";
pub const SNAPSHOT_SELECTION_START_PARAMETER: &str = "snapshot_selection_start_date";
pub const SNAPSHOT_SELECTION_END_PARAMETER: &str = "snapshot_selection_end_date";
pub const INLINE_TEMPLATE_REQUEST_SOURCE: &str = "inline_template";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorError {
    message: String,
}

impl OperatorError {
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

impl Display for OperatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OperatorError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunExecutionOptions {
    pub input_source: RunInputSource,
    pub output_dir: PathBuf,
    pub provider_identity: Option<ProviderIdentity>,
    pub snapshot_id: Option<String>,
    pub engine_version: Option<String>,
    pub strategy_components: Option<StrategyComponentLabels>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunInputSource {
    Request(PathBuf),
    Spec(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunExecutionOutcome {
    pub output_dir: PathBuf,
    pub descriptor: BundleDescriptor,
    pub manifest: RunManifest,
    pub summary: RunSummary,
    pub report: RunExecutionReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunExecutionReport {
    pub output_dir: PathBuf,
    pub snapshot_id: String,
    pub provider_identity: String,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub row_count: usize,
    pub warning_count: usize,
    pub ending_cash: f64,
    pub ending_equity: f64,
    pub provenance: RunExecutionProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSpecPreview {
    pub run_source_kind: RunSourceKind,
    pub request_source: String,
    pub spec_source: Option<String>,
    pub snapshot_source_path: Option<String>,
    pub snapshot_id: String,
    pub provider_identity: String,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub row_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunExecutionProvenance {
    pub run_source_kind: RunSourceKind,
    pub request_source: String,
    pub spec_source: Option<String>,
    pub snapshot_source_path: Option<String>,
    pub snapshot_selection_start_date: Option<String>,
    pub snapshot_selection_end_date: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyComponentLabels {
    pub signal_id: String,
    pub filter_id: String,
    pub position_manager_id: String,
    pub execution_model_id: String,
}

impl StrategyComponentLabels {
    pub fn system_id(&self) -> String {
        format!(
            "signal={} filter={} position={} execution={}",
            self.signal_id, self.filter_id, self.position_manager_id, self.execution_model_id
        )
    }

    pub fn manifest_parameters(&self) -> [ManifestParameter; 4] {
        [
            ManifestParameter {
                name: STRATEGY_SIGNAL_PARAMETER.to_string(),
                value: self.signal_id.clone(),
            },
            ManifestParameter {
                name: STRATEGY_FILTER_PARAMETER.to_string(),
                value: self.filter_id.clone(),
            },
            ManifestParameter {
                name: STRATEGY_POSITION_PARAMETER.to_string(),
                value: self.position_manager_id.clone(),
            },
            ManifestParameter {
                name: STRATEGY_EXECUTION_PARAMETER.to_string(),
                value: self.execution_model_id.clone(),
            },
        ]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorRunManifestSpec {
    pub provider_identity: Option<ProviderIdentity>,
    pub snapshot_id: Option<String>,
    pub engine_version: Option<String>,
    pub strategy_components: Option<StrategyComponentLabels>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorRunSpec {
    pub request_path: Option<String>,
    pub request: Option<RunRequest>,
    pub snapshot_source: Option<OperatorSnapshotSourceSpec>,
    pub request_template: Option<OperatorRunRequestTemplate>,
    #[serde(default)]
    pub manifest: OperatorRunManifestSpec,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorSnapshotSourceSpec {
    pub snapshot_dir: String,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorRunRequestTemplate {
    pub entry_intents: Vec<EntryIntent>,
    pub reference_flow: ReferenceFlowSpec,
    pub gap_policy: GapPolicy,
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedRunInput {
    request: RunRequest,
    request_source: String,
    spec_source: Option<String>,
    default_snapshot_source: PathBuf,
    provider_identity: ProviderIdentity,
    snapshot_id: Option<String>,
    engine_version: String,
    strategy_components: Option<StrategyComponentLabels>,
    run_source_kind: RunSourceKind,
    snapshot_source_provenance: Option<SnapshotSourceProvenance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunSourceKind {
    Request,
    Snapshot,
}

impl RunSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SnapshotSourceProvenance {
    snapshot_path: String,
    selection_start_date: String,
    selection_end_date: String,
}

pub fn execute_run(options: &RunExecutionOptions) -> Result<RunExecutionOutcome, OperatorError> {
    let input = resolve_run_input(options)?;
    execute_resolved_run(input, options.output_dir.clone())
}

pub fn execute_run_spec(
    spec: &OperatorRunSpec,
    output_dir: PathBuf,
) -> Result<RunExecutionOutcome, OperatorError> {
    let input = resolve_run_spec_definition(spec.clone(), None)?;
    execute_resolved_run(input, output_dir)
}

fn execute_resolved_run(
    input: ResolvedRunInput,
    output_dir: PathBuf,
) -> Result<RunExecutionOutcome, OperatorError> {
    let result = run_reference_flow(&input.request)
        .map_err(|err| OperatorError::invalid(err.to_string()))?;
    let manifest = build_manifest(&input)?;
    let summary = RunSummary {
        row_count: result.ledger.len(),
        warning_count: manifest.warnings.len(),
        ending_cash: result.cash.cash,
        ending_equity: result.cash.equity,
    };
    let report = build_execution_report(&output_dir, &input, &manifest, &summary);
    let ledger: Vec<PersistedLedgerRow> =
        result.ledger.iter().map(PersistedLedgerRow::from).collect();
    let descriptor = write_replay_bundle(&output_dir, &manifest, &summary, &ledger)
        .map_err(|err| OperatorError::invalid(err.to_string()))?;

    Ok(RunExecutionOutcome {
        output_dir,
        descriptor,
        manifest,
        summary,
        report,
    })
}

pub fn preview_run_spec(
    spec: &OperatorRunSpec,
    spec_path: Option<&Path>,
) -> Result<RunSpecPreview, OperatorError> {
    let input = resolve_run_spec_definition(spec.clone(), spec_path)?;
    validate_run_request(&input.request).map_err(|err| OperatorError::invalid(err.to_string()))?;

    let first_bar = input.request.bars.first().ok_or_else(|| {
        OperatorError::invalid("run requests must include at least one daily bar")
    })?;
    let last_bar = input.request.bars.last().ok_or_else(|| {
        OperatorError::invalid("run requests must include at least one daily bar")
    })?;

    Ok(RunSpecPreview {
        run_source_kind: input.run_source_kind,
        request_source: input.request_source,
        spec_source: input.spec_source,
        snapshot_source_path: input
            .snapshot_source_provenance
            .as_ref()
            .map(|source| source.snapshot_path.clone()),
        snapshot_id: input
            .snapshot_id
            .unwrap_or_else(|| default_snapshot_id(&input.default_snapshot_source)),
        provider_identity: input.provider_identity.as_str().to_string(),
        symbol: input.request.symbol,
        start_date: first_bar.date.clone(),
        end_date: last_bar.date.clone(),
        row_count: input.request.bars.len(),
    })
}

fn build_execution_report(
    output_dir: &Path,
    input: &ResolvedRunInput,
    manifest: &RunManifest,
    summary: &RunSummary,
) -> RunExecutionReport {
    let snapshot_source = input.snapshot_source_provenance.as_ref();

    RunExecutionReport {
        output_dir: output_dir.to_path_buf(),
        snapshot_id: manifest.data_snapshot_id.clone(),
        provider_identity: manifest.provider_identity.clone(),
        symbol: manifest.symbol_or_universe.clone(),
        start_date: manifest.date_range.start_date.clone(),
        end_date: manifest.date_range.end_date.clone(),
        row_count: summary.row_count,
        warning_count: summary.warning_count,
        ending_cash: summary.ending_cash,
        ending_equity: summary.ending_equity,
        provenance: RunExecutionProvenance {
            run_source_kind: input.run_source_kind,
            request_source: input.request_source.clone(),
            spec_source: input.spec_source.clone(),
            snapshot_source_path: snapshot_source.map(|source| source.snapshot_path.clone()),
            snapshot_selection_start_date: snapshot_source
                .map(|source| source.selection_start_date.clone()),
            snapshot_selection_end_date: snapshot_source
                .map(|source| source.selection_end_date.clone()),
        },
    }
}

fn read_run_request(path: &Path) -> Result<RunRequest, OperatorError> {
    let raw =
        fs::read_to_string(path).map_err(|err| OperatorError::io("failed to read", path, &err))?;
    serde_json::from_str(&raw).map_err(|err| OperatorError::json("failed to parse", path, &err))
}

fn read_run_spec(path: &Path) -> Result<OperatorRunSpec, OperatorError> {
    let raw =
        fs::read_to_string(path).map_err(|err| OperatorError::io("failed to read", path, &err))?;
    serde_json::from_str(&raw).map_err(|err| OperatorError::json("failed to parse", path, &err))
}

fn resolve_run_input(options: &RunExecutionOptions) -> Result<ResolvedRunInput, OperatorError> {
    match &options.input_source {
        RunInputSource::Request(request_path) => Ok(ResolvedRunInput {
            request: read_run_request(request_path)?,
            request_source: request_source_label_from_path(request_path),
            spec_source: None,
            default_snapshot_source: request_path.clone(),
            provider_identity: options
                .provider_identity
                .unwrap_or(ProviderIdentity::Fixture),
            snapshot_id: options.snapshot_id.clone(),
            engine_version: options
                .engine_version
                .clone()
                .unwrap_or_else(|| DEFAULT_ENGINE_VERSION.to_string()),
            strategy_components: options.strategy_components.clone(),
            run_source_kind: RunSourceKind::Request,
            snapshot_source_provenance: None,
        }),
        RunInputSource::Spec(spec_path) => resolve_run_spec_input(spec_path),
    }
}

fn resolve_run_spec_input(spec_path: &Path) -> Result<ResolvedRunInput, OperatorError> {
    let spec = read_run_spec(spec_path)?;
    resolve_run_spec_definition(spec, Some(spec_path))
}

fn resolve_run_spec_definition(
    spec: OperatorRunSpec,
    spec_path: Option<&Path>,
) -> Result<ResolvedRunInput, OperatorError> {
    let OperatorRunManifestSpec {
        provider_identity: manifest_provider_identity,
        snapshot_id: manifest_snapshot_id,
        engine_version: manifest_engine_version,
        strategy_components: manifest_strategy_components,
    } = spec.manifest;
    let (
        request,
        request_source,
        default_snapshot_source,
        provider_identity,
        snapshot_id,
        run_source_kind,
        snapshot_source_provenance,
    ) = match (
        spec.request_path,
        spec.request,
        spec.snapshot_source,
        spec.request_template,
    ) {
        (Some(request_path), None, None, None) => {
            let resolved_request_path = resolve_spec_relative_path(spec_path, &request_path);
            (
                read_run_request(&resolved_request_path)?,
                request_path,
                resolved_request_path,
                manifest_provider_identity.unwrap_or(ProviderIdentity::Fixture),
                manifest_snapshot_id.clone(),
                RunSourceKind::Request,
                None,
            )
        }
        (None, Some(request), None, None) => (
            request,
            "inline".to_string(),
            spec_path
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("run-spec.json")),
            manifest_provider_identity.unwrap_or(ProviderIdentity::Fixture),
            manifest_snapshot_id.clone(),
            RunSourceKind::Request,
            None,
        ),
        (None, None, Some(snapshot_source), Some(request_template)) => {
            if manifest_provider_identity.is_some() || manifest_snapshot_id.is_some() {
                return Err(OperatorError::invalid(
                    "snapshot-backed run specs must not override provider_identity or snapshot_id in manifest",
                ));
            }

            validate_snapshot_source_spec(&snapshot_source)?;
            let OperatorSnapshotSourceSpec {
                snapshot_dir,
                symbol,
                start_date,
                end_date,
            } = snapshot_source;
            let resolved_snapshot_dir = resolve_spec_relative_path(spec_path, &snapshot_dir);
            let resolved_source = resolve_snapshot_run_source(
                &resolved_snapshot_dir,
                &SnapshotRunSliceRequest {
                    symbol,
                    start_date,
                    end_date,
                },
            )
            .map_err(|err| {
                OperatorError::invalid(format!(
                    "resolve snapshot source {}: {err}",
                    resolved_snapshot_dir.display()
                ))
            })?;

            (
                RunRequest {
                    symbol: resolved_source.symbol.clone(),
                    bars: resolved_source.bars,
                    entry_intents: request_template.entry_intents,
                    reference_flow: request_template.reference_flow,
                    gap_policy: request_template.gap_policy,
                },
                INLINE_TEMPLATE_REQUEST_SOURCE.to_string(),
                resolved_snapshot_dir,
                resolved_source.provider_identity,
                Some(resolved_source.snapshot_id),
                RunSourceKind::Snapshot,
                Some(SnapshotSourceProvenance {
                    snapshot_path: snapshot_dir,
                    selection_start_date: resolved_source.selected_start_date,
                    selection_end_date: resolved_source.selected_end_date,
                }),
            )
        }
        (Some(_), Some(_), _, _)
        | (Some(_), _, Some(_), _)
        | (Some(_), _, _, Some(_))
        | (_, Some(_), Some(_), _)
        | (_, Some(_), _, Some(_)) => {
            return Err(OperatorError::invalid(
                "run spec must choose exactly one input mode: request_path, request, or snapshot_source + request_template",
            ));
        }
        (None, None, Some(_), None) | (None, None, None, Some(_)) => {
            return Err(OperatorError::invalid(
                "snapshot-backed run specs must define both `snapshot_source` and `request_template`",
            ));
        }
        (None, None, None, None) => {
            return Err(OperatorError::invalid(
                "run spec must include either `request_path`, `request`, or `snapshot_source` plus `request_template`",
            ));
        }
    };

    Ok(ResolvedRunInput {
        request,
        request_source,
        spec_source: spec_path.map(source_file_label),
        default_snapshot_source,
        provider_identity,
        snapshot_id,
        engine_version: manifest_engine_version
            .unwrap_or_else(|| DEFAULT_ENGINE_VERSION.to_string()),
        strategy_components: manifest_strategy_components,
        run_source_kind,
        snapshot_source_provenance,
    })
}

fn resolve_spec_relative_path(spec_path: Option<&Path>, raw_path: &str) -> PathBuf {
    let request_path = Path::new(raw_path);
    if request_path.is_absolute() {
        request_path.to_path_buf()
    } else {
        spec_path
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."))
            .join(request_path)
    }
}

fn request_source_label_from_path(path: &Path) -> String {
    source_file_label(path)
}

fn source_file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("request.json")
        .to_string()
}

fn validate_snapshot_source_spec(
    snapshot_source: &OperatorSnapshotSourceSpec,
) -> Result<(), OperatorError> {
    if snapshot_source.snapshot_dir.trim().is_empty() {
        return Err(OperatorError::invalid(
            "snapshot-backed run specs require a non-empty snapshot_source.snapshot_dir",
        ));
    }
    if snapshot_source.symbol.trim().is_empty() {
        return Err(OperatorError::invalid(
            "snapshot-backed run specs require a non-empty snapshot_source.symbol",
        ));
    }
    if snapshot_source.start_date.trim().is_empty() {
        return Err(OperatorError::invalid(
            "snapshot-backed run specs require a non-empty snapshot_source.start_date",
        ));
    }
    if snapshot_source.end_date.trim().is_empty() {
        return Err(OperatorError::invalid(
            "snapshot-backed run specs require a non-empty snapshot_source.end_date",
        ));
    }
    Ok(())
}

fn build_manifest(input: &ResolvedRunInput) -> Result<RunManifest, OperatorError> {
    let first_bar = input.request.bars.first().ok_or_else(|| {
        OperatorError::invalid("run requests must include at least one daily bar")
    })?;
    let last_bar = input.request.bars.last().ok_or_else(|| {
        OperatorError::invalid("run requests must include at least one daily bar")
    })?;
    let mut parameters = vec![
        ManifestParameter {
            name: RUN_SOURCE_KIND_PARAMETER.to_string(),
            value: input.run_source_kind.as_str().to_string(),
        },
        ManifestParameter {
            name: RUN_REQUEST_SOURCE_PARAMETER.to_string(),
            value: input.request_source.clone(),
        },
    ];
    if let Some(spec_source) = &input.spec_source {
        parameters.push(ManifestParameter {
            name: RUN_SPEC_SOURCE_PARAMETER.to_string(),
            value: spec_source.clone(),
        });
    }
    if let Some(snapshot_source) = &input.snapshot_source_provenance {
        parameters.push(ManifestParameter {
            name: SNAPSHOT_SOURCE_PATH_PARAMETER.to_string(),
            value: snapshot_source.snapshot_path.clone(),
        });
        parameters.push(ManifestParameter {
            name: SNAPSHOT_SELECTION_START_PARAMETER.to_string(),
            value: snapshot_source.selection_start_date.clone(),
        });
        parameters.push(ManifestParameter {
            name: SNAPSHOT_SELECTION_END_PARAMETER.to_string(),
            value: snapshot_source.selection_end_date.clone(),
        });
    }

    if let Some(strategy_components) = &input.strategy_components {
        parameters.extend(strategy_components.manifest_parameters());
    }

    Ok(RunManifest {
        schema_version: SCHEMA_VERSION,
        engine_version: input.engine_version.clone(),
        data_snapshot_id: input
            .snapshot_id
            .clone()
            .unwrap_or_else(|| default_snapshot_id(&input.default_snapshot_source)),
        provider_identity: input.provider_identity.as_str().to_string(),
        symbol_or_universe: input.request.symbol.clone(),
        universe_mode: "single_symbol".to_string(),
        historical_limitations: Vec::new(),
        date_range: DateRange {
            start_date: first_bar.date.clone(),
            end_date: last_bar.date.clone(),
        },
        reference_flow: ReferenceFlowDefinition {
            kind: "m1_reference_flow".to_string(),
            entry_shares: input.request.reference_flow.entry_shares,
            protective_stop_fraction: input.request.reference_flow.protective_stop_fraction,
        },
        parameters,
        cost_model: input.request.reference_flow.cost_model.clone(),
        gap_policy: input.request.gap_policy,
        seed: None,
        warnings: Vec::new(),
    })
}

fn default_snapshot_id(request_path: &Path) -> String {
    request_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| format!("adhoc:{value}"))
        .unwrap_or_else(|| "adhoc:request".to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use trendlab_artifact::load_replay_bundle;
    use trendlab_core::accounting::CostModel;
    use trendlab_core::engine::{ReferenceFlowSpec, RunRequest};
    use trendlab_core::market::DailyBar;
    use trendlab_core::orders::{EntryIntent, GapPolicy, OrderIntent};

    use crate::{
        DEFAULT_ENGINE_VERSION, OperatorRunManifestSpec, OperatorRunRequestTemplate,
        OperatorRunSpec, OperatorSnapshotSourceSpec, RUN_REQUEST_SOURCE_PARAMETER,
        RUN_SOURCE_KIND_PARAMETER, RUN_SPEC_SOURCE_PARAMETER, RunExecutionOptions, RunInputSource,
        RunSourceKind, SNAPSHOT_SELECTION_END_PARAMETER, SNAPSHOT_SELECTION_START_PARAMETER,
        SNAPSHOT_SOURCE_PATH_PARAMETER, execute_run, execute_run_spec, preview_run_spec,
    };

    #[test]
    fn execute_run_writes_replay_bundle_from_request_path() {
        let request_path = test_output_dir("operator-request").join("request.json");
        let output_dir = test_output_dir("operator-bundle");
        write_request(&request_path, &sample_request());

        let outcome = execute_run(&RunExecutionOptions {
            input_source: RunInputSource::Request(request_path.clone()),
            output_dir: output_dir.clone(),
            provider_identity: None,
            snapshot_id: None,
            engine_version: None,
            strategy_components: None,
        })
        .unwrap();

        assert_eq!(outcome.manifest.engine_version, DEFAULT_ENGINE_VERSION);
        assert_eq!(outcome.summary.row_count, 2);
        assert_eq!(outcome.output_dir, output_dir);
        assert!(output_dir.join("bundle.json").is_file());

        let bundle = load_replay_bundle(&output_dir).unwrap();
        assert_eq!(bundle.manifest.symbol_or_universe, "TEST");
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER),
            "request"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "request.json"
        );

        remove_dir_all_if_exists(request_path.parent().unwrap());
        remove_dir_all_if_exists(&output_dir);
    }

    #[test]
    fn execute_run_preserves_inline_spec_provenance_in_report() {
        let spec_path = test_output_dir("operator-inline-spec").join("run-spec.json");
        let output_dir = test_output_dir("operator-inline-bundle");
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: None,
                request: Some(sample_request()),
                snapshot_source: None,
                request_template: None,
                manifest: OperatorRunManifestSpec {
                    provider_identity: None,
                    snapshot_id: Some("snapshot:inline".to_string()),
                    engine_version: Some("operator-inline".to_string()),
                    strategy_components: None,
                },
            },
        );

        let outcome = execute_run(&RunExecutionOptions {
            input_source: RunInputSource::Spec(spec_path.clone()),
            output_dir: output_dir.clone(),
            provider_identity: None,
            snapshot_id: None,
            engine_version: None,
            strategy_components: None,
        })
        .unwrap();

        assert_eq!(outcome.report.snapshot_id, "snapshot:inline");
        assert_eq!(outcome.report.provider_identity, "fixture");
        assert_eq!(
            outcome.report.provenance.run_source_kind,
            RunSourceKind::Request
        );
        assert_eq!(outcome.report.provenance.request_source, "inline");
        assert_eq!(
            outcome.report.provenance.spec_source.as_deref(),
            Some("run-spec.json")
        );
        assert_eq!(
            manifest_parameter_value(&outcome.manifest, RUN_SPEC_SOURCE_PARAMETER),
            "run-spec.json"
        );

        remove_dir_all_if_exists(spec_path.parent().unwrap());
        remove_dir_all_if_exists(&output_dir);
    }

    #[test]
    fn execute_run_resolves_relative_request_path_spec() {
        let spec_dir = test_output_dir("operator-relative-spec");
        let request_path = spec_dir.join("inputs").join("request.json");
        let spec_path = spec_dir.join("run-spec.json");
        let output_dir = test_output_dir("operator-relative-bundle");
        write_request(&request_path, &sample_request());
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: Some("inputs/request.json".to_string()),
                request: None,
                snapshot_source: None,
                request_template: None,
                manifest: OperatorRunManifestSpec::default(),
            },
        );

        let outcome = execute_run(&RunExecutionOptions {
            input_source: RunInputSource::Spec(spec_path.clone()),
            output_dir: output_dir.clone(),
            provider_identity: None,
            snapshot_id: None,
            engine_version: None,
            strategy_components: None,
        })
        .unwrap();

        assert_eq!(outcome.report.snapshot_id, "adhoc:request");
        assert_eq!(
            outcome.report.provenance.request_source,
            "inputs/request.json"
        );
        assert_eq!(
            outcome.report.provenance.spec_source.as_deref(),
            Some("run-spec.json")
        );
        assert_eq!(
            manifest_parameter_value(&outcome.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "inputs/request.json"
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&output_dir);
    }

    #[test]
    fn execute_run_resolves_snapshot_spec_with_caller_safe_provenance() {
        let spec_dir = test_output_dir("operator-snapshot-spec");
        let snapshot_dir = spec_dir.join("snapshots").join("sample");
        let spec_path = spec_dir.join("run-spec.json");
        let output_dir = test_output_dir("operator-snapshot-bundle");
        write_sample_snapshot_bundle(&snapshot_dir);
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: None,
                request: None,
                snapshot_source: Some(OperatorSnapshotSourceSpec {
                    snapshot_dir: "snapshots/sample".to_string(),
                    symbol: "TEST".to_string(),
                    start_date: "2025-01-03".to_string(),
                    end_date: "2025-01-07".to_string(),
                }),
                request_template: Some(sample_request_template()),
                manifest: OperatorRunManifestSpec {
                    provider_identity: None,
                    snapshot_id: None,
                    engine_version: Some("operator-snapshot".to_string()),
                    strategy_components: None,
                },
            },
        );

        let outcome = execute_run(&RunExecutionOptions {
            input_source: RunInputSource::Spec(spec_path.clone()),
            output_dir: output_dir.clone(),
            provider_identity: None,
            snapshot_id: None,
            engine_version: None,
            strategy_components: None,
        })
        .unwrap();

        assert_eq!(
            outcome.report.provenance.run_source_kind,
            RunSourceKind::Snapshot
        );
        assert_eq!(outcome.report.provenance.request_source, "inline_template");
        assert_eq!(
            outcome.report.snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(outcome.report.provider_identity, "tiingo");
        assert_eq!(outcome.report.start_date, "2025-01-03");
        assert_eq!(outcome.report.end_date, "2025-01-07");
        assert_eq!(outcome.report.row_count, 3);
        assert_eq!(
            outcome.report.provenance.snapshot_source_path.as_deref(),
            Some("snapshots/sample")
        );
        assert_eq!(
            outcome
                .report
                .provenance
                .snapshot_selection_start_date
                .as_deref(),
            Some("2025-01-03")
        );
        assert_eq!(
            outcome
                .report
                .provenance
                .snapshot_selection_end_date
                .as_deref(),
            Some("2025-01-07")
        );
        assert_eq!(
            manifest_parameter_value(&outcome.manifest, SNAPSHOT_SOURCE_PATH_PARAMETER),
            "snapshots/sample"
        );
        assert_eq!(
            manifest_parameter_value(&outcome.manifest, SNAPSHOT_SELECTION_START_PARAMETER),
            "2025-01-03"
        );
        assert_eq!(
            manifest_parameter_value(&outcome.manifest, SNAPSHOT_SELECTION_END_PARAMETER),
            "2025-01-07"
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&output_dir);
    }

    #[test]
    fn preview_run_spec_validates_inline_snapshot_spec_without_execution() {
        let snapshot_dir = test_output_dir("operator-preview-snapshot");
        write_sample_snapshot_bundle(&snapshot_dir);

        let preview = preview_run_spec(
            &OperatorRunSpec {
                request_path: None,
                request: None,
                snapshot_source: Some(OperatorSnapshotSourceSpec {
                    snapshot_dir: snapshot_dir.display().to_string(),
                    symbol: "TEST".to_string(),
                    start_date: "2025-01-03".to_string(),
                    end_date: "2025-01-07".to_string(),
                }),
                request_template: Some(sample_request_template()),
                manifest: OperatorRunManifestSpec::default(),
            },
            None,
        )
        .unwrap();

        assert_eq!(preview.run_source_kind, RunSourceKind::Snapshot);
        assert_eq!(preview.request_source, "inline_template");
        assert_eq!(
            preview.snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(preview.provider_identity, "tiingo");
        assert_eq!(preview.symbol, "TEST");
        assert_eq!(preview.start_date, "2025-01-03");
        assert_eq!(preview.end_date, "2025-01-07");
        assert_eq!(preview.row_count, 3);
        let expected_path = snapshot_dir.display().to_string();
        assert_eq!(
            preview.snapshot_source_path.as_deref(),
            Some(expected_path.as_str())
        );

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn preview_run_spec_rejects_invalid_request_template_inputs() {
        let snapshot_dir = test_output_dir("operator-preview-invalid-template");
        write_sample_snapshot_bundle(&snapshot_dir);

        let error = preview_run_spec(
            &OperatorRunSpec {
                request_path: None,
                request: None,
                snapshot_source: Some(OperatorSnapshotSourceSpec {
                    snapshot_dir: snapshot_dir.display().to_string(),
                    symbol: "TEST".to_string(),
                    start_date: "2025-01-03".to_string(),
                    end_date: "2025-01-07".to_string(),
                }),
                request_template: Some(OperatorRunRequestTemplate {
                    entry_intents: vec![EntryIntent {
                        signal_date: "2025-01-06".to_string(),
                        intent: OrderIntent::QueueMarketEntry,
                        shares: 0,
                    }],
                    reference_flow: ReferenceFlowSpec {
                        initial_cash: 1000.0,
                        entry_shares: 0,
                        protective_stop_fraction: 0.10,
                        cost_model: CostModel::default(),
                    },
                    gap_policy: GapPolicy::M1Default,
                }),
                manifest: OperatorRunManifestSpec::default(),
            },
            None,
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "reference flow entry_shares must be greater than zero"
        );

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn execute_run_spec_executes_inline_snapshot_spec_without_cli_path() {
        let snapshot_dir = test_output_dir("operator-inline-launch-snapshot");
        let output_dir = test_output_dir("operator-inline-launch-bundle");
        write_sample_snapshot_bundle(&snapshot_dir);

        let outcome = execute_run_spec(
            &OperatorRunSpec {
                request_path: None,
                request: None,
                snapshot_source: Some(OperatorSnapshotSourceSpec {
                    snapshot_dir: snapshot_dir.display().to_string(),
                    symbol: "TEST".to_string(),
                    start_date: "2025-01-03".to_string(),
                    end_date: "2025-01-07".to_string(),
                }),
                request_template: Some(sample_request_template()),
                manifest: OperatorRunManifestSpec::default(),
            },
            output_dir.clone(),
        )
        .unwrap();

        assert_eq!(outcome.output_dir, output_dir);
        assert!(outcome.output_dir.join("bundle.json").is_file());
        assert_eq!(
            outcome.report.snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(outcome.report.start_date, "2025-01-03");
        assert_eq!(outcome.report.end_date, "2025-01-07");

        remove_dir_all_if_exists(&snapshot_dir);
        remove_dir_all_if_exists(&output_dir);
    }

    fn sample_request() -> RunRequest {
        RunRequest {
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
        }
    }

    fn sample_request_template() -> OperatorRunRequestTemplate {
        OperatorRunRequestTemplate {
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
        }
    }

    fn write_request(path: &Path, request: &RunRequest) {
        fs::create_dir_all(
            path.parent()
                .expect("request.json should have a parent directory"),
        )
        .unwrap();
        fs::write(path, serde_json::to_vec_pretty(request).unwrap()).unwrap();
    }

    fn write_run_spec(path: &Path, spec: &OperatorRunSpec) {
        fs::create_dir_all(
            path.parent()
                .expect("run-spec.json should have a parent directory"),
        )
        .unwrap();
        fs::write(path, serde_json::to_vec_pretty(spec).unwrap()).unwrap();
    }

    fn manifest_parameter_value<'a>(
        manifest: &'a trendlab_artifact::RunManifest,
        name: &str,
    ) -> &'a str {
        manifest
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .map(|parameter| parameter.value.as_str())
            .unwrap_or_else(|| panic!("missing manifest parameter `{name}`"))
    }

    fn remove_dir_all_if_exists(path: &Path) {
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
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
            .expect("trendlab-operator lives under crates/");
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
