use super::*;

use std::collections::BTreeMap;

use crate::probe::forecast as probe_forecast;

pub fn summarize_probe_cycles_for_record(items: &[probe_forecast::ProbeCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_legacy_probe_records_from_cycle(items: &[probe_forecast::ProbeCycle]) -> Vec<LegacyProbeRecord> {
    let mut out: Vec<LegacyProbeRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_probe_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_legacy_probe_record(item: &LegacyProbeRecord) -> String {
    probe_forecast::probe_forecast_bucket(item.measured_value())
}
