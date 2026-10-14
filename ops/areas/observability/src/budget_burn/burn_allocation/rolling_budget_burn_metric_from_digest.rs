use super::*;

use std::collections::BTreeMap;

use crate::probe::digest as probe_digest;

pub fn summarize_probe_digests_for_metric(items: &[probe_digest::ProbeDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_rolling_budget_burn_metrics_from_digest(
    items: &[probe_digest::ProbeDigest],
) -> Vec<RollingBudgetBurnMetric> {
    let mut out: Vec<RollingBudgetBurnMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_budget_burn_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_rolling_budget_burn_metric(item: &RollingBudgetBurnMetric) -> String {
    probe_digest::lenient_probe_group_bucket(item.effective_load())
}
