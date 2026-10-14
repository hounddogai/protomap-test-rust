use super::*;

use std::collections::BTreeMap;

use crate::failover::segment as failover_segment;

pub fn summarize_failover_proposals_for_slot(items: &[failover_segment::FailoverProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_rolling_delegation_slots_from_proposal(
    items: &[failover_segment::FailoverProposal],
) -> Vec<RollingDelegationSlot> {
    let mut out: Vec<RollingDelegationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_delegation_slot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rolling_delegation_slot(item: &RollingDelegationSlot) -> String {
    failover_segment::classify_failover_proposal(item.measured_value())
}
