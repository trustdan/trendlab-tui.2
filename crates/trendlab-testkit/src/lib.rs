#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use trendlab_artifact::{
    DateRange, ManifestParameter, PersistedLedgerRow, ReferenceFlowDefinition, RunManifest,
    RunSummary, SCHEMA_VERSION, write_replay_bundle,
};
use trendlab_core::engine::{ReferenceFlowSpec, RunRequest, RunResult, run_reference_flow};
use trendlab_core::market::DailyBar;
use trendlab_core::orders::{EntryIntent, GapPolicy, OrderIntent};

pub const M1_ENTRY_HOLD_OPEN_POSITION: &str = "m1_entry_hold_open_position";
pub const M1_INTRABAR_STOP_EXIT: &str = "m1_intrabar_stop_exit";
pub const M1_GAP_THROUGH_STOP_EXIT: &str = "m1_gap_through_stop_exit";

pub const FROZEN_M1_SCENARIOS: [&str; 3] = [
    M1_ENTRY_HOLD_OPEN_POSITION,
    M1_INTRABAR_STOP_EXIT,
    M1_GAP_THROUGH_STOP_EXIT,
];

#[derive(Clone, Debug, PartialEq)]
pub struct ScenarioManifest {
    pub name: String,
    pub symbol: String,
    pub initial_cash: f64,
    pub entry_shares: u32,
    pub protective_stop_fraction: f64,
    pub oracle: bool,
    pub gap_policy: GapPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioPaths {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub bars: PathBuf,
    pub entry_intents: PathBuf,
    pub expected_ledger: PathBuf,
}

pub mod fixtures {
    use super::*;

    pub fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("trendlab-testkit lives under crates/")
            .to_path_buf()
    }

    pub fn fixtures_dir() -> PathBuf {
        workspace_root().join("fixtures")
    }

    pub fn scenario_paths(name: &str) -> ScenarioPaths {
        let root = fixtures_dir().join(name);
        ScenarioPaths {
            manifest: root.join("scenario.txt"),
            bars: root.join("bars.csv"),
            entry_intents: root.join("entry-intents.csv"),
            expected_ledger: root.join("expected-ledger.csv"),
            root,
        }
    }

    pub fn load_manifest(name: &str) -> Result<ScenarioManifest, String> {
        let paths = scenario_paths(name);
        let raw = fs::read_to_string(&paths.manifest)
            .map_err(|err| format!("failed to read {}: {err}", paths.manifest.display()))?;
        let map = parse_key_value_file(&raw)?;

        Ok(ScenarioManifest {
            name: required(&map, "name")?.to_string(),
            symbol: required(&map, "symbol")?.to_string(),
            initial_cash: parse_f64(required(&map, "initial_cash")?, "initial_cash")?,
            entry_shares: parse_u32(required(&map, "entry_shares")?, "entry_shares")?,
            protective_stop_fraction: parse_f64(
                required(&map, "protective_stop_fraction")?,
                "protective_stop_fraction",
            )?,
            oracle: parse_bool(required(&map, "oracle")?, "oracle")?,
            gap_policy: GapPolicy::parse(required(&map, "gap_policy")?)
                .ok_or_else(|| "unknown gap_policy".to_string())?,
        })
    }

    pub fn load_reference_flow_spec(name: &str) -> Result<ReferenceFlowSpec, String> {
        let manifest = load_manifest(name)?;

        Ok(ReferenceFlowSpec {
            initial_cash: manifest.initial_cash,
            entry_shares: manifest.entry_shares,
            protective_stop_fraction: manifest.protective_stop_fraction,
            cost_model: trendlab_core::accounting::CostModel::default(),
        })
    }

    pub fn load_run_request(name: &str) -> Result<RunRequest, String> {
        let manifest = load_manifest(name)?;

        Ok(RunRequest {
            symbol: manifest.symbol,
            bars: load_bars(name)?,
            entry_intents: load_entry_intents(name)?,
            reference_flow: load_reference_flow_spec(name)?,
            gap_policy: manifest.gap_policy,
        })
    }

    pub fn load_bars(name: &str) -> Result<Vec<DailyBar>, String> {
        let raw = read_required_file(&scenario_paths(name).bars)?;
        let rows = parse_csv_rows(&raw)?;
        let mut bars = Vec::new();

        for row in rows {
            bars.push(DailyBar {
                date: required_column(&row, "date")?.to_string(),
                raw_open: parse_f64(required_column(&row, "raw_open")?, "raw_open")?,
                raw_high: parse_f64(required_column(&row, "raw_high")?, "raw_high")?,
                raw_low: parse_f64(required_column(&row, "raw_low")?, "raw_low")?,
                raw_close: parse_f64(required_column(&row, "raw_close")?, "raw_close")?,
                analysis_close: parse_f64(
                    required_column(&row, "analysis_close")?,
                    "analysis_close",
                )?,
            });
        }

        Ok(bars)
    }

