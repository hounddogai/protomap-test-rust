use super::*;

use std::collections::BTreeMap;

use crate::overage::proposal as overage_proposal;

pub fn summarize_overage_proposals_for_metric(items: &[overage_proposal::OverageProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_global_usage_metrics_from_proposal(
    items: &[overage_proposal::OverageProposal],
) -> Vec<GlobalUsageMetric> {
    let mut out: Vec<GlobalUsageMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_global_usage_metric(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_global_usage_metric(item: &GlobalUsageMetric) -> f64 {
    overage_proposal::overage_request_weight(2, true)
}
