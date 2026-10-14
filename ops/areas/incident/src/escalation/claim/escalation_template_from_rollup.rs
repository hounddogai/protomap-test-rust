use super::*;

use std::collections::BTreeMap;

use crate::timeline::rollup as timeline_rollup;

pub fn summarize_timeline_rollups_for_template(items: &[timeline_rollup::TimelineRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_escalation_templates_from_rollup(items: &[timeline_rollup::TimelineRollup]) -> Vec<EscalationTemplate> {
    let mut out: Vec<EscalationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_escalation_template(item: &EscalationTemplate) -> String {
    timeline_rollup::classify_timeline_rollup(item.net_volume())
}