    pub fn load_entry_intents(name: &str) -> Result<Vec<EntryIntent>, String> {
        let raw = read_required_file(&scenario_paths(name).entry_intents)?;
        let rows = parse_csv_rows(&raw)?;
        let mut intents = Vec::new();

        for row in rows {
            let intent = OrderIntent::parse(required_column(&row, "intent")?)
                .ok_or_else(|| "unknown order intent".to_string())?;

            intents.push(EntryIntent {
                signal_date: required_column(&row, "signal_date")?.to_string(),
                intent,
                shares: parse_u32(required_column(&row, "shares")?, "shares")?,
            });
        }

        Ok(intents)
    }

    pub fn load_expected_ledger(name: &str) -> Result<Vec<PersistedLedgerRow>, String> {
        let raw = read_required_file(&scenario_paths(name).expected_ledger)?;
        let rows = parse_csv_rows(&raw)?;
        let mut ledger = Vec::new();

        for row in rows {
            ledger.push(PersistedLedgerRow {
                date: required_column(&row, "date")?.to_string(),
                raw_open: parse_f64(required_column(&row, "raw_open")?, "raw_open")?,
                raw_high: parse_f64(required_column(&row, "raw_high")?, "raw_high")?,
                raw_low: parse_f64(required_column(&row, "raw_low")?, "raw_low")?,
                raw_close: parse_f64(required_column(&row, "raw_close")?, "raw_close")?,
                analysis_close: parse_f64(
                    required_column(&row, "analysis_close")?,
                    "analysis_close",
                )?,
                position_shares: parse_u32(
                    required_column(&row, "position_shares")?,
                    "position_shares",
                )?,
                signal_output: required_column(&row, "signal_output")?.to_string(),
                filter_outcome: required_column(&row, "filter_outcome")?.to_string(),
                pending_order_state: required_column(&row, "pending_order_state")?.to_string(),
                fill_price: parse_optional_f64(required_column(&row, "fill_price")?, "fill_price")?,
                prior_stop: parse_optional_f64(required_column(&row, "prior_stop")?, "prior_stop")?,
                next_stop: parse_optional_f64(required_column(&row, "next_stop")?, "next_stop")?,
                cash: parse_f64(required_column(&row, "cash")?, "cash")?,
                equity: parse_f64(required_column(&row, "equity")?, "equity")?,
                reason_codes: split_reason_codes(required_column(&row, "reason_codes")?),
            });
        }

        Ok(ledger)
    }

    fn read_required_file(path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
    }

    fn parse_key_value_file(input: &str) -> Result<BTreeMap<String, String>, String> {
        let mut values = BTreeMap::new();

        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid key=value line `{line}`"))?;

            values.insert(key.trim().to_string(), value.trim().to_string());
        }

        Ok(values)
    }

    fn parse_csv_rows(input: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
        let mut lines = input.lines().filter(|line| !line.trim().is_empty());
        let header_line = lines
            .next()
            .ok_or_else(|| "missing CSV header".to_string())?;
        let headers: Vec<String> = header_line
            .split(',')
            .map(|part| part.trim().to_string())
            .collect();

        let mut rows = Vec::new();

        for line in lines {
            let values: Vec<String> = line
                .split(',')
                .map(|part| part.trim().to_string())
                .collect();
            if values.len() != headers.len() {
                return Err(format!(
                    "column mismatch for line `{line}`: expected {}, got {}",
                    headers.len(),
                    values.len()
                ));
            }

            let row = headers
                .iter()
                .cloned()
                .zip(values.into_iter())
                .collect::<BTreeMap<_, _>>();
            rows.push(row);
        }

        Ok(rows)
    }

    fn required<'a>(map: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
        map.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing required key `{key}`"))
    }

    fn required_column<'a>(
        row: &'a BTreeMap<String, String>,
        key: &str,
    ) -> Result<&'a str, String> {
        row.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing required column `{key}`"))
    }

    fn parse_bool(value: &str, field: &str) -> Result<bool, String> {
        match value.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(format!("invalid boolean for `{field}`")),
        }
    }

    fn parse_f64(value: &str, field: &str) -> Result<f64, String> {
        value
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("invalid decimal for `{field}`"))
    }

    fn parse_optional_f64(value: &str, field: &str) -> Result<Option<f64>, String> {
        if value.trim().is_empty() {
            Ok(None)
        } else {
            parse_f64(value, field).map(Some)
        }
    }

    fn parse_u32(value: &str, field: &str) -> Result<u32, String> {
        value
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("invalid integer for `{field}`"))
    }

    fn split_reason_codes(value: &str) -> Vec<String> {
        value
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(ToString::to_string)
            .collect()
    }
}

