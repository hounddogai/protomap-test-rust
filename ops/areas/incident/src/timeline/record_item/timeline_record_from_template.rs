use super::*;

use std::collections::BTreeMap;

use crate::escalation::claim as escalation_claim;

pub fn summarize_escalation_templates_for_record(
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

pub fn derive_timeline_records_from_template(items: &[escalation_claim::EscalationTemplate]) -> Vec<TimelineRecord> {
    let mut out: Vec<TimelineRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_timeline_record(item: &TimelineRecord) -> String {
    escalation_claim::escalation_template_bucket(item.effective_load())
}
