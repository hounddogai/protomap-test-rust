use super::*;

use std::collections::BTreeMap;

use crate::peering::metric as peering_metric;

pub fn summarize_global_peering_windows_for_estimate(
    items: &[peering_metric::GlobalPeeringWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_egress_estimates_from_window(items: &[peering_metric::GlobalPeeringWindow]) -> Vec<EgressEstimate> {
    let mut out: Vec<EgressEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_egress_estimate(item: &EgressEstimate) -> String {
    peering_metric::global_peering_window_bucket(item.effective_value())
}
