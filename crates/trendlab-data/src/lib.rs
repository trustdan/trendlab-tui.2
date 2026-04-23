#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataError {
    message: String,
}

impl DataError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for DataError {}

pub mod provider {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum ProviderIdentity {
        #[serde(rename = "fixture")]
        Fixture,
        #[serde(rename = "tiingo")]
        Tiingo,
    }

    impl ProviderIdentity {
        pub const FIXTURE: &'static str = "fixture";
        pub const TIINGO: &'static str = "tiingo";

        pub fn as_str(self) -> &'static str {
            match self {
                Self::Fixture => Self::FIXTURE,
                Self::Tiingo => Self::TIINGO,
            }
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value.trim() {
                Self::FIXTURE => Some(Self::Fixture),
                Self::TIINGO => Some(Self::Tiingo),
                _ => None,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum TiingoCorporateActionKind {
        #[serde(rename = "split")]
        Split,
        #[serde(rename = "cash_dividend")]
        CashDividend,
    }

    impl TiingoCorporateActionKind {
        pub const SPLIT: &'static str = "split";
        pub const CASH_DIVIDEND: &'static str = "cash_dividend";

        pub fn as_str(self) -> &'static str {
            match self {
                Self::Split => Self::SPLIT,
                Self::CashDividend => Self::CASH_DIVIDEND,
            }
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value.trim() {
                Self::SPLIT => Some(Self::Split),
                Self::CASH_DIVIDEND => Some(Self::CashDividend),
                _ => None,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct TiingoDailyBar {
        pub symbol: String,
        pub date: String,
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
        pub volume: u64,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct TiingoCorporateAction {
        pub symbol: String,
        pub ex_date: String,
        pub kind: TiingoCorporateActionKind,
        pub split_ratio: Option<f64>,
        pub cash_amount: Option<f64>,
    }
}

pub mod snapshot {
    use std::collections::BTreeSet;

    use crate::provider::ProviderIdentity;
    use crate::{DataError, SNAPSHOT_SCHEMA_VERSION};

    pub const SNAPSHOT_DESCRIPTOR_FILE_NAME: &str = "snapshot.json";
    pub const SNAPSHOT_DAILY_DIRECTORY_NAME: &str = "daily";
    pub const SNAPSHOT_ACTIONS_DIRECTORY_NAME: &str = "actions";

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotMetadata {
        pub schema_version: u32,
        pub snapshot_id: String,
        pub provider_identity: ProviderIdentity,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct RawDailyBar {
        pub symbol: String,
        pub date: String,
        pub raw_open: f64,
        pub raw_high: f64,
        pub raw_low: f64,
        pub raw_close: f64,
        pub volume: u64,
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum CorporateAction {
        Split {
            symbol: String,
            ex_date: String,
            ratio: f64,
        },
        CashDividend {
            symbol: String,
            ex_date: String,
            cash_amount: f64,
        },
    }

    impl CorporateAction {
        pub fn symbol(&self) -> &str {
            match self {
                Self::Split { symbol, .. } | Self::CashDividend { symbol, .. } => symbol,
            }
        }

        pub fn ex_date(&self) -> &str {
            match self {
                Self::Split { ex_date, .. } | Self::CashDividend { ex_date, .. } => ex_date,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct StoredSymbolData {
        pub metadata: SnapshotMetadata,
        pub symbol: String,
        pub raw_bars: Vec<RawDailyBar>,
        pub corporate_actions: Vec<CorporateAction>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotRequestedWindow {
        pub start_date: String,
        pub end_date: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotCaptureMetadata {
        pub capture_mode: String,
        pub entrypoint: String,
        pub captured_at_unix_epoch_seconds: Option<u64>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotCompatibility {
        pub daily_directory: String,
        pub actions_directory: String,
    }

    impl SnapshotCompatibility {
        pub fn canonical() -> Self {
            Self {
                daily_directory: SNAPSHOT_DAILY_DIRECTORY_NAME.to_string(),
                actions_directory: SNAPSHOT_ACTIONS_DIRECTORY_NAME.to_string(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotSymbolSummary {
        pub symbol: String,
        pub raw_bar_count: usize,
        pub corporate_action_count: usize,
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SnapshotBundleDescriptor {
        pub schema_version: u32,
        pub snapshot_id: String,
        pub provider_identity: ProviderIdentity,
        pub requested_window: SnapshotRequestedWindow,
        pub capture: SnapshotCaptureMetadata,
        pub compatibility: SnapshotCompatibility,
        pub symbols: Vec<SnapshotSymbolSummary>,
    }

    impl SnapshotBundleDescriptor {
        pub fn from_stored_symbols(
            snapshot_id: impl Into<String>,
            provider_identity: ProviderIdentity,
            requested_window: SnapshotRequestedWindow,
            capture: SnapshotCaptureMetadata,
            symbols: &[StoredSymbolData],
        ) -> Result<Self, DataError> {
            let snapshot_id = snapshot_id.into();
            if snapshot_id.trim().is_empty() {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires a non-empty snapshot_id",
                ));
            }

            if requested_window.start_date.trim().is_empty()
                || requested_window.end_date.trim().is_empty()
            {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires non-empty requested start_date and end_date",
                ));
            }

            if requested_window.end_date < requested_window.start_date {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires end_date on or after start_date",
                ));
            }

            if capture.capture_mode.trim().is_empty() {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires a non-empty capture_mode",
                ));
            }

            if capture.entrypoint.trim().is_empty() {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires a non-empty entrypoint",
                ));
            }

            if symbols.is_empty() {
                return Err(DataError::invalid(
                    "snapshot bundle descriptor requires at least one stored symbol",
                ));
            }

            let mut seen_symbols = BTreeSet::new();
            let mut symbol_summaries = Vec::with_capacity(symbols.len());

            for stored in symbols {
                if stored.metadata.schema_version != SNAPSHOT_SCHEMA_VERSION {
                    return Err(DataError::invalid(format!(
                        "stored symbol `{}` schema version {} does not match supported version {}",
                        stored.symbol, stored.metadata.schema_version, SNAPSHOT_SCHEMA_VERSION
                    )));
                }
                if stored.metadata.snapshot_id != snapshot_id {
                    return Err(DataError::invalid(format!(
                        "stored symbol `{}` snapshot_id `{}` does not match descriptor snapshot_id `{snapshot_id}`",
                        stored.symbol, stored.metadata.snapshot_id
                    )));
                }
                if stored.metadata.provider_identity != provider_identity {
                    return Err(DataError::invalid(format!(
                        "stored symbol `{}` provider_identity `{}` does not match descriptor provider_identity `{}`",
                        stored.symbol,
                        stored.metadata.provider_identity.as_str(),
                        provider_identity.as_str()
                    )));
                }
                if !seen_symbols.insert(stored.symbol.clone()) {
                    return Err(DataError::invalid(format!(
                        "snapshot bundle descriptor contains duplicate symbol `{}`",
                        stored.symbol
                    )));
                }

                symbol_summaries.push(SnapshotSymbolSummary {
                    symbol: stored.symbol.clone(),
                    raw_bar_count: stored.raw_bars.len(),
                    corporate_action_count: stored.corporate_actions.len(),
                });
            }

            Ok(Self {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                snapshot_id,
                provider_identity,
                requested_window,
                capture,
                compatibility: SnapshotCompatibility::canonical(),
                symbols: symbol_summaries,
            })
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct PersistedSnapshotBundle {
        pub descriptor: SnapshotBundleDescriptor,
        pub symbols: Vec<StoredSymbolData>,
    }
}

fn validate_stored_symbol_data(stored: &snapshot::StoredSymbolData) -> Result<(), DataError> {
    if stored.metadata.schema_version != SNAPSHOT_SCHEMA_VERSION {
        return Err(DataError::invalid(format!(
            "stored symbol data schema version {} does not match supported version {}",
            stored.metadata.schema_version, SNAPSHOT_SCHEMA_VERSION
        )));
    }

    if stored.metadata.snapshot_id.trim().is_empty() {
        return Err(DataError::invalid(
            "stored symbol data must include a non-empty snapshot_id",
        ));
    }

    if stored.symbol.trim().is_empty() {
        return Err(DataError::invalid(
            "stored symbol data must include a non-empty symbol",
        ));
    }

    if stored.raw_bars.is_empty() {
        return Err(DataError::invalid(
            "stored symbol data must include at least one raw daily bar",
        ));
    }

    let mut previous_date: Option<&str> = None;
    for raw_bar in &stored.raw_bars {
        if raw_bar.symbol != stored.symbol {
            return Err(DataError::invalid(format!(
                "raw daily bar symbol `{}` does not match stored symbol `{}`",
                raw_bar.symbol, stored.symbol
            )));
        }

        if let Some(previous_date) = previous_date
            && raw_bar.date.as_str() <= previous_date
        {
            return Err(DataError::invalid(
                "stored raw daily bars must be in strictly increasing date order with unique dates",
            ));
        }
        previous_date = Some(raw_bar.date.as_str());
    }

    for action in &stored.corporate_actions {
        if action.symbol() != stored.symbol {
            return Err(DataError::invalid(format!(
                "corporate action symbol `{}` does not match stored symbol `{}`",
                action.symbol(),
                stored.symbol
            )));
        }
    }

    Ok(())
}

pub mod ingest {
    use crate::provider::{
        ProviderIdentity, TiingoCorporateAction, TiingoCorporateActionKind, TiingoDailyBar,
    };
    use crate::snapshot::{CorporateAction, RawDailyBar, SnapshotMetadata, StoredSymbolData};
    use crate::{DataError, SNAPSHOT_SCHEMA_VERSION};

    pub fn ingest_tiingo_symbol_history(
        metadata: SnapshotMetadata,
        symbol: &str,
        raw_bars: &[TiingoDailyBar],
        corporate_actions: &[TiingoCorporateAction],
    ) -> Result<StoredSymbolData, DataError> {
        validate_metadata(&metadata)?;

        if metadata.provider_identity != ProviderIdentity::Tiingo {
            return Err(DataError::invalid(
                "tiingo ingestion requires snapshot metadata provider_identity `tiingo`",
            ));
        }

        if symbol.trim().is_empty() {
            return Err(DataError::invalid(
                "symbol history ingestion requires a non-empty symbol",
            ));
        }

        let mut stored_bars = raw_bars
            .iter()
            .map(|bar| ingest_tiingo_daily_bar(symbol, bar))
            .collect::<Result<Vec<_>, _>>()?;
        stored_bars.sort_by(|left, right| left.date.cmp(&right.date));
        validate_raw_bars(symbol, &stored_bars)?;

        let mut stored_actions = corporate_actions
            .iter()
            .map(|action| ingest_tiingo_corporate_action(symbol, action))
            .collect::<Result<Vec<_>, _>>()?;
        stored_actions.sort_by(|left, right| left.ex_date().cmp(right.ex_date()));
        validate_corporate_actions(symbol, &stored_actions)?;

        Ok(StoredSymbolData {
            metadata,
            symbol: symbol.to_string(),
            raw_bars: stored_bars,
            corporate_actions: stored_actions,
        })
    }

    fn validate_metadata(metadata: &SnapshotMetadata) -> Result<(), DataError> {
        if metadata.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(DataError::invalid(format!(
                "snapshot schema version {} does not match supported version {}",
                metadata.schema_version, SNAPSHOT_SCHEMA_VERSION
            )));
        }

        if metadata.snapshot_id.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot metadata must include a non-empty snapshot_id",
            ));
        }

        Ok(())
    }

    fn ingest_tiingo_daily_bar(
        symbol: &str,
        bar: &TiingoDailyBar,
    ) -> Result<RawDailyBar, DataError> {
        if bar.symbol != symbol {
            return Err(DataError::invalid(format!(
                "tiingo daily bar symbol `{}` does not match requested symbol `{symbol}`",
                bar.symbol
            )));
        }

        if bar.date.trim().is_empty() {
            return Err(DataError::invalid(
                "tiingo daily bars must include a non-empty date",
            ));
        }

        if [bar.open, bar.high, bar.low, bar.close]
            .into_iter()
            .any(|value| value <= 0.0)
        {
            return Err(DataError::invalid(
                "tiingo daily bars must use positive OHLC prices",
            ));
        }

        if bar.high < bar.open.max(bar.close).max(bar.low) {
            return Err(DataError::invalid(
                "tiingo daily bars must satisfy high >= max(open, close, low)",
            ));
        }

        if bar.low > bar.open.min(bar.close).min(bar.high) {
            return Err(DataError::invalid(
                "tiingo daily bars must satisfy low <= min(open, close, high)",
            ));
        }

        Ok(RawDailyBar {
            symbol: bar.symbol.clone(),
            date: bar.date.clone(),
            raw_open: bar.open,
            raw_high: bar.high,
            raw_low: bar.low,
            raw_close: bar.close,
            volume: bar.volume,
        })
    }

    fn ingest_tiingo_corporate_action(
        symbol: &str,
        action: &TiingoCorporateAction,
    ) -> Result<CorporateAction, DataError> {
        if action.symbol != symbol {
            return Err(DataError::invalid(format!(
                "tiingo corporate action symbol `{}` does not match requested symbol `{symbol}`",
                action.symbol
            )));
        }

        if action.ex_date.trim().is_empty() {
            return Err(DataError::invalid(
                "tiingo corporate actions must include a non-empty ex_date",
            ));
        }

        match action.kind {
            TiingoCorporateActionKind::Split => {
                let Some(split_ratio) = action.split_ratio else {
                    return Err(DataError::invalid(
                        "split corporate actions require a split_ratio",
                    ));
                };
                if split_ratio <= 0.0 {
                    return Err(DataError::invalid(
                        "split corporate actions require a positive split_ratio",
                    ));
                }

                Ok(CorporateAction::Split {
                    symbol: action.symbol.clone(),
                    ex_date: action.ex_date.clone(),
                    ratio: split_ratio,
                })
            }
            TiingoCorporateActionKind::CashDividend => {
                let Some(cash_amount) = action.cash_amount else {
                    return Err(DataError::invalid(
                        "cash_dividend corporate actions require a cash_amount",
                    ));
                };
                if cash_amount < 0.0 {
                    return Err(DataError::invalid(
                        "cash_dividend corporate actions must not use a negative cash_amount",
                    ));
                }

                Ok(CorporateAction::CashDividend {
                    symbol: action.symbol.clone(),
                    ex_date: action.ex_date.clone(),
                    cash_amount,
                })
            }
        }
    }

    fn validate_raw_bars(symbol: &str, raw_bars: &[RawDailyBar]) -> Result<(), DataError> {
        if raw_bars.is_empty() {
            return Err(DataError::invalid(
                "symbol history ingestion requires at least one raw daily bar",
            ));
        }

        let mut previous_date: Option<&str> = None;
        for bar in raw_bars {
            if bar.symbol != symbol {
                return Err(DataError::invalid(format!(
                    "raw daily bar symbol `{}` does not match stored symbol `{symbol}`",
                    bar.symbol
                )));
            }

            if let Some(previous_date) = previous_date
                && bar.date.as_str() <= previous_date
            {
                return Err(DataError::invalid(
                    "raw daily bars must be in strictly increasing date order with unique dates",
                ));
            }
            previous_date = Some(bar.date.as_str());
        }

        Ok(())
    }

    fn validate_corporate_actions(
        symbol: &str,
        corporate_actions: &[CorporateAction],
    ) -> Result<(), DataError> {
        for action in corporate_actions {
            if action.symbol() != symbol {
                return Err(DataError::invalid(format!(
                    "corporate action symbol `{}` does not match stored symbol `{symbol}`",
                    action.symbol()
                )));
            }
        }

        Ok(())
    }
}

pub mod actions {
    use std::collections::BTreeMap;

    use crate::DataError;
    use crate::snapshot::CorporateAction;

    #[derive(Clone, Debug, PartialEq)]
    pub struct CorporateActionEffect {
        pub ex_date: String,
        pub split_ratio: f64,
        pub cash_dividend_per_share: f64,
    }

    impl CorporateActionEffect {
        pub fn has_split(&self) -> bool {
            round4(self.split_ratio) != 1.0
        }

        pub fn has_cash_dividend(&self) -> bool {
            round4(self.cash_dividend_per_share) != 0.0
        }
    }

    pub fn build_corporate_action_effects(
        corporate_actions: &[CorporateAction],
    ) -> Result<Vec<CorporateActionEffect>, DataError> {
        let mut grouped_effects = BTreeMap::<&str, CorporateActionAccumulator>::new();

        for action in corporate_actions {
            match action {
                CorporateAction::Split { ex_date, ratio, .. } => {
                    if *ratio <= 0.0 {
                        return Err(DataError::invalid(
                            "split corporate actions require a positive ratio",
                        ));
                    }

                    grouped_effects
                        .entry(ex_date.as_str())
                        .or_default()
                        .split_ratio *= *ratio;
                }
                CorporateAction::CashDividend {
                    ex_date,
                    cash_amount,
                    ..
                } => {
                    if *cash_amount < 0.0 {
                        return Err(DataError::invalid(
                            "cash-dividend corporate actions must not use a negative cash_amount",
                        ));
                    }

                    grouped_effects
                        .entry(ex_date.as_str())
                        .or_default()
                        .cash_dividend_per_share += *cash_amount;
                }
            }
        }

        Ok(grouped_effects
            .into_iter()
            .map(|(ex_date, accumulator)| CorporateActionEffect {
                ex_date: ex_date.to_string(),
                split_ratio: round4(accumulator.split_ratio),
                cash_dividend_per_share: round4(accumulator.cash_dividend_per_share),
            })
            .collect())
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CorporateActionAccumulator {
        split_ratio: f64,
        cash_dividend_per_share: f64,
    }

    impl Default for CorporateActionAccumulator {
        fn default() -> Self {
            Self {
                split_ratio: 1.0,
                cash_dividend_per_share: 0.0,
            }
        }
    }

    fn round4(value: f64) -> f64 {
        (value * 10_000.0).round() / 10_000.0
    }
}

pub mod snapshot_store {
    use std::collections::BTreeSet;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::path::Path;

    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use crate::DataError;
    use crate::snapshot::{
        CorporateAction, PersistedSnapshotBundle, RawDailyBar, SNAPSHOT_ACTIONS_DIRECTORY_NAME,
        SNAPSHOT_DAILY_DIRECTORY_NAME, SNAPSHOT_DESCRIPTOR_FILE_NAME, SnapshotBundleDescriptor,
        SnapshotCompatibility, SnapshotMetadata,
    };
    use crate::validate_stored_symbol_data;

    pub fn write_snapshot_bundle(
        snapshot_dir: &Path,
        bundle: &PersistedSnapshotBundle,
    ) -> Result<(), DataError> {
        validate_snapshot_bundle(bundle)?;

        fs::create_dir_all(snapshot_dir).map_err(|err| {
            DataError::invalid(format!(
                "failed to create snapshot directory {}: {err}",
                snapshot_dir.display()
            ))
        })?;

        let daily_dir = snapshot_dir.join(SNAPSHOT_DAILY_DIRECTORY_NAME);
        let actions_dir = snapshot_dir.join(SNAPSHOT_ACTIONS_DIRECTORY_NAME);
        fs::create_dir_all(&daily_dir).map_err(|err| {
            DataError::invalid(format!(
                "failed to create snapshot daily directory {}: {err}",
                daily_dir.display()
            ))
        })?;
        fs::create_dir_all(&actions_dir).map_err(|err| {
            DataError::invalid(format!(
                "failed to create snapshot actions directory {}: {err}",
                actions_dir.display()
            ))
        })?;

        write_json(
            &snapshot_dir.join(SNAPSHOT_DESCRIPTOR_FILE_NAME),
            &bundle.descriptor,
        )?;

        for stored in &bundle.symbols {
            write_json_lines(
                &daily_dir.join(symbol_file_name(&stored.symbol)?),
                &stored.raw_bars,
            )?;
            write_json_lines(
                &actions_dir.join(symbol_file_name(&stored.symbol)?),
                &stored.corporate_actions,
            )?;
        }

        Ok(())
    }

    pub fn load_snapshot_bundle(snapshot_dir: &Path) -> Result<PersistedSnapshotBundle, DataError> {
        let descriptor: SnapshotBundleDescriptor =
            read_json(&snapshot_dir.join(SNAPSHOT_DESCRIPTOR_FILE_NAME))?;
        validate_snapshot_descriptor(&descriptor)?;

        let mut symbols = Vec::with_capacity(descriptor.symbols.len());
        for symbol_summary in &descriptor.symbols {
            let file_name = symbol_file_name(&symbol_summary.symbol)?;
            let raw_bars: Vec<RawDailyBar> = read_json_lines(
                &snapshot_dir
                    .join(SNAPSHOT_DAILY_DIRECTORY_NAME)
                    .join(&file_name),
            )?;
            let corporate_actions: Vec<CorporateAction> = read_json_lines(
                &snapshot_dir
                    .join(SNAPSHOT_ACTIONS_DIRECTORY_NAME)
                    .join(file_name),
            )?;

            if raw_bars.len() != symbol_summary.raw_bar_count {
                return Err(DataError::invalid(format!(
                    "snapshot symbol `{}` expected {} raw daily bars but loaded {}",
                    symbol_summary.symbol,
                    symbol_summary.raw_bar_count,
                    raw_bars.len()
                )));
            }
            if corporate_actions.len() != symbol_summary.corporate_action_count {
                return Err(DataError::invalid(format!(
                    "snapshot symbol `{}` expected {} corporate actions but loaded {}",
                    symbol_summary.symbol,
                    symbol_summary.corporate_action_count,
                    corporate_actions.len()
                )));
            }

            let stored = crate::snapshot::StoredSymbolData {
                metadata: SnapshotMetadata {
                    schema_version: descriptor.schema_version,
                    snapshot_id: descriptor.snapshot_id.clone(),
                    provider_identity: descriptor.provider_identity,
                },
                symbol: symbol_summary.symbol.clone(),
                raw_bars,
                corporate_actions,
            };
            validate_stored_symbol_data(&stored)?;
            symbols.push(stored);
        }

        Ok(PersistedSnapshotBundle {
            descriptor,
            symbols,
        })
    }

    fn validate_snapshot_bundle(bundle: &PersistedSnapshotBundle) -> Result<(), DataError> {
        validate_snapshot_descriptor(&bundle.descriptor)?;

        if bundle.symbols.len() != bundle.descriptor.symbols.len() {
            return Err(DataError::invalid(format!(
                "snapshot bundle descriptor lists {} symbols but bundle stores {} symbols",
                bundle.descriptor.symbols.len(),
                bundle.symbols.len()
            )));
        }

        let mut seen_symbols = BTreeSet::new();
        for stored in &bundle.symbols {
            validate_stored_symbol_data(stored)?;

            if stored.metadata.snapshot_id != bundle.descriptor.snapshot_id {
                return Err(DataError::invalid(format!(
                    "stored symbol `{}` snapshot_id `{}` does not match descriptor snapshot_id `{}`",
                    stored.symbol, stored.metadata.snapshot_id, bundle.descriptor.snapshot_id
                )));
            }
            if stored.metadata.provider_identity != bundle.descriptor.provider_identity {
                return Err(DataError::invalid(format!(
                    "stored symbol `{}` provider_identity `{}` does not match descriptor provider_identity `{}`",
                    stored.symbol,
                    stored.metadata.provider_identity.as_str(),
                    bundle.descriptor.provider_identity.as_str()
                )));
            }
            if !seen_symbols.insert(stored.symbol.clone()) {
                return Err(DataError::invalid(format!(
                    "snapshot bundle stores duplicate symbol `{}`",
                    stored.symbol
                )));
            }

            let symbol_summary = bundle
                .descriptor
                .symbols
                .iter()
                .find(|summary| summary.symbol == stored.symbol)
                .ok_or_else(|| {
                    DataError::invalid(format!(
                        "snapshot bundle descriptor is missing symbol `{}`",
                        stored.symbol
                    ))
                })?;

            if symbol_summary.raw_bar_count != stored.raw_bars.len() {
                return Err(DataError::invalid(format!(
                    "snapshot descriptor raw_bar_count for `{}` is {} but stored data has {} bars",
                    stored.symbol,
                    symbol_summary.raw_bar_count,
                    stored.raw_bars.len()
                )));
            }
            if symbol_summary.corporate_action_count != stored.corporate_actions.len() {
                return Err(DataError::invalid(format!(
                    "snapshot descriptor corporate_action_count for `{}` is {} but stored data has {} actions",
                    stored.symbol,
                    symbol_summary.corporate_action_count,
                    stored.corporate_actions.len()
                )));
            }
        }

        Ok(())
    }

    fn validate_snapshot_descriptor(
        descriptor: &SnapshotBundleDescriptor,
    ) -> Result<(), DataError> {
        if descriptor.schema_version != crate::SNAPSHOT_SCHEMA_VERSION {
            return Err(DataError::invalid(format!(
                "snapshot descriptor schema version {} does not match supported version {}",
                descriptor.schema_version,
                crate::SNAPSHOT_SCHEMA_VERSION
            )));
        }

        if descriptor.snapshot_id.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot descriptor must include a non-empty snapshot_id",
            ));
        }

        if descriptor.requested_window.start_date.trim().is_empty()
            || descriptor.requested_window.end_date.trim().is_empty()
        {
            return Err(DataError::invalid(
                "snapshot descriptor must include non-empty requested start_date and end_date",
            ));
        }

        if descriptor.requested_window.end_date < descriptor.requested_window.start_date {
            return Err(DataError::invalid(
                "snapshot descriptor requires end_date on or after start_date",
            ));
        }

        if descriptor.capture.capture_mode.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot descriptor must include a non-empty capture_mode",
            ));
        }

        if descriptor.capture.entrypoint.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot descriptor must include a non-empty entrypoint",
            ));
        }

        if descriptor.compatibility != SnapshotCompatibility::canonical() {
            return Err(DataError::invalid(format!(
                "snapshot descriptor compatibility must use canonical directories `{}` and `{}`",
                SNAPSHOT_DAILY_DIRECTORY_NAME, SNAPSHOT_ACTIONS_DIRECTORY_NAME
            )));
        }

        if descriptor.symbols.is_empty() {
            return Err(DataError::invalid(
                "snapshot descriptor must include at least one symbol",
            ));
        }

        let mut seen_symbols = BTreeSet::new();
        for symbol_summary in &descriptor.symbols {
            if symbol_summary.symbol.trim().is_empty() {
                return Err(DataError::invalid(
                    "snapshot descriptor symbols must include non-empty symbol names",
                ));
            }
            if !seen_symbols.insert(symbol_summary.symbol.clone()) {
                return Err(DataError::invalid(format!(
                    "snapshot descriptor contains duplicate symbol `{}`",
                    symbol_summary.symbol
                )));
            }
        }

        Ok(())
    }

    fn symbol_file_name(symbol: &str) -> Result<String, DataError> {
        if symbol.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot symbol file names require a non-empty symbol",
            ));
        }

        if symbol
            .chars()
            .any(|ch| matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        {
            return Err(DataError::invalid(format!(
                "snapshot symbol `{symbol}` cannot be represented as a canonical file name",
            )));
        }

        Ok(format!("{symbol}.jsonl"))
    }

    fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), DataError> {
        let file = File::create(path).map_err(|err| {
            DataError::invalid(format!("failed to create {}: {err}", path.display()))
        })?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, value).map_err(|err| {
            DataError::invalid(format!("failed to serialize {}: {err}", path.display()))
        })?;
        writer.write_all(b"\n").map_err(|err| {
            DataError::invalid(format!("failed to finalize {}: {err}", path.display()))
        })?;
        writer
            .flush()
            .map_err(|err| DataError::invalid(format!("failed to flush {}: {err}", path.display())))
    }

    fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, DataError> {
        let file = File::open(path).map_err(|err| {
            DataError::invalid(format!("failed to open {}: {err}", path.display()))
        })?;
        serde_json::from_reader(BufReader::new(file))
            .map_err(|err| DataError::invalid(format!("failed to parse {}: {err}", path.display())))
    }

    fn write_json_lines<T: Serialize>(path: &Path, rows: &[T]) -> Result<(), DataError> {
        let file = File::create(path).map_err(|err| {
            DataError::invalid(format!("failed to create {}: {err}", path.display()))
        })?;
        let mut writer = BufWriter::new(file);

        for row in rows {
            serde_json::to_writer(&mut writer, row).map_err(|err| {
                DataError::invalid(format!("failed to serialize {}: {err}", path.display()))
            })?;
            writer.write_all(b"\n").map_err(|err| {
                DataError::invalid(format!("failed to write {}: {err}", path.display()))
            })?;
        }

        writer
            .flush()
            .map_err(|err| DataError::invalid(format!("failed to flush {}: {err}", path.display())))
    }

    fn read_json_lines<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, DataError> {
        let file = File::open(path).map_err(|err| {
            DataError::invalid(format!("failed to open {}: {err}", path.display()))
        })?;
        let reader = BufReader::new(file);
        let mut rows = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|err| {
                DataError::invalid(format!("failed to read {}: {err}", path.display()))
            })?;
            if line.trim().is_empty() {
                continue;
            }
            rows.push(serde_json::from_str(&line).map_err(|err| {
                DataError::invalid(format!(
                    "failed to parse JSON line in {}: {err}",
                    path.display()
                ))
            })?);
        }

        Ok(rows)
    }
}

