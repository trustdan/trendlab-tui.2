#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use trendlab_artifact::{
    BootstrapDistributionSummary, LeaderboardView, ManifestParameter, PersistedLedgerRow,
    ReplayBundle, ResearchAggregateMember, ResearchAggregateReport,
    ResearchBootstrapAggregateReport, ResearchBootstrapWalkForwardReport,
    ResearchBootstrapWalkForwardSplit, ResearchLeaderboardReport, ResearchLeaderboardRow,
    ResearchReport, ResearchWalkForwardReport, ResearchWalkForwardSplit,
    ResearchWalkForwardSplitChild, RunManifest, SCHEMA_VERSION, diff_replay_bundles,
    load_replay_bundle, load_research_report_bundle, write_research_report_bundle,
};
use trendlab_data::audit::audit_daily_bars;
use trendlab_data::inspect::inspect_snapshot_bundle;
use trendlab_data::provider::ProviderIdentity;
use trendlab_data::snapshot_store::load_snapshot_bundle;
use trendlab_operator::{
    RUN_REQUEST_SOURCE_PARAMETER, RUN_SOURCE_KIND_PARAMETER, RUN_SPEC_SOURCE_PARAMETER,
    RunExecutionOptions, RunInputSource, RunSourceKind, SNAPSHOT_SELECTION_END_PARAMETER,
    SNAPSHOT_SELECTION_START_PARAMETER, SNAPSHOT_SOURCE_PATH_PARAMETER,
    STRATEGY_EXECUTION_PARAMETER, STRATEGY_FILTER_PARAMETER, STRATEGY_POSITION_PARAMETER,
    STRATEGY_SIGNAL_PARAMETER, StrategyComponentLabels, execute_run,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliResponse {
    pub exit_code: u8,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CliError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunOptions {
    input_source: RunInputSource,
    output_dir: PathBuf,
    provider_identity: Option<ProviderIdentity>,
    snapshot_id: Option<String>,
    engine_version: Option<String>,
    strategy_components: Option<StrategyComponentLabels>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResearchAggregateOptions {
    bundle_dirs: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
struct ComparableResearchBundle {
    symbol: String,
    bundle_path: PathBuf,
    bundle: ReplayBundle,
}

#[derive(Clone, Debug, PartialEq)]
struct ComparableResearchBundleSet {
    engine_version: String,
    snapshot_id: String,
    provider_identity: String,
    date_range: String,
    gap_policy: String,
    historical_limitations: String,
    members: Vec<ComparableResearchBundle>,
}

#[derive(Clone, Debug, PartialEq)]
struct AttributedResearchBundle {
    member: ComparableResearchBundle,
    strategy_components: StrategyComponentLabels,
}

#[derive(Clone, Debug, PartialEq)]
struct ComparableAttributedResearchBundleSet {
    engine_version: String,
    snapshot_id: String,
    provider_identity: String,
    date_range: String,
    gap_policy: String,
    historical_limitations: String,
    members: Vec<AttributedResearchBundle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WalkForwardOptions {
    train_bars: usize,
    test_bars: usize,
    step_bars: usize,
    bundle_dirs: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapOptions {
    samples: usize,
    seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapAggregateOptions {
    bootstrap: BootstrapOptions,
    bundle_dirs: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapWalkForwardOptions {
    bootstrap: BootstrapOptions,
    walk_forward: WalkForwardOptions,
    output_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LeaderboardOptions {
    view: LeaderboardView,
    bundle_dirs: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

#[derive(Clone, Copy)]
struct ResearchBundleExpectation<'a> {
    engine_version: &'a str,
    snapshot_id: &'a str,
    provider_identity: &'a str,
    date_range: &'a str,
    gap_policy: &'a str,
    historical_limitations: &'a str,
}

const RUN_USAGE: &str = "usage: trendlab-cli run (--request <path> [--provider <fixture|tiingo>] [--snapshot-id <id>] [--engine-version <version>] [--signal-id <id> --filter-id <id> --position-manager-id <id> --execution-model-id <id>] | --spec <path>) --output <dir>";
const EXPLAIN_USAGE: &str = "usage: trendlab-cli explain <bundle-dir>";
const DIFF_USAGE: &str = "usage: trendlab-cli diff <left-bundle-dir> <right-bundle-dir>";
const AUDIT_DATA_USAGE: &str = "usage: trendlab-cli audit data <bundle-dir>";
const AUDIT_SNAPSHOT_USAGE: &str = "usage: trendlab-cli audit snapshot <snapshot-dir>";
const RESEARCH_AGGREGATE_USAGE: &str = "usage: trendlab-cli research aggregate [--output <dir>] <bundle-dir> <bundle-dir> [more-bundle-dirs...]";
const RESEARCH_EXPLAIN_USAGE: &str = "usage: trendlab-cli research explain <report-dir>";
const RESEARCH_WALK_FORWARD_USAGE: &str = "usage: trendlab-cli research walk-forward --train-bars <n> --test-bars <n> [--step-bars <n>] [--output <dir>] <bundle-dir> <bundle-dir> [more-bundle-dirs...]";
const RESEARCH_BOOTSTRAP_AGGREGATE_USAGE: &str = "usage: trendlab-cli research bootstrap aggregate --samples <n> [--seed <n>] [--output <dir>] <bundle-dir> <bundle-dir> [more-bundle-dirs...]";
const RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE: &str = "usage: trendlab-cli research bootstrap walk-forward --samples <n> [--seed <n>] --train-bars <n> --test-bars <n> [--step-bars <n>] [--output <dir>] <bundle-dir> <bundle-dir> [more-bundle-dirs...]";
const RESEARCH_LEADERBOARD_USAGE: &str = "usage: trendlab-cli research leaderboard <signal|position-manager|execution-model|system> [--output <dir>] <bundle-dir> <bundle-dir> [more-bundle-dirs...]";

pub fn dispatch<I, S>(args: I) -> CliResponse
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let mut iter = args.into_iter();

    match iter.next().as_deref() {
        Some("run") => respond(run_command(iter.collect())),
        Some("explain") => respond(explain_command(iter.collect())),
        Some("diff") => respond(diff_command(iter.collect())),
        Some("audit") => respond(audit_command(iter.collect())),
        Some("research") => respond(research_command(iter.collect())),
        _ => usage_response(),
    }
}

fn respond(result: Result<String, CliError>) -> CliResponse {
    match result {
        Ok(stdout) => CliResponse {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        },
        Err(err) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn usage_response() -> CliResponse {
    CliResponse {
        exit_code: 1,
        stdout: String::new(),
        stderr: usage_text(),
    }
}

fn run_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_run_options(args)?;
    let outcome = execute_run(&RunExecutionOptions {
        input_source: options.input_source,
        output_dir: options.output_dir,
        provider_identity: options.provider_identity,
        snapshot_id: options.snapshot_id,
        engine_version: options.engine_version,
        strategy_components: options.strategy_components,
    })
    .map_err(|err| CliError::invalid(err.to_string()))?;

    Ok(format!(
        "wrote replay bundle to {}\nsnapshot_id: {}\nsymbol: {}\nprovider: {}\nrows: {}\nending_cash: {:.4}\nending_equity: {:.4}",
        outcome.report.output_dir.display(),
        outcome.report.snapshot_id,
        outcome.report.symbol,
        outcome.report.provider_identity,
        outcome.report.row_count,
        outcome.report.ending_cash,
        outcome.report.ending_equity
    ))
}

fn explain_command(args: Vec<String>) -> Result<String, CliError> {
    let [bundle_dir]: [String; 1] = args
        .try_into()
        .map_err(|_| CliError::invalid(EXPLAIN_USAGE))?;
    let bundle_dir = PathBuf::from(bundle_dir);
    let bundle = load_bundle(&bundle_dir)?;
    let run_source_kind = manifest_parameter(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER)
        .map(|parameter| parameter.value.as_str())
        .unwrap_or("request");
    let mut lines = vec![
        format!("bundle: {}", bundle_dir.display()),
        format!("schema_version: {}", bundle.descriptor.schema_version),
        format!("engine_version: {}", bundle.manifest.engine_version),
        format!("snapshot_id: {}", bundle.manifest.data_snapshot_id),
        format!("provider: {}", bundle.manifest.provider_identity),
        format!("run_source_kind: {run_source_kind}"),
        format!("symbol: {}", bundle.manifest.symbol_or_universe),
        format!("universe_mode: {}", bundle.manifest.universe_mode),
        format!(
            "date_range: {}..{}",
            bundle.manifest.date_range.start_date, bundle.manifest.date_range.end_date
        ),
        format!(
            "request_source: {}",
            manifest_parameter_value_or(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER, "none")
        ),
        format!(
            "spec_source: {}",
            manifest_parameter_value_or(&bundle.manifest, RUN_SPEC_SOURCE_PARAMETER, "none")
        ),
        format!("gap_policy: {}", bundle.manifest.gap_policy.as_str()),
        format!(
            "reference_flow: kind={} entry_shares={} protective_stop_fraction={:.4}",
            bundle.manifest.reference_flow.kind,
            bundle.manifest.reference_flow.entry_shares,
            bundle.manifest.reference_flow.protective_stop_fraction,
        ),
        format!(
            "cost_model: commission_per_fill={:.4} slippage_per_share={:.4}",
            bundle.manifest.cost_model.commission_per_fill,
            bundle.manifest.cost_model.slippage_per_share
        ),
        format!(
            "warnings: {}",
            format_string_list(&bundle.manifest.warnings)
        ),
        format!("rows: {}", bundle.summary.row_count),
        format!("warning_count: {}", bundle.summary.warning_count),
        format!("ending_cash: {:.4}", bundle.summary.ending_cash),
        format!("ending_equity: {:.4}", bundle.summary.ending_equity),
    ];

    if run_source_kind == RunSourceKind::Snapshot.as_str() {
        lines.push(format!(
            "snapshot_source_path: {}",
            manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SOURCE_PATH_PARAMETER, "none")
        ));
        lines.push(format!(
            "snapshot_selection: {}..{}",
            manifest_parameter_value_or(
                &bundle.manifest,
                SNAPSHOT_SELECTION_START_PARAMETER,
                "none"
            ),
            manifest_parameter_value_or(&bundle.manifest, SNAPSHOT_SELECTION_END_PARAMETER, "none")
        ));
    }

    lines.extend([
        format!(
            "parameters: {}",
            format_manifest_parameters(&bundle.manifest.parameters)
        ),
        "ledger:".to_string(),
    ]);

    for row in &bundle.ledger {
        lines.push(format!(
            "{} shares={} signal={} filter={} pending={} fill={} prior_stop={} next_stop={} cash={:.4} equity={:.4} reasons={}",
            row.date,
            row.position_shares,
            row.signal_output,
            row.filter_outcome,
            row.pending_order_state,
            format_optional(row.fill_price),
            format_optional(row.prior_stop),
            format_optional(row.next_stop),
            row.cash,
            row.equity,
            format_reason_codes(&row.reason_codes),
        ));
    }

    Ok(lines.join("\n"))
}

fn diff_command(args: Vec<String>) -> Result<String, CliError> {
    let [left_bundle_dir, right_bundle_dir]: [String; 2] =
        args.try_into().map_err(|_| CliError::invalid(DIFF_USAGE))?;
    let left_bundle_dir = PathBuf::from(left_bundle_dir);
    let right_bundle_dir = PathBuf::from(right_bundle_dir);
    let left_bundle = load_bundle(&left_bundle_dir)?;
    let right_bundle = load_bundle(&right_bundle_dir)?;
    let diff = diff_replay_bundles(&left_bundle, &right_bundle);
    let mut lines = vec![
        format!("left: {}", left_bundle_dir.display()),
        format!("right: {}", right_bundle_dir.display()),
    ];

    if diff.is_empty() {
        lines.push("equal: yes".to_string());
        return Ok(lines.join("\n"));
    }

    lines.push("equal: no".to_string());
    lines.push(format!("manifest_diffs: {}", diff.manifest_diffs.len()));
    lines.push(format!("summary_diffs: {}", diff.summary_diffs.len()));
    lines.push(format!("ledger_row_diffs: {}", diff.ledger_row_diffs.len()));

    for entry in diff.manifest_diffs {
        lines.push(format!(
            "manifest.{}: left={} right={}",
            entry.field, entry.left, entry.right
        ));
    }

    for entry in diff.summary_diffs {
        lines.push(format!(
            "summary.{}: left={} right={}",
            entry.field, entry.left, entry.right
        ));
    }

    for row in diff.ledger_row_diffs {
        let left_date = format_optional_text(row.left_date.as_deref());
        let right_date = format_optional_text(row.right_date.as_deref());
        for entry in row.field_diffs {
            lines.push(format!(
                "ledger[{}].{}: left_date={} right_date={} left={} right={}",
                row.index, entry.field, left_date, right_date, entry.left, entry.right
            ));
        }
    }

    Ok(lines.join("\n"))
}

fn audit_command(args: Vec<String>) -> Result<String, CliError> {
    let mut iter = args.into_iter();

    match iter.next().as_deref() {
        Some("data") => audit_data_command(iter.collect()),
        Some("snapshot") => audit_snapshot_command(iter.collect()),
        _ => Err(CliError::invalid(format!(
            "{}\n{}",
            AUDIT_DATA_USAGE, AUDIT_SNAPSHOT_USAGE
        ))),
    }
}

fn research_command(args: Vec<String>) -> Result<String, CliError> {
    let mut iter = args.into_iter();

    match iter.next().as_deref() {
        Some("aggregate") => research_aggregate_command(iter.collect()),
        Some("explain") => research_explain_command(iter.collect()),
        Some("walk-forward") => research_walk_forward_command(iter.collect()),
        Some("bootstrap") => research_bootstrap_command(iter.collect()),
        Some("leaderboard") => research_leaderboard_command(iter.collect()),
        _ => Err(CliError::invalid(format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            RESEARCH_AGGREGATE_USAGE,
            RESEARCH_EXPLAIN_USAGE,
            RESEARCH_WALK_FORWARD_USAGE,
            RESEARCH_BOOTSTRAP_AGGREGATE_USAGE,
            RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
            RESEARCH_LEADERBOARD_USAGE
        ))),
    }
}

fn audit_data_command(args: Vec<String>) -> Result<String, CliError> {
    let [bundle_dir]: [String; 1] = args
        .try_into()
        .map_err(|_| CliError::invalid(AUDIT_DATA_USAGE))?;
    let bundle_dir = PathBuf::from(bundle_dir);
    let bundle = load_bundle(&bundle_dir)?;
    let bars: Vec<_> = bundle
        .ledger
        .iter()
        .map(PersistedLedgerRow::market_bar)
        .collect();
    let report = audit_daily_bars(&bars);
    let mut lines = vec![
        format!("bundle: {}", bundle_dir.display()),
        format!("snapshot_id: {}", bundle.manifest.data_snapshot_id),
        format!("provider: {}", bundle.manifest.provider_identity),
        format!("symbol: {}", bundle.manifest.symbol_or_universe),
        format!("bars: {}", report.bar_count),
        format!(
            "date_range: {}..{}",
            format_optional_text(report.start_date.as_deref()),
            format_optional_text(report.end_date.as_deref())
        ),
        format!(
            "analysis_adjusted_bars: {}",
            report.analysis_adjusted_bar_count
        ),
        format!(
            "analysis_matches_raw_close: {}",
            report.analysis_matches_raw_close_count
        ),
        format!(
            "max_analysis_close_gap: {}",
            format_optional(report.max_analysis_close_gap)
        ),
        format!(
            "max_analysis_close_gap_date: {}",
            format_optional_text(report.max_analysis_close_gap_date.as_deref())
        ),
    ];

    if report.findings.is_empty() {
        lines.push("findings: none".to_string());
    } else {
        lines.push(format!("findings: {}", report.findings.len()));
        for finding in report.findings {
            lines.push(format!(
                "finding: date={} code={} detail={}",
                format_optional_text(finding.date.as_deref()),
                finding.code,
                finding.detail
            ));
        }
    }

    Ok(lines.join("\n"))
}

fn audit_snapshot_command(args: Vec<String>) -> Result<String, CliError> {
    let [snapshot_dir]: [String; 1] = args
        .try_into()
        .map_err(|_| CliError::invalid(AUDIT_SNAPSHOT_USAGE))?;
    let snapshot_dir = PathBuf::from(snapshot_dir);
    let bundle = load_snapshot_bundle(&snapshot_dir).map_err(|err| {
        CliError::invalid(format!(
            "load snapshot bundle {}: {err}",
            snapshot_dir.display()
        ))
    })?;
    let report = inspect_snapshot_bundle(&bundle).map_err(|err| {
        CliError::invalid(format!(
            "inspect snapshot bundle {}: {err}",
            snapshot_dir.display()
        ))
    })?;

    let mut lines = vec![
        format!("snapshot: {}", snapshot_dir.display()),
        format!("snapshot_id: {}", report.snapshot_id),
        format!("provider: {}", report.provider_identity.as_str()),
        format!(
            "requested_window: {}..{}",
            report.requested_start_date, report.requested_end_date
        ),
        format!("capture_mode: {}", report.capture_mode),
        format!("entrypoint: {}", report.entrypoint),
        format!(
            "captured_at_unix_epoch_seconds: {}",
            format_optional_u64(report.captured_at_unix_epoch_seconds)
        ),
        format!("symbols: {}", report.symbol_count),
    ];

    for symbol in report.symbols {
        lines.push(format!("symbol: {}", symbol.symbol));
        lines.push(format!(
            "raw_window: {}..{}",
            format_optional_text(symbol.raw_start_date.as_deref()),
            format_optional_text(symbol.raw_end_date.as_deref())
        ));
        lines.push(format!("raw_bars: {}", symbol.raw_bar_count));
        lines.push(format!(
            "corporate_actions: {}",
            symbol.corporate_action_count
        ));
        lines.push(format!("split_actions: {}", symbol.split_action_count));
        lines.push(format!(
            "cash_dividends: {}",
            symbol.cash_dividend_action_count
        ));
        lines.push(format!(
            "normalized_bars: daily={} weekly={} monthly={}",
            symbol.normalized_daily_bar_count, symbol.weekly_bar_count, symbol.monthly_bar_count
        ));
        lines.push(format!(
            "analysis_adjusted_bars: {}",
            symbol.analysis_adjusted_bar_count
        ));
        lines.push(format!(
            "analysis_matches_raw_close: {}",
            symbol.analysis_matches_raw_close_count
        ));
        lines.push(format!(
            "max_analysis_close_gap: {}",
            format_optional(symbol.max_analysis_close_gap)
        ));
        lines.push(format!(
            "max_analysis_close_gap_date: {}",
            format_optional_text(symbol.max_analysis_close_gap_date.as_deref())
        ));

        if symbol.corporate_action_effects.is_empty() {
            lines.push("normalization_inputs: none".to_string());
        } else {
            lines.push(format!(
                "normalization_inputs: {}",
                symbol.corporate_action_effects.len()
            ));
            for effect in symbol.corporate_action_effects {
                lines.push(format!(
                    "normalization_input: ex_date={} split_ratio={:.4} cash_dividend_per_share={:.4}",
                    effect.ex_date, effect.split_ratio, effect.cash_dividend_per_share
                ));
            }
        }

        if symbol.findings.is_empty() {
            lines.push("findings: none".to_string());
        } else {
            lines.push(format!("findings: {}", symbol.findings.len()));
            for finding in symbol.findings {
                lines.push(format!(
                    "finding: date={} code={} detail={}",
                    format_optional_text(finding.date.as_deref()),
                    finding.code,
                    finding.detail
                ));
            }
        }
    }

    Ok(lines.join("\n"))
}

fn research_aggregate_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_research_aggregate_options(args)?;
    let bundles = options
        .bundle_dirs
        .iter()
        .cloned()
        .map(|bundle_dir| load_bundle(&bundle_dir).map(|bundle| (bundle_dir, bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    let comparable_set = build_comparable_research_bundle_set(&bundles)?;
    let report = ResearchReport::Aggregate(build_research_aggregate_report(&comparable_set));

    emit_research_report(&report, options.output_dir.as_deref())
}

fn research_explain_command(args: Vec<String>) -> Result<String, CliError> {
    let [report_dir]: [String; 1] = args
        .try_into()
        .map_err(|_| CliError::invalid(RESEARCH_EXPLAIN_USAGE))?;
    let report_dir = PathBuf::from(report_dir);
    let report = load_research_report(&report_dir)?;

    Ok(format_saved_research_report(&report_dir, &report))
}

fn research_walk_forward_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_walk_forward_options(args)?;
    let bundles = options
        .bundle_dirs
        .iter()
        .cloned()
        .map(|bundle_dir| load_bundle(&bundle_dir).map(|bundle| (bundle_dir, bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    let comparable_set = build_comparable_research_bundle_set(&bundles)?;
    let report = ResearchReport::WalkForward(build_research_walk_forward_report(
        &comparable_set,
        &options,
    )?);

    emit_research_report(&report, options.output_dir.as_deref())
}

fn research_bootstrap_command(args: Vec<String>) -> Result<String, CliError> {
    let mut iter = args.into_iter();

    match iter.next().as_deref() {
        Some("aggregate") => research_bootstrap_aggregate_command(iter.collect()),
        Some("walk-forward") => research_bootstrap_walk_forward_command(iter.collect()),
        _ => Err(CliError::invalid(format!(
            "{}\n{}",
            RESEARCH_BOOTSTRAP_AGGREGATE_USAGE, RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE
        ))),
    }
}

fn research_bootstrap_aggregate_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_bootstrap_aggregate_options(args)?;
    let bundles = options
        .bundle_dirs
        .iter()
        .cloned()
        .map(|bundle_dir| load_bundle(&bundle_dir).map(|bundle| (bundle_dir, bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    let comparable_set = build_comparable_research_bundle_set(&bundles)?;
    let report = ResearchReport::BootstrapAggregate(build_research_bootstrap_aggregate_report(
        &comparable_set,
        &options.bootstrap,
    ));

    emit_research_report(&report, options.output_dir.as_deref())
}

fn research_bootstrap_walk_forward_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_bootstrap_walk_forward_options(args)?;
    let bundles = options
        .walk_forward
        .bundle_dirs
        .iter()
        .cloned()
        .map(|bundle_dir| load_bundle(&bundle_dir).map(|bundle| (bundle_dir, bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    let comparable_set = build_comparable_research_bundle_set(&bundles)?;
    let report =
        ResearchReport::BootstrapWalkForward(build_research_bootstrap_walk_forward_report(
            &comparable_set,
            &options.walk_forward,
            &options.bootstrap,
        )?);

    emit_research_report(&report, options.output_dir.as_deref())
}

fn research_leaderboard_command(args: Vec<String>) -> Result<String, CliError> {
    let options = parse_leaderboard_options(args)?;
    let bundles = options
        .bundle_dirs
        .iter()
        .cloned()
        .map(|bundle_dir| load_bundle(&bundle_dir).map(|bundle| (bundle_dir, bundle)))
        .collect::<Result<Vec<_>, _>>()?;
    let comparable_set = build_comparable_attributed_research_bundle_set(&bundles)?;
    let report = ResearchReport::Leaderboard(build_research_leaderboard_report(
        &comparable_set,
        options.view,
    )?);

    emit_research_report(&report, options.output_dir.as_deref())
}

fn emit_research_report(
    report: &ResearchReport,
    output_dir: Option<&Path>,
) -> Result<String, CliError> {
    if let Some(output_dir) = output_dir {
        write_research_report_bundle(output_dir, report)
            .map_err(|err| CliError::invalid(err.to_string()))?;
        Ok(format!(
            "wrote research report to {}\n{}",
            output_dir.display(),
            format_saved_research_report(output_dir, report)
        ))
    } else {
        Ok(format_research_report(report))
    }
}

fn build_comparable_research_bundle_set(
    bundles: &[(PathBuf, ReplayBundle)],
) -> Result<ComparableResearchBundleSet, CliError> {
    build_research_bundle_set(bundles, true)
}

fn build_comparable_attributed_research_bundle_set(
    bundles: &[(PathBuf, ReplayBundle)],
) -> Result<ComparableAttributedResearchBundleSet, CliError> {
    let comparable_set = build_research_bundle_set(bundles, false)?;
    let members = comparable_set
        .members
        .into_iter()
        .map(|member| {
            let strategy_components = extract_strategy_components(&member.bundle.manifest)?;

            Ok(AttributedResearchBundle {
                member,
                strategy_components,
            })
        })
        .collect::<Result<Vec<_>, CliError>>()?;

    Ok(ComparableAttributedResearchBundleSet {
        engine_version: comparable_set.engine_version,
        snapshot_id: comparable_set.snapshot_id,
        provider_identity: comparable_set.provider_identity,
        date_range: comparable_set.date_range,
        gap_policy: comparable_set.gap_policy,
        historical_limitations: comparable_set.historical_limitations,
        members,
    })
}

fn build_research_bundle_set(
    bundles: &[(PathBuf, ReplayBundle)],
    require_distinct_symbols: bool,
) -> Result<ComparableResearchBundleSet, CliError> {
    let Some((_, first_bundle)) = bundles.first() else {
        return Err(CliError::invalid(RESEARCH_AGGREGATE_USAGE));
    };

    if first_bundle.manifest.universe_mode != "single_symbol" {
        return Err(CliError::invalid(format!(
            "research aggregate only supports single_symbol bundles; got universe_mode `{}`",
            first_bundle.manifest.universe_mode
        )));
    }

    let expected_engine_version = first_bundle.manifest.engine_version.clone();
    let expected_snapshot_id = first_bundle.manifest.data_snapshot_id.clone();
    let expected_provider_identity = first_bundle.manifest.provider_identity.clone();
    let expected_date_range = format!(
        "{}..{}",
        first_bundle.manifest.date_range.start_date, first_bundle.manifest.date_range.end_date
    );
    let expected_gap_policy = first_bundle.manifest.gap_policy.as_str().to_string();
    let expected_reference_flow_kind = first_bundle.manifest.reference_flow.kind.clone();
    let expected_entry_shares = first_bundle
        .manifest
        .reference_flow
        .entry_shares
        .to_string();
    let expected_stop_fraction = format_f64(
        first_bundle
            .manifest
            .reference_flow
            .protective_stop_fraction,
    );
    let expected_cost_model = format!(
        "commission_per_fill={} slippage_per_share={}",
        format_f64(first_bundle.manifest.cost_model.commission_per_fill),
        format_f64(first_bundle.manifest.cost_model.slippage_per_share)
    );
    let expected_historical_limitations =
        format_string_list(&first_bundle.manifest.historical_limitations);

    let mut seen_symbols = BTreeSet::new();
    let mut members = bundles
        .iter()
        .map(|(bundle_dir, bundle)| {
            ensure_matching_aggregate_field(
                bundle_dir,
                "universe_mode",
                "single_symbol",
                &bundle.manifest.universe_mode,
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "engine_version",
                &expected_engine_version,
                &bundle.manifest.engine_version,
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "snapshot_id",
                &expected_snapshot_id,
                &bundle.manifest.data_snapshot_id,
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "provider",
                &expected_provider_identity,
                &bundle.manifest.provider_identity,
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "date_range",
                &expected_date_range,
                &format!(
                    "{}..{}",
                    bundle.manifest.date_range.start_date, bundle.manifest.date_range.end_date
                ),
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "gap_policy",
                &expected_gap_policy,
                bundle.manifest.gap_policy.as_str(),
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "reference_flow.kind",
                &expected_reference_flow_kind,
                &bundle.manifest.reference_flow.kind,
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "reference_flow.entry_shares",
                &expected_entry_shares,
                &bundle.manifest.reference_flow.entry_shares.to_string(),
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "reference_flow.protective_stop_fraction",
                &expected_stop_fraction,
                &format_f64(bundle.manifest.reference_flow.protective_stop_fraction),
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "cost_model",
                &expected_cost_model,
                &format!(
                    "commission_per_fill={} slippage_per_share={}",
                    format_f64(bundle.manifest.cost_model.commission_per_fill),
                    format_f64(bundle.manifest.cost_model.slippage_per_share)
                ),
            )?;
            ensure_matching_aggregate_field(
                bundle_dir,
                "historical_limitations",
                &expected_historical_limitations,
                &format_string_list(&bundle.manifest.historical_limitations),
            )?;

            if require_distinct_symbols
                && !seen_symbols.insert(bundle.manifest.symbol_or_universe.clone())
            {
                return Err(CliError::invalid(format!(
                    "research aggregate requires distinct symbols; duplicate symbol `{}`",
                    bundle.manifest.symbol_or_universe
                )));
            }

            Ok(ComparableResearchBundle {
                symbol: bundle.manifest.symbol_or_universe.clone(),
                bundle_path: bundle_dir.to_path_buf(),
                bundle: bundle.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    members.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then_with(|| left.bundle_path.cmp(&right.bundle_path))
    });

    Ok(ComparableResearchBundleSet {
        engine_version: expected_engine_version,
        snapshot_id: expected_snapshot_id,
        provider_identity: expected_provider_identity,
        date_range: expected_date_range,
        gap_policy: expected_gap_policy,
        historical_limitations: expected_historical_limitations,
        members,
    })
}

fn build_research_aggregate_report(
    comparable_set: &ComparableResearchBundleSet,
) -> ResearchAggregateReport {
    let members = comparable_set
        .members
        .iter()
        .map(build_research_aggregate_member)
        .collect::<Vec<_>>();
    let total_starting_equity = members
        .iter()
        .map(|member| parse_f64(&member.starting_equity))
        .sum::<f64>();
    let total_ending_equity = members
        .iter()
        .map(|member| parse_f64(&member.ending_equity))
        .sum::<f64>();
    let total_trade_count = members
        .iter()
        .map(|member| member.trade_count)
        .sum::<usize>();
    let total_warning_count = members
        .iter()
        .map(|member| member.warning_count)
        .sum::<usize>();
    let total_row_count = members.iter().map(|member| member.row_count).sum::<usize>();
    let net_equity_change_total = total_ending_equity - total_starting_equity;
    let average_net_equity_change = net_equity_change_total / members.len() as f64;

    ResearchAggregateReport {
        engine_version: comparable_set.engine_version.clone(),
        snapshot_id: comparable_set.snapshot_id.clone(),
        provider_identity: comparable_set.provider_identity.clone(),
        date_range: comparable_set.date_range.clone(),
        gap_policy: comparable_set.gap_policy.clone(),
        historical_limitations: comparable_set.historical_limitations.clone(),
        symbol_count: members.len(),
        total_row_count,
        total_warning_count,
        total_trade_count,
        starting_equity_total: format_f64(total_starting_equity),
        ending_equity_total: format_f64(total_ending_equity),
        net_equity_change_total: format_signed_f64(net_equity_change_total),
        average_net_equity_change: format_signed_f64(average_net_equity_change),
        symbols: members.iter().map(|member| member.symbol.clone()).collect(),
        members,
    }
}

fn build_research_aggregate_member(member: &ComparableResearchBundle) -> ResearchAggregateMember {
    let first_row = member
        .bundle
        .ledger
        .first()
        .expect("comparable research bundles should contain at least one ledger row");

    let starting_equity = first_row.equity;
    let ending_equity = member.bundle.summary.ending_equity;

    ResearchAggregateMember {
        symbol: member.symbol.clone(),
        bundle_path: member.bundle_path.clone(),
        row_count: member.bundle.summary.row_count,
        warning_count: member.bundle.summary.warning_count,
        trade_count: count_entry_trades(&member.bundle.ledger),
        starting_equity: format_f64(starting_equity),
        ending_equity: format_f64(ending_equity),
        net_equity_change: format_signed_f64(ending_equity - starting_equity),
    }
}

fn ensure_matching_aggregate_field(
    bundle_dir: &Path,
    field: &str,
    expected: &str,
    actual: &str,
) -> Result<(), CliError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CliError::invalid(format!(
            "research aggregate requires matching {field}; {} had `{actual}` but expected `{expected}`",
            bundle_dir.display()
        )))
    }
}

fn build_research_walk_forward_report(
    comparable_set: &ComparableResearchBundleSet,
    options: &WalkForwardOptions,
) -> Result<ResearchWalkForwardReport, CliError> {
    let shared_dates = validate_walk_forward_dates(comparable_set)?;
    if options.train_bars + options.test_bars > shared_dates.len() {
        return Err(CliError::invalid(format!(
            "research walk-forward requires train_bars + test_bars <= shared row count; got {} + {} > {}",
            options.train_bars,
            options.test_bars,
            shared_dates.len()
        )));
    }

    let mut splits = Vec::new();
    let mut start_index = 0_usize;
    let total_rows = shared_dates.len();

    while start_index + options.train_bars + options.test_bars <= total_rows {
        let train_start_index = start_index;
        let train_end_index = start_index + options.train_bars - 1;
        let test_start_index = train_end_index + 1;
        let test_end_index = test_start_index + options.test_bars - 1;

        splits.push(ResearchWalkForwardSplit {
            sequence: splits.len() + 1,
            train_start_index,
            train_end_index,
            test_start_index,
            test_end_index,
            train_row_range: format!("{train_start_index}..{train_end_index}"),
            train_date_range: format!(
                "{}..{}",
                shared_dates[train_start_index], shared_dates[train_end_index]
            ),
            test_row_range: format!("{test_start_index}..{test_end_index}"),
            test_date_range: format!(
                "{}..{}",
                shared_dates[test_start_index], shared_dates[test_end_index]
            ),
            children: comparable_set
                .members
                .iter()
                .map(|member| ResearchWalkForwardSplitChild {
                    symbol: member.symbol.clone(),
                    bundle_path: member.bundle_path.clone(),
                })
                .collect(),
        });

        start_index += options.step_bars;
    }

    Ok(ResearchWalkForwardReport {
        engine_version: comparable_set.engine_version.clone(),
        snapshot_id: comparable_set.snapshot_id.clone(),
        provider_identity: comparable_set.provider_identity.clone(),
        date_range: comparable_set.date_range.clone(),
        gap_policy: comparable_set.gap_policy.clone(),
        historical_limitations: comparable_set.historical_limitations.clone(),
        symbols: comparable_set
            .members
            .iter()
            .map(|member| member.symbol.clone())
            .collect(),
        train_bars: options.train_bars,
        test_bars: options.test_bars,
        step_bars: options.step_bars,
        split_count: splits.len(),
        splits,
    })
}

fn build_research_bootstrap_aggregate_report(
    comparable_set: &ComparableResearchBundleSet,
    options: &BootstrapOptions,
) -> ResearchBootstrapAggregateReport {
    let baseline = build_research_aggregate_report(comparable_set);
    let distribution = build_bootstrap_distribution_summary(
        &baseline
            .members
            .iter()
            .map(|member| parse_f64(&member.net_equity_change))
            .collect::<Vec<_>>(),
        options,
        "average_net_equity_change",
    );

    ResearchBootstrapAggregateReport {
        baseline,
        distribution,
    }
}

fn build_research_bootstrap_walk_forward_report(
    comparable_set: &ComparableResearchBundleSet,
    walk_forward: &WalkForwardOptions,
    bootstrap: &BootstrapOptions,
) -> Result<ResearchBootstrapWalkForwardReport, CliError> {
    let baseline = build_research_walk_forward_report(comparable_set, walk_forward)?;
    let splits = baseline
        .splits
        .iter()
        .map(|split| {
            let member_changes = comparable_set
                .members
                .iter()
                .map(|member| {
                    compute_test_window_net_equity_change(
                        &member.bundle.ledger,
                        split.test_start_index,
                        split.test_end_index,
                        &member.bundle_path,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let total_change = member_changes.iter().sum::<f64>();
            let average_change = total_change / member_changes.len() as f64;

            Ok(ResearchBootstrapWalkForwardSplit {
                sequence: split.sequence,
                train_row_range: split.train_row_range.clone(),
                train_date_range: split.train_date_range.clone(),
                test_row_range: split.test_row_range.clone(),
                test_date_range: split.test_date_range.clone(),
                baseline_test_total_net_equity_change: format_signed_f64(total_change),
                baseline_test_average_net_equity_change: format_signed_f64(average_change),
                children: split.children.clone(),
            })
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    let distribution = build_bootstrap_distribution_summary(
        &splits
            .iter()
            .map(|split| parse_f64(&split.baseline_test_average_net_equity_change))
            .collect::<Vec<_>>(),
        bootstrap,
        "mean_split_test_average_net_equity_change",
    );

    Ok(ResearchBootstrapWalkForwardReport {
        baseline,
        distribution,
        splits,
    })
}

fn build_research_leaderboard_report(
    comparable_set: &ComparableAttributedResearchBundleSet,
    view: LeaderboardView,
) -> Result<ResearchLeaderboardReport, CliError> {
    let fixed_signal_id = match view {
        LeaderboardView::Signal | LeaderboardView::System => None,
        LeaderboardView::PositionManager | LeaderboardView::ExecutionModel => Some(
            require_single_component_value(comparable_set, "signal_id", view, |member| {
                member.strategy_components.signal_id.as_str()
            })?,
        ),
    };
    let fixed_filter_id = match view {
        LeaderboardView::System => None,
        _ => Some(require_single_component_value(
            comparable_set,
            "filter_id",
            view,
            |member| member.strategy_components.filter_id.as_str(),
        )?),
    };
    let fixed_position_manager_id = match view {
        LeaderboardView::Signal => Some(require_single_component_value(
            comparable_set,
            "position_manager_id",
            view,
            |member| member.strategy_components.position_manager_id.as_str(),
        )?),
        LeaderboardView::ExecutionModel => Some(require_single_component_value(
            comparable_set,
            "position_manager_id",
            view,
            |member| member.strategy_components.position_manager_id.as_str(),
        )?),
        LeaderboardView::PositionManager | LeaderboardView::System => None,
    };
    let fixed_execution_model_id = match view {
        LeaderboardView::Signal => Some(require_single_component_value(
            comparable_set,
            "execution_model_id",
            view,
            |member| member.strategy_components.execution_model_id.as_str(),
        )?),
        LeaderboardView::PositionManager => Some(require_single_component_value(
            comparable_set,
            "execution_model_id",
            view,
            |member| member.strategy_components.execution_model_id.as_str(),
        )?),
        LeaderboardView::ExecutionModel | LeaderboardView::System => None,
    };

    let mut grouped_members = BTreeMap::<String, Vec<AttributedResearchBundle>>::new();
    for member in &comparable_set.members {
        grouped_members
            .entry(leaderboard_group_key(view, &member.strategy_components))
            .or_default()
            .push(member.clone());
    }

    let mut rows = grouped_members
        .into_iter()
        .map(|(label, members)| {
            build_research_leaderboard_row(view, label, comparable_set, members)
        })
        .collect::<Result<Vec<_>, CliError>>()?;

    rows.sort_by(|left, right| {
        parse_f64(&right.aggregate.average_net_equity_change)
            .total_cmp(&parse_f64(&left.aggregate.average_net_equity_change))
            .then_with(|| left.label.cmp(&right.label))
    });

    for (index, row) in rows.iter_mut().enumerate() {
        row.rank = index + 1;
    }

    let symbols = rows
        .first()
        .map(|row| row.aggregate.symbols.clone())
        .unwrap_or_default();

    for row in rows.iter().skip(1) {
        if row.aggregate.symbols != symbols {
            return Err(CliError::invalid(format!(
                "research leaderboard requires matching symbol sets across rows; {} had `{}` but expected `{}`",
                row.label,
                row.aggregate.symbols.join("|"),
                symbols.join("|")
            )));
        }
    }

    Ok(ResearchLeaderboardReport {
        view,
        engine_version: comparable_set.engine_version.clone(),
        snapshot_id: comparable_set.snapshot_id.clone(),
        provider_identity: comparable_set.provider_identity.clone(),
        date_range: comparable_set.date_range.clone(),
        gap_policy: comparable_set.gap_policy.clone(),
        historical_limitations: comparable_set.historical_limitations.clone(),
        symbol_count: symbols.len(),
        symbols,
        fixed_signal_id,
        fixed_filter_id,
        fixed_position_manager_id,
        fixed_execution_model_id,
        rows,
    })
}

fn build_research_leaderboard_row(
    view: LeaderboardView,
    label: String,
    comparable_set: &ComparableAttributedResearchBundleSet,
    members: Vec<AttributedResearchBundle>,
) -> Result<ResearchLeaderboardRow, CliError> {
    let comparable_members = build_leaderboard_group_members(view, &label, members)?;
    let aggregate = build_research_aggregate_report(&ComparableResearchBundleSet {
        engine_version: comparable_set.engine_version.clone(),
        snapshot_id: comparable_set.snapshot_id.clone(),
        provider_identity: comparable_set.provider_identity.clone(),
        date_range: comparable_set.date_range.clone(),
        gap_policy: comparable_set.gap_policy.clone(),
        historical_limitations: comparable_set.historical_limitations.clone(),
        members: comparable_members
            .iter()
            .map(|member| member.member.clone())
            .collect(),
    });
    let strategy_components = &comparable_members[0].strategy_components;

    Ok(ResearchLeaderboardRow {
        rank: 0,
        label,
        signal_id: strategy_components.signal_id.clone(),
        filter_id: strategy_components.filter_id.clone(),
        position_manager_id: strategy_components.position_manager_id.clone(),
        execution_model_id: strategy_components.execution_model_id.clone(),
        aggregate,
    })
}

fn build_leaderboard_group_members(
    view: LeaderboardView,
    label: &str,
    members: Vec<AttributedResearchBundle>,
) -> Result<Vec<AttributedResearchBundle>, CliError> {
    let mut seen_symbols = BTreeSet::new();

    for member in &members {
        if !seen_symbols.insert(member.member.symbol.clone()) {
            return Err(CliError::invalid(format!(
                "research leaderboard {} view requires distinct symbols per row; {} repeated symbol `{}`",
                view.as_str(),
                label,
                member.member.symbol
            )));
        }
    }

    let mut members = members;
    members.sort_by(|left, right| left.member.symbol.cmp(&right.member.symbol));
    Ok(members)
}

fn leaderboard_group_key(view: LeaderboardView, labels: &StrategyComponentLabels) -> String {
    match view {
        LeaderboardView::Signal => labels.signal_id.clone(),
        LeaderboardView::PositionManager => labels.position_manager_id.clone(),
        LeaderboardView::ExecutionModel => labels.execution_model_id.clone(),
        LeaderboardView::System => labels.system_id(),
    }
}

fn require_single_component_value(
    comparable_set: &ComparableAttributedResearchBundleSet,
    field: &str,
    view: LeaderboardView,
    selector: impl Fn(&AttributedResearchBundle) -> &str,
) -> Result<String, CliError> {
    let mut values = comparable_set
        .members
        .iter()
        .map(selector)
        .collect::<BTreeSet<_>>();

    if values.len() == 1 {
        Ok(values.pop_first().unwrap().to_string())
    } else {
        Err(CliError::invalid(format!(
            "research leaderboard {} view requires fixed {field} across all bundles",
            view.as_str()
        )))
    }
}

fn build_bootstrap_distribution_summary(
    values: &[f64],
    options: &BootstrapOptions,
    metric: &str,
) -> BootstrapDistributionSummary {
    let baseline_metric = mean_f64(values);
    let mut rng = BootstrapRng::new(options.seed);
    let mut sample_metrics = Vec::with_capacity(options.samples);

    for _ in 0..options.samples {
        let mut sample_total = 0.0;

        for _ in 0..values.len() {
            sample_total += values[rng.next_index(values.len())];
        }

        sample_metrics.push(sample_total / values.len() as f64);
    }

    let mut sorted_metrics = sample_metrics.clone();
    sorted_metrics.sort_by(f64::total_cmp);

    BootstrapDistributionSummary {
        seed: options.seed,
        sample_count: options.samples,
        resample_size: values.len(),
        metric: metric.to_string(),
        baseline_metric: format_signed_f64(baseline_metric),
        bootstrap_mean: format_signed_f64(mean_f64(&sample_metrics)),
        bootstrap_median: format_signed_f64(percentile(&sorted_metrics, 0.50)),
        bootstrap_min: format_signed_f64(sorted_metrics[0]),
        bootstrap_max: format_signed_f64(sorted_metrics[sorted_metrics.len() - 1]),
        bootstrap_interval_95_lower: format_signed_f64(percentile(&sorted_metrics, 0.025)),
        bootstrap_interval_95_upper: format_signed_f64(percentile(&sorted_metrics, 0.975)),
    }
}

fn compute_test_window_net_equity_change(
    ledger: &[PersistedLedgerRow],
    test_start_index: usize,
    test_end_index: usize,
    bundle_path: &Path,
) -> Result<f64, CliError> {
    if test_start_index == 0 {
        return Err(CliError::invalid(format!(
            "research bootstrap walk-forward requires a prior train row before test rows in {}",
            bundle_path.display()
        )));
    }

    let start_equity = ledger
        .get(test_start_index - 1)
        .ok_or_else(|| {
            CliError::invalid(format!(
                "research bootstrap walk-forward requires row {} in {}",
                test_start_index - 1,
                bundle_path.display()
            ))
        })?
        .equity;
    let end_equity = ledger
        .get(test_end_index)
        .ok_or_else(|| {
            CliError::invalid(format!(
                "research bootstrap walk-forward requires row {} in {}",
                test_end_index,
                bundle_path.display()
            ))
        })?
        .equity;

    Ok(end_equity - start_equity)
}

fn validate_walk_forward_dates(
    comparable_set: &ComparableResearchBundleSet,
) -> Result<Vec<String>, CliError> {
    let Some(first_member) = comparable_set.members.first() else {
        return Err(CliError::invalid(RESEARCH_WALK_FORWARD_USAGE));
    };
    let expected_dates = first_member
        .bundle
        .ledger
        .iter()
        .map(|row| row.date.clone())
        .collect::<Vec<_>>();

    if expected_dates.is_empty() {
        return Err(CliError::invalid(format!(
            "research walk-forward requires at least one ledger row per bundle; {} had none",
            first_member.bundle_path.display()
        )));
    }

    for member in comparable_set.members.iter().skip(1) {
        let actual_dates = member
            .bundle
            .ledger
            .iter()
            .map(|row| row.date.as_str())
            .collect::<Vec<_>>();
        let expected_date_refs = expected_dates
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();

        if actual_dates != expected_date_refs {
            return Err(CliError::invalid(format!(
                "research walk-forward requires matching ledger date sequences; {} did not match {}",
                member.bundle_path.display(),
                first_member.bundle_path.display()
            )));
        }
    }

    Ok(expected_dates)
}

fn parse_research_aggregate_options(
    args: Vec<String>,
) -> Result<ResearchAggregateOptions, CliError> {
    let mut output_dir = None;
    let mut bundle_dirs = Vec::new();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--output" => {
                output_dir = Some(parse_output_dir(
                    iter.next().as_deref(),
                    RESEARCH_AGGREGATE_USAGE,
                )?)
            }
            other => bundle_dirs.push(PathBuf::from(other)),
        }
    }

    if bundle_dirs.len() < 2 {
        return Err(CliError::invalid(RESEARCH_AGGREGATE_USAGE));
    }

    Ok(ResearchAggregateOptions {
        bundle_dirs,
        output_dir,
    })
}

fn parse_walk_forward_options(args: Vec<String>) -> Result<WalkForwardOptions, CliError> {
    let mut train_bars = None;
    let mut test_bars = None;
    let mut step_bars = None;
    let mut output_dir = None;
    let mut bundle_dirs = Vec::new();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--train-bars" => {
                train_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--train-bars",
                    RESEARCH_WALK_FORWARD_USAGE,
                )?);
            }
            "--test-bars" => {
                test_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--test-bars",
                    RESEARCH_WALK_FORWARD_USAGE,
                )?);
            }
            "--step-bars" => {
                step_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--step-bars",
                    RESEARCH_WALK_FORWARD_USAGE,
                )?);
            }
            "--output" => {
                output_dir = Some(parse_output_dir(
                    iter.next().as_deref(),
                    RESEARCH_WALK_FORWARD_USAGE,
                )?);
            }
            other => bundle_dirs.push(PathBuf::from(other)),
        }
    }

    let Some(train_bars) = train_bars else {
        return Err(CliError::invalid(RESEARCH_WALK_FORWARD_USAGE));
    };
    let Some(test_bars) = test_bars else {
        return Err(CliError::invalid(RESEARCH_WALK_FORWARD_USAGE));
    };
    let step_bars = step_bars.unwrap_or(test_bars);

    if bundle_dirs.len() < 2 {
        return Err(CliError::invalid(RESEARCH_WALK_FORWARD_USAGE));
    }

    Ok(WalkForwardOptions {
        train_bars,
        test_bars,
        step_bars,
        bundle_dirs,
        output_dir,
    })
}

fn parse_bootstrap_aggregate_options(
    args: Vec<String>,
) -> Result<BootstrapAggregateOptions, CliError> {
    let mut samples = None;
    let mut seed = 0_u64;
    let mut output_dir = None;
    let mut bundle_dirs = Vec::new();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--samples" => {
                samples = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--samples",
                    RESEARCH_BOOTSTRAP_AGGREGATE_USAGE,
                )?);
            }
            "--seed" => {
                seed = parse_u64(
                    iter.next().as_deref(),
                    "--seed",
                    RESEARCH_BOOTSTRAP_AGGREGATE_USAGE,
                )?;
            }
            "--output" => {
                output_dir = Some(parse_output_dir(
                    iter.next().as_deref(),
                    RESEARCH_BOOTSTRAP_AGGREGATE_USAGE,
                )?);
            }
            other => bundle_dirs.push(PathBuf::from(other)),
        }
    }

    let Some(samples) = samples else {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_AGGREGATE_USAGE));
    };
    if bundle_dirs.len() < 2 {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_AGGREGATE_USAGE));
    }

    Ok(BootstrapAggregateOptions {
        bootstrap: BootstrapOptions { samples, seed },
        bundle_dirs,
        output_dir,
    })
}

