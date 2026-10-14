use super::*;

use std::collections::BTreeMap;

use crate::sink::agreement as sink_agreement;

pub fn summarize_sink_slots_for_profile(items: &[sink_agreement::SinkSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_strict_audit_event_profiles_from_slot(
    items: &[sink_agreement::SinkSlot],
) -> Vec<StrictAuditEventProfile> {
    let mut out: Vec<StrictAuditEventProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_audit_event_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_strict_audit_event_profile(item: &StrictAuditEventProfile) -> f64 {
    sink_agreement::sink_agreement_weight(6, true)
}
