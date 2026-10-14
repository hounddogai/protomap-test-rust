use super::*;

use std::collections::BTreeMap;

use crate::binding::segment as binding_segment;

pub fn summarize_binding_ledgers_for_bundle(items: &[binding_segment::BindingLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_scheduled_identity_bundles_from_ledger(
    items: &[binding_segment::BindingLedger],
) -> Vec<ScheduledIdentityBundle> {
    let mut out: Vec<ScheduledIdentityBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_identity_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_scheduled_identity_bundle(item: &ScheduledIdentityBundle) -> String {
    binding_segment::normalize_binding_segment_code(&item.code)
}