fn parse_bootstrap_walk_forward_options(
    args: Vec<String>,
) -> Result<BootstrapWalkForwardOptions, CliError> {
    let mut samples = None;
    let mut seed = 0_u64;
    let mut train_bars = None;
    let mut test_bars = None;
    let mut step_bars = None;
    let mut output_dir = None;
    let mut bundle_dirs = Vec::new();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--samples" => {
                samples = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--samples",
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?);
            }
            "--seed" => {
                seed = parse_u64(
                    iter.next().as_deref(),
                    "--seed",
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?;
            }
            "--train-bars" => {
                train_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--train-bars",
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?);
            }
            "--test-bars" => {
                test_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--test-bars",
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?);
            }
            "--step-bars" => {
                step_bars = Some(parse_positive_usize(
                    iter.next().as_deref(),
                    "--step-bars",
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?);
            }
            "--output" => {
                output_dir = Some(parse_output_dir(
                    iter.next().as_deref(),
                    RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE,
                )?);
            }
            other => bundle_dirs.push(PathBuf::from(other)),
        }
    }

    let Some(samples) = samples else {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE));
    };
    let Some(train_bars) = train_bars else {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE));
    };
    let Some(test_bars) = test_bars else {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE));
    };
    let step_bars = step_bars.unwrap_or(test_bars);

    if bundle_dirs.len() < 2 {
        return Err(CliError::invalid(RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE));
    }

    Ok(BootstrapWalkForwardOptions {
        bootstrap: BootstrapOptions { samples, seed },
        walk_forward: WalkForwardOptions {
            train_bars,
            test_bars,
            step_bars,
            bundle_dirs,
            output_dir: None,
        },
        output_dir,
    })
}

