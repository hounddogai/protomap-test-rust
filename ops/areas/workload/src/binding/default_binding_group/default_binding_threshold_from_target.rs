use super::*;

use std::collections::BTreeMap;

use crate::federation::federation_target as federation_federation_target;

pub fn summarize_seasonal_federation_targets_for_threshold(
    items: &[federation_federation_target::SeasonalFederationTarget],
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

pub fn derive_default_binding_thresholds_from_target(
    items: &[federation_federation_target::SeasonalFederationTarget],
) -> Vec<DefaultBindingThreshold> {
    let mut out: Vec<DefaultBindingThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_default_binding_threshold(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_default_binding_threshold(item: &DefaultBindingThreshold) -> String {
    federation_federation_target::classify_federation_variance(item.effective_total())
}
