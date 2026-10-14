use super::*;

use std::collections::BTreeMap;

use crate::peering::ledger as peering_ledger;

pub fn summarize_global_peering_ledgers_for_digest(
    items: &[peering_ledger::GlobalPeeringLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_adaptive_flow_log_digests_from_ledger(
    items: &[peering_ledger::GlobalPeeringLedger],
) -> Vec<AdaptiveFlowLogDigest> {
    let mut out: Vec<AdaptiveFlowLogDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_flow_log_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_adaptive_flow_log_digest(item: &AdaptiveFlowLogDigest) -> f64 {
    peering_ledger::regional_peering_quota_weight(2, true)
}
