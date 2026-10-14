use super::*;

use std::collections::BTreeMap;

use crate::postmortem::draft as postmortem_draft;

pub fn summarize_fallback_postmortem_entries_for_threshold(
    items: &[postmortem_draft::FallbackPostmortemEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_escalation_thresholds_from_entry(
    items: &[postmortem_draft::FallbackPostmortemEntry],
) -> Vec<EscalationThreshold> {
    let mut out: Vec<EscalationThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_threshold(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_escalation_threshold(item: &EscalationThreshold) -> f64 {
    postmortem_draft::legacy_postmortem_plan_weight(9, true)
}