pub mod normalize {
    use trendlab_core::market::DailyBar;

    use crate::DataError;
    use crate::actions::{CorporateActionEffect, build_corporate_action_effects};
    use crate::snapshot::{CorporateAction, SnapshotMetadata, StoredSymbolData};

    #[derive(Clone, Debug, PartialEq)]
    pub struct NormalizedSymbolData {
        pub metadata: SnapshotMetadata,
        pub symbol: String,
        pub bars: Vec<DailyBar>,
        pub corporate_actions: Vec<CorporateAction>,
        pub corporate_action_effects: Vec<CorporateActionEffect>,
    }

    pub fn normalize_symbol_history(
        stored: &StoredSymbolData,
    ) -> Result<NormalizedSymbolData, DataError> {
        crate::validate_stored_symbol_data(stored)?;

        let corporate_action_effects = build_corporate_action_effects(&stored.corporate_actions)?;
        let mut cumulative_future_split_factor = 1.0;
        let mut normalized_bars = Vec::with_capacity(stored.raw_bars.len());

        for raw_bar in stored.raw_bars.iter().rev() {
            normalized_bars.push(DailyBar {
                date: raw_bar.date.clone(),
                raw_open: raw_bar.raw_open,
                raw_high: raw_bar.raw_high,
                raw_low: raw_bar.raw_low,
                raw_close: raw_bar.raw_close,
                analysis_close: round4(raw_bar.raw_close / cumulative_future_split_factor),
            });

            if let Some(action_effect) = corporate_action_effects
                .iter()
                .find(|effect| effect.ex_date == raw_bar.date)
                && action_effect.has_split()
            {
                cumulative_future_split_factor *= action_effect.split_ratio;
            }
        }

        normalized_bars.reverse();

        Ok(NormalizedSymbolData {
            metadata: stored.metadata.clone(),
            symbol: stored.symbol.clone(),
            bars: normalized_bars,
            corporate_actions: stored.corporate_actions.clone(),
            corporate_action_effects,
        })
    }
    fn round4(value: f64) -> f64 {
        (value * 10_000.0).round() / 10_000.0
    }
}