pub mod golden {
    use super::*;

    pub fn assert_non_empty_case_name(name: &str) -> Result<(), &'static str> {
        if name.trim().is_empty() {
            Err("fixture case names must not be empty")
        } else {
            Ok(())
        }
    }

    pub fn persisted_row_count(rows: &[PersistedLedgerRow]) -> usize {
        rows.len()
    }

    pub fn assert_m1_reconciles(rows: &[PersistedLedgerRow]) -> Result<(), String> {
        for row in rows {
            let expected_equity = round4(row.cash + row.raw_close * f64::from(row.position_shares));
            if round4(row.equity) != expected_equity {
                return Err(format!(
                    "equity reconciliation failed on {}: expected {}, got {}",
                    row.date, expected_equity, row.equity
                ));
            }

            if row.position_shares == 0 && row.next_stop.is_some() {
                return Err(format!(
                    "flat rows must not carry a next_stop on {}",
                    row.date
                ));
            }

            if row.position_shares > 0 && row.next_stop.is_none() {
                return Err(format!(
                    "open-position rows must carry an active next_stop on {}",
                    row.date
                ));
            }
        }

        Ok(())
    }

    fn round4(value: f64) -> f64 {
        (value * 10_000.0).round() / 10_000.0
    }
}

pub mod oracle {
    use super::*;

    pub fn first_reason_code(rows: &[PersistedLedgerRow]) -> Option<&str> {
        rows.first()
            .and_then(|row| row.reason_codes.first().map(String::as_str))
    }
}

pub mod bundle {
    use super::*;

    pub fn build_manifest(name: &str) -> Result<RunManifest, String> {
        let scenario = fixtures::load_manifest(name)?;
        let bars = fixtures::load_bars(name)?;
        let first_bar = bars
            .first()
            .ok_or_else(|| "fixture bundle requires at least one bar".to_string())?;
        let last_bar = bars
            .last()
            .ok_or_else(|| "fixture bundle requires at least one bar".to_string())?;

        Ok(RunManifest {
            schema_version: SCHEMA_VERSION,
            engine_version: "m1-reference-flow".to_string(),
            data_snapshot_id: format!("fixture:{name}"),
            provider_identity: "fixture".to_string(),
            symbol_or_universe: scenario.symbol,
            universe_mode: "single_symbol".to_string(),
            historical_limitations: Vec::new(),
            date_range: DateRange {
                start_date: first_bar.date.clone(),
                end_date: last_bar.date.clone(),
            },
            reference_flow: ReferenceFlowDefinition {
                kind: "m1_reference_flow".to_string(),
                entry_shares: scenario.entry_shares,
                protective_stop_fraction: scenario.protective_stop_fraction,
            },
            parameters: vec![ManifestParameter {
                name: "fixture_scenario".to_string(),
                value: name.to_string(),
            }],
            cost_model: trendlab_core::accounting::CostModel::default(),
            gap_policy: scenario.gap_policy,
            seed: None,
            warnings: Vec::new(),
        })
    }

    pub fn build_summary(result: &RunResult, warning_count: usize) -> RunSummary {
        RunSummary {
            row_count: result.ledger.len(),
            warning_count,
            ending_cash: result.cash.cash,
            ending_equity: result.cash.equity,
        }
    }

    pub fn persisted_ledger(result: &RunResult) -> Vec<PersistedLedgerRow> {
        result.ledger.iter().map(PersistedLedgerRow::from).collect()
    }

