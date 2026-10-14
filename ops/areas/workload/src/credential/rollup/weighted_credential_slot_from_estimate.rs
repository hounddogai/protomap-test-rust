use super::*;

use std::collections::BTreeMap;

use crate::credential::estimate as credential_estimate;

pub fn summarize_global_credential_estimates_for_slot(
    items: &[credential_estimate::GlobalCredentialEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_weighted_credential_slots_from_estimate(
    items: &[credential_estimate::GlobalCredentialEstimate],
) -> Vec<WeightedCredentialSlot> {
    let mut out: Vec<WeightedCredentialSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_credential_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_weighted_credential_slot_extra(item: &WeightedCredentialSlot) -> String {
    credential_estimate::classify_scheduled_credential_segment(item.projected_total())
}