pub mod resample {
    use trendlab_core::market::DailyBar;

    use crate::DataError;
    use crate::actions::CorporateActionEffect;
    use crate::normalize::NormalizedSymbolData;
    use crate::snapshot::{CorporateAction, SnapshotMetadata};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ResampleFrequency {
        Weekly,
        Monthly,
    }

    impl ResampleFrequency {
        pub const WEEKLY: &'static str = "weekly";
        pub const MONTHLY: &'static str = "monthly";

        pub fn as_str(self) -> &'static str {
            match self {
                Self::Weekly => Self::WEEKLY,
                Self::Monthly => Self::MONTHLY,
            }
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value.trim() {
                Self::WEEKLY => Some(Self::Weekly),
                Self::MONTHLY => Some(Self::Monthly),
                _ => None,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct ResampledSymbolData {
        pub metadata: SnapshotMetadata,
        pub symbol: String,
        pub frequency: ResampleFrequency,
        pub bars: Vec<DailyBar>,
        pub corporate_actions: Vec<CorporateAction>,
        pub corporate_action_effects: Vec<CorporateActionEffect>,
    }

    pub fn resample_symbol_history(
        normalized: &NormalizedSymbolData,
        frequency: ResampleFrequency,
    ) -> Result<ResampledSymbolData, DataError> {
        if normalized.symbol.trim().is_empty() {
            return Err(DataError::invalid(
                "resampling requires a normalized symbol with a non-empty symbol",
            ));
        }

        Ok(ResampledSymbolData {
            metadata: normalized.metadata.clone(),
            symbol: normalized.symbol.clone(),
            frequency,
            bars: resample_bars(&normalized.bars, frequency)?,
            corporate_actions: normalized.corporate_actions.clone(),
            corporate_action_effects: normalized.corporate_action_effects.clone(),
        })
    }

    pub fn resample_bars(
        daily_bars: &[DailyBar],
        frequency: ResampleFrequency,
    ) -> Result<Vec<DailyBar>, DataError> {
        validate_daily_bars(daily_bars)?;

        let mut resampled_bars = Vec::new();
        let mut parsed_dates = daily_bars
            .iter()
            .map(|bar| Ok((bucket_key(parse_date(&bar.date)?, frequency), bar)))
            .collect::<Result<Vec<_>, DataError>>()?
            .into_iter();

        let Some((first_bucket_key, first_bar)) = parsed_dates.next() else {
            return Err(DataError::invalid(
                "resampling requires at least one normalized daily bar",
            ));
        };

        let mut current_bucket_key = first_bucket_key;
        let mut accumulator = ResampleAccumulator::new(first_bar);

        for (next_bucket_key, bar) in parsed_dates {
            if next_bucket_key == current_bucket_key {
                accumulator.push(bar);
            } else {
                resampled_bars.push(accumulator.finish());
                current_bucket_key = next_bucket_key;
                accumulator = ResampleAccumulator::new(bar);
            }
        }

        resampled_bars.push(accumulator.finish());
        Ok(resampled_bars)
    }

    fn validate_daily_bars(daily_bars: &[DailyBar]) -> Result<(), DataError> {
        if daily_bars.is_empty() {
            return Err(DataError::invalid(
                "resampling requires at least one normalized daily bar",
            ));
        }

        let mut previous_date: Option<&str> = None;
        for bar in daily_bars {
            let _ = parse_date(&bar.date)?;

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
                return Err(DataError::invalid(
                    "normalized daily bars must use positive raw and analysis prices",
                ));
            }

            if bar.raw_high < bar.raw_open.max(bar.raw_close).max(bar.raw_low) {
                return Err(DataError::invalid(
                    "normalized daily bars must satisfy raw_high >= max(raw_open, raw_close, raw_low)",
                ));
            }

            if bar.raw_low > bar.raw_open.min(bar.raw_close).min(bar.raw_high) {
                return Err(DataError::invalid(
                    "normalized daily bars must satisfy raw_low <= min(raw_open, raw_close, raw_high)",
                ));
            }

            if let Some(previous_date) = previous_date
                && bar.date.as_str() <= previous_date
            {
                return Err(DataError::invalid(
                    "normalized daily bars must be in strictly increasing date order with unique dates",
                ));
            }

            previous_date = Some(bar.date.as_str());
        }

        Ok(())
    }

