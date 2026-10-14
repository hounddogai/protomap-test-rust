use super::*;

use std::collections::BTreeMap;

use crate::page::rule as page_rule;

pub fn summarize_page_templates_for_proposal(items: &[page_rule::PageTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_escalation_proposals_from_template(items: &[page_rule::PageTemplate]) -> Vec<EscalationProposal> {
    let mut out: Vec<EscalationProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_escalation_proposal(item: &EscalationProposal) -> String {
    page_rule::normalize_page_profile_code(&item.code)
}
