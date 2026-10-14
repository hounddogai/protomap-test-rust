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

pub fn derive_postmortem_cycles_from_template(items: &[escalation_claim::EscalationTemplate]) -> Vec<PostmortemCycle> {
    let mut out: Vec<PostmortemCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_postmortem_cycle(item: &PostmortemCycle) -> String {
    escalation_claim::escalation_template_bucket(item.projected_total())
}