    fn bucket_key(date: CivilDate, frequency: ResampleFrequency) -> BucketKey {
        match frequency {
            ResampleFrequency::Weekly => BucketKey::Weekly(date.week_start_days()),
            ResampleFrequency::Monthly => BucketKey::Monthly(date.year, date.month),
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum BucketKey {
        Weekly(i64),
        Monthly(i32, u8),
    }

    struct ResampleAccumulator {
        date: String,
        raw_open: f64,
        raw_high: f64,
        raw_low: f64,
        raw_close: f64,
        analysis_close: f64,
    }

    impl ResampleAccumulator {
        fn new(bar: &DailyBar) -> Self {
            Self {
                date: bar.date.clone(),
                raw_open: bar.raw_open,
                raw_high: bar.raw_high,
                raw_low: bar.raw_low,
                raw_close: bar.raw_close,
                analysis_close: bar.analysis_close,
            }
        }

        fn push(&mut self, bar: &DailyBar) {
            self.date = bar.date.clone();
            self.raw_high = self.raw_high.max(bar.raw_high);
            self.raw_low = self.raw_low.min(bar.raw_low);
            self.raw_close = bar.raw_close;
            self.analysis_close = bar.analysis_close;
        }

        fn finish(self) -> DailyBar {
            DailyBar {
                date: self.date,
                raw_open: self.raw_open,
                raw_high: self.raw_high,
                raw_low: self.raw_low,
                raw_close: self.raw_close,
                analysis_close: self.analysis_close,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CivilDate {
        year: i32,
        month: u8,
        day: u8,
    }

    impl CivilDate {
        fn week_start_days(self) -> i64 {
            let days = days_from_civil(self.year, self.month, self.day);
            days - weekday_monday0(days)
        }
    }

    fn parse_date(value: &str) -> Result<CivilDate, DataError> {
        let mut parts = value.trim().split('-');
        let year = parts
            .next()
            .ok_or_else(|| DataError::invalid("dates must use YYYY-MM-DD format"))?
            .parse::<i32>()
            .map_err(|_| DataError::invalid("dates must use YYYY-MM-DD format"))?;
        let month = parts
            .next()
            .ok_or_else(|| DataError::invalid("dates must use YYYY-MM-DD format"))?
            .parse::<u8>()
            .map_err(|_| DataError::invalid("dates must use YYYY-MM-DD format"))?;
        let day = parts
            .next()
            .ok_or_else(|| DataError::invalid("dates must use YYYY-MM-DD format"))?
            .parse::<u8>()
            .map_err(|_| DataError::invalid("dates must use YYYY-MM-DD format"))?;

        if parts.next().is_some() {
            return Err(DataError::invalid("dates must use YYYY-MM-DD format"));
        }

        if !(1..=12).contains(&month) {
            return Err(DataError::invalid("dates must use a valid calendar month"));
        }

        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return Err(DataError::invalid("dates must use a valid calendar day"));
        }

        Ok(CivilDate { year, month, day })
    }

    fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
        let month = i64::from(month);
        let day = i64::from(day);
        let year = i64::from(year) - if month <= 2 { 1 } else { 0 };
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

        era * 146_097 + day_of_era - 719_468
    }

    fn weekday_monday0(days_since_epoch: i64) -> i64 {
        (days_since_epoch + 3).rem_euclid(7)
    }
}

pub mod audit {
    use trendlab_core::market::DailyBar;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct DataAuditFinding {
        pub date: Option<String>,
        pub code: String,
        pub detail: String,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct DataAuditReport {
        pub bar_count: usize,
        pub start_date: Option<String>,
        pub end_date: Option<String>,
        pub analysis_adjusted_bar_count: usize,
        pub analysis_matches_raw_close_count: usize,
        pub max_analysis_close_gap: Option<f64>,
        pub max_analysis_close_gap_date: Option<String>,
        pub findings: Vec<DataAuditFinding>,
    }

    impl DataAuditReport {
        pub fn is_clean(&self) -> bool {
            self.findings.is_empty()
        }
    }

    pub fn audit_daily_bars(bars: &[DailyBar]) -> DataAuditReport {
        let mut report = DataAuditReport {
            bar_count: bars.len(),
            start_date: bars.first().map(|bar| bar.date.clone()),
            end_date: bars.last().map(|bar| bar.date.clone()),
            ..DataAuditReport::default()
        };

        if bars.is_empty() {
            report.findings.push(DataAuditFinding {
                date: None,
                code: "empty_series".to_string(),
                detail: "data audit requires at least one daily bar".to_string(),
            });
            return report;
        }

        let mut previous_date: Option<&str> = None;
        let mut max_gap = 0.0;
        let mut max_gap_date = None;

        for bar in bars {
            if let Some(previous_date) = previous_date
                && bar.date.as_str() <= previous_date
            {
                report.findings.push(DataAuditFinding {
                    date: Some(bar.date.clone()),
                    code: "dates_not_strictly_increasing".to_string(),
                    detail: format!(
                        "date `{}` must be greater than prior date `{previous_date}`",
                        bar.date
                    ),
                });
            }
            previous_date = Some(bar.date.as_str());

            if [bar.raw_open, bar.raw_high, bar.raw_low, bar.raw_close]
                .into_iter()
                .any(|value| value <= 0.0)
            {
                report.findings.push(DataAuditFinding {
                    date: Some(bar.date.clone()),
                    code: "non_positive_raw_price".to_string(),
                    detail: "raw OHLC values must all be positive".to_string(),
                });
            }

            if bar.raw_high < bar.raw_open.max(bar.raw_close).max(bar.raw_low) {
                report.findings.push(DataAuditFinding {
                    date: Some(bar.date.clone()),
                    code: "raw_high_below_bar_range".to_string(),
                    detail: "raw_high must be at least max(raw_open, raw_close, raw_low)"
                        .to_string(),
                });
            }

            if bar.raw_low > bar.raw_open.min(bar.raw_close).min(bar.raw_high) {
                report.findings.push(DataAuditFinding {
                    date: Some(bar.date.clone()),
                    code: "raw_low_above_bar_range".to_string(),
                    detail: "raw_low must be at most min(raw_open, raw_close, raw_high)"
                        .to_string(),
                });
            }

            if bar.analysis_close <= 0.0 {
                report.findings.push(DataAuditFinding {
                    date: Some(bar.date.clone()),
                    code: "non_positive_analysis_close".to_string(),
                    detail: "analysis_close must be positive".to_string(),
                });
            }

            let analysis_gap = round4((bar.analysis_close - bar.raw_close).abs());
            if analysis_gap == 0.0 {
                report.analysis_matches_raw_close_count += 1;
            } else {
                report.analysis_adjusted_bar_count += 1;
                if analysis_gap > max_gap {
                    max_gap = analysis_gap;
                    max_gap_date = Some(bar.date.clone());
                }
            }
        }

        if max_gap_date.is_some() {
            report.max_analysis_close_gap = Some(max_gap);
            report.max_analysis_close_gap_date = max_gap_date;
        }

        report
    }

    fn round4(value: f64) -> f64 {
        (value * 10_000.0).round() / 10_000.0
    }
}

pub mod inspect {
    use crate::DataError;
    use crate::actions::CorporateActionEffect;
    use crate::audit::{DataAuditFinding, audit_daily_bars};
    use crate::normalize::normalize_symbol_history;
    use crate::provider::ProviderIdentity;
    use crate::resample::{ResampleFrequency, resample_symbol_history};
    use crate::snapshot::{CorporateAction, PersistedSnapshotBundle};

    #[derive(Clone, Debug, PartialEq)]
    pub struct SnapshotInspectionReport {
        pub snapshot_id: String,
        pub provider_identity: ProviderIdentity,
        pub requested_start_date: String,
        pub requested_end_date: String,
        pub capture_mode: String,
        pub entrypoint: String,
        pub captured_at_unix_epoch_seconds: Option<u64>,
        pub symbol_count: usize,
        pub symbols: Vec<SymbolSnapshotInspection>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SymbolSnapshotInspection {
        pub symbol: String,
        pub raw_bar_count: usize,
        pub raw_start_date: Option<String>,
        pub raw_end_date: Option<String>,
        pub corporate_action_count: usize,
        pub split_action_count: usize,
        pub cash_dividend_action_count: usize,
        pub normalized_daily_bar_count: usize,
        pub weekly_bar_count: usize,
        pub monthly_bar_count: usize,
        pub analysis_adjusted_bar_count: usize,
        pub analysis_matches_raw_close_count: usize,
        pub max_analysis_close_gap: Option<f64>,
        pub max_analysis_close_gap_date: Option<String>,
        pub corporate_action_effects: Vec<CorporateActionEffect>,
        pub findings: Vec<DataAuditFinding>,
    }

    pub fn inspect_snapshot_bundle(
        bundle: &PersistedSnapshotBundle,
    ) -> Result<SnapshotInspectionReport, DataError> {
        let mut symbol_reports = Vec::with_capacity(bundle.symbols.len());

        for stored in &bundle.symbols {
            let normalized = normalize_symbol_history(stored)?;
            let weekly = resample_symbol_history(&normalized, ResampleFrequency::Weekly)?;
            let monthly = resample_symbol_history(&normalized, ResampleFrequency::Monthly)?;
            let audit = audit_daily_bars(&normalized.bars);

            let mut split_action_count = 0;
            let mut cash_dividend_action_count = 0;
            for action in &stored.corporate_actions {
                match action {
                    CorporateAction::Split { .. } => split_action_count += 1,
                    CorporateAction::CashDividend { .. } => cash_dividend_action_count += 1,
                }
            }

            symbol_reports.push(SymbolSnapshotInspection {
                symbol: stored.symbol.clone(),
                raw_bar_count: stored.raw_bars.len(),
                raw_start_date: stored.raw_bars.first().map(|bar| bar.date.clone()),
                raw_end_date: stored.raw_bars.last().map(|bar| bar.date.clone()),
                corporate_action_count: stored.corporate_actions.len(),
                split_action_count,
                cash_dividend_action_count,
                normalized_daily_bar_count: normalized.bars.len(),
                weekly_bar_count: weekly.bars.len(),
                monthly_bar_count: monthly.bars.len(),
                analysis_adjusted_bar_count: audit.analysis_adjusted_bar_count,
                analysis_matches_raw_close_count: audit.analysis_matches_raw_close_count,
                max_analysis_close_gap: audit.max_analysis_close_gap,
                max_analysis_close_gap_date: audit.max_analysis_close_gap_date,
                corporate_action_effects: normalized.corporate_action_effects,
                findings: audit.findings,
            });
        }

        Ok(SnapshotInspectionReport {
            snapshot_id: bundle.descriptor.snapshot_id.clone(),
            provider_identity: bundle.descriptor.provider_identity,
            requested_start_date: bundle.descriptor.requested_window.start_date.clone(),
            requested_end_date: bundle.descriptor.requested_window.end_date.clone(),
            capture_mode: bundle.descriptor.capture.capture_mode.clone(),
            entrypoint: bundle.descriptor.capture.entrypoint.clone(),
            captured_at_unix_epoch_seconds: bundle
                .descriptor
                .capture
                .captured_at_unix_epoch_seconds,
            symbol_count: symbol_reports.len(),
            symbols: symbol_reports,
        })
    }
}

pub mod run_source {
    use std::path::Path;

    use trendlab_core::market::DailyBar;

    use crate::DataError;
    use crate::normalize::normalize_symbol_history;
    use crate::provider::ProviderIdentity;
    use crate::snapshot::PersistedSnapshotBundle;
    use crate::snapshot_store::load_snapshot_bundle;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SnapshotRunSliceRequest {
        pub symbol: String,
        pub start_date: String,
        pub end_date: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SnapshotRunFormOptions {
        pub snapshot_id: String,
        pub provider_identity: ProviderIdentity,
        pub symbols: Vec<SnapshotRunSymbolOptions>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SnapshotRunSymbolOptions {
        pub symbol: String,
        pub available_dates: Vec<String>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct ResolvedSnapshotRunSource {
        pub snapshot_id: String,
        pub provider_identity: ProviderIdentity,
        pub symbol: String,
        pub selected_start_date: String,
        pub selected_end_date: String,
        pub bars: Vec<DailyBar>,
    }

    pub fn resolve_snapshot_run_source(
        snapshot_dir: &Path,
        request: &SnapshotRunSliceRequest,
    ) -> Result<ResolvedSnapshotRunSource, DataError> {
        let bundle = load_snapshot_bundle(snapshot_dir)?;
        resolve_snapshot_bundle_slice(&bundle, request)
    }

    pub fn snapshot_run_form_options(
        bundle: &PersistedSnapshotBundle,
    ) -> Result<SnapshotRunFormOptions, DataError> {
        let mut symbols = Vec::with_capacity(bundle.symbols.len());

        for stored in &bundle.symbols {
            let normalized = normalize_symbol_history(stored)?;
            symbols.push(SnapshotRunSymbolOptions {
                symbol: normalized.symbol,
                available_dates: normalized.bars.into_iter().map(|bar| bar.date).collect(),
            });
        }

        symbols.sort_by(|left, right| left.symbol.cmp(&right.symbol));

        Ok(SnapshotRunFormOptions {
            snapshot_id: bundle.descriptor.snapshot_id.clone(),
            provider_identity: bundle.descriptor.provider_identity,
            symbols,
        })
    }

    pub fn resolve_snapshot_bundle_slice(
        bundle: &PersistedSnapshotBundle,
        request: &SnapshotRunSliceRequest,
    ) -> Result<ResolvedSnapshotRunSource, DataError> {
        if request.symbol.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot run source requires a non-empty symbol",
            ));
        }

        if request.start_date.trim().is_empty() || request.end_date.trim().is_empty() {
            return Err(DataError::invalid(
                "snapshot run source requires non-empty start_date and end_date",
            ));
        }

        if request.end_date < request.start_date {
            return Err(DataError::invalid(
                "snapshot run source requires end_date on or after start_date",
            ));
        }

        let stored = bundle
            .symbols
            .iter()
            .find(|stored| stored.symbol == request.symbol)
            .ok_or_else(|| {
                DataError::invalid(format!(
                    "snapshot `{}` does not contain symbol `{}`",
                    bundle.descriptor.snapshot_id, request.symbol
                ))
            })?;
        let normalized = normalize_symbol_history(stored)?;
        let start_index = normalized
            .bars
            .iter()
            .position(|bar| bar.date == request.start_date)
            .ok_or_else(|| {
                DataError::invalid(format!(
                    "snapshot `{}` symbol `{}` does not contain start_date `{}`",
                    bundle.descriptor.snapshot_id, request.symbol, request.start_date
                ))
            })?;
        let end_index = normalized
            .bars
            .iter()
            .position(|bar| bar.date == request.end_date)
            .ok_or_else(|| {
                DataError::invalid(format!(
                    "snapshot `{}` symbol `{}` does not contain end_date `{}`",
                    bundle.descriptor.snapshot_id, request.symbol, request.end_date
                ))
            })?;

        if end_index < start_index {
            return Err(DataError::invalid(format!(
                "snapshot `{}` symbol `{}` has end_date `{}` before start_date `{}` in bar order",
                bundle.descriptor.snapshot_id, request.symbol, request.end_date, request.start_date
            )));
        }

        Ok(ResolvedSnapshotRunSource {
            snapshot_id: bundle.descriptor.snapshot_id.clone(),
            provider_identity: bundle.descriptor.provider_identity,
            symbol: normalized.symbol,
            selected_start_date: request.start_date.clone(),
            selected_end_date: request.end_date.clone(),
            bars: normalized.bars[start_index..=end_index].to_vec(),
        })
    }
}

pub mod live {
    use std::time::Duration;

    use reqwest::blocking::Client;

    use crate::DataError;
    use crate::SNAPSHOT_SCHEMA_VERSION;
    use crate::ingest::ingest_tiingo_symbol_history;
    use crate::provider::{
        ProviderIdentity, TiingoCorporateAction, TiingoCorporateActionKind, TiingoDailyBar,
    };
    use crate::resample::ResampleFrequency;
    use crate::snapshot::{SnapshotMetadata, StoredSymbolData};

    pub const TIINGO_API_TOKEN_ENV: &str = "TIINGO_API_TOKEN";
    const TIINGO_HISTORICAL_PRICES_URL: &str = "https://api.tiingo.com/tiingo/daily/";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LiveSymbolHistoryRequest {
        pub symbol: String,
        pub start_date: String,
        pub end_date: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SmokeCheckPlan {
        pub provider_identity: ProviderIdentity,
        pub required_env_var: String,
        pub symbol: String,
        pub start_date: String,
        pub end_date: String,
        pub expected_resamples: Vec<ResampleFrequency>,
        pub invariants: Vec<String>,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct LiveFetchedSymbolHistory {
        pub provider_identity: ProviderIdentity,
        pub snapshot_id: String,
        pub symbol: String,
        pub first_date: String,
        pub last_date: String,
        pub provider_daily_bar_count: usize,
        pub provider_corporate_action_count: usize,
        pub stored: StoredSymbolData,
    }

    pub trait ProviderAdapter {
        fn provider_identity(&self) -> ProviderIdentity;
        fn smoke_plan(
            &self,
            request: &LiveSymbolHistoryRequest,
        ) -> Result<SmokeCheckPlan, DataError>;
        fn fetch_symbol_history(
            &self,
            request: &LiveSymbolHistoryRequest,
            api_token: &str,
        ) -> Result<LiveFetchedSymbolHistory, DataError>;
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct TiingoAdapter;

    impl ProviderAdapter for TiingoAdapter {
        fn provider_identity(&self) -> ProviderIdentity {
            ProviderIdentity::Tiingo
        }

        fn smoke_plan(
            &self,
            request: &LiveSymbolHistoryRequest,
        ) -> Result<SmokeCheckPlan, DataError> {
            validate_live_request(request)?;

            Ok(SmokeCheckPlan {
                provider_identity: self.provider_identity(),
                required_env_var: TIINGO_API_TOKEN_ENV.to_string(),
                symbol: request.symbol.clone(),
                start_date: request.start_date.clone(),
                end_date: request.end_date.clone(),
                expected_resamples: vec![ResampleFrequency::Weekly, ResampleFrequency::Monthly],
                invariants: vec![
                    "fetch the Tiingo historical prices endpoint over real HTTP outside normal validation"
                        .to_string(),
                    "ingest provider-native rows into stored symbol history before normalization"
                        .to_string(),
                    "normalize into trendlab-core daily bars with split-adjusted analysis_close only"
                        .to_string(),
                    "keep dividend cashflows explicit instead of silently folding them into analysis_close"
                        .to_string(),
                    "resample canonical daily bars into weekly and monthly bars inside trendlab-data"
                        .to_string(),
                ],
            })
        }

        fn fetch_symbol_history(
            &self,
            request: &LiveSymbolHistoryRequest,
            api_token: &str,
        ) -> Result<LiveFetchedSymbolHistory, DataError> {
            validate_live_request(request)?;

            if api_token.trim().is_empty() {
                return Err(DataError::invalid(
                    "tiingo live fetch requires a non-empty API token",
                ));
            }

            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|err| {
                    DataError::invalid(format!("failed to build Tiingo HTTP client: {err}"))
                })?;
            let url = build_tiingo_historical_prices_url(request, api_token)?;
            let response = client
                .get(url)
                .header(reqwest::header::ACCEPT, "application/json")
                .send()
                .map_err(|err| {
                    DataError::invalid(format!("tiingo live fetch request failed: {err}"))
                })?;
            let status = response.status();
            let body = response.text().map_err(|err| {
                DataError::invalid(format!("failed to read Tiingo response body: {err}"))
            })?;

            if !status.is_success() {
                return Err(DataError::invalid(format!(
                    "tiingo live fetch failed with HTTP {}: {}",
                    status,
                    summarize_provider_body(&body)
                )));
            }

            let price_rows = decode_tiingo_price_rows(&body)?;
            if price_rows.is_empty() {
                return Err(DataError::invalid(
                    "tiingo live fetch returned no daily price rows",
                ));
            }

            let (daily_bars, corporate_actions) =
                convert_tiingo_price_rows(&request.symbol, &price_rows)?;
            let metadata = build_live_snapshot_metadata(request, &daily_bars);
            let stored = ingest_tiingo_symbol_history(
                metadata,
                &request.symbol,
                &daily_bars,
                &corporate_actions,
            )?;
            let first_date = stored
                .raw_bars
                .first()
                .map(|bar| bar.date.clone())
                .ok_or_else(|| {
                    DataError::invalid("tiingo live fetch did not produce stored daily bars")
                })?;
            let last_date = stored
                .raw_bars
                .last()
                .map(|bar| bar.date.clone())
                .ok_or_else(|| {
                    DataError::invalid("tiingo live fetch did not produce stored daily bars")
                })?;

            Ok(LiveFetchedSymbolHistory {
                provider_identity: self.provider_identity(),
                snapshot_id: stored.metadata.snapshot_id.clone(),
                symbol: stored.symbol.clone(),
                first_date,
                last_date,
                provider_daily_bar_count: daily_bars.len(),
                provider_corporate_action_count: corporate_actions.len(),
                stored,
            })
        }
    }

    fn validate_live_request(request: &LiveSymbolHistoryRequest) -> Result<(), DataError> {
        if request.symbol.trim().is_empty() {
            return Err(DataError::invalid(
                "live provider requests require a non-empty symbol",
            ));
        }

        if request.start_date.trim().is_empty() || request.end_date.trim().is_empty() {
            return Err(DataError::invalid(
                "live provider requests require non-empty start_date and end_date",
            ));
        }

        if request.end_date < request.start_date {
            return Err(DataError::invalid(
                "live provider requests require end_date on or after start_date",
            ));
        }

        Ok(())
    }

    fn build_tiingo_historical_prices_url(
        request: &LiveSymbolHistoryRequest,
        api_token: &str,
    ) -> Result<reqwest::Url, DataError> {
        let mut url = reqwest::Url::parse(TIINGO_HISTORICAL_PRICES_URL)
            .map_err(|err| DataError::invalid(format!("invalid Tiingo base URL: {err}")))?;
        url.path_segments_mut()
            .map_err(|_| DataError::invalid("invalid Tiingo historical prices URL shape"))?
            .pop_if_empty()
            .extend([request.symbol.as_str(), "prices"]);
        url.query_pairs_mut()
            .append_pair("startDate", &request.start_date)
            .append_pair("endDate", &request.end_date)
            .append_pair("format", "json")
            .append_pair("token", api_token);
        Ok(url)
    }

    fn build_live_snapshot_metadata(
        request: &LiveSymbolHistoryRequest,
        daily_bars: &[TiingoDailyBar],
    ) -> SnapshotMetadata {
        let first_date = daily_bars
            .first()
            .map(|bar| bar.date.as_str())
            .unwrap_or(request.start_date.as_str());
        let last_date = daily_bars
            .last()
            .map(|bar| bar.date.as_str())
            .unwrap_or(request.end_date.as_str());

        SnapshotMetadata {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            snapshot_id: format!(
                "live:{}:{}:{}:{}",
                ProviderIdentity::Tiingo.as_str(),
                request.symbol,
                first_date,
                last_date
            ),
            provider_identity: ProviderIdentity::Tiingo,
        }
    }

    fn decode_tiingo_price_rows(body: &str) -> Result<Vec<TiingoHistoricalPriceRow>, DataError> {
        serde_json::from_str(body).map_err(|err| {
            DataError::invalid(format!("failed to parse Tiingo JSON payload: {err}"))
        })
    }

    fn convert_tiingo_price_rows(
        symbol: &str,
        price_rows: &[TiingoHistoricalPriceRow],
    ) -> Result<(Vec<TiingoDailyBar>, Vec<TiingoCorporateAction>), DataError> {
        let mut daily_bars = Vec::with_capacity(price_rows.len());
        let mut corporate_actions = Vec::new();

        for row in price_rows {
            let normalized_date = normalize_tiingo_date(&row.date)?;
            let split_factor = row.split_factor.unwrap_or(1.0);
            let div_cash = row.div_cash.unwrap_or(0.0);

            if split_factor <= 0.0 {
                return Err(DataError::invalid(
                    "tiingo live fetch returned a non-positive splitFactor",
                ));
            }

            if div_cash < 0.0 {
                return Err(DataError::invalid(
                    "tiingo live fetch returned a negative divCash",
                ));
            }

            daily_bars.push(TiingoDailyBar {
                symbol: symbol.to_string(),
                date: normalized_date.clone(),
                open: row.open,
                high: row.high,
                low: row.low,
                close: row.close,
                volume: row.volume,
            });

            if (split_factor - 1.0).abs() > f64::EPSILON {
                corporate_actions.push(TiingoCorporateAction {
                    symbol: symbol.to_string(),
                    ex_date: normalized_date.clone(),
                    kind: TiingoCorporateActionKind::Split,
                    split_ratio: Some(split_factor),
                    cash_amount: None,
                });
            }

            if div_cash > 0.0 {
                corporate_actions.push(TiingoCorporateAction {
                    symbol: symbol.to_string(),
                    ex_date: normalized_date,
                    kind: TiingoCorporateActionKind::CashDividend,
                    split_ratio: None,
                    cash_amount: Some(div_cash),
                });
            }
        }

        Ok((daily_bars, corporate_actions))
    }

    fn normalize_tiingo_date(value: &str) -> Result<String, DataError> {
        let trimmed = value.trim();
        if trimmed.len() < 10 {
            return Err(DataError::invalid(
                "tiingo live fetch returned a malformed date",
            ));
        }

        let date = &trimmed[..10];
        if !looks_like_iso_date(date) {
            return Err(DataError::invalid(
                "tiingo live fetch returned a non-ISO date",
            ));
        }

        Ok(date.to_string())
    }

    fn looks_like_iso_date(value: &str) -> bool {
        value.len() == 10
            && value.as_bytes()[4] == b'-'
            && value.as_bytes()[7] == b'-'
            && value
                .chars()
                .enumerate()
                .all(|(index, ch)| matches!(index, 4 | 7) || ch.is_ascii_digit())
    }

    fn summarize_provider_body(body: &str) -> String {
        let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
        let summary: String = normalized.chars().take(160).collect();
        if normalized.chars().count() > 160 {
            format!("{summary}...")
        } else if summary.is_empty() {
            "empty response body".to_string()
        } else {
            summary
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Deserialize)]
    struct TiingoHistoricalPriceRow {
        date: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
        #[serde(default, rename = "divCash")]
        div_cash: Option<f64>,
        #[serde(default, rename = "splitFactor")]
        split_factor: Option<f64>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn tiingo_historical_prices_url_carries_window_and_token() {
            let url = build_tiingo_historical_prices_url(
                &LiveSymbolHistoryRequest {
                    symbol: "SPY".to_string(),
                    start_date: "2025-01-02".to_string(),
                    end_date: "2025-01-10".to_string(),
                },
                "secret-token",
            )
            .unwrap();

            assert_eq!(
                url.as_str(),
                "https://api.tiingo.com/tiingo/daily/SPY/prices?startDate=2025-01-02&endDate=2025-01-10&format=json&token=secret-token"
            );
        }

        #[test]
        fn tiingo_price_rows_decode_into_bars_and_actions() {
            let rows = decode_tiingo_price_rows(
                r#"
                [
                  {
                    "date": "2025-01-06T00:00:00.000Z",
                    "open": 100.0,
                    "high": 102.0,
                    "low": 99.0,
                    "close": 101.0,
                    "volume": 12345,
                    "divCash": 0.25,
                    "splitFactor": 2.0
                  },
                  {
                    "date": "2025-01-07T00:00:00.000Z",
                    "open": 51.0,
                    "high": 53.0,
                    "low": 50.0,
                    "close": 52.0,
                    "volume": 23456,
                    "divCash": 0.0,
                    "splitFactor": 1.0
                  }
                ]
                "#,
            )
            .unwrap();

            let (daily_bars, corporate_actions) = convert_tiingo_price_rows("SPY", &rows).unwrap();

            assert_eq!(daily_bars.len(), 2);
            assert_eq!(daily_bars[0].date, "2025-01-06");
            assert_eq!(daily_bars[1].date, "2025-01-07");
            assert_eq!(corporate_actions.len(), 2);
            assert_eq!(
                corporate_actions[0],
                TiingoCorporateAction {
                    symbol: "SPY".to_string(),
                    ex_date: "2025-01-06".to_string(),
                    kind: TiingoCorporateActionKind::Split,
                    split_ratio: Some(2.0),
                    cash_amount: None,
                }
            );
            assert_eq!(
                corporate_actions[1],
                TiingoCorporateAction {
                    symbol: "SPY".to_string(),
                    ex_date: "2025-01-06".to_string(),
                    kind: TiingoCorporateActionKind::CashDividend,
                    split_ratio: None,
                    cash_amount: Some(0.25),
                }
            );
        }

        #[test]
        fn tiingo_price_rows_reject_non_positive_split_factor() {
            let rows = decode_tiingo_price_rows(
                r#"
                [
                  {
                    "date": "2025-01-06T00:00:00.000Z",
                    "open": 100.0,
                    "high": 102.0,
                    "low": 99.0,
                    "close": 101.0,
                    "volume": 12345,
                    "splitFactor": 0.0
                  }
                ]
                "#,
            )
            .unwrap();

            let error = convert_tiingo_price_rows("SPY", &rows).unwrap_err();

            assert_eq!(
                error.to_string(),
                "tiingo live fetch returned a non-positive splitFactor"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::SNAPSHOT_SCHEMA_VERSION;
    use crate::actions::build_corporate_action_effects;
    use crate::audit::audit_daily_bars;
    use crate::ingest::ingest_tiingo_symbol_history;
    use crate::inspect::inspect_snapshot_bundle;
    use crate::live::{
        LiveSymbolHistoryRequest, ProviderAdapter, TIINGO_API_TOKEN_ENV, TiingoAdapter,
    };
    use crate::normalize::normalize_symbol_history;
    use crate::provider::{
        ProviderIdentity, TiingoCorporateAction, TiingoCorporateActionKind, TiingoDailyBar,
    };
    use crate::resample::{ResampleFrequency, resample_symbol_history};
    use crate::run_source::{
        SnapshotRunSliceRequest, resolve_snapshot_bundle_slice, snapshot_run_form_options,
    };
    use crate::snapshot::{
        CorporateAction, PersistedSnapshotBundle, SnapshotBundleDescriptor,
        SnapshotCaptureMetadata, SnapshotMetadata, SnapshotRequestedWindow,
    };
    use crate::snapshot_store::{load_snapshot_bundle, write_snapshot_bundle};
    use trendlab_core::market::DailyBar;

    #[test]
    fn provider_identity_round_trips() {
        assert_eq!(
            ProviderIdentity::parse(ProviderIdentity::TIINGO),
            Some(ProviderIdentity::Tiingo)
        );
        assert_eq!(ProviderIdentity::Tiingo.as_str(), ProviderIdentity::TIINGO);
    }

    #[test]
    fn tiingo_action_kind_round_trips() {
        assert_eq!(
            TiingoCorporateActionKind::parse(TiingoCorporateActionKind::SPLIT),
            Some(TiingoCorporateActionKind::Split)
        );
        assert_eq!(
            TiingoCorporateActionKind::CashDividend.as_str(),
            TiingoCorporateActionKind::CASH_DIVIDEND
        );
    }

    #[test]
    fn resample_frequency_round_trips() {
        assert_eq!(
            ResampleFrequency::parse(ResampleFrequency::WEEKLY),
            Some(ResampleFrequency::Weekly)
        );
        assert_eq!(
            ResampleFrequency::Monthly.as_str(),
            ResampleFrequency::MONTHLY
        );
    }

    #[test]
    fn tiingo_fixture_normalizes_split_adjusted_analysis_close_without_dividend_adjustment() {
        let bars = load_tiingo_bars("m2_tiingo_split_adjustment").unwrap();
        let actions = load_tiingo_actions("m2_tiingo_split_adjustment").unwrap();
        let stored = ingest_tiingo_symbol_history(
            sample_metadata("m2_tiingo_split_adjustment"),
            "TEST",
            &bars,
            &actions,
        )
        .unwrap();

        let normalized = normalize_symbol_history(&stored).unwrap();

        assert_eq!(normalized.symbol, "TEST");
        assert_eq!(normalized.bars.len(), 4);
        assert_eq!(normalized.corporate_actions.len(), 2);
        assert_eq!(normalized.corporate_action_effects.len(), 2);
        assert_eq!(normalized.bars[0].raw_close, 102.0);
        assert_eq!(normalized.bars[0].analysis_close, 51.0);
        assert_eq!(normalized.bars[1].raw_close, 104.0);
        assert_eq!(normalized.bars[1].analysis_close, 52.0);
        assert_eq!(normalized.bars[2].raw_close, 51.0);
        assert_eq!(normalized.bars[2].analysis_close, 51.0);
        assert_eq!(normalized.bars[3].raw_close, 53.0);
        assert_eq!(normalized.bars[3].analysis_close, 53.0);
        assert_eq!(normalized.corporate_action_effects[0].ex_date, "2025-01-06");
        assert_eq!(normalized.corporate_action_effects[0].split_ratio, 2.0);
        assert_eq!(
            normalized.corporate_action_effects[0].cash_dividend_per_share,
            0.0
        );
        assert!(normalized.corporate_action_effects[0].has_split());
        assert_eq!(normalized.corporate_action_effects[1].ex_date, "2025-01-07");
        assert_eq!(normalized.corporate_action_effects[1].split_ratio, 1.0);
        assert_eq!(
            normalized.corporate_action_effects[1].cash_dividend_per_share,
            0.25
        );
        assert!(normalized.corporate_action_effects[1].has_cash_dividend());
    }

    #[test]
    fn corporate_action_effects_group_same_day_split_and_dividend() {
        let effects = build_corporate_action_effects(&[
            CorporateAction::Split {
                symbol: "TEST".to_string(),
                ex_date: "2025-02-03".to_string(),
                ratio: 2.0,
            },
            CorporateAction::CashDividend {
                symbol: "TEST".to_string(),
                ex_date: "2025-02-03".to_string(),
                cash_amount: 0.25,
            },
        ])
        .unwrap();

        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].ex_date, "2025-02-03");
        assert_eq!(effects[0].split_ratio, 2.0);
        assert_eq!(effects[0].cash_dividend_per_share, 0.25);
    }

    #[test]
    fn tiingo_fixture_resamples_weekly_and_monthly_from_normalized_bars() {
        let bars = load_tiingo_bars("m2_tiingo_resampling").unwrap();
        let actions = load_tiingo_actions("m2_tiingo_resampling").unwrap();
        let stored = ingest_tiingo_symbol_history(
            sample_metadata("m2_tiingo_resampling"),
            "TEST",
            &bars,
            &actions,
        )
        .unwrap();
        let normalized = normalize_symbol_history(&stored).unwrap();

        let weekly = resample_symbol_history(&normalized, ResampleFrequency::Weekly).unwrap();
        let monthly = resample_symbol_history(&normalized, ResampleFrequency::Monthly).unwrap();

        assert_eq!(weekly.bars.len(), 2);
        assert_eq!(weekly.bars[0].date, "2025-01-31");
        assert_eq!(weekly.bars[0].raw_open, 100.0);
        assert_eq!(weekly.bars[0].raw_high, 104.0);
        assert_eq!(weekly.bars[0].raw_low, 99.0);
        assert_eq!(weekly.bars[0].raw_close, 103.0);
        assert_eq!(weekly.bars[0].analysis_close, 51.5);
        assert_eq!(weekly.bars[1].date, "2025-02-07");
        assert_eq!(weekly.bars[1].raw_open, 52.0);
        assert_eq!(weekly.bars[1].raw_high, 57.0);
        assert_eq!(weekly.bars[1].raw_low, 51.0);
        assert_eq!(weekly.bars[1].raw_close, 56.0);
        assert_eq!(weekly.bars[1].analysis_close, 56.0);

        assert_eq!(monthly.bars.len(), 2);
        assert_eq!(monthly.bars[0].date, "2025-01-31");
        assert_eq!(monthly.bars[0].raw_open, 100.0);
        assert_eq!(monthly.bars[0].raw_high, 104.0);
        assert_eq!(monthly.bars[0].raw_low, 99.0);
        assert_eq!(monthly.bars[0].raw_close, 103.0);
        assert_eq!(monthly.bars[0].analysis_close, 51.5);
        assert_eq!(monthly.bars[1].date, "2025-02-07");
        assert_eq!(monthly.bars[1].raw_open, 52.0);
        assert_eq!(monthly.bars[1].raw_high, 57.0);
        assert_eq!(monthly.bars[1].raw_low, 51.0);
        assert_eq!(monthly.bars[1].raw_close, 56.0);
        assert_eq!(monthly.bars[1].analysis_close, 56.0);
        assert_eq!(
            monthly.corporate_action_effects,
            normalized.corporate_action_effects
        );
    }

    #[test]
    fn tiingo_adapter_smoke_plan_is_explicit_about_m2_invariants() {
        let adapter = TiingoAdapter;
        let plan = adapter
            .smoke_plan(&LiveSymbolHistoryRequest {
                symbol: "SPY".to_string(),
                start_date: "2025-01-02".to_string(),
                end_date: "2025-01-10".to_string(),
            })
            .unwrap();

        assert_eq!(plan.provider_identity, ProviderIdentity::Tiingo);
        assert_eq!(plan.required_env_var, TIINGO_API_TOKEN_ENV);
        assert_eq!(
            plan.expected_resamples,
            vec![ResampleFrequency::Weekly, ResampleFrequency::Monthly]
        );
        assert!(
            plan.invariants
                .iter()
                .any(|item| item.contains("split-adjusted analysis_close only"))
        );
    }

    #[test]
    fn tiingo_adapter_smoke_plan_rejects_backwards_windows() {
        let adapter = TiingoAdapter;
        let error = adapter
            .smoke_plan(&LiveSymbolHistoryRequest {
                symbol: "SPY".to_string(),
                start_date: "2025-01-10".to_string(),
                end_date: "2025-01-02".to_string(),
            })
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "live provider requests require end_date on or after start_date"
        );
    }

    #[test]
    fn data_audit_reports_analysis_close_divergence_without_flagging_clean_bars() {
        let report = audit_daily_bars(&[
            DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.5,
                analysis_close: 50.25,
            },
            DailyBar {
                date: "2025-01-03".to_string(),
                raw_open: 102.0,
                raw_high: 103.0,
                raw_low: 101.0,
                raw_close: 102.5,
                analysis_close: 102.5,
            },
        ]);

        assert_eq!(report.bar_count, 2);
        assert_eq!(report.analysis_adjusted_bar_count, 1);
        assert_eq!(report.analysis_matches_raw_close_count, 1);
        assert_eq!(report.max_analysis_close_gap, Some(50.25));
        assert_eq!(
            report.max_analysis_close_gap_date,
            Some("2025-01-02".to_string())
        );
        assert!(report.is_clean());
    }

    #[test]
    fn data_audit_flags_order_and_price_shape_issues() {
        let report = audit_daily_bars(&[
            DailyBar {
                date: "2025-01-03".to_string(),
                raw_open: 100.0,
                raw_high: 101.0,
                raw_low: 99.0,
                raw_close: 100.0,
                analysis_close: 100.0,
            },
            DailyBar {
                date: "2025-01-02".to_string(),
                raw_open: 100.0,
                raw_high: 99.0,
                raw_low: 101.0,
                raw_close: -1.0,
                analysis_close: 0.0,
            },
        ]);

        assert_eq!(report.bar_count, 2);
        assert_eq!(report.findings.len(), 5);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "dates_not_strictly_increasing")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "non_positive_raw_price")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "raw_high_below_bar_range")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "raw_low_above_bar_range")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "non_positive_analysis_close")
        );
        assert!(!report.is_clean());
    }

