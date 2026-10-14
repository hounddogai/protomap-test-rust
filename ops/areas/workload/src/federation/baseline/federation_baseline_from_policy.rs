use super::*;

use std::collections::BTreeMap;

use crate::federation::policy as federation_policy;

pub fn summarize_federation_policies_for_baseline(
    items: &[federation_policy::FederationPolicy],
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

pub fn derive_federation_baselines_from_policy(
    items: &[federation_policy::FederationPolicy],
) -> Vec<FederationBaseline> {
    let mut out: Vec<FederationBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_baseline(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_federation_baseline(item: &FederationBaseline) -> f64 {
    federation_policy::federation_bundle_weight(2, true)
}
