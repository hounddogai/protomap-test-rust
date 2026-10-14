use super::*;

use std::collections::BTreeMap;

use crate::egress::profile as egress_profile;

pub fn summarize_egress_profiles_for_bundle(items: &[egress_profile::EgressProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_global_allowlist_bundles_from_profile(
    items: &[egress_profile::EgressProfile],
) -> Vec<GlobalAllowlistBundle> {
    let mut out: Vec<GlobalAllowlistBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_global_allowlist_bundle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_global_allowlist_bundle(item: &GlobalAllowlistBundle) -> String {
    egress_profile::normalize_egress_tier_code(&item.code)
}
