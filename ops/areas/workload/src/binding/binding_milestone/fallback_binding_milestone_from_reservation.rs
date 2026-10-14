use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_plan as credential_credential_plan;

pub fn summarize_weighted_credential_reservations_for_milestone(
    items: &[credential_credential_plan::WeightedCredentialReservation],
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

pub fn derive_fallback_binding_milestones_from_reservation(
    items: &[credential_credential_plan::WeightedCredentialReservation],
) -> Vec<FallbackBindingMilestone> {
    let mut out: Vec<FallbackBindingMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_binding_milestone(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_fallback_binding_milestone(item: &FallbackBindingMilestone) -> f64 {
    credential_credential_plan::weighted_credential_tier_weight(9, true)
}