    #[test]
    fn tiingo_ingest_rejects_duplicate_daily_bar_dates() {
        let bar = TiingoDailyBar {
            symbol: "TEST".to_string(),
            date: "2025-01-02".to_string(),
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.0,
            volume: 1_000,
        };
        let bars = vec![bar.clone(), bar];

        let error = ingest_tiingo_symbol_history(
            sample_metadata("m2_tiingo_split_adjustment"),
            "TEST",
            &bars,
            &[],
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "raw daily bars must be in strictly increasing date order with unique dates"
        );
    }

    #[test]
    fn tiingo_ingest_rejects_non_positive_split_ratios() {
        let bars = load_tiingo_bars("m2_tiingo_split_adjustment").unwrap();
        let actions = vec![TiingoCorporateAction {
            symbol: "TEST".to_string(),
            ex_date: "2025-01-06".to_string(),
            kind: TiingoCorporateActionKind::Split,
            split_ratio: Some(0.0),
            cash_amount: None,
        }];

        let error = ingest_tiingo_symbol_history(
            sample_metadata("m2_tiingo_split_adjustment"),
            "TEST",
            &bars,
            &actions,
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "split corporate actions require a positive split_ratio"
        );
    }

    #[test]
    fn snapshot_bundle_round_trips_through_snapshot_json_and_jsonl_layout() {
        let bars = load_tiingo_bars("m2_tiingo_split_adjustment").unwrap();
        let actions = load_tiingo_actions("m2_tiingo_split_adjustment").unwrap();
        let stored = ingest_tiingo_symbol_history(
            SnapshotMetadata {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                snapshot_id: "live:tiingo:TEST:2025-01-03:2025-01-08".to_string(),
                provider_identity: ProviderIdentity::Tiingo,
            },
            "TEST",
            &bars,
            &actions,
        )
        .unwrap();
        let descriptor = SnapshotBundleDescriptor::from_stored_symbols(
            stored.metadata.snapshot_id.clone(),
            stored.metadata.provider_identity,
            sample_requested_window(),
            sample_capture_metadata(),
            std::slice::from_ref(&stored),
        )
        .unwrap();
        let bundle = PersistedSnapshotBundle {
            descriptor,
            symbols: vec![stored.clone()],
        };
        let snapshot_dir = temp_test_dir("snapshot_round_trip");

        write_snapshot_bundle(&snapshot_dir, &bundle).unwrap();
        let reopened = load_snapshot_bundle(&snapshot_dir).unwrap();
        let normalized = normalize_symbol_history(&reopened.symbols[0]).unwrap();
        let monthly = resample_symbol_history(&normalized, ResampleFrequency::Monthly).unwrap();

        assert_eq!(reopened, bundle);
        assert_eq!(normalized.symbol, "TEST");
        assert_eq!(normalized.bars.len(), 4);
        assert_eq!(monthly.bars.len(), 1);

        fs::remove_dir_all(snapshot_dir).unwrap();
    }