fn parse_leaderboard_options(args: Vec<String>) -> Result<LeaderboardOptions, CliError> {
    let mut iter = args.into_iter();
    let Some(raw_view) = iter.next() else {
        return Err(CliError::invalid(RESEARCH_LEADERBOARD_USAGE));
    };
    let Some(view) = LeaderboardView::parse(raw_view.as_str()) else {
        return Err(CliError::invalid(RESEARCH_LEADERBOARD_USAGE));
    };
    let mut output_dir = None;
    let mut bundle_dirs = Vec::new();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--output" => {
                output_dir = Some(parse_output_dir(
                    iter.next().as_deref(),
                    RESEARCH_LEADERBOARD_USAGE,
                )?);
            }
            other => bundle_dirs.push(PathBuf::from(other)),
        }
    }

    if bundle_dirs.len() < 2 {
        return Err(CliError::invalid(RESEARCH_LEADERBOARD_USAGE));
    }

    Ok(LeaderboardOptions {
        view,
        bundle_dirs,
        output_dir,
    })
}

fn parse_output_dir(value: Option<&str>, usage: &str) -> Result<PathBuf, CliError> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| CliError::invalid(usage))
}

fn parse_positive_usize(value: Option<&str>, flag: &str, usage: &str) -> Result<usize, CliError> {
    let Some(value) = value else {
        return Err(CliError::invalid(usage));
    };
    let parsed = value
        .parse::<usize>()
        .map_err(|_| CliError::invalid(format!("invalid value for {flag}: `{value}`")))?;

    if parsed == 0 {
        Err(CliError::invalid(format!(
            "{flag} must be greater than zero"
        )))
    } else {
        Ok(parsed)
    }
}

fn parse_u64(value: Option<&str>, flag: &str, usage: &str) -> Result<u64, CliError> {
    let Some(value) = value else {
        return Err(CliError::invalid(usage));
    };

    value
        .parse::<u64>()
        .map_err(|_| CliError::invalid(format!("invalid value for {flag}: `{value}`")))
}

