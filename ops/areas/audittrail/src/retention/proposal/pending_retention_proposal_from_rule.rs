use super::*;

use std::collections::BTreeMap;

use crate::redaction::rule as redaction_rule;

pub fn summarize_redaction_rules_for_proposal(items: &[redaction_rule::RedactionRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_pending_retention_proposals_from_rule(
    items: &[redaction_rule::RedactionRule],
) -> Vec<PendingRetentionProposal> {
    let mut out: Vec<PendingRetentionProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_retention_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_retention_proposal(item: &PendingRetentionProposal) -> String {
    redaction_rule::classify_redaction_rule(item.net_value())
}
