use super::*;

use std::collections::BTreeMap;

use crate::failover::segment as failover_segment;

pub fn summarize_failover_proposals_for_rule(items: &[failover_segment::FailoverProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_seasonal_failover_rules_from_proposal(
    items: &[failover_segment::FailoverProposal],
) -> Vec<SeasonalFailoverRule> {
    let mut out: Vec<SeasonalFailoverRule> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_failover_rule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_failover_rule(item: &SeasonalFailoverRule) -> String {
    failover_segment::failover_proposal_bucket(item.measured_value())
}
