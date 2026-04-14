#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use trendlab_artifact::load_replay_bundle;
use trendlab_data::live::{
    LiveSymbolHistoryRequest, ProviderAdapter, TIINGO_API_TOKEN_ENV, TiingoAdapter,
};
use trendlab_data::normalize::normalize_symbol_history;
use trendlab_data::resample::{ResampleFrequency, resample_symbol_history};
use trendlab_data::snapshot::{
    PersistedSnapshotBundle, SnapshotBundleDescriptor, SnapshotCaptureMetadata,
    SnapshotRequestedWindow,
};
use trendlab_data::snapshot_store::{load_snapshot_bundle, write_snapshot_bundle};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("validate") => validate(),
        Some("validate-live") => validate_live(args.collect()),
        Some("capture-live-snapshot") => capture_live_snapshot(args.collect()),
        Some("write-fixture-bundle") => write_fixture_bundle_command(args.collect()),
        Some("inspect-ledger") => inspect_ledger(args.collect()),
        _ => usage(),
    }
}

fn validate() -> ExitCode {
    let workspace_root = workspace_root();

    let commands: [(&str, &[&str]); 3] = [
        ("fmt", &["fmt", "--all", "--check"]),
        (
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("test", &["test", "--workspace"]),
    ];

    for (label, args) in commands {
        println!("running cargo {label}...");

        match run_cargo(&workspace_root, args) {
            Ok(0) => {}
            Ok(code) => return ExitCode::from(code),
            Err(err) => {
                eprintln!("failed to run cargo {label}: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

fn validate_live(args: Vec<String>) -> ExitCode {
    let mut provider = None;
    let mut symbol = None;
    let mut start_date = None;
    let mut end_date = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = iter.next(),
            "--symbol" => symbol = iter.next(),
            "--start" => start_date = iter.next(),
            "--end" => end_date = iter.next(),
            other => {
                eprintln!("unexpected argument for validate-live: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    match provider.as_deref() {
        Some("tiingo") => validate_live_tiingo(symbol, start_date, end_date),
        Some(other) => {
            eprintln!("unknown live provider `{other}`");
            ExitCode::FAILURE
        }
        None => {
            eprintln!(
                "usage: cargo xtask validate-live --provider tiingo [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
            );
            ExitCode::FAILURE
        }
    }
}

fn capture_live_snapshot(args: Vec<String>) -> ExitCode {
    let mut provider = None;
    let mut symbol = None;
    let mut start_date = None;
    let mut end_date = None;
    let mut output = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = iter.next(),
            "--symbol" => symbol = iter.next(),
            "--start" => start_date = iter.next(),
            "--end" => end_date = iter.next(),
            "--output" => output = iter.next(),
            other => {
                eprintln!("unexpected argument for capture-live-snapshot: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let Some(output) = output else {
        eprintln!(
            "usage: cargo xtask capture-live-snapshot --provider tiingo --output <dir> [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
        );
        return ExitCode::FAILURE;
    };

    match provider.as_deref() {
        Some("tiingo") => capture_live_snapshot_tiingo(
            symbol,
            start_date,
            end_date,
            resolve_workspace_path(&output),
        ),
        Some(other) => {
            eprintln!("unknown live provider `{other}`");
            ExitCode::FAILURE
        }
        None => {
            eprintln!(
                "usage: cargo xtask capture-live-snapshot --provider tiingo --output <dir> [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
            );
            ExitCode::FAILURE
        }
    }
}

fn validate_live_tiingo(
    symbol: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> ExitCode {
    let adapter = TiingoAdapter;
    let request = LiveSymbolHistoryRequest {
        symbol: symbol.unwrap_or_else(|| "SPY".to_string()),
        start_date: start_date.unwrap_or_else(|| "2025-01-02".to_string()),
        end_date: end_date.unwrap_or_else(|| "2025-01-10".to_string()),
    };
    let plan = match adapter.smoke_plan(&request) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("invalid tiingo live-smoke request: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("validate-live is intentionally excluded from cargo xtask validate");
    println!("provider: {}", plan.provider_identity.as_str());
    println!("symbol: {}", plan.symbol);
    println!("start_date: {}", plan.start_date);
    println!("end_date: {}", plan.end_date);
    println!("required_env_var: {}", plan.required_env_var);
    println!(
        "expected_resamples: {}",
        plan.expected_resamples
            .iter()
            .map(|frequency| frequency.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("smoke_invariants:");
    for invariant in &plan.invariants {
        println!("  - {invariant}");
    }

    if env::var_os(TIINGO_API_TOKEN_ENV).is_none() {
        eprintln!(
            "set {TIINGO_API_TOKEN_ENV} before running the Tiingo live-smoke lane; normal validation does not require it"
        );
        return ExitCode::FAILURE;
    }

    let api_token = match env::var(TIINGO_API_TOKEN_ENV) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to read {TIINGO_API_TOKEN_ENV} even though it appears set: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "configuration check passed: {TIINGO_API_TOKEN_ENV} is set; executing the optional live Tiingo fetch outside cargo xtask validate"
    );

    let fetched = match adapter.fetch_symbol_history(&request, &api_token) {
        Ok(fetched) => fetched,
        Err(err) => {
            eprintln!("live Tiingo fetch failed: {err}");
            return ExitCode::FAILURE;
        }
    };
    let normalized = match normalize_symbol_history(&fetched.stored) {
        Ok(normalized) => normalized,
        Err(err) => {
            eprintln!("failed to normalize fetched Tiingo history: {err}");
            return ExitCode::FAILURE;
        }
    };
    let weekly = match resample_symbol_history(&normalized, ResampleFrequency::Weekly) {
        Ok(weekly) => weekly,
        Err(err) => {
            eprintln!("failed to resample fetched Tiingo history to weekly bars: {err}");
            return ExitCode::FAILURE;
        }
    };
    let monthly = match resample_symbol_history(&normalized, ResampleFrequency::Monthly) {
        Ok(monthly) => monthly,
        Err(err) => {
            eprintln!("failed to resample fetched Tiingo history to monthly bars: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("live_fetch:");
    println!("  snapshot_id: {}", fetched.snapshot_id);
    println!(
        "  provider_daily_bars: {}",
        fetched.provider_daily_bar_count
    );
    println!(
        "  provider_corporate_actions: {}",
        fetched.provider_corporate_action_count
    );
    println!("  returned_first_date: {}", fetched.first_date);
    println!("  returned_last_date: {}", fetched.last_date);
    println!("pipeline_counts:");
    println!("  stored_raw_bars: {}", fetched.stored.raw_bars.len());
    println!(
        "  stored_corporate_actions: {}",
        fetched.stored.corporate_actions.len()
    );
    println!("  normalized_daily_bars: {}", normalized.bars.len());
    println!("  weekly_bars: {}", weekly.bars.len());
    println!("  monthly_bars: {}", monthly.bars.len());
    println!(
        "live Tiingo smoke passed: fetched, ingested, normalized, and resampled real provider data outside cargo xtask validate"
    );
    ExitCode::SUCCESS
}

fn capture_live_snapshot_tiingo(
    symbol: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    output_dir: PathBuf,
) -> ExitCode {
    let adapter = TiingoAdapter;
    let request = LiveSymbolHistoryRequest {
        symbol: symbol.unwrap_or_else(|| "SPY".to_string()),
        start_date: start_date.unwrap_or_else(|| "2025-01-02".to_string()),
        end_date: end_date.unwrap_or_else(|| "2025-01-10".to_string()),
    };

    if env::var_os(TIINGO_API_TOKEN_ENV).is_none() {
        eprintln!(
            "set {TIINGO_API_TOKEN_ENV} before running live snapshot capture; normal validation does not require it"
        );
        return ExitCode::FAILURE;
    }

    let api_token = match env::var(TIINGO_API_TOKEN_ENV) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to read {TIINGO_API_TOKEN_ENV} even though it appears set: {err}");
            return ExitCode::FAILURE;
        }
    };

    let fetched = match adapter.fetch_symbol_history(&request, &api_token) {
        Ok(fetched) => fetched,
        Err(err) => {
            eprintln!("live Tiingo fetch failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    let descriptor = match SnapshotBundleDescriptor::from_stored_symbols(
        fetched.snapshot_id.clone(),
        fetched.provider_identity,
        SnapshotRequestedWindow {
            start_date: request.start_date.clone(),
            end_date: request.end_date.clone(),
        },
        SnapshotCaptureMetadata {
            capture_mode: "live_provider_fetch".to_string(),
            entrypoint: "cargo xtask capture-live-snapshot".to_string(),
            captured_at_unix_epoch_seconds: captured_at_unix_epoch_seconds(),
        },
        std::slice::from_ref(&fetched.stored),
    ) {
        Ok(descriptor) => descriptor,
        Err(err) => {
            eprintln!("failed to build snapshot descriptor: {err}");
            return ExitCode::FAILURE;
        }
    };
    let bundle = PersistedSnapshotBundle {
        descriptor,
        symbols: vec![fetched.stored.clone()],
    };

    if let Err(err) = write_snapshot_bundle(&output_dir, &bundle) {
        eprintln!("failed to write live snapshot bundle: {err}");
        return ExitCode::FAILURE;
    }

    let reopened = match load_snapshot_bundle(&output_dir) {
        Ok(bundle) => bundle,
        Err(err) => {
            eprintln!("failed to reopen written snapshot bundle: {err}");
            return ExitCode::FAILURE;
        }
    };

    if reopened != bundle {
        eprintln!("reopened snapshot bundle does not match the just-written snapshot bundle");
        return ExitCode::FAILURE;
    }

    let normalized = match normalize_symbol_history(&reopened.symbols[0]) {
        Ok(normalized) => normalized,
        Err(err) => {
            eprintln!("failed to normalize reopened snapshot bundle: {err}");
            return ExitCode::FAILURE;
        }
    };
    let weekly = match resample_symbol_history(&normalized, ResampleFrequency::Weekly) {
        Ok(weekly) => weekly,
        Err(err) => {
            eprintln!("failed to resample reopened snapshot bundle to weekly bars: {err}");
            return ExitCode::FAILURE;
        }
    };
    let monthly = match resample_symbol_history(&normalized, ResampleFrequency::Monthly) {
        Ok(monthly) => monthly,
        Err(err) => {
            eprintln!("failed to resample reopened snapshot bundle to monthly bars: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("snapshot_capture:");
    println!("  output_dir: {}", output_dir.display());
    println!("  snapshot_id: {}", reopened.descriptor.snapshot_id);
    println!(
        "  provider_identity: {}",
        reopened.descriptor.provider_identity.as_str()
    );
    println!(
        "  requested_window: {}..{}",
        reopened.descriptor.requested_window.start_date,
        reopened.descriptor.requested_window.end_date
    );
    println!("  symbol_count: {}", reopened.descriptor.symbols.len());
    println!("  first_symbol: {}", reopened.symbols[0].symbol);
    println!("  stored_raw_bars: {}", reopened.symbols[0].raw_bars.len());
    println!(
        "  stored_corporate_actions: {}",
        reopened.symbols[0].corporate_actions.len()
    );
    println!("  normalized_daily_bars: {}", normalized.bars.len());
    println!("  weekly_bars: {}", weekly.bars.len());
    println!("  monthly_bars: {}", monthly.bars.len());
    println!(
        "live snapshot capture passed: fetched, persisted, reopened, normalized, and resampled without a second provider call"
    );
    ExitCode::SUCCESS
}

fn write_fixture_bundle_command(args: Vec<String>) -> ExitCode {
    let mut scenario = None;
    let mut output = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scenario" => scenario = iter.next(),
            "--output" => output = iter.next(),
            other => {
                eprintln!("unexpected argument for write-fixture-bundle: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let Some(scenario) = scenario else {
        eprintln!("usage: cargo xtask write-fixture-bundle --scenario <name> --output <dir>");
        return ExitCode::FAILURE;
    };
    let Some(output) = output else {
        eprintln!("usage: cargo xtask write-fixture-bundle --scenario <name> --output <dir>");
        return ExitCode::FAILURE;
    };

    let bundle_dir = workspace_root().join(output);

    match trendlab_testkit::bundle::write_fixture_bundle(&scenario, &bundle_dir) {
        Ok(()) => {
            println!(
                "wrote fixture replay bundle for `{scenario}` to {}",
                bundle_dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("failed to write fixture replay bundle: {err}");
            ExitCode::FAILURE
        }
    }
}

fn inspect_ledger(args: Vec<String>) -> ExitCode {
    let [bundle_dir]: [String; 1] = match args.try_into() {
        Ok(values) => values,
        Err(_) => {
            eprintln!("usage: cargo xtask inspect-ledger <bundle-dir>");
            return ExitCode::FAILURE;
        }
    };

    let bundle_dir = workspace_root().join(bundle_dir);
    let bundle = match load_replay_bundle(&bundle_dir) {
        Ok(bundle) => bundle,
        Err(err) => {
            eprintln!("failed to load replay bundle: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("bundle: {}", bundle_dir.display());
    println!("schema_version: {}", bundle.descriptor.schema_version);
    println!("symbol: {}", bundle.manifest.symbol_or_universe);
    println!("gap_policy: {}", bundle.manifest.gap_policy.as_str());
    println!("rows: {}", bundle.summary.row_count);
    println!("ending_cash: {:.4}", bundle.summary.ending_cash);
    println!("ending_equity: {:.4}", bundle.summary.ending_equity);

    for row in bundle.ledger {
        println!(
            "{} shares={} fill={} prior_stop={} next_stop={} cash={:.4} equity={:.4} reasons={}",
            row.date,
            row.position_shares,
            format_optional(row.fill_price),
            format_optional(row.prior_stop),
            format_optional(row.next_stop),
            row.cash,
            row.equity,
            format_reason_codes(&row.reason_codes),
        );
    }

    ExitCode::SUCCESS
}

fn format_optional(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{value:.4}"),
        None => "none".to_string(),
    }
}

fn format_reason_codes(reason_codes: &[String]) -> String {
    if reason_codes.is_empty() {
        "none".to_string()
    } else {
        reason_codes.join("|")
    }
}

fn run_cargo(workspace_root: &Path, args: &[&str]) -> std::io::Result<u8> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(workspace_root)
        .status()?;

    Ok(status.code().unwrap_or(1) as u8)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives directly under the workspace root")
        .to_path_buf()
}

fn resolve_workspace_path(value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace_root().join(path)
    }
}

fn captured_at_unix_epoch_seconds() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn usage() -> ExitCode {
    eprintln!("usage:");
    eprintln!("  cargo xtask validate");
    eprintln!(
        "  cargo xtask validate-live --provider tiingo [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
    );
    eprintln!(
        "  cargo xtask capture-live-snapshot --provider tiingo --output <dir> [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
    );
    eprintln!("  cargo xtask write-fixture-bundle --scenario <name> --output <dir>");
    eprintln!("  cargo xtask inspect-ledger <bundle-dir>");
    ExitCode::FAILURE
}
