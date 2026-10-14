use super::*;

use std::collections::BTreeMap;

use crate::credential::bundle as credential_bundle;

pub fn summarize_seasonal_credential_variances_for_slot(
    items: &[credential_bundle::SeasonalCredentialVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_binding_slots_from_variance(items: &[credential_bundle::SeasonalCredentialVariance]) -> Vec<BindingSlot> {
    let mut out: Vec<BindingSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_binding_slot(item: &BindingSlot) -> f64 {
    credential_bundle::seasonal_credential_variance_weight(1, true)
}