    #[test]
    fn snapshot_bundle_load_rejects_missing_daily_file() {
        let bundle = sample_snapshot_bundle();
        let snapshot_dir = temp_test_dir("snapshot_missing_daily");
        let daily_file = snapshot_dir.join("daily").join("TEST.jsonl");

        write_snapshot_bundle(&snapshot_dir, &bundle).unwrap();
        fs::remove_file(&daily_file).unwrap();

        let error = load_snapshot_bundle(&snapshot_dir).unwrap_err();

        assert!(
            error
                .to_string()
                .contains(&format!("failed to open {}", daily_file.display()))
        );

        fs::remove_dir_all(snapshot_dir).unwrap();
    }

    #[test]
    fn snapshot_bundle_load_rejects_descriptor_count_drift() {
        let bundle = sample_snapshot_bundle();
        let snapshot_dir = temp_test_dir("snapshot_count_drift");
        let descriptor_path = snapshot_dir.join("snapshot.json");

        write_snapshot_bundle(&snapshot_dir, &bundle).unwrap();
        let mut descriptor = bundle.descriptor.clone();
        descriptor.symbols[0].raw_bar_count += 1;
        fs::write(
            &descriptor_path,
            format!("{}\n", serde_json::to_string_pretty(&descriptor).unwrap()),
        )
        .unwrap();

        let error = load_snapshot_bundle(&snapshot_dir).unwrap_err();

        assert_eq!(
            error.to_string(),
            "snapshot symbol `TEST` expected 5 raw daily bars but loaded 4"
        );

        fs::remove_dir_all(snapshot_dir).unwrap();
    }

