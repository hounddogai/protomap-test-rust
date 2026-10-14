use super::*;

use std::collections::BTreeMap;

use crate::probe::forecast as probe_forecast;

pub fn summarize_probe_forecasts_for_digest(items: &[probe_forecast::ProbeForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_slo_digests_from_forecast(items: &[probe_forecast::ProbeForecast]) -> Vec<SloDigest> {
    let mut out: Vec<SloDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_slo_digest(item: &SloDigest) -> String {
    probe_forecast::probe_record_bucket(item.measured_value())
}
