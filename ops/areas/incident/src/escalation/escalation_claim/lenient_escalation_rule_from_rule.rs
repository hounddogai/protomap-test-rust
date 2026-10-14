use super::*;

use std::collections::BTreeMap;

use crate::postmortem::postmortem_profile as postmortem_postmortem_profile;

pub fn summarize_postmortem_rules_for_rule(
    items: &[postmortem_postmortem_profile::PostmortemRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_lenient_escalation_rules_from_rule(
    items: &[postmortem_postmortem_profile::PostmortemRule],
) -> Vec<LenientEscalationRule> {
    let mut out: Vec<LenientEscalationRule> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_escalation_rule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_lenient_escalation_rule(item: &LenientEscalationRule) -> String {
    postmortem_postmortem_profile::classify_batched_postmortem_profile(item.measured_value())
}
