use super::*;

use std::collections::BTreeMap;

use crate::rule::proposal as rule_proposal;

pub fn summarize_weighted_rule_proposals_for_claim(
    items: &[rule_proposal::WeightedRuleProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_pending_allowlist_claims_from_proposal(
    items: &[rule_proposal::WeightedRuleProposal],
) -> Vec<PendingAllowlistClaim> {
    let mut out: Vec<PendingAllowlistClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_allowlist_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_pending_allowlist_claim(item: &PendingAllowlistClaim) -> String {
    rule_proposal::strict_rule_variance_bucket(item.effective_volume())
}
