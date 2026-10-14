use super::*;

use std::collections::BTreeMap;

use crate::failover::adjustment as failover_adjustment;

pub fn summarize_weighted_failover_groups_for_proposal(
    items: &[failover_adjustment::WeightedFailoverGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_failover_proposals_from_group(
    items: &[failover_adjustment::WeightedFailoverGroup],
) -> Vec<FailoverProposal> {
    let mut out: Vec<FailoverProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_failover_proposal(item: &FailoverProposal) -> String {
    failover_adjustment::failover_bundle_bucket(item.projected_value())
}