fn extract_strategy_components(
    manifest: &RunManifest,
) -> Result<StrategyComponentLabels, CliError> {
    let signal_id = required_manifest_parameter(manifest, STRATEGY_SIGNAL_PARAMETER)?;
    let filter_id = required_manifest_parameter(manifest, STRATEGY_FILTER_PARAMETER)?;
    let position_manager_id = required_manifest_parameter(manifest, STRATEGY_POSITION_PARAMETER)?;
    let execution_model_id = required_manifest_parameter(manifest, STRATEGY_EXECUTION_PARAMETER)?;

    Ok(StrategyComponentLabels {
        signal_id,
        filter_id,
        position_manager_id,
        execution_model_id,
    })
}

fn required_manifest_parameter(manifest: &RunManifest, name: &str) -> Result<String, CliError> {
    manifest
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .map(|parameter| parameter.value.clone())
        .ok_or_else(|| {
            CliError::invalid(format!(
                "research leaderboard requires manifest parameter `{name}` on bundle for symbol `{}`",
                manifest.symbol_or_universe
            ))
        })
}

fn format_research_report(report: &ResearchReport) -> String {
    match report {
        ResearchReport::Aggregate(report) => format_research_aggregate_report(report),
        ResearchReport::WalkForward(report) => format_research_walk_forward_report(report),
        ResearchReport::BootstrapAggregate(report) => {
            format_research_bootstrap_aggregate_report(report)
        }
        ResearchReport::BootstrapWalkForward(report) => {
            format_research_bootstrap_walk_forward_report(report)
        }
        ResearchReport::Leaderboard(report) => format_research_leaderboard_report(report),
    }
}

fn format_saved_research_report(report_dir: &Path, report: &ResearchReport) -> String {
    format!(
        "report: {}\nschema_version: {}\nreport_kind: {}\n{}",
        report_dir.display(),
        SCHEMA_VERSION,
        report.kind(),
        format_research_report(report)
    )
}

fn format_research_aggregate_report(report: &ResearchAggregateReport) -> String {
    let mut lines = vec![
        "research aggregate".to_string(),
        format!("engine_version: {}", report.engine_version),
        format!("snapshot_id: {}", report.snapshot_id),
        format!("provider: {}", report.provider_identity),
        format!("date_range: {}", report.date_range),
        format!("gap_policy: {}", report.gap_policy),
        format!("historical_limitations: {}", report.historical_limitations),
        format!("symbol_count: {}", report.symbol_count),
        format!("symbols: {}", report.symbols.join("|")),
        format!("total_rows: {}", report.total_row_count),
        format!("total_warnings: {}", report.total_warning_count),
        format!("total_trades: {}", report.total_trade_count),
        format!("starting_equity_total: {}", report.starting_equity_total),
        format!("ending_equity_total: {}", report.ending_equity_total),
        format!(
            "net_equity_change_total: {}",
            report.net_equity_change_total
        ),
        format!(
            "average_net_equity_change: {}",
            report.average_net_equity_change
        ),
        "members:".to_string(),
    ];

    lines.extend(report.members.iter().map(|member| {
        format!(
            "member: symbol={} start={} end={} change={} trades={} rows={} warnings={} bundle={}",
            member.symbol,
            member.starting_equity,
            member.ending_equity,
            member.net_equity_change,
            member.trade_count,
            member.row_count,
            member.warning_count,
            member.bundle_path.display()
        )
    }));

    lines.join("\n")
}

fn format_research_walk_forward_report(report: &ResearchWalkForwardReport) -> String {
    let mut lines = vec![
        "research walk-forward".to_string(),
        format!("engine_version: {}", report.engine_version),
        format!("snapshot_id: {}", report.snapshot_id),
        format!("provider: {}", report.provider_identity),
        format!("date_range: {}", report.date_range),
        format!("gap_policy: {}", report.gap_policy),
        format!("historical_limitations: {}", report.historical_limitations),
        format!("symbols: {}", report.symbols.join("|")),
        format!("train_bars: {}", report.train_bars),
        format!("test_bars: {}", report.test_bars),
        format!("step_bars: {}", report.step_bars),
        format!("split_count: {}", report.split_count),
    ];

    for split in &report.splits {
        lines.push(format!(
            "split: id={} train_rows={} train_dates={} test_rows={} test_dates={}",
            split.sequence,
            split.train_row_range,
            split.train_date_range,
            split.test_row_range,
            split.test_date_range
        ));
        lines.extend(split.children.iter().map(|child| {
            format!(
                "child: split={} symbol={} bundle={}",
                split.sequence,
                child.symbol,
                child.bundle_path.display()
            )
        }));
    }

    lines.join("\n")
}

fn format_research_bootstrap_aggregate_report(report: &ResearchBootstrapAggregateReport) -> String {
    let baseline = &report.baseline;
    let distribution = &report.distribution;
    let mut lines = vec![
        "research bootstrap aggregate".to_string(),
        format!("engine_version: {}", baseline.engine_version),
        format!("snapshot_id: {}", baseline.snapshot_id),
        format!("provider: {}", baseline.provider_identity),
        format!("date_range: {}", baseline.date_range),
        format!("gap_policy: {}", baseline.gap_policy),
        format!(
            "historical_limitations: {}",
            baseline.historical_limitations
        ),
        format!("symbol_count: {}", baseline.symbol_count),
        format!("symbols: {}", baseline.symbols.join("|")),
        format!("total_rows: {}", baseline.total_row_count),
        format!("total_warnings: {}", baseline.total_warning_count),
        format!("total_trades: {}", baseline.total_trade_count),
        format!(
            "baseline_starting_equity_total: {}",
            baseline.starting_equity_total
        ),
        format!(
            "baseline_ending_equity_total: {}",
            baseline.ending_equity_total
        ),
        format!(
            "baseline_net_equity_change_total: {}",
            baseline.net_equity_change_total
        ),
        format!(
            "baseline_average_net_equity_change: {}",
            baseline.average_net_equity_change
        ),
        format!("seed: {}", distribution.seed),
        format!("samples: {}", distribution.sample_count),
        "resample_unit: symbol".to_string(),
        format!("resample_size: {}", distribution.resample_size),
        format!("metric: {}", distribution.metric),
        format!("baseline_metric: {}", distribution.baseline_metric),
        format!("bootstrap_mean: {}", distribution.bootstrap_mean),
        format!("bootstrap_median: {}", distribution.bootstrap_median),
        format!("bootstrap_min: {}", distribution.bootstrap_min),
        format!("bootstrap_max: {}", distribution.bootstrap_max),
        format!(
            "bootstrap_interval_95: {}..{}",
            distribution.bootstrap_interval_95_lower, distribution.bootstrap_interval_95_upper
        ),
        "members:".to_string(),
    ];

    lines.extend(baseline.members.iter().map(|member| {
        format!(
            "member: symbol={} start={} end={} change={} trades={} rows={} warnings={} bundle={}",
            member.symbol,
            member.starting_equity,
            member.ending_equity,
            member.net_equity_change,
            member.trade_count,
            member.row_count,
            member.warning_count,
            member.bundle_path.display()
        )
    }));

    lines.join("\n")
}

fn format_research_bootstrap_walk_forward_report(
    report: &ResearchBootstrapWalkForwardReport,
) -> String {
    let baseline = &report.baseline;
    let distribution = &report.distribution;
    let mut lines = vec![
        "research bootstrap walk-forward".to_string(),
        format!("engine_version: {}", baseline.engine_version),
        format!("snapshot_id: {}", baseline.snapshot_id),
        format!("provider: {}", baseline.provider_identity),
        format!("date_range: {}", baseline.date_range),
        format!("gap_policy: {}", baseline.gap_policy),
        format!(
            "historical_limitations: {}",
            baseline.historical_limitations
        ),
        format!("symbols: {}", baseline.symbols.join("|")),
        format!("train_bars: {}", baseline.train_bars),
        format!("test_bars: {}", baseline.test_bars),
        format!("step_bars: {}", baseline.step_bars),
        format!("split_count: {}", baseline.split_count),
        format!("seed: {}", distribution.seed),
        format!("samples: {}", distribution.sample_count),
        "resample_unit: walk_forward_split".to_string(),
        format!("resample_size: {}", distribution.resample_size),
        format!("metric: {}", distribution.metric),
        format!("baseline_metric: {}", distribution.baseline_metric),
        format!("bootstrap_mean: {}", distribution.bootstrap_mean),
        format!("bootstrap_median: {}", distribution.bootstrap_median),
        format!("bootstrap_min: {}", distribution.bootstrap_min),
        format!("bootstrap_max: {}", distribution.bootstrap_max),
        format!(
            "bootstrap_interval_95: {}..{}",
            distribution.bootstrap_interval_95_lower, distribution.bootstrap_interval_95_upper
        ),
    ];

    for split in &report.splits {
        lines.push(format!(
            "split: id={} train_rows={} train_dates={} test_rows={} test_dates={} baseline_test_total_net_equity_change={} baseline_test_average_net_equity_change={}",
            split.sequence,
            split.train_row_range,
            split.train_date_range,
            split.test_row_range,
            split.test_date_range,
            split.baseline_test_total_net_equity_change,
            split.baseline_test_average_net_equity_change
        ));
        lines.extend(split.children.iter().map(|child| {
            format!(
                "child: split={} symbol={} bundle={}",
                split.sequence,
                child.symbol,
                child.bundle_path.display()
            )
        }));
    }

    lines.join("\n")
}

fn format_research_leaderboard_report(report: &ResearchLeaderboardReport) -> String {
    let mut lines = vec![
        format!("research leaderboard {}", report.view.as_str()),
        format!("engine_version: {}", report.engine_version),
        format!("snapshot_id: {}", report.snapshot_id),
        format!("provider: {}", report.provider_identity),
        format!("date_range: {}", report.date_range),
        format!("gap_policy: {}", report.gap_policy),
        format!("historical_limitations: {}", report.historical_limitations),
        format!("symbol_count: {}", report.symbol_count),
        format!("symbols: {}", report.symbols.join("|")),
        format!("row_count: {}", report.rows.len()),
    ];

    if let Some(value) = &report.fixed_signal_id {
        lines.push(format!("fixed_signal_id: {}", value));
    }
    if let Some(value) = &report.fixed_filter_id {
        lines.push(format!("fixed_filter_id: {}", value));
    }
    if let Some(value) = &report.fixed_position_manager_id {
        lines.push(format!("fixed_position_manager_id: {}", value));
    }
    if let Some(value) = &report.fixed_execution_model_id {
        lines.push(format!("fixed_execution_model_id: {}", value));
    }

    for row in &report.rows {
        lines.push(format!(
            "row: rank={} label={} signal={} filter={} position={} execution={} start={} end={} change={} average_change={} trades={} warnings={} bundles={}",
            row.rank,
            row.label,
            row.signal_id,
            row.filter_id,
            row.position_manager_id,
            row.execution_model_id,
            row.aggregate.starting_equity_total,
            row.aggregate.ending_equity_total,
            row.aggregate.net_equity_change_total,
            row.aggregate.average_net_equity_change,
            row.aggregate.total_trade_count,
            row.aggregate.total_warning_count,
            row.aggregate.symbol_count
        ));
        lines.extend(row.aggregate.members.iter().map(|member| {
            format!(
                "member: rank={} label={} symbol={} change={} trades={} rows={} warnings={} bundle={}",
                row.rank,
                row.label,
                member.symbol,
                member.net_equity_change,
                member.trade_count,
                member.row_count,
                member.warning_count,
                member.bundle_path.display()
            )
        }));
    }

    lines.join("\n")
}

fn parse_run_options(args: Vec<String>) -> Result<RunOptions, CliError> {
    let mut request_path = None;
    let mut spec_path = None;
    let mut output_dir = None;
    let mut provider_identity = None;
    let mut snapshot_id = None;
    let mut engine_version = None;
    let mut signal_id = None;
    let mut filter_id = None;
    let mut position_manager_id = None;
    let mut execution_model_id = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--request" => request_path = iter.next().map(PathBuf::from),
            "--spec" => spec_path = iter.next().map(PathBuf::from),
            "--output" => output_dir = iter.next().map(PathBuf::from),
            "--provider" => {
                let Some(raw_provider) = iter.next() else {
                    return Err(CliError::invalid(RUN_USAGE));
                };
                provider_identity =
                    Some(ProviderIdentity::parse(&raw_provider).ok_or_else(|| {
                        CliError::invalid(format!("unknown provider `{raw_provider}`"))
                    })?);
            }
            "--snapshot-id" => snapshot_id = iter.next(),
            "--engine-version" => {
                let Some(value) = iter.next() else {
                    return Err(CliError::invalid(RUN_USAGE));
                };
                engine_version = Some(value);
            }
            "--signal-id" => signal_id = iter.next(),
            "--filter-id" => filter_id = iter.next(),
            "--position-manager-id" => position_manager_id = iter.next(),
            "--execution-model-id" => execution_model_id = iter.next(),
            other => {
                return Err(CliError::invalid(format!(
                    "unexpected argument for run: {other}"
                )));
            }
        }
    }

    let Some(output_dir) = output_dir else {
        return Err(CliError::invalid(RUN_USAGE));
    };
    let input_source = match (request_path, spec_path) {
        (Some(request_path), None) => RunInputSource::Request(request_path),
        (None, Some(spec_path)) => RunInputSource::Spec(spec_path),
        _ => return Err(CliError::invalid(RUN_USAGE)),
    };
    let strategy_components = match (
        signal_id,
        filter_id,
        position_manager_id,
        execution_model_id,
    ) {
        (None, None, None, None) => None,
        (Some(signal_id), Some(filter_id), Some(position_manager_id), Some(execution_model_id)) => {
            Some(StrategyComponentLabels {
                signal_id,
                filter_id,
                position_manager_id,
                execution_model_id,
            })
        }
        _ => {
            return Err(CliError::invalid(
                "run requires --signal-id, --filter-id, --position-manager-id, and --execution-model-id together",
            ));
        }
    };
    if matches!(input_source, RunInputSource::Spec(_))
        && (provider_identity.is_some()
            || snapshot_id.is_some()
            || engine_version.is_some()
            || strategy_components.is_some())
    {
        return Err(CliError::invalid(
            "run --spec cannot be combined with --provider, --snapshot-id, --engine-version, or strategy-component flags",
        ));
    }

    Ok(RunOptions {
        input_source,
        output_dir,
        provider_identity,
        snapshot_id,
        engine_version,
        strategy_components,
    })
}

fn load_bundle(bundle_dir: &Path) -> Result<ReplayBundle, CliError> {
    load_replay_bundle(bundle_dir).map_err(|err| CliError::invalid(err.to_string()))
}

fn load_research_report(report_dir: &Path) -> Result<ResearchReport, CliError> {
    let report = load_research_report_bundle(report_dir)
        .map_err(|err| CliError::invalid(err.to_string()))?;
    validate_research_report_provenance(&report)?;
    Ok(report)
}

fn validate_research_report_provenance(report: &ResearchReport) -> Result<(), CliError> {
    match report {
        ResearchReport::Aggregate(report) => validate_aggregate_report_provenance(report),
        ResearchReport::WalkForward(report) => validate_walk_forward_report_provenance(report),
        ResearchReport::BootstrapAggregate(report) => {
            validate_bootstrap_aggregate_report_provenance(report)
        }
        ResearchReport::BootstrapWalkForward(report) => {
            validate_bootstrap_walk_forward_report_provenance(report)
        }
        ResearchReport::Leaderboard(report) => validate_leaderboard_report_provenance(report),
    }
}

fn validate_aggregate_report_provenance(report: &ResearchAggregateReport) -> Result<(), CliError> {
    let expectation = aggregate_expectation(report);

    for member in &report.members {
        validate_aggregate_member_bundle(
            &format!("research aggregate member `{}`", member.symbol),
            member,
            expectation,
        )?;
    }

    Ok(())
}

fn validate_walk_forward_report_provenance(
    report: &ResearchWalkForwardReport,
) -> Result<(), CliError> {
    let expectation = walk_forward_expectation(report);

    for split in &report.splits {
        for child in &split.children {
            let bundle = validate_bundle_against_expectation(
                &format!(
                    "research walk-forward split {} child `{}`",
                    split.sequence, child.symbol
                ),
                &child.bundle_path,
                &child.symbol,
                expectation,
            )?;
            validate_split_indices_against_bundle(
                &format!(
                    "research walk-forward split {} child `{}`",
                    split.sequence, child.symbol
                ),
                &bundle,
                split.test_end_index,
            )?;
        }
    }

    Ok(())
}

fn validate_bootstrap_aggregate_report_provenance(
    report: &ResearchBootstrapAggregateReport,
) -> Result<(), CliError> {
    validate_aggregate_report_provenance(&report.baseline)?;

    let expected_distribution = build_bootstrap_distribution_summary(
        &report
            .baseline
            .members
            .iter()
            .map(|member| parse_f64(&member.net_equity_change))
            .collect::<Vec<_>>(),
        &BootstrapOptions {
            samples: report.distribution.sample_count,
            seed: report.distribution.seed,
        },
        "average_net_equity_change",
    );

    validate_bootstrap_distribution_matches(
        "research bootstrap aggregate distribution",
        &report.distribution,
        &expected_distribution,
    )
}

