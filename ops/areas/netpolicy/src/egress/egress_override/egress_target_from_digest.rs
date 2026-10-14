use super::*;

use std::collections::BTreeMap;

use crate::flow_log::score as flow_log_score;

pub fn summarize_flow_log_digests_for_target(items: &[flow_log_score::FlowLogDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_egress_targets_from_digest(items: &[flow_log_score::FlowLogDigest]) -> Vec<EgressTarget> {
    let mut out: Vec<EgressTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_egress_target(item: &EgressTarget) -> String {
    flow_log_score::classify_flow_log_score(item.measured_value())
}
