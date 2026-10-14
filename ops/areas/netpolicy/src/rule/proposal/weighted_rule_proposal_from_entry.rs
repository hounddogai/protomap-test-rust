use super::*;

use std::collections::BTreeMap;

use crate::flow_log::digest as flow_log_digest;

pub fn summarize_lenient_flow_log_entries_for_proposal(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_weighted_rule_proposals_from_entry(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> Vec<WeightedRuleProposal> {
    let mut out: Vec<WeightedRuleProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_rule_proposal(item: &WeightedRuleProposal) -> String {
    flow_log_digest::rolling_flow_log_checkpoint_bucket(item.adjusted_value())
}