fn validate_bootstrap_walk_forward_report_provenance(
    report: &ResearchBootstrapWalkForwardReport,
) -> Result<(), CliError> {
    let expectation = walk_forward_expectation(&report.baseline);
    let mut split_average_changes = Vec::with_capacity(report.splits.len());

    for (stored_split, baseline_split) in report.splits.iter().zip(report.baseline.splits.iter()) {
        let mut member_changes = Vec::with_capacity(stored_split.children.len());

        for child in &stored_split.children {
            let context = format!(
                "research bootstrap walk-forward split {} child `{}`",
                stored_split.sequence, child.symbol
            );
            let bundle = validate_bundle_against_expectation(
                &context,
                &child.bundle_path,
                &child.symbol,
                expectation,
            )?;
            validate_split_indices_against_bundle(
                &context,
                &bundle,
                baseline_split.test_end_index,
            )?;
            member_changes.push(compute_test_window_net_equity_change(
                &bundle.ledger,
                baseline_split.test_start_index,
                baseline_split.test_end_index,
                &child.bundle_path,
            )?);
        }

        let total_change = member_changes.iter().sum::<f64>();
        let average_change = total_change / member_changes.len() as f64;
        validate_report_metric(
            &format!(
                "research bootstrap walk-forward split {} baseline_test_total_net_equity_change",
                stored_split.sequence
            ),
            &stored_split.baseline_test_total_net_equity_change,
            total_change,
        )?;
        validate_report_metric(
            &format!(
                "research bootstrap walk-forward split {} baseline_test_average_net_equity_change",
                stored_split.sequence
            ),
            &stored_split.baseline_test_average_net_equity_change,
            average_change,
        )?;
        split_average_changes.push(average_change);
    }

    let expected_distribution = build_bootstrap_distribution_summary(
        &split_average_changes,
        &BootstrapOptions {
            samples: report.distribution.sample_count,
            seed: report.distribution.seed,
        },
        "mean_split_test_average_net_equity_change",
    );

    validate_bootstrap_distribution_matches(
        "research bootstrap walk-forward distribution",
        &report.distribution,
        &expected_distribution,
    )
}

fn validate_leaderboard_report_provenance(
    report: &ResearchLeaderboardReport,
) -> Result<(), CliError> {
    let mut prior_average_change = None;
    let mut prior_label = None::<String>;

    for row in &report.rows {
        let expectation = aggregate_expectation(&row.aggregate);

        for member in &row.aggregate.members {
            let context = format!(
                "research leaderboard row {} member `{}`",
                row.rank, member.symbol
            );
            let bundle = validate_aggregate_member_bundle(&context, member, expectation)?;

            validate_bundle_strategy_component(
                &context,
                &bundle,
                &member.bundle_path,
                STRATEGY_SIGNAL_PARAMETER,
                &row.signal_id,
            )?;
            validate_bundle_strategy_component(
                &context,
                &bundle,
                &member.bundle_path,
                STRATEGY_FILTER_PARAMETER,
                &row.filter_id,
            )?;
            validate_bundle_strategy_component(
                &context,
                &bundle,
                &member.bundle_path,
                STRATEGY_POSITION_PARAMETER,
                &row.position_manager_id,
            )?;
            validate_bundle_strategy_component(
                &context,
                &bundle,
                &member.bundle_path,
                STRATEGY_EXECUTION_PARAMETER,
                &row.execution_model_id,
            )?;
        }

        let average_change = parse_f64(&row.aggregate.average_net_equity_change);
        if let Some(previous) = prior_average_change {
            if average_change > previous {
                return Err(CliError::invalid(format!(
                    "research leaderboard row {} average_net_equity_change {} must not exceed prior row average_net_equity_change {}",
                    row.rank,
                    row.aggregate.average_net_equity_change,
                    format_signed_f64(previous)
                )));
            }

            if round4_cli(average_change) == round4_cli(previous)
                && let Some(previous_label) = &prior_label
                && row.label < *previous_label
            {
                return Err(CliError::invalid(format!(
                    "research leaderboard row {} label `{}` must not sort before prior row label `{}` when average_net_equity_change ties",
                    row.rank, row.label, previous_label
                )));
            }
        }

        prior_average_change = Some(average_change);
        prior_label = Some(row.label.clone());
    }

    Ok(())
}

fn validate_aggregate_member_bundle(
    context: &str,
    member: &ResearchAggregateMember,
    expectation: ResearchBundleExpectation<'_>,
) -> Result<ReplayBundle, CliError> {
    let bundle = validate_bundle_against_expectation(
        context,
        &member.bundle_path,
        &member.symbol,
        expectation,
    )?;
    let first_row = bundle.ledger.first().ok_or_else(|| {
        CliError::invalid(format!(
            "{context} requires at least one ledger row in {}",
            member.bundle_path.display()
        ))
    })?;

    validate_report_usize_field(
        &format!("{context} row_count"),
        bundle.summary.row_count,
        member.row_count,
    )?;
    validate_report_usize_field(
        &format!("{context} warning_count"),
        bundle.summary.warning_count,
        member.warning_count,
    )?;
    validate_report_usize_field(
        &format!("{context} trade_count"),
        count_entry_trades(&bundle.ledger),
        member.trade_count,
    )?;
    validate_report_metric(
        &format!("{context} starting_equity"),
        &member.starting_equity,
        first_row.equity,
    )?;
    validate_report_metric(
        &format!("{context} ending_equity"),
        &member.ending_equity,
        bundle.summary.ending_equity,
    )?;
    validate_report_metric(
        &format!("{context} net_equity_change"),
        &member.net_equity_change,
        bundle.summary.ending_equity - first_row.equity,
    )?;

    Ok(bundle)
}

fn validate_bundle_against_expectation(
    context: &str,
    bundle_path: &Path,
    expected_symbol: &str,
    expectation: ResearchBundleExpectation<'_>,
) -> Result<ReplayBundle, CliError> {
    let bundle = load_replay_bundle(bundle_path).map_err(|err| {
        CliError::invalid(format!(
            "{context} requires replay bundle {}: {err}",
            bundle_path.display()
        ))
    })?;

    validate_bundle_text_field(
        &format!("{context} symbol"),
        &bundle.manifest.symbol_or_universe,
        expected_symbol,
    )?;
    validate_bundle_text_field(
        &format!("{context} engine_version"),
        &bundle.manifest.engine_version,
        expectation.engine_version,
    )?;
    validate_bundle_text_field(
        &format!("{context} snapshot_id"),
        &bundle.manifest.data_snapshot_id,
        expectation.snapshot_id,
    )?;
    validate_bundle_text_field(
        &format!("{context} provider"),
        &bundle.manifest.provider_identity,
        expectation.provider_identity,
    )?;
    validate_bundle_text_field(
        &format!("{context} date_range"),
        &format!(
            "{}..{}",
            bundle.manifest.date_range.start_date, bundle.manifest.date_range.end_date
        ),
        expectation.date_range,
    )?;
    validate_bundle_text_field(
        &format!("{context} gap_policy"),
        bundle.manifest.gap_policy.as_str(),
        expectation.gap_policy,
    )?;
    validate_bundle_text_field(
        &format!("{context} historical_limitations"),
        &format_string_list(&bundle.manifest.historical_limitations),
        expectation.historical_limitations,
    )?;

    Ok(bundle)
}

fn validate_split_indices_against_bundle(
    context: &str,
    bundle: &ReplayBundle,
    expected_last_index: usize,
) -> Result<(), CliError> {
    if bundle.summary.row_count <= expected_last_index {
        Err(CliError::invalid(format!(
            "{context} requires row {} in replay bundle but {} only has {} rows",
            expected_last_index, bundle.manifest.symbol_or_universe, bundle.summary.row_count
        )))
    } else {
        Ok(())
    }
}

fn validate_bundle_strategy_component(
    context: &str,
    bundle: &ReplayBundle,
    bundle_path: &Path,
    parameter_name: &str,
    expected_value: &str,
) -> Result<(), CliError> {
    let actual_value = bundle
        .manifest
        .parameters
        .iter()
        .find(|parameter| parameter.name == parameter_name)
        .map(|parameter| parameter.value.as_str())
        .ok_or_else(|| {
            CliError::invalid(format!(
                "{context} requires manifest parameter `{parameter_name}` on bundle {}",
                bundle_path.display()
            ))
        })?;

    validate_bundle_text_field(
        &format!("{context} {parameter_name}"),
        actual_value,
        expected_value,
    )
}

fn validate_bootstrap_distribution_matches(
    context: &str,
    actual: &BootstrapDistributionSummary,
    expected: &BootstrapDistributionSummary,
) -> Result<(), CliError> {
    validate_report_usize_field(
        &format!("{context} seed"),
        actual.seed as usize,
        expected.seed as usize,
    )?;
    validate_report_usize_field(
        &format!("{context} sample_count"),
        actual.sample_count,
        expected.sample_count,
    )?;
    validate_report_usize_field(
        &format!("{context} resample_size"),
        actual.resample_size,
        expected.resample_size,
    )?;
    validate_bundle_text_field(
        &format!("{context} metric"),
        &actual.metric,
        &expected.metric,
    )?;
    validate_report_metric(
        &format!("{context} baseline_metric"),
        &actual.baseline_metric,
        parse_f64(&expected.baseline_metric),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_mean"),
        &actual.bootstrap_mean,
        parse_f64(&expected.bootstrap_mean),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_median"),
        &actual.bootstrap_median,
        parse_f64(&expected.bootstrap_median),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_min"),
        &actual.bootstrap_min,
        parse_f64(&expected.bootstrap_min),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_max"),
        &actual.bootstrap_max,
        parse_f64(&expected.bootstrap_max),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_interval_95_lower"),
        &actual.bootstrap_interval_95_lower,
        parse_f64(&expected.bootstrap_interval_95_lower),
    )?;
    validate_report_metric(
        &format!("{context} bootstrap_interval_95_upper"),
        &actual.bootstrap_interval_95_upper,
        parse_f64(&expected.bootstrap_interval_95_upper),
    )
}

fn validate_bundle_text_field(context: &str, actual: &str, expected: &str) -> Result<(), CliError> {
    if actual == expected {
        Ok(())
    } else {
        Err(CliError::invalid(format!(
            "{context} `{actual}` does not match expected `{expected}`"
        )))
    }
}

fn validate_report_usize_field(
    context: &str,
    actual: usize,
    expected: usize,
) -> Result<(), CliError> {
    if actual == expected {
        Ok(())
    } else {
        Err(CliError::invalid(format!(
            "{context} {actual} does not match expected {expected}"
        )))
    }
}

fn validate_report_metric(context: &str, actual_text: &str, expected: f64) -> Result<(), CliError> {
    let actual = parse_f64(actual_text);
    if round4_cli(actual) == round4_cli(expected) {
        Ok(())
    } else {
        Err(CliError::invalid(format!(
            "{context} {} does not match expected {}",
            actual_text,
            format_signed_f64(expected)
        )))
    }
}

fn aggregate_expectation(report: &ResearchAggregateReport) -> ResearchBundleExpectation<'_> {
    ResearchBundleExpectation {
        engine_version: &report.engine_version,
        snapshot_id: &report.snapshot_id,
        provider_identity: &report.provider_identity,
        date_range: &report.date_range,
        gap_policy: &report.gap_policy,
        historical_limitations: &report.historical_limitations,
    }
}

fn walk_forward_expectation(report: &ResearchWalkForwardReport) -> ResearchBundleExpectation<'_> {
    ResearchBundleExpectation {
        engine_version: &report.engine_version,
        snapshot_id: &report.snapshot_id,
        provider_identity: &report.provider_identity,
        date_range: &report.date_range,
        gap_policy: &report.gap_policy,
        historical_limitations: &report.historical_limitations,
    }
}

fn format_optional(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{value:.4}"),
        None => "none".to_string(),
    }
}

fn format_optional_u64(value: Option<u64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "none".to_string(),
    }
}

fn format_optional_text(value: Option<&str>) -> String {
    value.unwrap_or("none").to_string()
}

fn manifest_parameter<'a>(manifest: &'a RunManifest, name: &str) -> Option<&'a ManifestParameter> {
    manifest
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
}

fn manifest_parameter_value_or<'a>(
    manifest: &'a RunManifest,
    name: &str,
    default: &'a str,
) -> &'a str {
    manifest_parameter(manifest, name)
        .map(|parameter| parameter.value.as_str())
        .unwrap_or(default)
}

fn format_f64(value: f64) -> String {
    format!("{value:.4}")
}

fn format_signed_f64(value: f64) -> String {
    if value >= 0.0 {
        format!("+{value:.4}")
    } else {
        format!("{value:.4}")
    }
}

fn parse_f64(value: &str) -> f64 {
    value
        .parse::<f64>()
        .expect("formatted numeric output should parse back to f64")
}

fn mean_f64(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    let position = percentile * (sorted_values.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted_values[lower_index]
    } else {
        let lower_value = sorted_values[lower_index];
        let upper_value = sorted_values[upper_index];
        let weight = position - lower_index as f64;

        lower_value + ((upper_value - lower_value) * weight)
    }
}

