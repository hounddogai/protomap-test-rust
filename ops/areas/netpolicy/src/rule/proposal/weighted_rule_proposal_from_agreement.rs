use super::*;

use std::collections::BTreeMap;

use crate::allowlist::agreement as allowlist_agreement;

pub fn summarize_allowlist_agreements_for_proposal(
    items: &[allowlist_agreement::AllowlistAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_weighted_rule_proposals_from_agreement(
    items: &[allowlist_agreement::AllowlistAgreement],
) -> Vec<WeightedRuleProposal> {
    let mut out: Vec<WeightedRuleProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_proposal(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_weighted_rule_proposal(item: &WeightedRuleProposal) -> String {
    allowlist_agreement::manual_allowlist_exception_bucket(item.adjusted_value())
}
