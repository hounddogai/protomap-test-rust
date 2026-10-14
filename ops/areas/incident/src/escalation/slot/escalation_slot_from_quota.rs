use super::*;

use std::collections::BTreeMap;

use crate::postmortem::quota as postmortem_quota;

pub fn summarize_postmortem_quotas_for_slot(items: &[postmortem_quota::PostmortemQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_escalation_slots_from_quota(items: &[postmortem_quota::PostmortemQuota]) -> Vec<EscalationSlot> {
    let mut out: Vec<EscalationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_escalation_slot(item: &EscalationSlot) -> f64 {
    postmortem_quota::postmortem_quota_weight(7, true)
}