    #[test]
    fn snapshot_inspection_surfaces_provider_actions_and_normalization_inputs() {
        let bundle = sample_snapshot_bundle();

        let report = inspect_snapshot_bundle(&bundle).unwrap();

        assert_eq!(report.snapshot_id, "live:tiingo:TEST:2025-01-03:2025-01-08");
        assert_eq!(report.provider_identity, ProviderIdentity::Tiingo);
        assert_eq!(report.requested_start_date, "2025-01-02");
        assert_eq!(report.requested_end_date, "2025-01-10");
        assert_eq!(report.capture_mode, "live_provider_fetch");
        assert_eq!(report.entrypoint, "cargo xtask capture-live-snapshot");
        assert_eq!(report.symbol_count, 1);
        assert_eq!(report.symbols.len(), 1);

        let symbol = &report.symbols[0];
        assert_eq!(symbol.symbol, "TEST");
        assert_eq!(symbol.raw_bar_count, 4);
        assert_eq!(symbol.raw_start_date.as_deref(), Some("2025-01-02"));
        assert_eq!(symbol.raw_end_date.as_deref(), Some("2025-01-07"));
        assert_eq!(symbol.corporate_action_count, 2);
        assert_eq!(symbol.split_action_count, 1);
        assert_eq!(symbol.cash_dividend_action_count, 1);
        assert_eq!(symbol.normalized_daily_bar_count, 4);
        assert_eq!(symbol.weekly_bar_count, 2);
        assert_eq!(symbol.monthly_bar_count, 1);
        assert_eq!(symbol.analysis_adjusted_bar_count, 2);
        assert_eq!(symbol.analysis_matches_raw_close_count, 2);
        assert_eq!(symbol.max_analysis_close_gap, Some(52.0));
        assert_eq!(
            symbol.max_analysis_close_gap_date.as_deref(),
            Some("2025-01-03")
        );
        assert!(symbol.findings.is_empty());
        assert_eq!(symbol.corporate_action_effects.len(), 2);
        assert_eq!(symbol.corporate_action_effects[0].ex_date, "2025-01-06");
        assert_eq!(symbol.corporate_action_effects[0].split_ratio, 2.0);
        assert_eq!(
            symbol.corporate_action_effects[1].cash_dividend_per_share,
            0.25
        );
    }