    pub fn write_fixture_bundle(name: &str, bundle_dir: &Path) -> Result<(), String> {
        let request = fixtures::load_run_request(name)?;
        let result = run_reference_flow(&request).map_err(|err| err.to_string())?;
        let manifest = build_manifest(name)?;
        let summary = build_summary(&result, manifest.warnings.len());
        let ledger = persisted_ledger(&result);

        write_replay_bundle(bundle_dir, &manifest, &summary, &ledger)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use trendlab_artifact::PersistedLedgerRow;
    use trendlab_artifact::SCHEMA_VERSION;
    use trendlab_artifact::load_replay_bundle;
    use trendlab_core::engine::run_reference_flow;

    use super::bundle;
    use super::fixtures;
    use super::golden;
    use super::{
        FROZEN_M1_SCENARIOS, M1_ENTRY_HOLD_OPEN_POSITION, M1_GAP_THROUGH_STOP_EXIT,
        M1_INTRABAR_STOP_EXIT,
    };

    #[test]
    fn frozen_m1_scenarios_have_expected_files() {
        for name in FROZEN_M1_SCENARIOS {
            golden::assert_non_empty_case_name(name).unwrap();

            let paths = fixtures::scenario_paths(name);
            assert!(paths.root.is_dir(), "missing scenario dir for {name}");
            assert!(
                paths.manifest.is_file(),
                "missing scenario manifest for {name}"
            );
            assert!(paths.bars.is_file(), "missing bars.csv for {name}");
            assert!(
                paths.entry_intents.is_file(),
                "missing entry-intents.csv for {name}"
            );
            assert!(
                paths.expected_ledger.is_file(),
                "missing expected-ledger.csv for {name}"
            );
        }
    }

    #[test]
    fn fixture_inputs_load_deterministically() {
        let manifest = fixtures::load_manifest(M1_ENTRY_HOLD_OPEN_POSITION).unwrap();
        let request = fixtures::load_run_request(M1_ENTRY_HOLD_OPEN_POSITION).unwrap();

        assert_eq!(manifest.name, M1_ENTRY_HOLD_OPEN_POSITION);
        assert_eq!(manifest.entry_shares, 1);
        assert_eq!(request.reference_flow.protective_stop_fraction, 0.10);
        assert_eq!(request.bars.len(), 4);
        assert_eq!(request.entry_intents.len(), 1);
    }

    #[test]
    fn oracle_ledger_loads_for_intrabar_stop_case() {
        let manifest = fixtures::load_manifest(M1_INTRABAR_STOP_EXIT).unwrap();
        let ledger = fixtures::load_expected_ledger(M1_INTRABAR_STOP_EXIT).unwrap();

        assert!(manifest.oracle);
        assert_eq!(golden::persisted_row_count(&ledger), 3);
        assert_eq!(
            super::oracle::first_reason_code(&ledger),
            Some("entry_intent_queued")
        );
    }

    #[test]
    fn gap_through_case_marks_non_oracle_but_has_expected_ledger_shell() {
        let manifest = fixtures::load_manifest(M1_GAP_THROUGH_STOP_EXIT).unwrap();
        let ledger = fixtures::load_expected_ledger(M1_GAP_THROUGH_STOP_EXIT).unwrap();

        assert!(!manifest.oracle);
        assert_eq!(golden::persisted_row_count(&ledger), 3);
    }

    #[test]
    fn week_three_entry_hold_run_matches_expected_ledger_fixture() {
        let request = fixtures::load_run_request(M1_ENTRY_HOLD_OPEN_POSITION).unwrap();
        let actual = run_reference_flow(&request).unwrap();
        let expected = fixtures::load_expected_ledger(M1_ENTRY_HOLD_OPEN_POSITION).unwrap();
        let persisted: Vec<_> = actual.ledger.iter().map(PersistedLedgerRow::from).collect();

        golden::assert_m1_reconciles(&persisted).unwrap();
        assert_eq!(persisted, expected);
    }

    #[test]
    fn week_four_stop_runs_match_expected_ledger_fixtures() {
        for name in [M1_INTRABAR_STOP_EXIT, M1_GAP_THROUGH_STOP_EXIT] {
            let request = fixtures::load_run_request(name).unwrap();
            let actual = run_reference_flow(&request).unwrap();
            let expected = fixtures::load_expected_ledger(name).unwrap();
            let persisted: Vec<_> = actual.ledger.iter().map(PersistedLedgerRow::from).collect();

            golden::assert_m1_reconciles(&persisted).unwrap();
            assert_eq!(persisted, expected, "ledger mismatch for {name}");
        }
    }

    #[test]
    fn week_five_replay_bundles_round_trip_for_frozen_fixtures() {
        for name in FROZEN_M1_SCENARIOS {
            let bundle_dir = test_output_dir(name);

            bundle::write_fixture_bundle(name, &bundle_dir).unwrap();

            let loaded = load_replay_bundle(&bundle_dir).unwrap();
            let expected = fixtures::load_expected_ledger(name).unwrap();

            golden::assert_m1_reconciles(&loaded.ledger).unwrap();
            assert_eq!(loaded.ledger, expected, "loaded ledger mismatch for {name}");
            assert_eq!(loaded.summary.row_count, expected.len());
            assert_eq!(loaded.manifest.schema_version, SCHEMA_VERSION);

            fs::remove_dir_all(bundle_dir).unwrap();
        }
    }

    fn test_output_dir(label: &str) -> std::path::PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("trendlab-testkit lives under crates/");
        workspace_root
            .join("target")
            .join("test-output")
            .join(format!(
                "{label}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ))
    }

    #[test]
    fn frozen_m1_expected_ledgers_reconcile() {
        for name in FROZEN_M1_SCENARIOS {
            let ledger = fixtures::load_expected_ledger(name).unwrap();
            golden::assert_m1_reconciles(&ledger).unwrap();
        }
    }
}
