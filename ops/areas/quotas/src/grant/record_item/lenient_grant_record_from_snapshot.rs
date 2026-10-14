use super::*;

use std::collections::BTreeMap;

use crate::forecast::snapshot as forecast_snapshot;

pub fn summarize_forecast_snapshots_for_record(items: &[forecast_snapshot::ForecastSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_grant_records_from_snapshot(
    items: &[forecast_snapshot::ForecastSnapshot],
) -> Vec<LenientGrantRecord> {
    let mut out: Vec<LenientGrantRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_grant_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_lenient_grant_record(item: &LenientGrantRecord) -> String {
    forecast_snapshot::classify_forecast_snapshot(item.weighted_exposure())
}
