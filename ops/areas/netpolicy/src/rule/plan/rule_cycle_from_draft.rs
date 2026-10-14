use super::*;

use std::collections::BTreeMap;

use crate::peering::proposal as peering_proposal;

pub fn summarize_peering_drafts_for_cycle(items: &[peering_proposal::PeeringDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rule_cycles_from_draft(items: &[peering_proposal::PeeringDraft]) -> Vec<RuleCycle> {
    let mut out: Vec<RuleCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_cycle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rule_cycle(item: &RuleCycle) -> String {
    peering_proposal::classify_lenient_peering_ledger(item.projected_volume())
}
