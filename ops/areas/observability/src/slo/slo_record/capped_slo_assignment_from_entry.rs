use super::*;

use std::collections::BTreeMap;

use crate::slo::entry as slo_entry;

pub fn summarize_capped_slo_entries_for_assignment(items: &[slo_entry::CappedSloEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_capped_slo_assignments_from_entry(items: &[slo_entry::CappedSloEntry]) -> Vec<CappedSloAssignment> {
    let mut out: Vec<CappedSloAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_slo_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_capped_slo_assignment(item: &CappedSloAssignment) -> String {
    slo_entry::normalize_legacy_slo_audit_code(&item.code)
}
