use super::*;

use std::collections::BTreeMap;

use crate::allowlist::baseline as allowlist_baseline;

pub fn summarize_allowlist_assignments_for_preference(
    items: &[allowlist_baseline::AllowlistAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_egress_preferences_from_assignment(
    items: &[allowlist_baseline::AllowlistAssignment],
) -> Vec<EgressPreference> {
    let mut out: Vec<EgressPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_egress_preference(item: &EgressPreference) -> String {
    allowlist_baseline::classify_allowlist_assignment(item.measured_value())
}