    #[test]
    fn snapshot_run_source_resolves_exact_symbol_slice_into_canonical_bars() {
        let bundle = sample_snapshot_bundle();

        let resolved = resolve_snapshot_bundle_slice(
            &bundle,
            &SnapshotRunSliceRequest {
                symbol: "TEST".to_string(),
                start_date: "2025-01-03".to_string(),
                end_date: "2025-01-07".to_string(),
            },
        )
        .unwrap();

        assert_eq!(
            resolved.snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(resolved.provider_identity, ProviderIdentity::Tiingo);
        assert_eq!(resolved.symbol, "TEST");
        assert_eq!(resolved.selected_start_date, "2025-01-03");
        assert_eq!(resolved.selected_end_date, "2025-01-07");
        assert_eq!(resolved.bars.len(), 3);
        assert_eq!(resolved.bars[0].date, "2025-01-03");
        assert_eq!(resolved.bars[0].analysis_close, 52.0);
        assert_eq!(resolved.bars[2].date, "2025-01-07");
        assert_eq!(resolved.bars[2].analysis_close, 53.0);
    }

    #[test]
    fn snapshot_run_source_rejects_missing_symbol() {
        let bundle = sample_snapshot_bundle();

        let error = resolve_snapshot_bundle_slice(
            &bundle,
            &SnapshotRunSliceRequest {
                symbol: "MISSING".to_string(),
                start_date: "2025-01-03".to_string(),
                end_date: "2025-01-07".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "snapshot `live:tiingo:TEST:2025-01-03:2025-01-08` does not contain symbol `MISSING`"
        );
    }

    #[test]
    fn snapshot_run_source_rejects_dates_that_do_not_land_on_snapshot_bars() {
        let bundle = sample_snapshot_bundle();

        let error = resolve_snapshot_bundle_slice(
            &bundle,
            &SnapshotRunSliceRequest {
                symbol: "TEST".to_string(),
                start_date: "2025-01-04".to_string(),
                end_date: "2025-01-07".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "snapshot `live:tiingo:TEST:2025-01-03:2025-01-08` symbol `TEST` does not contain start_date `2025-01-04`"
        );
    }

    #[test]
    fn snapshot_run_form_options_surface_exact_symbol_dates() {
        let bundle = sample_snapshot_bundle();

        let options = snapshot_run_form_options(&bundle).unwrap();

        assert_eq!(
            options.snapshot_id,
            "live:tiingo:TEST:2025-01-03:2025-01-08"
        );
        assert_eq!(options.provider_identity, ProviderIdentity::Tiingo);
        assert_eq!(options.symbols.len(), 1);
        assert_eq!(options.symbols[0].symbol, "TEST");
        assert_eq!(
            options.symbols[0].available_dates,
            vec![
                "2025-01-02".to_string(),
                "2025-01-03".to_string(),
                "2025-01-06".to_string(),
                "2025-01-07".to_string(),
            ]
        );
    }

    fn sample_metadata(name: &str) -> SnapshotMetadata {
        SnapshotMetadata {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            snapshot_id: format!("fixture:{name}"),
            provider_identity: ProviderIdentity::Tiingo,
        }
    }

    fn load_tiingo_bars(name: &str) -> Result<Vec<TiingoDailyBar>, String> {
        let raw = read_required_file(&fixture_dir(name).join("tiingo-daily-bars.csv"))?;
        let rows = parse_csv_rows(&raw)?;
        let mut bars = Vec::new();

        for row in rows {
            bars.push(TiingoDailyBar {
                symbol: required_column(&row, "symbol")?.to_string(),
                date: required_column(&row, "date")?.to_string(),
                open: parse_f64(required_column(&row, "open")?, "open")?,
                high: parse_f64(required_column(&row, "high")?, "high")?,
                low: parse_f64(required_column(&row, "low")?, "low")?,
                close: parse_f64(required_column(&row, "close")?, "close")?,
                volume: parse_u64(required_column(&row, "volume")?, "volume")?,
            });
        }

        Ok(bars)
    }

    fn load_tiingo_actions(name: &str) -> Result<Vec<TiingoCorporateAction>, String> {
        let raw = read_required_file(&fixture_dir(name).join("tiingo-actions.csv"))?;
        let rows = parse_csv_rows(&raw)?;
        let mut actions = Vec::new();

        for row in rows {
            let kind = TiingoCorporateActionKind::parse(required_column(&row, "kind")?)
                .ok_or_else(|| "unknown tiingo corporate action kind".to_string())?;

            actions.push(TiingoCorporateAction {
                symbol: required_column(&row, "symbol")?.to_string(),
                ex_date: required_column(&row, "ex_date")?.to_string(),
                kind,
                split_ratio: parse_optional_f64(
                    required_column(&row, "split_ratio")?,
                    "split_ratio",
                )?,
                cash_amount: parse_optional_f64(
                    required_column(&row, "cash_amount")?,
                    "cash_amount",
                )?,
            });
        }

        Ok(actions)
    }

    fn fixture_dir(name: &str) -> PathBuf {
        workspace_root().join("fixtures").join(name)
    }

    fn sample_requested_window() -> SnapshotRequestedWindow {
        SnapshotRequestedWindow {
            start_date: "2025-01-02".to_string(),
            end_date: "2025-01-10".to_string(),
        }
    }

    fn sample_capture_metadata() -> SnapshotCaptureMetadata {
        SnapshotCaptureMetadata {
            capture_mode: "live_provider_fetch".to_string(),
            entrypoint: "cargo xtask capture-live-snapshot".to_string(),
            captured_at_unix_epoch_seconds: Some(1_736_400_000),
        }
    }

    fn sample_snapshot_bundle() -> PersistedSnapshotBundle {
        let bars = load_tiingo_bars("m2_tiingo_split_adjustment").unwrap();
        let actions = load_tiingo_actions("m2_tiingo_split_adjustment").unwrap();
        let stored = ingest_tiingo_symbol_history(
            SnapshotMetadata {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                snapshot_id: "live:tiingo:TEST:2025-01-03:2025-01-08".to_string(),
                provider_identity: ProviderIdentity::Tiingo,
            },
            "TEST",
            &bars,
            &actions,
        )
        .unwrap();
        let descriptor = SnapshotBundleDescriptor::from_stored_symbols(
            stored.metadata.snapshot_id.clone(),
            stored.metadata.provider_identity,
            sample_requested_window(),
            sample_capture_metadata(),
            std::slice::from_ref(&stored),
        )
        .unwrap();

        PersistedSnapshotBundle {
            descriptor,
            symbols: vec![stored],
        }
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("trendlab-data lives under crates/")
            .to_path_buf()
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let mut path = std::env::temp_dir();
        path.push(OsString::from(format!(
            "trendlab-data-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    fn read_required_file(path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
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

            rows.push(
                headers
                    .iter()
                    .cloned()
                    .zip(values.into_iter())
                    .collect::<BTreeMap<_, _>>(),
            );
        }

        Ok(rows)
    }

    fn required_column<'a>(
        row: &'a BTreeMap<String, String>,
        key: &str,
    ) -> Result<&'a str, String> {
        row.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("missing required column `{key}`"))
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

    fn parse_u64(value: &str, field: &str) -> Result<u64, String> {
        value
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid integer for `{field}`"))
    }
}
