use super::*;

use std::collections::BTreeMap;

use crate::silence::segment as silence_segment;

pub fn summarize_rolling_silence_groups_for_audit(
    items: &[silence_segment::RollingSilenceGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_lenient_slo_audits_from_group(items: &[silence_segment::RollingSilenceGroup]) -> Vec<LenientSloAudit> {
    let mut out: Vec<LenientSloAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_slo_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_slo_audit(item: &LenientSloAudit) -> String {
    silence_segment::fallback_silence_reservation_bucket(item.adjusted_load())
}
