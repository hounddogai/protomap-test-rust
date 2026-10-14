use super::*;

use std::collections::BTreeMap;

use crate::freeze::audit as freeze_audit;

pub fn summarize_lenient_freeze_records_for_assignment(
    items: &[freeze_audit::LenientFreezeRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_adaptive_canary_assignments_from_record(
    items: &[freeze_audit::LenientFreezeRecord],
) -> Vec<AdaptiveCanaryAssignment> {
    let mut out: Vec<AdaptiveCanaryAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_canary_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_canary_assignment(item: &AdaptiveCanaryAssignment) -> String {
    freeze_audit::classify_lenient_freeze_record(item.measured_value())
}
