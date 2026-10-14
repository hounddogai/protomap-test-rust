use super::*;

use std::collections::BTreeMap;

use crate::flow_log::score as flow_log_score;

pub fn summarize_flow_log_digests_for_entry(items: &[flow_log_score::FlowLogDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_peering_entries_from_digest(items: &[flow_log_score::FlowLogDigest]) -> Vec<PeeringEntry> {
    let mut out: Vec<PeeringEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_entry(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_peering_entry(item: &PeeringEntry) -> String {
    flow_log_score::normalize_flow_log_score_code(&item.code)
}
