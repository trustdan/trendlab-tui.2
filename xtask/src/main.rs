#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use trendlab_artifact::load_replay_bundle;
use trendlab_data::live::{
    LiveSymbolHistoryRequest, ProviderAdapter, TIINGO_API_TOKEN_ENV, TiingoAdapter,
};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("validate") => validate(),
        Some("validate-live") => validate_live(args.collect()),
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

    println!(
        "configuration check passed: {TIINGO_API_TOKEN_ENV} is set; live HTTP execution remains outside default validation and is not required for cargo xtask validate"
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

fn usage() -> ExitCode {
    eprintln!("usage:");
    eprintln!("  cargo xtask validate");
    eprintln!(
        "  cargo xtask validate-live --provider tiingo [--symbol <symbol>] [--start <YYYY-MM-DD>] [--end <YYYY-MM-DD>]"
    );
    eprintln!("  cargo xtask write-fixture-bundle --scenario <name> --output <dir>");
    eprintln!("  cargo xtask inspect-ledger <bundle-dir>");
    ExitCode::FAILURE
}
