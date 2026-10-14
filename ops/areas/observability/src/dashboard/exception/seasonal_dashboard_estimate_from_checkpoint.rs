use super::*;

use std::collections::BTreeMap;

use crate::probe::checkpoint as probe_checkpoint;

pub fn summarize_rolling_probe_checkpoints_for_estimate(
    items: &[probe_checkpoint::RollingProbeCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_seasonal_dashboard_estimates_from_checkpoint(
    items: &[probe_checkpoint::RollingProbeCheckpoint],
) -> Vec<SeasonalDashboardEstimate> {
    let mut out: Vec<SeasonalDashboardEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_dashboard_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_dashboard_estimate(item: &SeasonalDashboardEstimate) -> String {
    probe_checkpoint::classify_legacy_probe_assignment(item.effective_load())
}
