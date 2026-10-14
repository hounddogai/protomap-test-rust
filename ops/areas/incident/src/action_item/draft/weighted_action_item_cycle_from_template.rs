use super::*;

use std::collections::BTreeMap;

use crate::escalation::claim as escalation_claim;

pub fn summarize_escalation_templates_for_cycle(
    items: &[escalation_claim::EscalationTemplate],
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

pub fn derive_weighted_action_item_cycles_from_template(
    items: &[escalation_claim::EscalationTemplate],
) -> Vec<WeightedActionItemCycle> {
    let mut out: Vec<WeightedActionItemCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_action_item_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_weighted_action_item_cycle(item: &WeightedActionItemCycle) -> String {
    escalation_claim::escalation_template_bucket(item.effective_total())
}
