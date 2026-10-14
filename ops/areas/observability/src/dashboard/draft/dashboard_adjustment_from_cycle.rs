use super::*;

use std::collections::BTreeMap;

use crate::probe::claim as probe_claim;

pub fn summarize_weighted_probe_cycles_for_adjustment(
    items: &[probe_claim::WeightedProbeCycle],
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

pub fn derive_dashboard_adjustments_from_cycle(items: &[probe_claim::WeightedProbeCycle]) -> Vec<DashboardAdjustment> {
    let mut out: Vec<DashboardAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_adjustment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_dashboard_adjustment(item: &DashboardAdjustment) -> String {
    probe_claim::normalize_legacy_probe_proposal_code(&item.code)
}