fn round4_cli(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn count_entry_trades(ledger: &[PersistedLedgerRow]) -> usize {
    let mut previous_shares = 0_u32;
    let mut trade_count = 0_usize;

    for row in ledger {
        if row.position_shares > previous_shares {
            trade_count += 1;
        }

        previous_shares = row.position_shares;
    }

    trade_count
}

fn format_reason_codes(reason_codes: &[String]) -> String {
    if reason_codes.is_empty() {
        "none".to_string()
    } else {
        reason_codes.join("|")
    }
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

fn usage_text() -> String {
    [
        "usage:",
        &format!("  {RUN_USAGE}"),
        &format!("  {EXPLAIN_USAGE}"),
        &format!("  {DIFF_USAGE}"),
        &format!("  {AUDIT_DATA_USAGE}"),
        &format!("  {RESEARCH_AGGREGATE_USAGE}"),
        &format!("  {RESEARCH_EXPLAIN_USAGE}"),
        &format!("  {RESEARCH_WALK_FORWARD_USAGE}"),
        &format!("  {RESEARCH_BOOTSTRAP_AGGREGATE_USAGE}"),
        &format!("  {RESEARCH_BOOTSTRAP_WALK_FORWARD_USAGE}"),
        &format!("  {RESEARCH_LEADERBOARD_USAGE}"),
    ]
    .join("\n")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapRng {
    state: u64,
}

impl BootstrapRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_index(&mut self, upper_bound: usize) -> usize {
        self.next_bounded_u64(upper_bound as u64) as usize
    }

    fn next_bounded_u64(&mut self, upper_bound: u64) -> u64 {
        if upper_bound == 1 {
            return 0;
        }

        let zone = u64::MAX - (u64::MAX % upper_bound);

        loop {
            let value = self.next_u64();
            if value < zone {
                return value % upper_bound;
            }
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D049BB133111EB);
        mixed ^ (mixed >> 31)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use trendlab_artifact::{
        LeaderboardView, ResearchReport, RunManifest, load_replay_bundle,
        load_research_report_bundle, write_replay_bundle, write_research_report_bundle,
    };
    use trendlab_core::accounting::CostModel;
    use trendlab_core::engine::{ReferenceFlowSpec, RunRequest};
    use trendlab_core::market::DailyBar;
    use trendlab_core::orders::{EntryIntent, GapPolicy, OrderIntent};
    use trendlab_data::provider::ProviderIdentity;

    use trendlab_operator::{
        DEFAULT_ENGINE_VERSION, OperatorRunManifestSpec, OperatorRunRequestTemplate,
        OperatorRunSpec, OperatorSnapshotSourceSpec, RUN_REQUEST_SOURCE_PARAMETER,
        RUN_SOURCE_KIND_PARAMETER, RUN_SPEC_SOURCE_PARAMETER, SNAPSHOT_SELECTION_END_PARAMETER,
        SNAPSHOT_SELECTION_START_PARAMETER, SNAPSHOT_SOURCE_PATH_PARAMETER,
        STRATEGY_EXECUTION_PARAMETER, STRATEGY_FILTER_PARAMETER, STRATEGY_POSITION_PARAMETER,
        STRATEGY_SIGNAL_PARAMETER, StrategyComponentLabels,
    };

    use crate::{CliResponse, dispatch};

    #[test]
    fn run_command_writes_bundle_and_explain_surfaces_audit_rows() {
        let request_path = test_output_dir("cli-request").join("request.json");
        let bundle_dir = test_output_dir("cli-bundle");
        write_request(&request_path, &sample_request());

        let run_response = dispatch([
            "run",
            "--request",
            request_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
        ]);

        assert_eq!(run_response.exit_code, 0, "{}", run_response.stderr);
        assert!(run_response.stdout.contains("wrote replay bundle"));
        assert!(bundle_dir.join("bundle.json").is_file());

        let bundle = load_replay_bundle(&bundle_dir).unwrap();
        assert_eq!(bundle.manifest.provider_identity, "fixture");
        assert_eq!(bundle.manifest.symbol_or_universe, "TEST");
        assert_eq!(bundle.summary.row_count, 2);
        assert_eq!(bundle.manifest.parameters.len(), 2);
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER),
            "request"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "request.json"
        );

        let explain_response = dispatch(["explain", bundle_dir.to_str().unwrap()]);

        assert_eq!(explain_response.exit_code, 0, "{}", explain_response.stderr);
        assert!(explain_response.stdout.contains("run_source_kind: request"));
        assert!(explain_response.stdout.contains("symbol: TEST"));
        assert!(
            explain_response
                .stdout
                .contains("request_source: request.json")
        );
        assert!(explain_response.stdout.contains("spec_source: none"));
        assert!(
            explain_response
                .stdout
                .contains("parameters: run_source_kind=request|run_request_source=request.json")
        );
        assert!(explain_response.stdout.contains("warnings: none"));
        assert!(explain_response.stdout.contains("rows: 2"));
        assert!(
            explain_response
                .stdout
                .contains("2025-01-02 shares=0 signal=queue_market_entry")
        );
        assert!(
            explain_response
                .stdout
                .contains("reasons=entry_intent_queued")
        );

        remove_dir_all_if_exists(request_path.parent().unwrap());
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_accepts_inline_spec_and_preserves_manifest_provenance() {
        let spec_path = test_output_dir("cli-inline-spec").join("run-spec.json");
        let bundle_dir = test_output_dir("cli-inline-spec-bundle");
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: None,
                request: Some(sample_request()),
                snapshot_source: None,
                request_template: None,
                manifest: OperatorRunManifestSpec {
                    provider_identity: Some(ProviderIdentity::Fixture),
                    snapshot_id: Some("snapshot:inline".to_string()),
                    engine_version: Some("m3-inline-spec".to_string()),
                    strategy_components: Some(StrategyComponentLabels {
                        signal_id: "breakout-close".to_string(),
                        filter_id: "pass-through".to_string(),
                        position_manager_id: "keep-position".to_string(),
                        execution_model_id: "next-open".to_string(),
                    }),
                },
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);

        let bundle = load_replay_bundle(&bundle_dir).unwrap();
        assert_eq!(bundle.manifest.engine_version, "m3-inline-spec");
        assert_eq!(bundle.manifest.data_snapshot_id, "snapshot:inline");
        assert_eq!(bundle.manifest.provider_identity, "fixture");
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER),
            "request"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "inline"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SPEC_SOURCE_PARAMETER),
            "run-spec.json"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, STRATEGY_SIGNAL_PARAMETER),
            "breakout-close"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, STRATEGY_FILTER_PARAMETER),
            "pass-through"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, STRATEGY_POSITION_PARAMETER),
            "keep-position"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, STRATEGY_EXECUTION_PARAMETER),
            "next-open"
        );

        remove_dir_all_if_exists(spec_path.parent().unwrap());
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_accepts_spec_with_relative_request_path() {
        let spec_dir = test_output_dir("cli-relative-spec");
        let request_path = spec_dir.join("inputs").join("request.json");
        let spec_path = spec_dir.join("run-spec.json");
        let bundle_dir = test_output_dir("cli-relative-spec-bundle");
        write_request(&request_path, &sample_request());
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: Some("inputs/request.json".to_string()),
                request: None,
                snapshot_source: None,
                request_template: None,
                manifest: OperatorRunManifestSpec {
                    provider_identity: Some(ProviderIdentity::Fixture),
                    snapshot_id: None,
                    engine_version: None,
                    strategy_components: None,
                },
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);

        let bundle = load_replay_bundle(&bundle_dir).unwrap();
        assert_eq!(bundle.manifest.engine_version, DEFAULT_ENGINE_VERSION);
        assert_eq!(bundle.manifest.data_snapshot_id, "adhoc:request");
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER),
            "request"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "inputs/request.json"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SPEC_SOURCE_PARAMETER),
            "run-spec.json"
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_rejects_spec_with_cli_manifest_overrides() {
        let spec_path = test_output_dir("cli-spec-overrides").join("run-spec.json");
        let bundle_dir = test_output_dir("cli-spec-overrides-bundle");
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: None,
                request: Some(sample_request()),
                snapshot_source: None,
                request_template: None,
                manifest: OperatorRunManifestSpec::default(),
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
        ]);

        assert_eq!(response.exit_code, 1);
        assert_eq!(
            response.stderr,
            "run --spec cannot be combined with --provider, --snapshot-id, --engine-version, or strategy-component flags"
        );

        remove_dir_all_if_exists(spec_path.parent().unwrap());
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_accepts_snapshot_source_spec_and_preserves_snapshot_provenance() {
        let spec_dir = test_output_dir("cli-snapshot-spec");
        let snapshot_dir = spec_dir.join("snapshots").join("sample");
        let spec_path = spec_dir.join("run-spec.json");
        let bundle_dir = test_output_dir("cli-snapshot-spec-bundle");
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
                    engine_version: Some("m9-snapshot-spec".to_string()),
                    strategy_components: None,
                },
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);

        let bundle = load_replay_bundle(&bundle_dir).unwrap();
        assert_eq!(bundle.manifest.engine_version, "m9-snapshot-spec");
        assert_eq!(
            bundle.manifest.data_snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(bundle.manifest.provider_identity, "tiingo");
        assert_eq!(bundle.manifest.symbol_or_universe, "TEST");
        assert_eq!(bundle.manifest.date_range.start_date, "2025-01-03");
        assert_eq!(bundle.manifest.date_range.end_date, "2025-01-07");
        assert_eq!(bundle.summary.row_count, 3);
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SOURCE_KIND_PARAMETER),
            "snapshot"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_REQUEST_SOURCE_PARAMETER),
            "inline_template"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, RUN_SPEC_SOURCE_PARAMETER),
            "run-spec.json"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, SNAPSHOT_SOURCE_PATH_PARAMETER),
            "snapshots/sample"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, SNAPSHOT_SELECTION_START_PARAMETER),
            "2025-01-03"
        );
        assert_eq!(
            manifest_parameter_value(&bundle.manifest, SNAPSHOT_SELECTION_END_PARAMETER),
            "2025-01-07"
        );

        let explain_response = dispatch(["explain", bundle_dir.to_str().unwrap()]);
        assert_eq!(explain_response.exit_code, 0, "{}", explain_response.stderr);
        assert!(
            explain_response
                .stdout
                .contains("run_source_kind: snapshot")
        );
        assert!(
            explain_response
                .stdout
                .contains("request_source: inline_template")
        );
        assert!(
            explain_response
                .stdout
                .contains("spec_source: run-spec.json")
        );
        assert!(
            explain_response
                .stdout
                .contains("snapshot_source_path: snapshots/sample")
        );
        assert!(
            explain_response
                .stdout
                .contains("snapshot_selection: 2025-01-03..2025-01-07")
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_rejects_snapshot_spec_manifest_identity_overrides() {
        let spec_dir = test_output_dir("cli-snapshot-spec-overrides");
        let snapshot_dir = spec_dir.join("snapshots").join("sample");
        let spec_path = spec_dir.join("run-spec.json");
        let bundle_dir = test_output_dir("cli-snapshot-spec-overrides-bundle");
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
                    provider_identity: Some(ProviderIdentity::Fixture),
                    snapshot_id: None,
                    engine_version: None,
                    strategy_components: None,
                },
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 1);
        assert_eq!(
            response.stderr,
            "snapshot-backed run specs must not override provider_identity or snapshot_id in manifest"
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_rejects_snapshot_spec_with_empty_snapshot_dir() {
        let spec_path = test_output_dir("cli-snapshot-empty-dir").join("run-spec.json");
        let bundle_dir = test_output_dir("cli-snapshot-empty-dir-bundle");
        write_run_spec(
            &spec_path,
            &OperatorRunSpec {
                request_path: None,
                request: None,
                snapshot_source: Some(OperatorSnapshotSourceSpec {
                    snapshot_dir: " ".to_string(),
                    symbol: "TEST".to_string(),
                    start_date: "2025-01-03".to_string(),
                    end_date: "2025-01-07".to_string(),
                }),
                request_template: Some(sample_request_template()),
                manifest: OperatorRunManifestSpec::default(),
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 1);
        assert_eq!(
            response.stderr,
            "snapshot-backed run specs require a non-empty snapshot_source.snapshot_dir"
        );

        remove_dir_all_if_exists(spec_path.parent().unwrap());
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_rejects_snapshot_spec_with_empty_end_date() {
        let spec_dir = test_output_dir("cli-snapshot-empty-end");
        let snapshot_dir = spec_dir.join("snapshots").join("sample");
        let spec_path = spec_dir.join("run-spec.json");
        let bundle_dir = test_output_dir("cli-snapshot-empty-end-bundle");
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
                    end_date: " ".to_string(),
                }),
                request_template: Some(sample_request_template()),
                manifest: OperatorRunManifestSpec::default(),
            },
        );

        let response = dispatch([
            "run",
            "--spec",
            spec_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 1);
        assert_eq!(
            response.stderr,
            "snapshot-backed run specs require a non-empty snapshot_source.end_date"
        );

        remove_dir_all_if_exists(&spec_dir);
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn run_command_rejects_unknown_provider() {
        let response = dispatch([
            "run",
            "--request",
            "request.json",
            "--output",
            "bundle-dir",
            "--provider",
            "unknown",
        ]);

        assert_eq!(response.exit_code, 1);
        assert_eq!(response.stderr, "unknown provider `unknown`");
    }

    #[test]
    fn diff_command_reports_manifest_summary_and_ledger_changes() {
        let left_request_path = test_output_dir("cli-diff-left-request").join("request.json");
        let right_request_path = test_output_dir("cli-diff-right-request").join("request.json");
        let left_bundle_dir = test_output_dir("cli-diff-left-bundle");
        let right_bundle_dir = test_output_dir("cli-diff-right-bundle");
        write_request(&left_request_path, &sample_request());
        write_request(
            &right_request_path,
            &sample_request_with_modified_terminal_bar(),
        );

        let left_run = dispatch([
            "run",
            "--request",
            left_request_path.to_str().unwrap(),
            "--output",
            left_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--engine-version",
            "m1-reference-flow-left",
        ]);
        let right_run = dispatch([
            "run",
            "--request",
            right_request_path.to_str().unwrap(),
            "--output",
            right_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--engine-version",
            "m1-reference-flow-right",
        ]);

        assert_eq!(left_run.exit_code, 0, "{}", left_run.stderr);
        assert_eq!(right_run.exit_code, 0, "{}", right_run.stderr);

        let diff_response = dispatch([
            "diff",
            left_bundle_dir.to_str().unwrap(),
            right_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(diff_response.exit_code, 0, "{}", diff_response.stderr);
        assert!(diff_response.stdout.contains("equal: no"));
        assert!(diff_response.stdout.contains(
            "manifest.engine_version: left=m1-reference-flow-left right=m1-reference-flow-right"
        ));
        assert!(
            diff_response
                .stdout
                .contains("summary.ending_equity: left=1001.0000 right=1002.0000")
        );
        assert!(
            diff_response
                .stdout
                .contains("ledger[1].raw_close: left_date=2025-01-03 right_date=2025-01-03 left=103.0000 right=104.0000")
        );

        remove_dir_all_if_exists(left_request_path.parent().unwrap());
        remove_dir_all_if_exists(right_request_path.parent().unwrap());
        remove_dir_all_if_exists(&left_bundle_dir);
        remove_dir_all_if_exists(&right_bundle_dir);
    }

    #[test]
    fn audit_data_command_summarizes_price_space_differences() {
        let request_path = test_output_dir("cli-audit-request").join("request.json");
        let bundle_dir = test_output_dir("cli-audit-bundle");
        write_request(&request_path, &sample_request_with_analysis_gap());

        let run_response = dispatch([
            "run",
            "--request",
            request_path.to_str().unwrap(),
            "--output",
            bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(run_response.exit_code, 0, "{}", run_response.stderr);

        let audit_response = dispatch(["audit", "data", bundle_dir.to_str().unwrap()]);

        assert_eq!(audit_response.exit_code, 0, "{}", audit_response.stderr);
        assert!(audit_response.stdout.contains("analysis_adjusted_bars: 1"));
        assert!(
            audit_response
                .stdout
                .contains("analysis_matches_raw_close: 1")
        );
        assert!(
            audit_response
                .stdout
                .contains("max_analysis_close_gap: 50.2500")
        );
        assert!(
            audit_response
                .stdout
                .contains("max_analysis_close_gap_date: 2025-01-02")
        );
        assert!(audit_response.stdout.contains("findings: none"));

        remove_dir_all_if_exists(request_path.parent().unwrap());
        remove_dir_all_if_exists(&bundle_dir);
    }

    #[test]
    fn audit_snapshot_command_summarizes_stored_snapshot_inputs() {
        let snapshot_dir = test_output_dir("cli-audit-snapshot");
        write_sample_snapshot_bundle(&snapshot_dir);

        let audit_response = dispatch(["audit", "snapshot", snapshot_dir.to_str().unwrap()]);

        assert_eq!(audit_response.exit_code, 0, "{}", audit_response.stderr);
        assert!(
            audit_response
                .stdout
                .contains("snapshot_id: live:tiingo:TEST:2025-01-03:2025-01-08")
        );
        assert!(audit_response.stdout.contains("provider: tiingo"));
        assert!(
            audit_response
                .stdout
                .contains("requested_window: 2025-01-02..2025-01-10")
        );
        assert!(audit_response.stdout.contains("symbol: TEST"));
        assert!(audit_response.stdout.contains("raw_bars: 4"));
        assert!(audit_response.stdout.contains("corporate_actions: 2"));
        assert!(audit_response.stdout.contains("split_actions: 1"));
        assert!(audit_response.stdout.contains("cash_dividends: 1"));
        assert!(
            audit_response
                .stdout
                .contains("normalized_bars: daily=4 weekly=2 monthly=1")
        );
        assert!(
            audit_response
                .stdout
                .contains("normalization_input: ex_date=2025-01-06 split_ratio=2.0000 cash_dividend_per_share=0.0000")
        );
        assert!(
            audit_response
                .stdout
                .contains("normalization_input: ex_date=2025-01-07 split_ratio=1.0000 cash_dividend_per_share=0.2500")
        );
        assert!(audit_response.stdout.contains("findings: none"));

        remove_dir_all_if_exists(&snapshot_dir);
    }

    #[test]
    fn research_aggregate_command_summarizes_cross_symbol_bundles_with_drilldown() {
        let alpha_request_path =
            test_output_dir("cli-aggregate-alpha-request").join("request.json");
        let beta_request_path = test_output_dir("cli-aggregate-beta-request").join("request.json");
        let alpha_bundle_dir = test_output_dir("cli-aggregate-alpha-bundle");
        let beta_bundle_dir = test_output_dir("cli-aggregate-beta-bundle");
        write_request(&alpha_request_path, &sample_request_for_symbol("ALPHA"));
        write_request(
            &beta_request_path,
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
        );

        let alpha_run = dispatch([
            "run",
            "--request",
            alpha_request_path.to_str().unwrap(),
            "--output",
            alpha_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_cross_symbol_aggregation",
            "--engine-version",
            "m6-aggregate-reference-flow",
        ]);
        let beta_run = dispatch([
            "run",
            "--request",
            beta_request_path.to_str().unwrap(),
            "--output",
            beta_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_cross_symbol_aggregation",
            "--engine-version",
            "m6-aggregate-reference-flow",
        ]);

        assert_eq!(alpha_run.exit_code, 0, "{}", alpha_run.stderr);
        assert_eq!(beta_run.exit_code, 0, "{}", beta_run.stderr);

        let aggregate_response = dispatch([
            "research",
            "aggregate",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(
            aggregate_response.exit_code, 0,
            "{}",
            aggregate_response.stderr
        );
        assert!(
            aggregate_response
                .stdout
                .contains("snapshot_id: fixture:m6_cross_symbol_aggregation")
        );
        assert!(aggregate_response.stdout.contains("symbol_count: 2"));
        assert!(aggregate_response.stdout.contains("symbols: ALPHA|BETA"));
        assert!(
            aggregate_response
                .stdout
                .contains("starting_equity_total: 2000.0000")
        );
        assert!(
            aggregate_response
                .stdout
                .contains("ending_equity_total: 1998.0000")
        );
        assert!(
            aggregate_response
                .stdout
                .contains("net_equity_change_total: -2.0000")
        );
        assert!(
            aggregate_response
                .stdout
                .contains("average_net_equity_change: -1.0000")
        );
        assert!(aggregate_response.stdout.contains("total_trades: 2"));
        assert!(
            aggregate_response
                .stdout
                .contains(&format!("bundle={}", alpha_bundle_dir.display()))
        );
        assert!(
            aggregate_response
                .stdout
                .contains(&format!("bundle={}", beta_bundle_dir.display()))
        );

        remove_dir_all_if_exists(alpha_request_path.parent().unwrap());
        remove_dir_all_if_exists(beta_request_path.parent().unwrap());
        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
    }

    #[test]
    fn research_aggregate_command_writes_shared_report_bundle() {
        let snapshot_id = "fixture:m6_aggregate_persisted";
        let engine_version = "m6-aggregate-persisted-reference-flow";
        let alpha_bundle_dir = write_labeled_bundle(
            "cli-aggregate-persisted-alpha",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle_dir = write_labeled_bundle(
            "cli-aggregate-persisted-beta",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            None,
        );
        let report_dir = test_output_dir("cli-aggregate-report");

        let response = dispatch([
            "research",
            "aggregate",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("wrote research report"));

        let report = load_research_report_bundle(&report_dir).unwrap();
        let ResearchReport::Aggregate(report) = report else {
            panic!("expected aggregate research report");
        };
        assert_eq!(
            report.symbols,
            vec!["ALPHA".to_string(), "BETA".to_string()]
        );
        assert_eq!(report.members.len(), 2);
        assert_eq!(report.members[0].bundle_path, alpha_bundle_dir);
        assert_eq!(report.members[1].bundle_path, beta_bundle_dir);
        assert_research_explain_matches_saved_output(&response, &report_dir);

        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_explain_rejects_missing_aggregate_member_bundle() {
        let snapshot_id = "fixture:m6_aggregate_missing_member";
        let engine_version = "m6-aggregate-missing-member-reference-flow";
        let alpha_bundle_dir = write_labeled_bundle(
            "cli-aggregate-missing-member-alpha",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle_dir = write_labeled_bundle(
            "cli-aggregate-missing-member-beta",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            None,
        );
        let report_dir = test_output_dir("cli-aggregate-missing-member-report");

        let response = dispatch([
            "research",
            "aggregate",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        fs::remove_dir_all(&alpha_bundle_dir).unwrap();

        let explain_response = dispatch(["research", "explain", report_dir.to_str().unwrap()]);

        assert_eq!(explain_response.exit_code, 1);
        assert!(
            explain_response
                .stderr
                .contains("research report requires replay bundle")
        );
        assert!(
            explain_response
                .stderr
                .contains(&alpha_bundle_dir.display().to_string())
        );

        remove_dir_all_if_exists(&beta_bundle_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_aggregate_command_rejects_mismatched_snapshot_ids() {
        let left_request_path = test_output_dir("cli-aggregate-left-request").join("request.json");
        let right_request_path =
            test_output_dir("cli-aggregate-right-request").join("request.json");
        let left_bundle_dir = test_output_dir("cli-aggregate-left-bundle");
        let right_bundle_dir = test_output_dir("cli-aggregate-right-bundle");
        write_request(&left_request_path, &sample_request_for_symbol("LEFT"));
        write_request(&right_request_path, &sample_request_for_symbol("RIGHT"));

        let left_run = dispatch([
            "run",
            "--request",
            left_request_path.to_str().unwrap(),
            "--output",
            left_bundle_dir.to_str().unwrap(),
            "--snapshot-id",
            "fixture:m6_left",
        ]);
        let right_run = dispatch([
            "run",
            "--request",
            right_request_path.to_str().unwrap(),
            "--output",
            right_bundle_dir.to_str().unwrap(),
            "--snapshot-id",
            "fixture:m6_right",
        ]);

        assert_eq!(left_run.exit_code, 0, "{}", left_run.stderr);
        assert_eq!(right_run.exit_code, 0, "{}", right_run.stderr);

        let aggregate_response = dispatch([
            "research",
            "aggregate",
            left_bundle_dir.to_str().unwrap(),
            right_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(aggregate_response.exit_code, 1);
        assert!(
            aggregate_response
                .stderr
                .contains("research aggregate requires matching snapshot_id")
        );

        remove_dir_all_if_exists(left_request_path.parent().unwrap());
        remove_dir_all_if_exists(right_request_path.parent().unwrap());
        remove_dir_all_if_exists(&left_bundle_dir);
        remove_dir_all_if_exists(&right_bundle_dir);
    }

    #[test]
    fn research_walk_forward_command_generates_deterministic_splits_with_bundle_links() {
        let alpha_request_path =
            test_output_dir("cli-walk-forward-alpha-request").join("request.json");
        let beta_request_path =
            test_output_dir("cli-walk-forward-beta-request").join("request.json");
        let alpha_bundle_dir = test_output_dir("cli-walk-forward-alpha-bundle");
        let beta_bundle_dir = test_output_dir("cli-walk-forward-beta-bundle");
        write_request(
            &alpha_request_path,
            &sample_walk_forward_request_for_symbol("ALPHA"),
        );
        write_request(
            &beta_request_path,
            &sample_walk_forward_request_for_symbol("BETA"),
        );

        let alpha_run = dispatch([
            "run",
            "--request",
            alpha_request_path.to_str().unwrap(),
            "--output",
            alpha_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_walk_forward",
            "--engine-version",
            "m6-walk-forward-reference-flow",
        ]);
        let beta_run = dispatch([
            "run",
            "--request",
            beta_request_path.to_str().unwrap(),
            "--output",
            beta_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_walk_forward",
            "--engine-version",
            "m6-walk-forward-reference-flow",
        ]);

        assert_eq!(alpha_run.exit_code, 0, "{}", alpha_run.stderr);
        assert_eq!(beta_run.exit_code, 0, "{}", beta_run.stderr);

        let walk_forward_response = dispatch([
            "research",
            "walk-forward",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            "--step-bars",
            "1",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(
            walk_forward_response.exit_code, 0,
            "{}",
            walk_forward_response.stderr
        );
        assert!(
            walk_forward_response
                .stdout
                .contains("research walk-forward")
        );
        assert!(walk_forward_response.stdout.contains("symbols: ALPHA|BETA"));
        assert!(walk_forward_response.stdout.contains("train_bars: 3"));
        assert!(walk_forward_response.stdout.contains("test_bars: 2"));
        assert!(walk_forward_response.stdout.contains("step_bars: 1"));
        assert!(walk_forward_response.stdout.contains("split_count: 2"));
        assert!(
            walk_forward_response
                .stdout
                .contains("split: id=1 train_rows=0..2 train_dates=2025-01-02..2025-01-06 test_rows=3..4 test_dates=2025-01-07..2025-01-08")
        );
        assert!(
            walk_forward_response
                .stdout
                .contains("split: id=2 train_rows=1..3 train_dates=2025-01-03..2025-01-07 test_rows=4..5 test_dates=2025-01-08..2025-01-09")
        );
        assert!(walk_forward_response.stdout.contains(&format!(
            "child: split=1 symbol=ALPHA bundle={}",
            alpha_bundle_dir.display()
        )));
        assert!(walk_forward_response.stdout.contains(&format!(
            "child: split=2 symbol=BETA bundle={}",
            beta_bundle_dir.display()
        )));

        remove_dir_all_if_exists(alpha_request_path.parent().unwrap());
        remove_dir_all_if_exists(beta_request_path.parent().unwrap());
        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
    }

    #[test]
    fn research_walk_forward_command_writes_shared_report_bundle() {
        let snapshot_id = "fixture:m6_walk_forward_persisted";
        let engine_version = "m6-walk-forward-persisted-reference-flow";
        let alpha_bundle_dir = write_labeled_bundle(
            "cli-walk-forward-persisted-alpha",
            &sample_walk_forward_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle_dir = write_labeled_bundle(
            "cli-walk-forward-persisted-beta",
            &sample_walk_forward_request_for_symbol("BETA"),
            snapshot_id,
            engine_version,
            None,
        );
        let report_dir = test_output_dir("cli-walk-forward-report");

        let response = dispatch([
            "research",
            "walk-forward",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            "--step-bars",
            "1",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("wrote research report"));

        let report = load_research_report_bundle(&report_dir).unwrap();
        let ResearchReport::WalkForward(report) = report else {
            panic!("expected walk-forward research report");
        };
        assert_eq!(report.split_count, 2);
        assert_eq!(report.splits[0].children.len(), 2);
        assert_eq!(report.splits[0].children[0].bundle_path, alpha_bundle_dir);
        assert_eq!(report.splits[0].children[1].bundle_path, beta_bundle_dir);
        assert_research_explain_matches_saved_output(&response, &report_dir);

        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_walk_forward_command_rejects_mismatched_ledger_date_sequences() {
        let alpha_request_path =
            test_output_dir("cli-walk-forward-mismatch-alpha-request").join("request.json");
        let beta_request_path =
            test_output_dir("cli-walk-forward-mismatch-beta-request").join("request.json");
        let alpha_bundle_dir = test_output_dir("cli-walk-forward-mismatch-alpha-bundle");
        let beta_bundle_dir = test_output_dir("cli-walk-forward-mismatch-beta-bundle");
        write_request(
            &alpha_request_path,
            &sample_walk_forward_request_for_symbol("ALPHA"),
        );
        write_request(
            &beta_request_path,
            &sample_walk_forward_request_with_shifted_date("BETA"),
        );

        let alpha_run = dispatch([
            "run",
            "--request",
            alpha_request_path.to_str().unwrap(),
            "--output",
            alpha_bundle_dir.to_str().unwrap(),
            "--snapshot-id",
            "fixture:m6_walk_forward_mismatch",
        ]);
        let beta_run = dispatch([
            "run",
            "--request",
            beta_request_path.to_str().unwrap(),
            "--output",
            beta_bundle_dir.to_str().unwrap(),
            "--snapshot-id",
            "fixture:m6_walk_forward_mismatch",
        ]);

        assert_eq!(alpha_run.exit_code, 0, "{}", alpha_run.stderr);
        assert_eq!(beta_run.exit_code, 0, "{}", beta_run.stderr);

        let walk_forward_response = dispatch([
            "research",
            "walk-forward",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(walk_forward_response.exit_code, 1);
        assert!(
            walk_forward_response
                .stderr
                .contains("research walk-forward requires matching ledger date sequences")
        );

        remove_dir_all_if_exists(alpha_request_path.parent().unwrap());
        remove_dir_all_if_exists(beta_request_path.parent().unwrap());
        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
    }

    #[test]
    fn research_bootstrap_aggregate_command_is_seeded_and_preserves_member_drilldown() {
        let alpha_request_path =
            test_output_dir("cli-bootstrap-aggregate-alpha-request").join("request.json");
        let beta_request_path =
            test_output_dir("cli-bootstrap-aggregate-beta-request").join("request.json");
        let alpha_bundle_dir = test_output_dir("cli-bootstrap-aggregate-alpha-bundle");
        let beta_bundle_dir = test_output_dir("cli-bootstrap-aggregate-beta-bundle");
        write_request(&alpha_request_path, &sample_request_for_symbol("ALPHA"));
        write_request(
            &beta_request_path,
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
        );

        let alpha_run = dispatch([
            "run",
            "--request",
            alpha_request_path.to_str().unwrap(),
            "--output",
            alpha_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_bootstrap_aggregate",
            "--engine-version",
            "m6-bootstrap-aggregate-reference-flow",
        ]);
        let beta_run = dispatch([
            "run",
            "--request",
            beta_request_path.to_str().unwrap(),
            "--output",
            beta_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_bootstrap_aggregate",
            "--engine-version",
            "m6-bootstrap-aggregate-reference-flow",
        ]);

        assert_eq!(alpha_run.exit_code, 0, "{}", alpha_run.stderr);
        assert_eq!(beta_run.exit_code, 0, "{}", beta_run.stderr);

        let bootstrap_response = dispatch([
            "research",
            "bootstrap",
            "aggregate",
            "--samples",
            "5",
            "--seed",
            "7",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);
        let repeat_response = dispatch([
            "research",
            "bootstrap",
            "aggregate",
            "--samples",
            "5",
            "--seed",
            "7",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(
            bootstrap_response.exit_code, 0,
            "{}",
            bootstrap_response.stderr
        );
        assert_eq!(bootstrap_response.stdout, repeat_response.stdout);
        assert!(
            bootstrap_response
                .stdout
                .contains("research bootstrap aggregate")
        );
        assert!(
            bootstrap_response
                .stdout
                .contains("baseline_average_net_equity_change: -1.0000")
        );
        assert!(bootstrap_response.stdout.contains("seed: 7"));
        assert!(bootstrap_response.stdout.contains("samples: 5"));
        assert!(bootstrap_response.stdout.contains("resample_unit: symbol"));
        assert!(
            bootstrap_response
                .stdout
                .contains("metric: average_net_equity_change")
        );
        assert!(bootstrap_response.stdout.contains("bootstrap_interval_95:"));
        assert!(
            bootstrap_response
                .stdout
                .contains(&format!("bundle={}", alpha_bundle_dir.display()))
        );
        assert!(
            bootstrap_response
                .stdout
                .contains(&format!("bundle={}", beta_bundle_dir.display()))
        );

        remove_dir_all_if_exists(alpha_request_path.parent().unwrap());
        remove_dir_all_if_exists(beta_request_path.parent().unwrap());
        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
    }

    #[test]
    fn research_bootstrap_aggregate_command_writes_shared_report_bundle() {
        let snapshot_id = "fixture:m6_bootstrap_aggregate_persisted";
        let engine_version = "m6-bootstrap-aggregate-persisted-reference-flow";
        let alpha_bundle_dir = write_labeled_bundle(
            "cli-bootstrap-aggregate-persisted-alpha",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle_dir = write_labeled_bundle(
            "cli-bootstrap-aggregate-persisted-beta",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            None,
        );
        let report_dir = test_output_dir("cli-bootstrap-aggregate-report");

        let response = dispatch([
            "research",
            "bootstrap",
            "aggregate",
            "--samples",
            "5",
            "--seed",
            "7",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("wrote research report"));

        let report = load_research_report_bundle(&report_dir).unwrap();
        let ResearchReport::BootstrapAggregate(report) = report else {
            panic!("expected bootstrap aggregate research report");
        };
        assert_eq!(report.distribution.seed, 7);
        assert_eq!(report.distribution.sample_count, 5);
        assert_eq!(report.baseline.members.len(), 2);
        assert_eq!(report.baseline.members[0].bundle_path, alpha_bundle_dir);
        assert_eq!(report.baseline.members[1].bundle_path, beta_bundle_dir);
        assert_research_explain_matches_saved_output(&response, &report_dir);

        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_bootstrap_walk_forward_command_writes_shared_report_bundle() {
        let snapshot_id = "fixture:m6_bootstrap_walk_forward_persisted";
        let engine_version = "m6-bootstrap-walk-forward-persisted-reference-flow";
        let alpha_bundle_dir = write_labeled_bundle(
            "cli-bootstrap-walk-forward-persisted-alpha",
            &sample_walk_forward_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle_dir = write_labeled_bundle(
            "cli-bootstrap-walk-forward-persisted-beta",
            &sample_walk_forward_request_for_symbol_with_closes(
                "BETA",
                [100.5, 101.5, 102.5, 101.5, 105.5, 106.5],
            ),
            snapshot_id,
            engine_version,
            None,
        );
        let report_dir = test_output_dir("cli-bootstrap-walk-forward-report");

        let response = dispatch([
            "research",
            "bootstrap",
            "walk-forward",
            "--samples",
            "6",
            "--seed",
            "11",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            "--step-bars",
            "1",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("wrote research report"));

        let report = load_research_report_bundle(&report_dir).unwrap();
        let ResearchReport::BootstrapWalkForward(report) = report else {
            panic!("expected bootstrap walk-forward research report");
        };
        assert_eq!(report.distribution.seed, 11);
        assert_eq!(report.distribution.sample_count, 6);
        assert_eq!(report.splits.len(), 2);
        assert_eq!(report.splits[0].children[0].bundle_path, alpha_bundle_dir);
        assert_eq!(report.splits[1].children[1].bundle_path, beta_bundle_dir);
        assert_research_explain_matches_saved_output(&response, &report_dir);

        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_bootstrap_walk_forward_command_is_seeded_and_keeps_split_drilldown() {
        let alpha_request_path =
            test_output_dir("cli-bootstrap-walk-forward-alpha-request").join("request.json");
        let beta_request_path =
            test_output_dir("cli-bootstrap-walk-forward-beta-request").join("request.json");
        let alpha_bundle_dir = test_output_dir("cli-bootstrap-walk-forward-alpha-bundle");
        let beta_bundle_dir = test_output_dir("cli-bootstrap-walk-forward-beta-bundle");
        write_request(
            &alpha_request_path,
            &sample_walk_forward_request_for_symbol("ALPHA"),
        );
        write_request(
            &beta_request_path,
            &sample_walk_forward_request_for_symbol_with_closes(
                "BETA",
                [100.5, 101.5, 102.5, 101.5, 105.5, 106.5],
            ),
        );

        let alpha_run = dispatch([
            "run",
            "--request",
            alpha_request_path.to_str().unwrap(),
            "--output",
            alpha_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_bootstrap_walk_forward",
            "--engine-version",
            "m6-bootstrap-walk-forward-reference-flow",
        ]);
        let beta_run = dispatch([
            "run",
            "--request",
            beta_request_path.to_str().unwrap(),
            "--output",
            beta_bundle_dir.to_str().unwrap(),
            "--provider",
            "fixture",
            "--snapshot-id",
            "fixture:m6_bootstrap_walk_forward",
            "--engine-version",
            "m6-bootstrap-walk-forward-reference-flow",
        ]);

        assert_eq!(alpha_run.exit_code, 0, "{}", alpha_run.stderr);
        assert_eq!(beta_run.exit_code, 0, "{}", beta_run.stderr);

        let bootstrap_response = dispatch([
            "research",
            "bootstrap",
            "walk-forward",
            "--samples",
            "6",
            "--seed",
            "11",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            "--step-bars",
            "1",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);
        let repeat_response = dispatch([
            "research",
            "bootstrap",
            "walk-forward",
            "--samples",
            "6",
            "--seed",
            "11",
            "--train-bars",
            "3",
            "--test-bars",
            "2",
            "--step-bars",
            "1",
            alpha_bundle_dir.to_str().unwrap(),
            beta_bundle_dir.to_str().unwrap(),
        ]);

        assert_eq!(
            bootstrap_response.exit_code, 0,
            "{}",
            bootstrap_response.stderr
        );
        assert_eq!(bootstrap_response.stdout, repeat_response.stdout);
        assert!(
            bootstrap_response
                .stdout
                .contains("research bootstrap walk-forward")
        );
        assert!(bootstrap_response.stdout.contains("seed: 11"));
        assert!(bootstrap_response.stdout.contains("samples: 6"));
        assert!(
            bootstrap_response
                .stdout
                .contains("resample_unit: walk_forward_split")
        );
        assert!(
            bootstrap_response
                .stdout
                .contains("metric: mean_split_test_average_net_equity_change")
        );
        assert!(
            bootstrap_response
                .stdout
                .contains("baseline_metric: +3.0000")
        );
        assert!(
            bootstrap_response
                .stdout
                .contains("split: id=1 train_rows=0..2 train_dates=2025-01-02..2025-01-06 test_rows=3..4 test_dates=2025-01-07..2025-01-08 baseline_test_total_net_equity_change=+5.0000 baseline_test_average_net_equity_change=+2.5000")
        );
        assert!(
            bootstrap_response
                .stdout
                .contains("split: id=2 train_rows=1..3 train_dates=2025-01-03..2025-01-07 test_rows=4..5 test_dates=2025-01-08..2025-01-09 baseline_test_total_net_equity_change=+7.0000 baseline_test_average_net_equity_change=+3.5000")
        );
        assert!(bootstrap_response.stdout.contains(&format!(
            "child: split=1 symbol=ALPHA bundle={}",
            alpha_bundle_dir.display()
        )));
        assert!(bootstrap_response.stdout.contains(&format!(
            "child: split=2 symbol=BETA bundle={}",
            beta_bundle_dir.display()
        )));

        remove_dir_all_if_exists(alpha_request_path.parent().unwrap());
        remove_dir_all_if_exists(beta_request_path.parent().unwrap());
        remove_dir_all_if_exists(&alpha_bundle_dir);
        remove_dir_all_if_exists(&beta_bundle_dir);
    }

    #[test]
    fn research_leaderboard_signal_view_ranks_signals_with_fixed_context() {
        let snapshot_id = "fixture:m6_leaderboard_signal";
        let engine_version = "m6-leaderboard-signal-reference-flow";
        let filter_id = "pass_filter";
        let position_manager_id = "keep_position_manager";
        let execution_model_id = "next_open_long";
        let close_signal = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id,
            position_manager_id,
            execution_model_id,
        };
        let stop_signal = TestStrategyLabels {
            signal_id: "stop_entry_breakout",
            filter_id,
            position_manager_id,
            execution_model_id,
        };
        let alpha_close_bundle = write_labeled_bundle(
            "cli-leaderboard-signal-alpha-close",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let beta_close_bundle = write_labeled_bundle(
            "cli-leaderboard-signal-beta-close",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let alpha_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-signal-alpha-stop",
            &sample_request_for_symbol_with_terminal_close("ALPHA", 106.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );
        let beta_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-signal-beta-stop",
            &sample_request_for_symbol_with_terminal_close("BETA", 102.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );

        let response = dispatch([
            "research",
            "leaderboard",
            "signal",
            alpha_close_bundle.to_str().unwrap(),
            beta_close_bundle.to_str().unwrap(),
            alpha_stop_bundle.to_str().unwrap(),
            beta_stop_bundle.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("research leaderboard signal"));
        assert!(response.stdout.contains("fixed_filter_id: pass_filter"));
        assert!(
            response
                .stdout
                .contains("fixed_position_manager_id: keep_position_manager")
        );
        assert!(
            response
                .stdout
                .contains("fixed_execution_model_id: next_open_long")
        );
        assert!(
            response
                .stdout
                .contains("row: rank=1 label=stop_entry_breakout")
        );
        assert!(
            response
                .stdout
                .contains("row: rank=2 label=close_confirmed_breakout")
        );
        assert!(
            response
                .stdout
                .contains("member: rank=1 label=stop_entry_breakout symbol=ALPHA")
        );
        assert!(
            response
                .stdout
                .contains(&format!("bundle={}", alpha_stop_bundle.display()))
        );

        remove_dir_all_if_exists(&alpha_close_bundle);
        remove_dir_all_if_exists(&beta_close_bundle);
        remove_dir_all_if_exists(&alpha_stop_bundle);
        remove_dir_all_if_exists(&beta_stop_bundle);
    }

    #[test]
    fn research_leaderboard_command_writes_shared_report_bundle() {
        let snapshot_id = "fixture:m6_leaderboard_persisted";
        let engine_version = "m6-leaderboard-persisted-reference-flow";
        let close_signal = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let stop_signal = TestStrategyLabels {
            signal_id: "stop_entry_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let alpha_close_bundle = write_labeled_bundle(
            "cli-leaderboard-persisted-alpha-close",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let beta_close_bundle = write_labeled_bundle(
            "cli-leaderboard-persisted-beta-close",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let alpha_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-persisted-alpha-stop",
            &sample_request_for_symbol_with_terminal_close("ALPHA", 106.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );
        let beta_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-persisted-beta-stop",
            &sample_request_for_symbol_with_terminal_close("BETA", 102.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );
        let report_dir = test_output_dir("cli-leaderboard-report");

        let response = dispatch([
            "research",
            "leaderboard",
            "signal",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_close_bundle.to_str().unwrap(),
            beta_close_bundle.to_str().unwrap(),
            alpha_stop_bundle.to_str().unwrap(),
            beta_stop_bundle.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("wrote research report"));

        let report = load_research_report_bundle(&report_dir).unwrap();
        let ResearchReport::Leaderboard(report) = report else {
            panic!("expected leaderboard research report");
        };
        assert_eq!(report.view, LeaderboardView::Signal);
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.rows[0].aggregate.members.len(), 2);
        assert_eq!(
            report.rows[0].aggregate.members[0].bundle_path,
            alpha_stop_bundle
        );
        assert_eq!(
            report.rows[1].aggregate.members[1].bundle_path,
            beta_close_bundle
        );
        assert_research_explain_matches_saved_output(&response, &report_dir);

        remove_dir_all_if_exists(&alpha_close_bundle);
        remove_dir_all_if_exists(&beta_close_bundle);
        remove_dir_all_if_exists(&alpha_stop_bundle);
        remove_dir_all_if_exists(&beta_stop_bundle);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_explain_rejects_leaderboard_bundle_with_missing_strategy_attribution() {
        let snapshot_id = "fixture:m6_leaderboard_missing_attr_on_reopen";
        let engine_version = "m6-leaderboard-missing-attr-on-reopen-reference-flow";
        let close_signal = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let stop_signal = TestStrategyLabels {
            signal_id: "stop_entry_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let alpha_close_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-attr-on-reopen-alpha-close",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let beta_close_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-attr-on-reopen-beta-close",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            Some(&close_signal),
        );
        let alpha_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-attr-on-reopen-alpha-stop",
            &sample_request_for_symbol_with_terminal_close("ALPHA", 106.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );
        let beta_stop_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-attr-on-reopen-beta-stop",
            &sample_request_for_symbol_with_terminal_close("BETA", 102.0),
            snapshot_id,
            engine_version,
            Some(&stop_signal),
        );
        let report_dir = test_output_dir("cli-leaderboard-missing-attr-on-reopen-report");

        let response = dispatch([
            "research",
            "leaderboard",
            "signal",
            "--output",
            report_dir.to_str().unwrap(),
            alpha_close_bundle.to_str().unwrap(),
            beta_close_bundle.to_str().unwrap(),
            alpha_stop_bundle.to_str().unwrap(),
            beta_stop_bundle.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        let stored_report = load_research_report_bundle(&report_dir).unwrap();

        let mut tampered = load_replay_bundle(&alpha_stop_bundle).unwrap();
        tampered
            .manifest
            .parameters
            .retain(|parameter| parameter.name != "strategy.signal_id");
        write_replay_bundle(
            &alpha_stop_bundle,
            &tampered.manifest,
            &tampered.summary,
            &tampered.ledger,
        )
        .unwrap();
        let report = load_research_report_bundle(&report_dir).unwrap_err();
        assert!(report.to_string().contains("linked replay bundle"));
        write_research_report_bundle(&report_dir, &stored_report).unwrap();

        let explain_response = dispatch(["research", "explain", report_dir.to_str().unwrap()]);

        assert_eq!(explain_response.exit_code, 1);
        assert!(
            explain_response
                .stderr
                .contains("requires manifest parameter `strategy.signal_id`")
        );
        assert!(
            explain_response
                .stderr
                .contains(&alpha_stop_bundle.display().to_string())
        );

        remove_dir_all_if_exists(&alpha_close_bundle);
        remove_dir_all_if_exists(&beta_close_bundle);
        remove_dir_all_if_exists(&alpha_stop_bundle);
        remove_dir_all_if_exists(&beta_stop_bundle);
        remove_dir_all_if_exists(&report_dir);
    }

    #[test]
    fn research_leaderboard_execution_view_ranks_execution_models_with_fixed_context() {
        let snapshot_id = "fixture:m6_leaderboard_execution";
        let engine_version = "m6-leaderboard-execution-reference-flow";
        let next_open = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let stop_entry = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "stop_entry_long",
        };
        let alpha_next_open = write_labeled_bundle(
            "cli-leaderboard-execution-alpha-next-open",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&next_open),
        );
        let beta_next_open = write_labeled_bundle(
            "cli-leaderboard-execution-beta-next-open",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            Some(&next_open),
        );
        let alpha_stop_entry = write_labeled_bundle(
            "cli-leaderboard-execution-alpha-stop-entry",
            &sample_request_for_symbol_with_terminal_close("ALPHA", 104.0),
            snapshot_id,
            engine_version,
            Some(&stop_entry),
        );
        let beta_stop_entry = write_labeled_bundle(
            "cli-leaderboard-execution-beta-stop-entry",
            &sample_request_for_symbol_with_terminal_close("BETA", 103.0),
            snapshot_id,
            engine_version,
            Some(&stop_entry),
        );

        let response = dispatch([
            "research",
            "leaderboard",
            "execution-model",
            alpha_next_open.to_str().unwrap(),
            beta_next_open.to_str().unwrap(),
            alpha_stop_entry.to_str().unwrap(),
            beta_stop_entry.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(
            response
                .stdout
                .contains("research leaderboard execution-model")
        );
        assert!(
            response
                .stdout
                .contains("fixed_signal_id: close_confirmed_breakout")
        );
        assert!(response.stdout.contains("fixed_filter_id: pass_filter"));
        assert!(
            response
                .stdout
                .contains("fixed_position_manager_id: keep_position_manager")
        );
        assert!(
            response
                .stdout
                .contains("row: rank=1 label=stop_entry_long")
        );
        assert!(response.stdout.contains("row: rank=2 label=next_open_long"));

        remove_dir_all_if_exists(&alpha_next_open);
        remove_dir_all_if_exists(&beta_next_open);
        remove_dir_all_if_exists(&alpha_stop_entry);
        remove_dir_all_if_exists(&beta_stop_entry);
    }

    #[test]
    fn research_leaderboard_system_view_ranks_full_systems_and_keeps_drilldown() {
        let snapshot_id = "fixture:m6_leaderboard_system";
        let engine_version = "m6-leaderboard-system-reference-flow";
        let close_next_open = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let stop_stop_entry = TestStrategyLabels {
            signal_id: "stop_entry_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "stop_entry_long",
        };
        let alpha_close_next_open = write_labeled_bundle(
            "cli-leaderboard-system-alpha-close-next-open",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&close_next_open),
        );
        let beta_close_next_open = write_labeled_bundle(
            "cli-leaderboard-system-beta-close-next-open",
            &sample_request_for_symbol_with_terminal_close("BETA", 99.0),
            snapshot_id,
            engine_version,
            Some(&close_next_open),
        );
        let alpha_stop_stop_entry = write_labeled_bundle(
            "cli-leaderboard-system-alpha-stop-stop-entry",
            &sample_request_for_symbol_with_terminal_close("ALPHA", 105.0),
            snapshot_id,
            engine_version,
            Some(&stop_stop_entry),
        );
        let beta_stop_stop_entry = write_labeled_bundle(
            "cli-leaderboard-system-beta-stop-stop-entry",
            &sample_request_for_symbol_with_terminal_close("BETA", 104.0),
            snapshot_id,
            engine_version,
            Some(&stop_stop_entry),
        );

        let response = dispatch([
            "research",
            "leaderboard",
            "system",
            alpha_close_next_open.to_str().unwrap(),
            beta_close_next_open.to_str().unwrap(),
            alpha_stop_stop_entry.to_str().unwrap(),
            beta_stop_stop_entry.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        assert!(response.stdout.contains("research leaderboard system"));
        assert!(
            response
                .stdout
                .contains("row: rank=1 label=signal=stop_entry_breakout filter=pass_filter position=keep_position_manager execution=stop_entry_long")
        );
        assert!(
            response
                .stdout
                .contains("row: rank=2 label=signal=close_confirmed_breakout filter=pass_filter position=keep_position_manager execution=next_open_long")
        );
        assert!(
            response
                .stdout
                .contains(&format!("bundle={}", alpha_stop_stop_entry.display()))
        );

        remove_dir_all_if_exists(&alpha_close_next_open);
        remove_dir_all_if_exists(&beta_close_next_open);
        remove_dir_all_if_exists(&alpha_stop_stop_entry);
        remove_dir_all_if_exists(&beta_stop_stop_entry);
    }

    #[test]
    fn research_leaderboard_rejects_missing_component_attribution() {
        let snapshot_id = "fixture:m6_leaderboard_missing_labels";
        let engine_version = "m6-leaderboard-missing-labels-reference-flow";
        let alpha_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-alpha",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            None,
        );
        let beta_bundle = write_labeled_bundle(
            "cli-leaderboard-missing-beta",
            &sample_request_for_symbol("BETA"),
            snapshot_id,
            engine_version,
            None,
        );

        let response = dispatch([
            "research",
            "leaderboard",
            "signal",
            alpha_bundle.to_str().unwrap(),
            beta_bundle.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 1);
        assert!(
            response
                .stderr
                .contains("research leaderboard requires manifest parameter `strategy.signal_id`")
        );

        remove_dir_all_if_exists(&alpha_bundle);
        remove_dir_all_if_exists(&beta_bundle);
    }

    #[test]
    fn research_leaderboard_rejects_signal_view_when_execution_context_varies() {
        let snapshot_id = "fixture:m6_leaderboard_context_mismatch";
        let engine_version = "m6-leaderboard-context-mismatch-reference-flow";
        let next_open = TestStrategyLabels {
            signal_id: "close_confirmed_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "next_open_long",
        };
        let stop_entry = TestStrategyLabels {
            signal_id: "stop_entry_breakout",
            filter_id: "pass_filter",
            position_manager_id: "keep_position_manager",
            execution_model_id: "stop_entry_long",
        };
        let alpha_bundle = write_labeled_bundle(
            "cli-leaderboard-context-alpha",
            &sample_request_for_symbol("ALPHA"),
            snapshot_id,
            engine_version,
            Some(&next_open),
        );
        let beta_bundle = write_labeled_bundle(
            "cli-leaderboard-context-beta",
            &sample_request_for_symbol("BETA"),
            snapshot_id,
            engine_version,
            Some(&stop_entry),
        );

        let response = dispatch([
            "research",
            "leaderboard",
            "signal",
            alpha_bundle.to_str().unwrap(),
            beta_bundle.to_str().unwrap(),
        ]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stderr.contains(
            "research leaderboard signal view requires fixed execution_model_id across all bundles"
        ));

        remove_dir_all_if_exists(&alpha_bundle);
        remove_dir_all_if_exists(&beta_bundle);
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

    fn sample_request_with_modified_terminal_bar() -> RunRequest {
        let mut request = sample_request();
        request.bars[1].raw_close = 104.0;
        request.bars[1].analysis_close = 104.0;
        request
    }

    fn sample_request_with_analysis_gap() -> RunRequest {
        let mut request = sample_request();
        request.bars[0].analysis_close = 50.25;
        request
    }

    fn sample_request_for_symbol(symbol: &str) -> RunRequest {
        let mut request = sample_request();
        request.symbol = symbol.to_string();
        request
    }

    fn sample_request_for_symbol_with_terminal_close(symbol: &str, close: f64) -> RunRequest {
        let mut request = sample_request_for_symbol(symbol);
        request.bars[1].raw_close = close;
        request.bars[1].analysis_close = close;
        request.bars[1].raw_low = request.bars[1].raw_low.min(close);
        request.bars[1].raw_high = request.bars[1].raw_high.max(close);
        request
    }

    fn sample_walk_forward_request_for_symbol(symbol: &str) -> RunRequest {
        RunRequest {
            symbol: symbol.to_string(),
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
                    raw_open: 101.0,
                    raw_high: 102.0,
                    raw_low: 100.0,
                    raw_close: 101.5,
                    analysis_close: 101.5,
                },
                DailyBar {
                    date: "2025-01-06".to_string(),
                    raw_open: 102.0,
                    raw_high: 103.0,
                    raw_low: 101.0,
                    raw_close: 102.5,
                    analysis_close: 102.5,
                },
                DailyBar {
                    date: "2025-01-07".to_string(),
                    raw_open: 103.0,
                    raw_high: 104.0,
                    raw_low: 102.0,
                    raw_close: 103.5,
                    analysis_close: 103.5,
                },
                DailyBar {
                    date: "2025-01-08".to_string(),
                    raw_open: 104.0,
                    raw_high: 105.0,
                    raw_low: 103.0,
                    raw_close: 104.5,
                    analysis_close: 104.5,
                },
                DailyBar {
                    date: "2025-01-09".to_string(),
                    raw_open: 105.0,
                    raw_high: 106.0,
                    raw_low: 104.0,
                    raw_close: 105.5,
                    analysis_close: 105.5,
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

    fn sample_walk_forward_request_with_shifted_date(symbol: &str) -> RunRequest {
        let mut request = sample_walk_forward_request_for_symbol(symbol);
        request.bars[1].date = "2025-01-04".to_string();
        request
    }

    fn sample_walk_forward_request_for_symbol_with_closes(
        symbol: &str,
        closes: [f64; 6],
    ) -> RunRequest {
        let mut request = sample_walk_forward_request_for_symbol(symbol);

        for (bar, close) in request.bars.iter_mut().zip(closes) {
            bar.raw_close = close;
            bar.analysis_close = close;
            bar.raw_high = bar.raw_high.max(close);
            bar.raw_low = bar.raw_low.min(close);
        }

        request
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

    fn manifest_parameter_value<'a>(manifest: &'a RunManifest, name: &str) -> &'a str {
        manifest
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .map(|parameter| parameter.value.as_str())
            .unwrap_or_else(|| panic!("missing manifest parameter `{name}`"))
    }

    #[derive(Clone, Copy)]
    struct TestStrategyLabels<'a> {
        signal_id: &'a str,
        filter_id: &'a str,
        position_manager_id: &'a str,
        execution_model_id: &'a str,
    }

    fn write_labeled_bundle(
        label: &str,
        request: &RunRequest,
        snapshot_id: &str,
        engine_version: &str,
        strategy_labels: Option<&TestStrategyLabels<'_>>,
    ) -> PathBuf {
        let request_path = test_output_dir(&format!("{label}-request")).join("request.json");
        let bundle_dir = test_output_dir(&format!("{label}-bundle"));
        write_request(&request_path, request);

        let mut args = vec![
            "run".to_string(),
            "--request".to_string(),
            request_path.display().to_string(),
            "--output".to_string(),
            bundle_dir.display().to_string(),
            "--provider".to_string(),
            "fixture".to_string(),
            "--snapshot-id".to_string(),
            snapshot_id.to_string(),
            "--engine-version".to_string(),
            engine_version.to_string(),
        ];

        if let Some(strategy_labels) = strategy_labels {
            args.extend([
                "--signal-id".to_string(),
                strategy_labels.signal_id.to_string(),
                "--filter-id".to_string(),
                strategy_labels.filter_id.to_string(),
                "--position-manager-id".to_string(),
                strategy_labels.position_manager_id.to_string(),
                "--execution-model-id".to_string(),
                strategy_labels.execution_model_id.to_string(),
            ]);
        }

        let response = dispatch(args);
        assert_eq!(response.exit_code, 0, "{}", response.stderr);
        bundle_dir
    }

    fn assert_research_explain_matches_saved_output(response: &CliResponse, report_dir: &Path) {
        let explain_response = dispatch(["research", "explain", report_dir.to_str().unwrap()]);

        assert_eq!(explain_response.exit_code, 0, "{}", explain_response.stderr);

        let expected = response
            .stdout
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(explain_response.stdout, expected);
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
            .expect("trendlab-cli lives under crates/");
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
